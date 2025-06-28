# src/tools/openai.rs

This file provides utilities for interacting with OpenAI-compatible large language models (LLMs), including chat completions and image-based prompts. It leverages the `async-openai` crate.

## Structs

### `Provider`
Configuration for an LLM provider, including base URL, model name, and API key.

### `LLM`
Represents a Large Language Model client, configured with a `Provider`.

### `ChatResponse`
Encapsulates the response from a chat completion request, including duration and the raw response.

#### Methods
*   `new(duration: Duration, response: CreateChatCompletionResponse)`: Creates a new `ChatResponse` instance.
*   `duration() -> Duration`: Returns the duration of the chat request.
*   `response() -> CreateChatCompletionResponse`: Returns the raw chat completion response.
*   `response_vec() -> Vec<String>`: Extracts content from the response choices into a vector of strings.
*   `response_string() -> String`: Concatenates all response content into a single string.
*   `response_json<T: for<'a> Deserialize<'a>>() -> Result<T, erx::Erx>`: Attempts to parse the response content as JSON.

### `PromptsBuilder`
A builder for constructing chat completion messages.

#### Methods
*   `default() -> Self`: Creates a new `PromptsBuilder` with an empty message list.
*   `add(&mut self, message: ChatCompletionRequestMessage) -> &mut Self`: Adds a generic message to the prompt.
*   `messages() -> Vec<ChatCompletionRequestMessage>`: Returns the built messages.
*   `user(&mut self, message: &str) -> &mut Self`: Adds a user message.
*   `system(&mut self, message: &str) -> &mut Self`: Adds a system message.
*   `assistant(&mut self, message: &str) -> &mut Self`: Adds an assistant message.
*   `developer(&mut self, message: &str) -> &mut Self`: Adds a developer message.
*   `function(&mut self, message: &str) -> &mut Self`: Adds a function message.
*   `tool(&mut self, message: &str) -> &mut Self`: Adds a tool message.
*   `image(&mut self, message: &str, url: &str) -> &mut Self`: Adds a user message with an image URL.

## Implementations

### `impl LLM`
*   `with_provider(provider: Provider) -> LLM`: Creates an `LLM` instance with the given provider.
*   `chat(&self, prompt: Vec<ChatCompletionRequestMessage>) -> Result<ChatResponse, erx::Erx>`: Sends a chat completion request to the LLM.
