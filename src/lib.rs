use std::env;

use base64::{engine::general_purpose, Engine};
use cloud_vision_flows::text_detection;
use http_req::{
    request::{Method, Request},
    uri::Uri,
};
use openai_flows::{chat_completion, ChatModel, ChatOptions, ChatResponse};
use serde_json::Value;
use tg_flows::{listen_to_update, ChatId, Telegram, Update, UpdateKind};

#[no_mangle]
pub fn run() {
    let telegram_token = env::var("telegram_token").unwrap();
    let openai_key_name = env::var("openai_key_name").unwrap_or("jaykchen".to_string());

    listen_to_update(telegram_token.clone(), |update| {
        handle(update, telegram_token, openai_key_name)
    });
}

// {{{ handle
fn handle(update: Update, telegram_token: String, openai_key_name: String) {
    if let UpdateKind::Message(msg) = update.kind {
        let chat_id = msg.chat.id;

        let tele = Telegram::new(telegram_token.clone());

        if let Some(text) = msg.text() {
            if text == "/start" {
                let init_message = "Hello! I am your medical lab report analyzer bot. Zoom in on where you need assistance with, take a photo and upload it as a file, or paste the photo in the chatbox to send me if you think it's clear enough.";
                _ = tele.send_message(chat_id, init_message.to_string());
                return;
            }
        }

        // TODO: check msg type when user upload photo *file*(s)

        _ = tele.send_message(chat_id, "please waiting...");

        let image_file_id = match (msg.document().is_some(), msg.photo().is_some()) {
            (true, false) => msg.document().unwrap().file.id.clone(),
            (false, true) => msg.photo().unwrap().last().unwrap().file.id.clone(),
            (_, _) => {
                _ = tele.send_message(chat_id, "not doc either photo");
                return;
            }
        };

        _ = tele.send_message(chat_id, format!("{:?}", msg.kind));

        match download_photo_data_base64(&telegram_token, &image_file_id) {
            Ok(data) => {
                let c = doctor(data, openai_key_name, chat_id);
                if let Some(c) = c {
                    if c.restarted {
                        // _ = tele.send_message(chat_id, "I am starting a new session. You can always type \"restart\" to terminate the current session.\n\n".to_string() + &c.choice);
                    } else {
                        _ = tele.send_message(chat_id, c.choice);
                    }
                }
            }
            Err(_e) => {
                eprintln!("Error downloading photo");
            }
        };
    }
}
// }}}

// {{{ download_photo_data_base64
fn download_photo_data_base64(
    token: &str,
    file_id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let file_url = format!(
        "https://api.telegram.org/bot{}/getFile?file_id={}",
        token, file_id
    );
    let file_uri: Uri = Uri::try_from(file_url.as_str()).unwrap();

    let mut file_response = Vec::new();
    Request::new(&file_uri)
        .method(Method::GET)
        .send(&mut file_response)?;

    let file_info: Value = serde_json::from_slice(&file_response)?;
    let file_path = file_info["result"]["file_path"]
        .as_str()
        .ok_or("file_path missing")?;

    // Download the file using the file path
    let file_download_url = format!("https://api.telegram.org/file/bot{}/{}", token, file_path);
    let file_download_uri: Uri = Uri::try_from(file_download_url.as_str()).unwrap();

    let mut file_data = Vec::new();
    Request::new(&file_download_uri)
        .method(Method::GET)
        .send(&mut file_data)?;
    let base64_encoded = general_purpose::STANDARD.encode(file_data);

    Ok(base64_encoded)
}
// }}}

// {{{ doctor
fn doctor(data: String, openai_key_name: String, chat_id: ChatId) -> Option<ChatResponse> {
    if let Ok(ocr_text) = text_detection(data) {
        let text = if !ocr_text.is_empty() {
            ocr_text
        } else {
            "".to_string()
        };

        let system = r#"You are a medical lab technican, you'll read a lab report and tell the user the most important findings of the report in short bullets, please use the following template: The major findings are:
                        1) [the name of the measurement] [status of the reading]
                        ...
                        one sentence summary about the subject's health status."#;
        let co = ChatOptions {
            // model: ChatModel::GPT4,
            model: ChatModel::GPT35Turbo,
            restart: false,
            // restart: text.eq_ignore_ascii_case("restart"),
            system_prompt: Some(system),
            retry_times: 3,
        };

        chat_completion(&openai_key_name, &chat_id.to_string(), &text, &co)
    } else {
        None
    }
}
// }}}
