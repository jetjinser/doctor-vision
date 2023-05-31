use serde_json::json;
use store_flows as store;

use crate::App;

impl App {
    pub fn store_image_id(&self, image_file_id: String) -> usize {
        let key = format!("{}:image_file_ids", self.msg.chat.id);
        let ids = store::get(&key).unwrap_or(json!([]));
        let mut ids = serde_json::from_value(ids).unwrap_or(vec![]);

        log::info!("Stored image: {}", image_file_id);

        ids.push(image_file_id);
        let len = ids.len();
        let idv = serde_json::to_value(ids).unwrap_or(json!([]));
        store::set(&key, idv, None);

        len
    }

    pub fn clear_image_ids(&self) {
        log::info!("Cleard images in store");

        let key = format!("{}:image_file_ids", self.msg.chat.id);

        store::del(&key);
    }

    pub fn get_image_ids(&self) -> Option<serde_json::Value> {
        let key = format!("{}:image_file_ids", self.msg.chat.id);

        store::get(&key)
    }
}
