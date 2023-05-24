use cloud_vision_flows::text_detection;
use openai_flows::chat::ChatResponse;

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

    pub async fn doctor_once(&self, id: String) {
        match self.download_photo_data_base64(id.to_string()) {
            Ok(data) => {
                let cp = self.doctor(data).await;

                if let Some(c) = cp {
                    self.send_msg(c.choice);
                }
            }
            Err(_) => {
                eprintln!("Error downloading photo: {}", id);
            }
        }
    }

    pub async fn doctor_batch(&self) {
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
                self.send_msg(cp.choice);
            } else {
                self.send_msg("Something went wrong...");
            }
        }

        self.clear_image_ids();
    }
}
