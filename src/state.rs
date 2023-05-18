use serde::{Deserialize, Serialize};
use store_flows::{self as store, Expire, ExpireKind};

use crate::{App, HELP};

#[derive(Serialize, Deserialize, Debug)]
pub enum State {
    Waiting,
    Receiving,
    Answering,
}

impl App {
    pub fn waiting(&self) {
        let state = serde_json::to_value(State::Waiting).unwrap();
        store::set(format!("{}:state", self.msg.chat.id).as_str(), state, None);
    }

    pub fn receiving(&self) {
        let state = serde_json::to_value(State::Receiving).unwrap();
        store::set(format!("{}:state", self.msg.chat.id).as_str(), state, None);
    }

    pub fn answering(&self) {
        let state = serde_json::to_value(State::Receiving).unwrap();
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
    pub fn waiting_stuff(&self) {
        if let Some(text) = self.msg.text() {
            if text == "/start" {
                self.receiving();
            }

            self.send_msg(HELP);
        }
    }

    pub fn receiving_stuff(&self) {
        let msg = &self.msg;

        if let Some(text) = msg.text() {
            if text == "/end" {
                self.answering();
                self.doctor_and_response();
                return;
            }

            let count = self.image_counts();
            self.send_msg(format!("received {count} photo(s)\n\n{HELP}"));
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

    pub fn answering_stuff(&self) {
        let msg = &self.msg;

        if let Some(text) = msg.text() {
            if text == "/bye" {
                self.waiting();
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
