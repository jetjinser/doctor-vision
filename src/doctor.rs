use cloud_vision_flows::text_detection;
use openai_flows::chat::ChatResponse;
use tg_flows::Message;

use crate::App;

impl App {
    pub async fn doctor(&self, data: String) -> Option<ChatResponse> {
        let text = text_detection(data);
        if let Ok(t) = text {
            self.chat(&t).await
        } else {
            None
        }
    }

    pub async fn doctor_once(&self, id: String, msg: Message) {
        match self.download_photo_data_base64(id.to_string()) {
            Ok(data) => {
                let cp = self.doctor(data).await;

                if let Some(c) = cp {
                    _ = self.tele.edit_message_text(msg.chat.id, msg.id, c.choice);
                } else {
                    _ = self
                        .tele
                        .edit_message_text(msg.chat.id, msg.id, "Something went wrong...");
                }
            }
            Err(_) => {
                eprintln!("Error downloading photo: {}", id);
            }
        }
    }

    pub async fn doctor_batch(&self, msg: Message) {
        if let Some(value) = self.get_image_ids() {
            let ids = value.as_array().unwrap();

            let texts = ids
                .iter()
                .filter_map(|id| {
                    let id = id.as_str().unwrap();
                    match self.download_photo_data_base64(id.to_string()) {
                        Ok(data) => Some(data),
                        Err(_) => {
                            eprintln!("Error downloading photo: {}", id);
                            None
                        }
                    }
                })
                .collect::<Vec<_>>()
                .join("\n---\n");

            if let Some(cp) = self.doctor(texts).await {
                _ = self.tele.edit_message_text(msg.chat.id, msg.id, cp.choice);
            } else {
                _ = self
                    .tele
                    .edit_message_text(msg.chat.id, msg.id, "Something went wrong...");
            }
        }

        self.clear_image_ids();
    }
}
