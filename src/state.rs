use std::format;

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
    pub async fn normal_stuff(&self) {
        if let Some(id) = self.get_image_id() {
            if self.is_group_media() {
                self.sw_pending();
                self.send_msg("You are uploading multiple photos. Please type /finish once you have uploaded all photos. Thank you");
                self.pending_stuff().await;
            } else {
                let ph_msg = self.send_msg("please wait a minute.").unwrap();
                self.doctor_once(id, ph_msg).await;
                self.sw_chat();
            }
        } else if self.msg.text().is_some() {
            self.send_msg(HELP);
        }
    }

    pub async fn pending_stuff(&self) {
        if let Some(text) = self.msg.text() {
            match text {
                "/finish" => {
                    let ph_msg = self.send_msg("please wait a minute.").unwrap();
                    self.doctor_batch(ph_msg).await;

                    self.sw_chat();
                }
                _ => {
                    self.send_msg("You are uploading multiple photos. Please type /finish once you have uploaded all photos. Thank you");
                }
            }
        }

        if let Some(id) = self.get_image_id() {
            let count = self.store_image_id(id);
            self.send_msg(format!("received {} photo(s)", count));
        }
    }

    pub async fn chat_stuff(&self) {
        if self.get_image_id().is_some() {
            self.sw_normal();
            self.normal_stuff().await;
        } else if let Some(text) = self.msg.text() {
            let ph_msg = self.send_msg("please wait a minute.").unwrap();

            let msg = if let Some(cp) = self.chat(text).await {
                cp.choice
            } else {
                String::from("Something went wrong...")
            };

            _ = self.tele.edit_message_text(ph_msg.chat.id, ph_msg.id, msg);
        }
    }
}
