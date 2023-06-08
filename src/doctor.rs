use cloud_vision_flows::text_detection;
use openai_flows::chat::ChatResponse;
use tg_flows::{ChatId, MessageId};

use crate::{first_x_string, App};

impl App {
    pub fn ocr(&self, data: String) -> Option<String> {
        log::debug!("doing ocr");

        match text_detection(data) {
            Ok(t) => {
                log::debug!("Got ocr_text via cloud vision: {}", first_x_string(15, &t));
                Some(t)
            }
            Err(e) => {
                log::warn!("Failed to get ocr_text via cloud vision: {}", e);
                None
            }
        }
    }

    pub async fn doctor(&self, text: String) -> Option<ChatResponse> {
        log::debug!("Doctoring");
        self.chat(&text).await
    }

    pub async fn doctor_once(&self, id: String, chat_id: ChatId, msg_id: MessageId) {
        log::debug!("Doctoring once");

        match self.download_photo_data_base64(id.clone()) {
            Ok(data) => {
                if let Some(text) = self.ocr(data) {
                    let cp = self.doctor(text).await;
                    if let Some(c) = cp {
                        self.edit_msg(chat_id, msg_id, c.choice);
                    } else {
                        self.edit_msg(chat_id, msg_id, "Something went wrong...");
                    }
                }
            }
            Err(e) => {
                log::warn!("Failed to download file: {}, reason: {}", id, e);
            }
        }
    }

    pub async fn doctor_batch(&self) {
        log::debug!("Doctoring batch");

        if let Some(value) = self.get_image_ids() {
            let ids = value.as_array().unwrap();

            let msg = self.send_msg("please wait a minute.").unwrap();

            let data = ids
                .iter()
                .filter_map(|id| {
                    self.download_photo_data_base64(id.as_str().unwrap().to_string())
                        .ok()
                        .and_then(|data| self.ocr(data))
                })
                .collect();

            let chat_id = msg.chat.id;
            let msg_id = msg.id;

            let cp = self.doctor(data).await;
            if let Some(c) = cp {
                self.edit_msg(chat_id, msg_id, c.choice);
            } else {
                self.edit_msg(chat_id, msg_id, "Something went wrong...");
            }
        }

        self.clear_image_ids();
    }
}
