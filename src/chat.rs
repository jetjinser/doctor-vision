use openai_flows::{
    chat::{ChatModel, ChatOptions, ChatResponse},
    FlowsAccount, OpenAIFlows,
};

use crate::{App, SYSTEM};

impl App {
    pub async fn chat(&self, text: &str) -> Option<ChatResponse> {
        let chat_options = ChatOptions {
            // model: ChatModel::GPT4,
            model: ChatModel::GPT35Turbo,
            restart: false,
            system_prompt: Some(SYSTEM),
        };

        let mut of = OpenAIFlows::new();
        of.set_flows_account(FlowsAccount::Provided(self.openai_key.clone()));
        of.set_retry_times(3);

        log::debug!(
            "Chat text: {}... waiting chatgpt completion",
            text.chars().take(15).collect::<String>()
        );

        of.chat_completion(&self.msg.chat.id.to_string(), text, &chat_options)
            .await
            .ok()
    }
}
