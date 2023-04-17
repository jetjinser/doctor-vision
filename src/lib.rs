use base64;
use cloud_vision_flows::text_detection;
use http_req::{
    request::{Method, Request},
    uri::Uri,
};
use openai_flows::{chat_completion, ChatModel, ChatOptions};
use serde_json::Value;
use std::env;
use tg_flows::{listen_to_update, Telegram, UpdateKind};
#[no_mangle]
pub fn run() {
    let telegram_token = std::env::var("telegram_token").unwrap();
    let tele = Telegram::new(telegram_token.clone());

    listen_to_update(&telegram_token, |update| {
        if let UpdateKind::Message(msg) = update.kind {
            let image_file_id = msg.document().unwrap().file.id.clone();
            let chat_id = msg.chat.id;
            let openai_key_name = "jaykchen";

            match download_photo_data_base64(&telegram_token, &image_file_id) {
                Ok(data) => {
                    if let Ok(ocr) = text_detection(data) {
                        let mut text = if !ocr.is_empty() { ocr } else { "".to_string() };

                        let system = "You are a medical lab technican, you'll read a lab report and tell the user the most important findings of the report";
                        let co = ChatOptions {
                            // model: ChatModel::GPT4,
                            model: ChatModel::GPT35Turbo,
                            restart: text.eq_ignore_ascii_case("restart"),
                            system_prompt: Some(system),
                            retry_times: 3,
                        };

                        let c = chat_completion(&openai_key_name, &chat_id.to_string(), &text, &co);

                        if let Some(c) = c {
                            if c.restarted {
                                _ = tele.send_message(chat_id, "I am starting a new conversation. You can always type \"restart\" to terminate the current conversation.\n\n".to_string() + &c.choice);
                            } else {
                                _ = tele.send_message(chat_id, c.choice);
                            }
                        }
                    }
                }
                Err(_e) => {
                    eprintln!("Error downloading photo");
                    return;
                }
            };

            // if let Some(c) = c {
            //     if c.restarted {
            //         _ = tele.send_message(chat_id, "I am starting a new conversation. You can always type \"restart\" to terminate the current conversation.\n\n".to_string() + &c.choice);
            //     } else {
            //         _ = tele.send_message(chat_id, c.choice);
            //     }
            // }
        }
    });
}

pub fn download_photo_data_base64(
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
    let base64_encoded = base64::encode(file_data);

    Ok(base64_encoded)
}
