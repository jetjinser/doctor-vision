use base64::{engine::general_purpose, Engine};
use http_req::{
    request::{Method, Request},
    uri::Uri,
};
use tg_flows::Message;

use crate::App;

impl App {
    pub fn send_msg<S>(&self, text: S) -> Option<Message>
    where
        S: Into<String>,
    {
        let text: String = text.into();
        log::debug!(
            "Sending message: {}...",
            text.chars().take(15).collect::<String>()
        );
        self.tele.send_message(self.msg.chat.id, text).ok()
    }

    pub fn download_photo_data_base64(
        &self,
        file_id: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let file = self.tele.get_file(file_id)?;
        let file_path = file.path;

        log::debug!("Downloading file from {}", file_path);

        // TODO: need to update sdk
        // let file_data = self.tele.download_file(file_path);

        // Download the file using the file path
        let file_download_url = format!(
            "https://api.telegram.org/file/bot{}/{}",
            self.tele_token, file_path
        );
        let file_download_uri: Uri = Uri::try_from(file_download_url.as_str())?;

        let mut file_data = Vec::new();
        Request::new(&file_download_uri)
            .method(Method::GET)
            .send(&mut file_data)?;
        let base64_encoded = general_purpose::STANDARD.encode(file_data);

        Ok(base64_encoded)
    }

    pub fn is_group_media(&self) -> bool {
        self.msg.media_group_id().is_some()
    }

    pub fn get_image_id(&self) -> Option<String> {
        let msg = &self.msg;
        match (msg.document(), msg.photo().map(|p| p.last())) {
            (Some(doc), None) => Some(doc.file.id.clone()),
            (None, Some(Some(ps))) => Some(ps.file.id.clone()),
            (_, _) => None,
        }
    }
}
