use base64;
use cloud_vision_flows::text_detection;
use http_req::{
    request::{Method, Request},
    response::{Headers, Response},
    uri::Uri,
};
use image::{self, Pixel};
use lambda_flows::{request_received, send_response};
use serde_json::{json, Value};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::Cursor;
use tg_flows::{listen_to_update, ChatId, InputFile, Telegram, UpdateKind};

#[no_mangle]
pub fn run() {
    let telegram_token = std::env::var("telegram_token").unwrap();
    let tele = Telegram::new(telegram_token.clone());
    // let file_path = "~/Downloads/lab_report_part.jpg"; // Replace with the path to your image file

    // send_photo_as_document(&bot, chat_id, file_path).await;

    listen_to_update(&telegram_token, |update| {
        if let UpdateKind::Message(msg) = update.kind {
            let image_file_id = msg.document().unwrap().file.id.clone();
            // let image_file_id = msg.photo().unwrap()[0].file.id.clone();
            let chat_id = msg.chat.id;
            tele.send_message(chat_id, image_file_id.clone());

            match download_photo_data_base64(&telegram_token, &image_file_id) {
                Ok(data) => {
                    let text = text_detection(data);

                    match text {
                        Ok(r) => {
                            tele.send_message(chat_id, r.clone());

                            send_response(
                                200,
                                vec![(
                                    String::from("content-type"),
                                    String::from("text/plain; charset=UTF-8"),
                                )],
                                r.as_bytes().to_vec(),
                            );
                        }
                        Err(e) => send_response(
                            500,
                            vec![(
                                String::from("content-type"),
                                String::from("text/plain; charset=UTF-8"),
                            )],
                            e.as_bytes().to_vec(),
                        ),
                    }
                }
                Err(e) => {
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

// async fn send_photo_as_document(bot: &Bot, chat_id: i64, file_path: &str) {
//     let input_file = teloxide::types::InputFile::File {
//         media: teloxide::types::MediaFile::from_file(file_path),
//         file_name: Some(String::from(file_path)),
//         thumb: None,
//     };

//     let document = teloxide::types::InputMediaDocument::new(input_file);

//     if let Err(e) = bot.send_document(chat_id, document).send().await {
//         eprintln!("Error while sending the document: {}", e);
//     }
// }
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
