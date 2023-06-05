use std::format;

use serde::{Deserialize, Serialize};
use store_flows as store;

use crate::{first_x_string, App, HELP};

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
        log::debug!("switched to normal");
    }

    pub fn sw_pending(&self) {
        let state = serde_json::to_value(State::Pending).unwrap();
        store::set(format!("{}:state", self.msg.chat.id).as_str(), state, None);
        log::debug!("switched to pending");
    }

    pub fn sw_chat(&self) {
        let state = serde_json::to_value(State::Chat).unwrap();
        store::set(format!("{}:state", self.msg.chat.id).as_str(), state, None);
        log::debug!("switched to chat");
    }

    pub fn state(&self) -> Option<State> {
        let state = store::get(format!("{}:state", self.msg.chat.id).as_str());
        state.and_then(|v| serde_json::from_value(v).ok())
    }
}

impl App {
    pub async fn normal_stuff(&self) {
        log::debug!("Doing normal stuff");

        if let Some(id) = self.get_image_id() {
            log::debug!("Got image id: {}", id);

            if self.is_group_media() {
                log::debug!("IS group media");

                self.sw_pending();

                // TODO: get last update

                self.send_msg("You are uploading multiple photos. Please type /finish once you have uploaded all photos. Thank you");

                self.pending_stuff().await;
            } else {
                log::debug!("NOT group media");

                let ph_msg = self.send_msg("please wait a minute.").unwrap();
                self.doctor_once(id, ph_msg.chat.id, ph_msg.id).await;
                self.sw_chat();
            }
        } else if let Some(text) = self.msg.text() {
            log::debug!("Got text: {}", first_x_string(15, text));

            self.send_msg(&*HELP);
        }

        log::debug!("Normal stuff done");
    }

    pub async fn pending_stuff(&self) {
        log::debug!("Doing pending stuff");

        if let Some(text) = self.msg.text() {
            log::debug!("Got text: {}", first_x_string(15, text));

            match text {
                "/finish" => {
                    self.doctor_batch().await;

                    self.sw_chat();
                }
                _ => {
                    self.send_msg("You are uploading multiple photos. Please type /finish once you have uploaded all photos. Thank you");
                }
            }
        }

        if let Some(id) = self.get_image_id() {
            log::debug!("Got image id: {}", id);

            self.store_image_id(id);
        }

        log::debug!("Pending stuff done");
    }

    pub async fn chat_stuff(&self) {
        log::debug!("Doing chat stuff");

        if let Some(id) = self.get_image_id() {
            log::debug!("Got image id: {}", id);

            self.sw_normal();
            self.normal_stuff().await;
        } else if let Some(text) = self.msg.text() {
            log::debug!("Got text: {}", first_x_string(15, text));

            let ph_msg = self.send_msg("please wait a minute.").unwrap();

            let msg = if let Some(cp) = self.chat(text).await {
                cp.choice
            } else {
                log::warn!("failed get chatgpt choise in chat");
                String::from("Something went wrong...")
            };

            self.edit_msg(ph_msg.chat.id, ph_msg.id, msg);
        }

        log::debug!("Chat stuff done");
    }
}
