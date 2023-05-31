use std::env;

use dotenv::dotenv;
use flowsnet_platform_sdk::logger;
use state::State;
use tg_flows::{listen_to_update, Message, Telegram, Update, UpdateKind};

mod chat;
mod doctor;
mod state;
mod store;
mod telegram;

const HELP: &str = "Howdy! I am here to help explain doctor notes, forms, prescriptions or lab reports to you. Snap a photo of your document and send it to me! If the document has multiple pages, please send multiple photos as a single message.";

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
#[tokio::main(flavor = "current_thread")]
pub async fn run() {
    dotenv().ok();
    logger::init();
    log::debug!("Running doctor-vision/main");

    let telegram_token = env::var("telegram_token").unwrap();
    let openai_key_name = env::var("openai_key_name").unwrap_or("jaykchen".to_string());

    listen_to_update(telegram_token.clone(), |update| {
        handler(update, telegram_token, openai_key_name)
    })
    .await;
}

async fn handler(update: Update, tele_token: String, openai_key: String) {
    if let UpdateKind::Message(msg) = update.kind {
        let app = App::new(tele_token, openai_key, msg.clone());

        match msg.text() {
            Some(text) if text == "/init" => {
                log::debug!("initializing");

                app.send_msg("initialized");
                app.sw_normal();

                log::debug!("initialized");
                return;
            }
            _ => (),
        }

        let state = app.state().unwrap_or_else(|| {
            log::debug!("No state is stored, fallback to Normal mode");
            State::Normal
        });
        match state {
            State::Normal => app.normal_stuff().await,
            State::Pending => app.pending_stuff().await,
            State::Chat => app.chat_stuff().await,
        }
    } else {
        log::debug!("Not Message update kind, ignored");
    }
}

fn first_x_string<S: AsRef<str>>(x: usize, str: S) -> String {
    str.as_ref().chars().take(x).collect()
}
