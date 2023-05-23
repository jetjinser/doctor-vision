use serde::{Deserialize, Serialize};
use store_flows::{self as store, Expire, ExpireKind};

use crate::{App, HELP};

#[derive(Serialize, Deserialize, Debug)]
pub enum State {
    Normal,
    Chat,
}

impl App {
    pub fn sw_normal(&self) {
        let state = serde_json::to_value(State::Normal).unwrap();
        store::set(format!("{}:state", self.msg.chat.id).as_str(), state, None);
    }

    pub fn sw_chat(&self) {
        let state = serde_json::to_value(State::Chat).unwrap();
        store::set(
            format!("{}:state", self.msg.chat.id).as_str(),
            state,
            Some(Expire {
                kind: ExpireKind::Ex,
                value: 300, // 5 mins
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
        if let Some(id) = self.get_image_id() {
            self.send_msg("please wait a minite.");
            self.doctor_once(id);
            self.sw_chat();
        } else if self.msg.text().is_some() {
            self.send_msg(HELP);
        }
    }

    pub fn chat_stuff(&self) {
        if self.get_image_id().is_some() {
            self.sw_normal();
            self.normal_stuff();
        } else if let Some(text) = self.msg.text() {
            let msg = if let Some(cp) = self.chat(text) {
                cp.choice
            } else {
                String::from("Something went wrong...")
            };

            self.send_msg(msg);
        }
    }
}
