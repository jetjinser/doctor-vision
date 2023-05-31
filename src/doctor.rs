use cloud_vision_flows::text_detection;
use openai_flows::chat::ChatResponse;
use tg_flows::Message;

use crate::{first_x_string, App};

impl App {
    pub async fn doctor(&self, data: String) -> Option<ChatResponse> {
        log::debug!("Doctoring");

        let text = text_detection(data);
        if let Ok(t) = text {
            log::debug!("Got ocr_text via cloud vision: {}", first_x_string(15, &t));
            self.chat(&t).await
        } else {
            log::warn!("Failed to get ocr_text via cloud vision");
            None
        }
    }

    pub async fn doctor_once(&self, id: String, msg: Message) {
        log::debug!("Doctoring once");

        match self.download_photo_data_base64(id.to_string()) {
            Ok(data) => {
                let cp = self.doctor(data).await;

                if let Some(c) = cp {
                    self.edit_msg(msg.chat.id, msg.id, &c.choice);
                } else {
                    self.edit_msg(msg.chat.id, msg.id, "Something went wrong...");
                }
            }
            Err(e) => {
                log::warn!("Failed to download file: {}, reason: {}", id, e);
            }
        }
    }

    pub async fn doctor_batch(&self, msg: Message) {
        log::debug!("Doctoring batch");

        if let Some(value) = self.get_image_ids() {
            let ids = value.as_array().unwrap();

            let texts = ids
                .iter()
                .filter_map(|id| {
                    let id = id.as_str().unwrap();
                    match self.download_photo_data_base64(id.to_string()) {
                        Ok(data) => Some(data),
                        Err(e) => {
                            log::warn!("Failed to download file: {}, reason: {}", id, e);
                            None
                        }
                    }
                })
                .collect::<Vec<_>>()
                .join("\n---\n");

            if let Some(cp) = self.doctor(texts).await {
                self.edit_msg(msg.chat.id, msg.id, cp.choice);
            } else {
                self.edit_msg(msg.chat.id, msg.id, "Something went wrong...");
            }
        }

        self.clear_image_ids();
    }
}
