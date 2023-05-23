use serde::{Deserialize, Serialize};
use store_flows as store;

use crate::{App, HELP};

#[derive(Serialize, Deserialize, Debug)]
pub enum State {
    Normal,
    Pending,
    Chat,
}

impl App {
    pub fn sw_normal(&self) {
        let state = serde_json::to_value(State::Normal).unwrap();
        store::set(format!("{}:state", self.msg.chat.id).as_str(), state, None);
    }

    pub fn sw_pending(&self) {
        let state = serde_json::to_value(State::Pending).unwrap();
        store::set(format!("{}:state", self.msg.chat.id).as_str(), state, None);
    }

    pub fn sw_chat(&self) {
        let state = serde_json::to_value(State::Chat).unwrap();
        store::set(format!("{}:state", self.msg.chat.id).as_str(), state, None);
    }

    pub fn state(&self) -> Option<State> {
        let state = store::get(format!("{}:state", self.msg.chat.id).as_str());
        state.and_then(|v| serde_json::from_value(v).ok())
    }
}

impl App {
    pub fn normal_stuff(&self) {
        if let Some(id) = self.get_image_id() {
            if self.is_group_media() {
                self.sw_pending();
                self.pending_stuff();
                self.send_msg("received multi-media, switch to pending state");
            } else {
                self.send_msg("please wait a minute.");
                self.doctor_once(id);
                self.sw_chat();
            }
        } else if self.msg.text().is_some() {
            self.send_msg(HELP);
        }
    }

    pub fn pending_stuff(&self) {
        if let Some(text) = self.msg.text() {
            if text == "/finish" {
                self.send_msg("please wait a minute.");
                self.doctor_batch();

                self.sw_chat();
            } else {
                self.send_msg("use `/finish` commmand to start doctor");
            }
        }

        if let Some(id) = self.get_image_id() {
            self.store_image_id(id);
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
