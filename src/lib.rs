use std::env;

use state::State;
use tg_flows::{listen_to_update, Message, Telegram, Update, UpdateKind};

mod chat;
mod doctor;
mod state;
mod store;
mod telegram;

const HELP: &str = "Hello! I am your medical lab report analyzer bot. Zoom in on where you need assistance with, take a photo and upload it as a file, or paste the photo in the chatbox to send me if you think it's clear enough.";

const SYSTEM: &str = r#"You are a medical lab technican, you'll read a lab report and tell the user the most important findings of the report in short bullets, please use the following template: The major findings are:
                        1) [the name of the measurement] [status of the reading]
                        ...
                        one sentence summary about the subject's health status."#;

struct App {
    tele: Telegram,
    tele_token: String,
    openai_key: String,
    msg: Message,
}

impl App {
    fn new(tele_token: String, openai_key: String, msg: Message) -> Self {
        let tele = Telegram::new(tele_token.clone());
        Self {
            tele,
            tele_token,
            openai_key,
            msg,
        }
    }
}

#[no_mangle]
pub fn run() {
    let telegram_token = env::var("telegram_token").unwrap();
    let openai_key_name = env::var("openai_key_name").unwrap_or("jaykchen".to_string());

    listen_to_update(telegram_token.clone(), |update| {
        handler(update, telegram_token, openai_key_name)
    });
}

fn handler(update: Update, tele_token: String, openai_key: String) {
    if let UpdateKind::Message(msg) = update.kind {
        let app = App::new(tele_token, openai_key, msg.clone());

        match msg.text() {
            Some(text) if text == "/init" => {
                app.send_msg("initialized");
                app.sw_normal();

                return;
            }
            _ => (),
        }

        let state = app.state().unwrap_or(State::Normal);
        match state {
            State::Normal => app.normal_stuff(),
            State::Pending => app.pending_stuff(),
            State::Chat => app.chat_stuff(),
        }
    }
}
