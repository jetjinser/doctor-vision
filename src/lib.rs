use std::env;

use tg_flows::{listen_to_update, Message, Telegram, Update, UpdateKind};

#[no_mangle]
#[tokio::main]
pub async fn run() {
    let telegram_token = env::var("telegram_token").unwrap();

    listen_to_update(telegram_token.clone(), |update| async {
        handler(update, telegram_token)
    })
    .await;
}

fn handler(update: Update, tele_token: String) {
    if let UpdateKind::Message(msg) = update.kind {
        let tele = Telegram::new(tele_token);

        let id = get_image_id(&msg).unwrap_or("...".to_string());
        let text = msg.text().unwrap_or("...");

        let re_msg = format!("text: {text}\nimage: {id}");

        _ = tele.send_message(msg.chat.id, re_msg);
    }
}

fn get_image_id(msg: &Message) -> Option<String> {
    match (msg.document(), msg.photo().map(|p| p.last())) {
        (Some(doc), None) => Some(doc.file.id.clone()),
        (None, Some(Some(ps))) => Some(ps.file.id.clone()),
        (_, _) => None,
    }
}
