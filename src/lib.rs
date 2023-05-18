use std::{env, format};

use base64::{engine::general_purpose, Engine};
use cloud_vision_flows::text_detection;
use http_req::{
    request::{Method, Request},
    uri::Uri,
};
use openai_flows::{chat_completion, ChatModel, ChatOptions, ChatResponse};
use serde_json::{json, Value};
use store_flows as store;
use tg_flows::{listen_to_update, ChatId, Telegram, Update, UpdateKind};

#[no_mangle]
pub fn run() {
    store::del("in_context");

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

        let system = r#"You are a medical lab technican, you'll read a lab report and tell the user the most important findings of the report in short bullets, please use the following template: The major findings are:
                        1) [the name of the measurement] [status of the reading]
                        ...
                        one sentence summary about the subject's health status."#;
        let mut co = ChatOptions {
            // model: ChatModel::GPT4,
            model: ChatModel::GPT35Turbo,
            restart: false,
            system_prompt: Some(system),
            retry_times: 3,
        };

        let tele = Telegram::new(telegram_token.clone());

        if let Some(text) = msg.text() {
            co.restart = text.eq_ignore_ascii_case("restart");

            _ = tele.send_message(chat_id, text);

            if text == "/end" {
                _ = tele.send_message(chat_id, "please waiting...");

                store::set("in_context", json!(1), None);

                let ids = store::get("image_file_ids").unwrap_or(json!([]));

                for idv in ids.as_array().unwrap_or(&vec![]) {
                    _ = tele.send_message(chat_id, idv.as_str().unwrap_or("..."));

                    if let Some(id) = idv.as_str() {
                        match download_photo_data_base64(&telegram_token, id) {
                            Ok(data) => {
                                let c = doctor(data, &openai_key_name, chat_id, &co);
                                if let Some(c) = c {
                                    _ = tele.send_message(chat_id, c.choice);
                                }
                            }
                            Err(_e) => {
                                eprintln!("Error downloading photo");
                            }
                        };
                    }
                }

                store::del("image_file_ids");

                return;
            }

            let in_context = store::get("in_context");

            match in_context {
                Some(_) => {
                    let c = chat_completion(&openai_key_name, &chat_id.to_string(), text, &co);
                    if let Some(cp) = c {
                        if cp.restarted {
                            store::del("in_context");

                            _ = tele.send_message(chat_id, format!("I am starting a new session. You can always type \"restart\" to terminate the current session.\n\n{}", cp.choice));
                        } else {
                            _ = tele.send_message(chat_id, cp.choice);
                        }
                    }
                }
                None => {
                    let init_message = "Hello! I am your medical lab report analyzer bot. Zoom in on where you need assistance with, take a photo and upload it as a file, or paste the photo in the chatbox to send me if you think it's clear enough.\nYou can start at any time by sending photo(s) and end it with `/end`";
                    _ = tele.send_message(chat_id, init_message);
                }
            }

            return;
        }

        let image_file_id = match (msg.document(), msg.photo().map(|p| p.last())) {
            (Some(doc), None) => doc.file.id.clone(),
            (None, Some(Some(ps))) => ps.file.id.clone(),
            (_, _) => return,
        };

        let ids = store::get("image_file_ids").unwrap_or(json!([]));

        let mut ids = serde_json::from_value(ids).unwrap_or(vec![]);
        ids.push(image_file_id);
        ids.dedup();

        _ = tele.send_message(chat_id, format!(":: {} ::", ids.join(", ")));

        let ids = serde_json::to_value(ids).unwrap_or(json!([]));
        store::set("image_file_ids", ids, None);
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
    let file_uri: Uri = Uri::try_from(file_url.as_str())?;

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
    let file_download_uri: Uri = Uri::try_from(file_download_url.as_str())?;

    let mut file_data = Vec::new();
    Request::new(&file_download_uri)
        .method(Method::GET)
        .send(&mut file_data)?;
    let base64_encoded = general_purpose::STANDARD.encode(file_data);

    Ok(base64_encoded)
}
// }}}

// {{{ doctor
fn doctor(
    data: String,
    openai_key_name: &str,
    chat_id: ChatId,
    co: &ChatOptions,
) -> Option<ChatResponse> {
    text_detection(data)
        .ok()
        .and_then(|ocr_text| chat_completion(openai_key_name, &chat_id.to_string(), &ocr_text, co))
}
// }}}
