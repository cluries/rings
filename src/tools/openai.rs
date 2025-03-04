// https://github.com/64bit/async-openai

use crate::erx;
use async_openai::config::OpenAIConfig;
use async_openai::types::{ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs, CreateChatCompletionResponse, ImageDetail};
use async_openai::Client;
use serde::Deserialize;
use std::time::{Duration, SystemTime};

#[derive(Debug)]
pub struct Provider {
    pub base: String,
    pub model: String,
    pub key: String,
}

pub struct LLM {
    provider: Provider,
}

pub struct ChatResponse {
    duration: Duration,
    response: CreateChatCompletionResponse,
}


pub struct MessageBuilder {
    messages: Vec<ChatCompletionRequestMessage>,
}


impl ChatResponse {
    pub fn new(duration: Duration, response: CreateChatCompletionResponse) -> Self {
        Self { duration, response }
    }


    pub fn duration(&self) -> Duration {
        self.duration
    }

    pub fn response(&self) -> CreateChatCompletionResponse {
        self.response.clone()
    }

    pub fn response_vec(&self) -> Vec<String> {
        if self.response.choices.len() < 1 {
            return Vec::new();
        }

        self.response.choices.iter().fold(
            Vec::new(), |mut acc, choice| {
                if let Some(_reason) = choice.finish_reason {}
                acc.push(choice.message.content.clone().unwrap_or_default());
                acc
            },
        )
    }

    pub fn response_string(&self) -> String {
        self.response_vec().join("")
    }

    pub fn response_json<T: for<'a> Deserialize<'a>>(&self) -> Result<T, erx::Erx> {
        let c = self.response_string();
        serde_json::from_str::<T>(&c).map_err(erx::smp)
    }
}

impl MessageBuilder {
    pub fn default() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    pub fn add(&mut self, message: ChatCompletionRequestMessage) -> &mut Self {
        self.messages.push(message);
        self
    }

    pub fn user(&mut self, message: &str) -> &mut Self {
        use async_openai::types::ChatCompletionRequestUserMessageArgs;

        self.messages.push(
            ChatCompletionRequestUserMessageArgs::default().content(message).build().unwrap().into()
        );
        self
    }

    pub fn system(&mut self, message: &str) -> &mut Self {
        use async_openai::types::ChatCompletionRequestSystemMessageArgs;
        self.messages.push(
            ChatCompletionRequestSystemMessageArgs::default().content(message).build().unwrap().into()
        );
        self
    }

    pub fn assistant(&mut self, message: &str) -> &mut Self {
        use async_openai::types::ChatCompletionRequestAssistantMessageArgs;
        self.messages.push(
            ChatCompletionRequestAssistantMessageArgs::default().content(message).build().unwrap().into()
        );
        self
    }

    pub fn developer(&mut self, message: &str) -> &mut Self {
        use async_openai::types::ChatCompletionRequestDeveloperMessageArgs;
        self.messages.push(
            ChatCompletionRequestDeveloperMessageArgs::default().content(message).build().unwrap().into()
        );
        self
    }

    pub fn function(&mut self, message: &str) -> &mut Self {
        use async_openai::types::ChatCompletionRequestFunctionMessageArgs;
        self.messages.push(
            ChatCompletionRequestFunctionMessageArgs::default().content(message).build().unwrap().into()
        );
        self
    }

    pub fn tool(&mut self, message: &str) -> &mut Self {
        use async_openai::types::ChatCompletionRequestToolMessageArgs;
        self.messages.push(
            ChatCompletionRequestToolMessageArgs::default().content(message).build().unwrap().into()
        );
        self
    }

    pub fn image(&mut self, url: &str) -> &mut Self {
        use async_openai::types::ChatCompletionRequestMessageContentPartImageArgs;
        self.messages.push(
            ChatCompletionRequestMessageContentPartImageArgs::default().image_url(
                async_openai::types::ImageUrl {
                    url: url.into(),
                    detail: Some(ImageDetail::Auto),
                }
            ).build().unwrap().into()
        );
        self
    }
}

impl Into<Vec<ChatCompletionRequestMessage>> for MessageBuilder {
    fn into(self) -> Vec<ChatCompletionRequestMessage> {
        self.messages
    }
}



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

    pub async fn single(&self, prompt: Vec<ChatCompletionRequestMessage>) -> Result<ChatResponse, erx::Erx> {
        let request = CreateChatCompletionRequestArgs::default().stream(
            false
        ).model(
            self.provider.model.clone()
        ).response_format(
            async_openai::types::ResponseFormat::Text
        ).messages(
            prompt
        ).build().map_err(erx::smp)?;


        let start = SystemTime::now();
        let response = self.cli().chat().create(request).await.map_err(erx::smp)?;
        let duration = SystemTime::now().duration_since(start).unwrap_or_default();
        Ok(ChatResponse::new(duration, response))
    }
}


