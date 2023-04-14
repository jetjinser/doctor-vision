use base64;
use cloud_vision_flows::text_detection;
use http_req::{
    request::{Method, Request},
    response::{Headers, Response},
    uri::Uri,
};
use image::GenericImageView;
use image::{self, Pixel};
use lambda_flows::{request_received, send_response};
use serde_json::{json, Value};
use std::fs::File;
use std::io::prelude::*;
use std::io::Cursor;
use tg_flows::{listen_to_update, ChatId, InputFile, Telegram, UpdateKind};
use url::Url;

#[no_mangle]
pub fn run() {
    let telegram_token = std::env::var("telegram_token").unwrap();
    let tele = Telegram::new(telegram_token.clone());

    listen_to_update(&telegram_token, |update| {
        if let UpdateKind::Message(msg) = update.kind {
            let image_file_id = msg.photo().unwrap()[0].file.id.clone();
            let chat_id = msg.chat.id;
            tele.send_message(chat_id, image_file_id.clone());
            tele.send_message(chat_id, msg.chat.first_name().unwrap());

            let photo_data = match download_photo_data(&telegram_token, &image_file_id) {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("Error downloading photo: {}", e);
                    return;
                }
            };

            let img = image::load_from_memory(&photo_data).unwrap();

            let mut image_buffer = Cursor::new(Vec::new());
            img.write_to(&mut image_buffer, image::ImageOutputFormat::Png)
                .expect("Error converting image to byte buffer");

            let image_bytes = image_buffer.into_inner();
            let head = image_bytes[..20]
                .iter()
                .map(|byte| format!("{:02x}", byte))
                .collect::<Vec<String>>()
                .join(" ");
            let tail = image_bytes[image_bytes.len() - 20..]
                .iter()
                .map(|byte| format!("{:02x}", byte))
                .collect::<Vec<String>>()
                .join(" ");
            tele.send_message(chat_id, head);
            tele.send_message(chat_id, tail);

            let image_base64 = base64::encode(&image_bytes);

            let text = text_detection(image_base64);

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

fn download_photo_data(token: &str, file_id: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let file_url = format!(
        "https://api.telegram.org/bot{}/getFile?file_id={}",
        token, file_id
    );
    let file_uri: Uri = Uri::try_from(file_url.as_str()).unwrap();

    let mut file_response = Vec::new();
    Request::new(&file_uri)
        .method(Method::GET)
        .send(&mut file_response)?;

    let response: serde_json::Value = serde_json::from_slice(&file_response)?;

    let file_path = response["result"]["file_path"].as_str().unwrap();
    let photo_url = format!("https://api.telegram.org/file/bot{}/{}", token, file_path);
    let photo_uri: Uri = Uri::try_from(photo_url.as_str()).unwrap();

    let mut photo_data = Vec::new();
    Request::new(&photo_uri)
        .method(Method::GET)
        .send(&mut photo_data)?;

    Ok(photo_data)
}
