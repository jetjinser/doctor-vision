use serde_json::json;
use store_flows as store;

use crate::App;

impl App {
    pub fn store_image_id(&self, image_file_id: String) {
        let key = format!("{}:image_file_ids", self.msg.chat.id);

        let ids = store::get(key.as_str()).unwrap_or(json!([]));

        let mut ids = serde_json::from_value(ids).unwrap_or(vec![]);
        ids.push(image_file_id);

        let ids = serde_json::to_value(ids).unwrap_or(json!([]));
        store::set(key.as_str(), ids, None);
    }

    pub fn image_counts(&self) -> usize {
        let ids = store::get(format!("{}:image_file_ids", self.msg.chat.id).as_str())
            .unwrap_or(json!([]));

        ids.as_array().unwrap().len()
    }
}
