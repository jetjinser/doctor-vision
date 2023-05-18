use serde::{Deserialize, Serialize};
use store_flows::{self as store, Expire, ExpireKind};

use crate::{App, HELP};

#[derive(Serialize, Deserialize, Debug)]
pub enum State {
    Normal,
    Batch,
    FAQ,
}

impl App {
    pub fn sw_normal(&self) {
        let state = serde_json::to_value(State::Normal).unwrap();
        store::set(format!("{}:state", self.msg.chat.id).as_str(), state, None);
    }

    pub fn sw_batch(&self) {
        let state = serde_json::to_value(State::Batch).unwrap();
        store::set(format!("{}:state", self.msg.chat.id).as_str(), state, None);
    }

    pub fn sw_faq(&self) {
        let state = serde_json::to_value(State::FAQ).unwrap();
        store::set(
            format!("{}:state", self.msg.chat.id).as_str(),
            state,
            Some(Expire {
                kind: ExpireKind::Ex,
                value: 120, // 2 mins
            }),
        );
    }

    pub fn state(&self) -> Option<State> {
        let state = store::get(format!("{}:state", self.msg.chat.id).as_str());
        state.and_then(|v| serde_json::from_value(v).ok())
    }
}

impl App {
    pub fn normal_stuff(&self) {
        if let Some(text) = self.msg.text() {
            match text {
                "/help" => {
                    self.send_msg(HELP);
                }
                "/start" => {
                    self.sw_batch();

                    // XXX: msg
                    self.send_msg("<another help message>");
                }
                _ => {}
            }
        }

        if let Some(id) = self.get_image_id() {
            self.doctor_in_normal(id);
        }
    }

    pub fn batch_stuff(&self) {
        let msg = &self.msg;

        if let Some(text) = msg.text() {
            match text {
                "/end" => {
                    self.sw_faq();
                    self.doctor_in_batch();
                }
                "/clear" => {
                    self.clear_image_ids();

                    // XXX: msg
                    self.send_msg("ok, cleared");
                }
                _ => {
                    let count = self.image_counts();
                    // XXX: msg
                    self.send_msg(format!("received {count} photo(s)"));
                }
            }
        }

        let image_file_id = match (msg.document(), msg.photo().map(|p| p.last())) {
            (Some(doc), None) => doc.file.id.clone(),
            (None, Some(Some(ps))) => ps.file.id.clone(),
            (_, _) => return,
        };

        self.store_image_id(image_file_id);
        // TODO: reply msg
        self.reply_msg("received it");
    }

    pub fn faq_stuff(&self) {
        let msg = &self.msg;

        if let Some(text) = msg.text() {
            if text == "/bye" {
                self.sw_normal();
                // XXX: msg
                self.send_msg("bye!");
                return;
            }

            if let Some(cp) = self.chat(text) {
                self.send_msg(cp.choice);
            }
        } else {
            self.send_msg(HELP);
        }
    }
}
