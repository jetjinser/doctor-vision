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
}
