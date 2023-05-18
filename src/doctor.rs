use cloud_vision_flows::text_detection;
use openai_flows::ChatResponse;
use store_flows as store;

use crate::App;

impl App {
    pub fn doctor(&self, data: String) -> Option<ChatResponse> {
        text_detection(data)
            .ok()
            .and_then(|ocr_text| self.chat(&ocr_text))
    }

    pub fn doctor_and_response(&self) {
        let key = format!("{}:image_file_ids", self.msg.chat.id);

        if let Some(value) = store::get(&key) {
            let ids = value.as_array().unwrap();

            for id in ids {
                let id = id.as_str().unwrap();
                match self.download_photo_data_base64(id.to_string()) {
                    Ok(data) => {
                        let cp = self.doctor(data);

                        if let Some(c) = cp {
                            self.send_msg(c.choice);
                        }
                    }
                    Err(_) => {
                        eprintln!("Error downloading photo: {}", id);
                    }
                }
            }
        }
        store::del(&key);
    }
}
