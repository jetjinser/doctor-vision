use cloud_vision_flows::text_detection;
use openai_flows::ChatResponse;

use crate::App;

impl App {
    pub fn doctor(&self, data: String) -> Option<ChatResponse> {
        text_detection(data)
            .ok()
            .and_then(|ocr_text| self.chat(&ocr_text))
    }

    pub fn doctor_once(&self, id: String) {
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

    pub fn doctor_batch(&self) {
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

            if let Some(cp) = self.doctor(texts) {
                self.send_msg(cp.choice);
            }
        }

        self.clear_image_ids();
    }
}
