// https://github.com/64bit/async-openai

use std::time::SystemTime;
use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::types::{ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage, ChatCompletionRequestUserMessage, CreateChatCompletionRequestArgs, CreateChatCompletionResponse};
use serde::Deserialize;
use crate::erx;

#[derive(Debug)]
pub struct Provider {
    pub base: String,
    pub model: String,
    pub key: String,
}

pub struct LLM {
    provider: Provider,
}

pub struct LLMAnswer {
    response: CreateChatCompletionResponse,
}


pub struct MessageBuilder;

impl MessageBuilder {}

impl LLM {
    pub fn with_provider(provider: Provider) -> LLM {
        LLM { provider }
    }

    fn cli(&self) -> Client<OpenAIConfig> {
        let mut config = OpenAIConfig::default();
        config = config.with_api_base(self.provider.base.as_str());
        config = config.with_api_key(self.provider.key.as_str());
        Client::with_config(config)
    }

    pub async fn single(&self, prompt: Vec<ChatCompletionRequestMessage>) -> Result<LLMAnswer, erx::Erx> {
        let start = SystemTime::now();

        let request = CreateChatCompletionRequestArgs::default()
            .stream(false)
            .model(self.provider.model.clone())
            // .response_format(aoi::types::ResponseFormat::JsonObject)
            .messages(prompt).build().map_err(erx::smp)?;
        let response = self.cli().chat().create(request).await.map_err(erx::smp)?;


        let results = response.choices.iter().fold(
            Vec::new(), |mut acc, choice| {
                acc.push(choice.message.content.clone().unwrap_or_default());
                acc
            });

        let duration = SystemTime::now().duration_since(start).unwrap_or_default();

        Ok(results.join("\n"))
    }

    pub async fn chat_json<T: for<'a> Deserialize<'a>>(prompts: Vec<String>, image_url: Option<String>) -> Result<T, erx::Erx> {}
}


