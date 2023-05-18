use openai_flows::{chat_completion, ChatModel, ChatOptions, ChatResponse};

use crate::{App, SYSTEM};

impl App {
    pub fn chat(&self, text: &str) -> Option<ChatResponse> {
        let chat_options = ChatOptions {
            // model: ChatModel::GPT4,
            model: ChatModel::GPT35Turbo,
            restart: false,
            system_prompt: Some(SYSTEM),
            retry_times: 3,
        };

        chat_completion(
            self.openai_key.as_str(),
            self.msg.chat.id.to_string().as_str(),
            text,
            &chat_options,
        )
    }
}
