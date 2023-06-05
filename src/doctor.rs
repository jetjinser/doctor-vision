use cloud_vision_flows::text_detection;
use openai_flows::chat::ChatResponse;
use tg_flows::{ChatId, MessageId};

use crate::{first_x_string, App};

impl App {
    pub async fn doctor(&self, id: String) -> Option<ChatResponse> {
        log::debug!("Doctoring");

        match self.download_photo_data_base64(id.clone()) {
            Ok(data) => match text_detection(data) {
                Ok(t) => {
                    log::debug!("Got ocr_text via cloud vision: {}", first_x_string(15, &t));
                    self.chat(&t).await
                }
                Err(e) => {
                    log::warn!("Failed to get ocr_text via cloud vision: {}", e);
                    None
                }
            },
            Err(e) => {
                log::warn!("Failed to download file: {}, reason: {}", id, e);
                None
            }
        }
    }

    pub async fn doctor_once(&self, id: String, chat_id: ChatId, msg_id: MessageId) {
        log::debug!("Doctoring once");

        let cp = self.doctor(id).await;
        if let Some(c) = cp {
            self.edit_msg(chat_id, msg_id, c.choice);
        } else {
            self.edit_msg(chat_id, msg_id, "Something went wrong...");
        }
    }

    pub async fn doctor_batch(&self) {
        log::debug!("Doctoring batch");

        if let Some(value) = self.get_image_ids() {
            let ids = value.as_array().unwrap();

            for id in ids {
                let msg = self.send_msg("please wait a minute.").unwrap();

                let chat_id = msg.chat.id;
                let msg_id = msg.id;

                let id = id.as_str().unwrap();
                self.doctor_once(id.to_string(), chat_id, msg_id).await;
            }
        }

        self.clear_image_ids();
    }
}
