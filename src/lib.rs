use base64;
use cloud_vision_flows::text_detection;
use http_req::{
    request::{Method, Request},
    response::{Headers, Response},
    uri::Uri,
};
use image;
use image::GenericImageView;
use lambda_flows::{request_received, send_response};
use std::fs::File;
use std::io::prelude::*;
use tg_flows::{listen_to_update, ChatId, InputFile, Telegram, UpdateKind};
use url::Url;
#[no_mangle]
pub fn run() {
    let telegram_token = std::env::var("telegram_token").unwrap();
    let tele = Telegram::new(telegram_token.clone());

    let crustaceans = "https://images.template.net/wp-content/uploads/2016/03/02064535/informal-lab-report-template.jpg";
    let url = Url::try_from(crustaceans).unwrap();

    listen_to_update(telegram_token, |update| {
        _ = tele.send_photo(ChatId(6221995180), InputFile::url(url));
        if let Ok(image) = ready_image(crustaceans) {
            let text = text_detection(image);
            match text {
                Ok(r) => {
                    tele.send_message(ChatId(6221995180), r.clone());

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

        if let UpdateKind::Message(msg) = update.kind {
            let mut text = msg.text().unwrap_or("");
            let chat_id = msg.chat.id;
        }
    });
}

pub  fn ready_image(inp: &str) -> Result<String, Box<dyn std::error::Error>> {
    let uri = Uri::try_from(inp)?;
    let mut writer = Vec::new();
    let _ = Request::new(&uri)
        .method(Method::GET)
        .send(&mut writer)?;
    // let img = image::load_from_memory(&writer)?;

    // // Write the image to a buffer in PNG format
    // let mut buffer: Vec<u8> = Vec::new();
    // img.write_to(&mut buffer, image::ImageOutputFormat::Png)?;

    // Convert the image data to base64
    let image_base64 = base64::encode(&writer);
    Ok(image_base64)
}
