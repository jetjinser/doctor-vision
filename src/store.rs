use serde_json::json;
use store_flows as store;

use crate::App;

impl App {
    pub fn store_image_id(&self, image_file_id: String) -> usize {
        let key = format!("{}:image_file_ids", self.msg.chat.id);

        let ids = store::get(&key).unwrap_or(json!([]));

        let mut ids = serde_json::from_value(ids).unwrap_or(vec![]);
        ids.push(image_file_id);
        let len = ids.len();

        let idv = serde_json::to_value(ids).unwrap_or(json!([]));
        store::set(&key, idv, None);

        len
    }

    pub fn clear_image_ids(&self) {
        let key = format!("{}:image_file_ids", self.msg.chat.id);

        store::del(&key);
    }

    pub fn get_image_ids(&self) -> Option<serde_json::Value> {
        let key = format!("{}:image_file_ids", self.msg.chat.id);

        store::get(&key)
    }

    pub fn count_images(&self) -> usize {
        let key = format!("{}:image_file_ids", self.msg.chat.id);

        let ids = store::get(&key).unwrap_or(json!([]));

        ids.as_array().unwrap().len()
    }
}
