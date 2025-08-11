// https://github.com/64bit/async-openai

use crate::erx;
use crate::tools::strings::IgnoreCase;
use async_openai::config::OpenAIConfig;
use async_openai::types::{
    ChatCompletionRequestMessage, ChatCompletionRequestMessageContentPartImage, ChatCompletionRequestMessageContentPartText,
    ChatCompletionRequestUserMessageContent, ChatCompletionRequestUserMessageContentPart, CreateChatCompletionRequestArgs,
    CreateChatCompletionResponse, ImageDetail, ImageUrl,
};
use async_openai::Client;

#[allow(unused)]
use serde::{Deserialize, Serialize};

use std::time::{Duration, SystemTime};

#[derive(Debug)]
pub struct Provider {
    pub base: String,
    pub model: String,
    pub key: String,
}

///
pub struct LLM {
    provider: Provider,
}

pub struct ChatResponse {
    duration: Duration,
    response: CreateChatCompletionResponse,
}

pub struct PromptsBuilder {
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

        self.response.choices.iter().fold(Vec::new(), |mut acc, choice| {
            if let Some(_reason) = choice.finish_reason {}
            acc.push(choice.message.content.clone().unwrap_or_default());
            acc
        })
    }

    pub fn response_string(&self) -> String {
        self.response_vec().join("")
    }

    pub fn response_json<T: for<'a> Deserialize<'a>>(&self) -> Result<T, erx::Erx> {
        let c = self.response_string();
        Self::try_parse_json(&c)
    }

    fn try_parse_json<T: for<'a> Deserialize<'a>>(content: &str) -> Result<T, erx::Erx> {
        let mut c = content.trim();
        if c.is_empty() {
            return Err("invalid content length".into());
        }

        const PREFIX: &str = "```json";
        const SUFFIX: &str = "```";

        if IgnoreCase::Prefix(PREFIX.into()).matches(c) {
            c = &c[PREFIX.len()..];
        }
        if IgnoreCase::Suffix(c.into()).matches(c) {
            c = &c[..c.len() - SUFFIX.len()];
        }

        serde_json::from_str::<T>(c).map_err(erx::smp)
    }
}

impl PromptsBuilder {
    pub fn default() -> Self {
        Self { messages: Vec::new() }
    }

    pub fn add(&mut self, message: ChatCompletionRequestMessage) -> &mut Self {
        self.messages.push(message);
        self
    }

    pub fn messages(&self) -> Vec<ChatCompletionRequestMessage> {
        self.messages.clone()
    }

    pub fn user(&mut self, message: &str) -> &mut Self {
        use async_openai::types::ChatCompletionRequestUserMessageArgs;

        self.messages.push(ChatCompletionRequestUserMessageArgs::default().content(message).build().unwrap().into());
        self
    }

    pub fn system(&mut self, message: &str) -> &mut Self {
        use async_openai::types::ChatCompletionRequestSystemMessageArgs;
        self.messages.push(ChatCompletionRequestSystemMessageArgs::default().content(message).build().unwrap().into());
        self
    }

    pub fn assistant(&mut self, message: &str) -> &mut Self {
        use async_openai::types::ChatCompletionRequestAssistantMessageArgs;
        self.messages.push(ChatCompletionRequestAssistantMessageArgs::default().content(message).build().unwrap().into());
        self
    }

    pub fn developer(&mut self, message: &str) -> &mut Self {
        use async_openai::types::ChatCompletionRequestDeveloperMessageArgs;
        self.messages.push(ChatCompletionRequestDeveloperMessageArgs::default().content(message).build().unwrap().into());
        self
    }

    pub fn function(&mut self, message: &str) -> &mut Self {
        use async_openai::types::ChatCompletionRequestFunctionMessageArgs;
        self.messages.push(ChatCompletionRequestFunctionMessageArgs::default().content(message).build().unwrap().into());
        self
    }

    pub fn tool(&mut self, message: &str) -> &mut Self {
        use async_openai::types::ChatCompletionRequestToolMessageArgs;
        self.messages.push(ChatCompletionRequestToolMessageArgs::default().content(message).build().unwrap().into());
        self
    }

    pub fn image(&mut self, message: &str, url: &str) -> &mut Self {
        // use async_openai::types::ChatCompletionRequestMessageContentPartImageArgs;

        let m = ChatCompletionRequestUserMessageContent::Array(vec![
            ChatCompletionRequestUserMessageContentPart::Text(ChatCompletionRequestMessageContentPartText { text: message.to_string() }),
            ChatCompletionRequestUserMessageContentPart::ImageUrl(ChatCompletionRequestMessageContentPartImage {
                image_url: ImageUrl { url: url.to_string(), detail: Some(ImageDetail::Auto) },
            }),
        ]);

        self.messages.push(async_openai::types::ChatCompletionRequestUserMessage::from(m).into());

        self
    }
}

impl Into<Vec<ChatCompletionRequestMessage>> for PromptsBuilder {
    fn into(self) -> Vec<ChatCompletionRequestMessage> {
        self.messages()
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

    pub async fn chat(&self, prompt: Vec<ChatCompletionRequestMessage>) -> Result<ChatResponse, erx::Erx> {
        let request = CreateChatCompletionRequestArgs::default()
            .stream(false)
            .model(self.provider.model.clone())
            .response_format(async_openai::types::ResponseFormat::Text)
            .messages(prompt)
            .build()
            .map_err(erx::smp)?;

        let start = SystemTime::now();
        let response = self.cli().chat().create(request).await.map_err(erx::smp)?;
        let duration = SystemTime::now().duration_since(start).unwrap_or_default();
        Ok(ChatResponse::new(duration, response))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vol_deepseek_v3() -> Provider {
        Provider {
            base: "https://ark.cn-beijing.volces.com/api/v3".to_string(),
            model: "ep-20250213220927-z6vhn".to_string(),
            key: "216bf172-5bda-4479-93f5-04bf683c87dd".to_string(),
        }
    }

    fn vol_doubao_vision_15_pro() -> Provider {
        Provider {
            base: "https://ark.cn-beijing.volces.com/api/v3".to_string(),
            model: "doubao-1-5-vision-pro-32k-250115".to_string(),
            key: "216bf172-5bda-4479-93f5-04bf683c87dd".to_string(),
        }
    }

    #[tokio::test]
    async fn test_chat_single() {
        let mut b = PromptsBuilder::default();
        b.user("孟加拉国和印度有世仇？主要争端是什么？用中文和英文分别回答。");

        let r = LLM::with_provider(vol_deepseek_v3()).chat(b.into()).await.unwrap();
        println!("{}", r.response_string());
    }

    #[tokio::test]
    async fn test_chat_single_with_vision() {
        let mut b = PromptsBuilder::default();
        let prompt = "推理这个图片是一个机构的有效证件不？ 如果是，JSON输出以下字段：\
    有效日期开始时间(date_start)，\
    有效日期结束时间(date_end)，\
    代码(code)，\
    代码类型(code_type)，\
    证书名称(license_name)，\
    证书类型(license_type)，\
    组织名称(org_name)，\
    组织类型(org_type).  时间字段格式化为2025-04-09这种格式";
        let url = "https://horizonpublicstorage.bangbangwang.cn/horizon/img/202311/11/mKhA3fBfpsbsaCNWSZF4PdONi8bdNrNehYGa77d2.jpg";
        b.image(prompt, url);

        let r = LLM::with_provider(vol_doubao_vision_15_pro()).chat(b.into()).await.unwrap();
        println!("{}", r.response_string());
    }
}
