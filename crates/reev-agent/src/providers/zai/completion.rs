//! ZAI completion model and related types

use rig::{
    completion::{
        self, AssistantContent, CompletionError, CompletionRequest, Usage as CompletionUsage,
    },
    message::{self, Message, Text, ToolCall as MessageToolCall, ToolFunction},
};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Serialize, Deserialize, Debug)]
pub struct CompletionResponse {
    pub choices: Vec<Choice>,
    pub usage: Option<ZaiUsage>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ZaiUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    #[serde(rename = "prompt_tokens_details")]
    pub prompt_tokens_details: Option<PromptTokensDetails>,
}

#[allow(unused)]
fn completion_usage_from_zai_usage(usage: Option<ZaiUsage>) -> CompletionUsage {
    usage
        .map(|u| CompletionUsage {
            input_tokens: u.prompt_tokens as u64,
            output_tokens: u.completion_tokens as u64,
            total_tokens: u.total_tokens as u64,
        })
        .unwrap_or_else(|| CompletionUsage {
            input_tokens: 0,
            output_tokens: 0,
            total_tokens: 0,
        })
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PromptTokensDetails {
    #[serde(rename = "cached_tokens")]
    pub cached_tokens: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Choice {
    pub index: u32,
    pub message: Option<ZaiMessage>,
    pub logprobs: Option<serde_json::Value>,
    #[serde(rename = "finish_reason")]
    pub finish_reason: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "role")]
pub enum ZaiMessage {
    #[serde(rename = "system")]
    System {
        content: String,
        name: Option<String>,
    },
    #[serde(rename = "user")]
    User {
        content: String,
        name: Option<String>,
    },
    #[serde(rename = "assistant")]
    Assistant {
        content: Option<String>,
        name: Option<String>,
        #[serde(rename = "tool_calls")]
        tool_calls: Option<Vec<MessageToolCall>>,
    },
    #[serde(rename = "tool")]
    ToolResult {
        #[serde(rename = "tool_call_id")]
        tool_call_id: String,
        content: String,
    },
}

impl ZaiMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self::System {
            content: content.into(),
            name: None,
        }
    }
}

impl From<message::ToolResult> for ZaiMessage {
    fn from(value: message::ToolResult) -> Self {
        Self::ToolResult {
            tool_call_id: value.id,
            content: format!("{:?}", value.content),
        }
    }
}

impl From<MessageToolCall> for ZaiToolCall {
    fn from(value: message::ToolCall) -> Self {
        Self {
            id: value.id,
            index: None,
            r#type: ToolType::Function,
            function: Function {
                name: value.function.name,
                arguments: value.function.arguments.to_string(),
            },
        }
    }
}

fn message_to_zai_messages(value: Message) -> Result<Vec<ZaiMessage>, String> {
    match value {
        Message::User { content } => {
            let mut messages = vec![];

            for content in content.into_iter() {
                match content {
                    message::UserContent::Text(text) => {
                        messages.push(ZaiMessage::User {
                            content: text.text,
                            name: None,
                        });
                    }
                    message::UserContent::ToolResult(tool_result) => {
                        messages.push(ZaiMessage::ToolResult {
                            tool_call_id: tool_result.id,
                            content: format!("{:?}", tool_result.content),
                        });
                    }
                    _ => {
                        // Skip other content types for now
                    }
                }
            }

            Ok(messages)
        }
        Message::Assistant { id: _, content } => {
            let mut messages = vec![];

            for content in content.into_iter() {
                match content {
                    message::AssistantContent::Text(text) => {
                        messages.push(ZaiMessage::Assistant {
                            content: Some(text.text),
                            name: None,
                            tool_calls: None,
                        });
                    }
                    message::AssistantContent::ToolCall(tool_call) => {
                        messages.push(ZaiMessage::Assistant {
                            content: None,
                            name: None,
                            tool_calls: Some(vec![MessageToolCall {
                                id: tool_call.id,
                                call_id: None,
                                function: ToolFunction {
                                    name: tool_call.function.name,
                                    arguments: tool_call.function.arguments.clone(),
                                },
                            }]),
                        });
                    }
                    _ => {
                        // Skip other content types for now
                    }
                }
            }

            Ok(messages)
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ZaiToolCall {
    pub id: String,
    pub index: Option<u32>,
    #[serde(rename = "type")]
    pub r#type: ToolType,
    pub function: Function,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Function {
    pub name: String,
    pub arguments: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ToolType {
    Function,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ZaiToolDefinition {
    #[serde(rename = "type")]
    pub r#type: ToolType,
    pub function: serde_json::Value,
}

impl From<rig::completion::ToolDefinition> for ZaiToolDefinition {
    fn from(value: rig::completion::ToolDefinition) -> Self {
        info!("ZAI: Converting tool definition: {}", value.name);
        info!("ZAI: Tool description: {}", value.description);

        // Validate tool definition before conversion
        if value.name.trim().is_empty() {
            error!("ZAI: CRITICAL - Tool has empty name: {:?}", value);
            panic!("Tool name cannot be empty");
        }

        if value.description.trim().is_empty() {
            error!("ZAI: CRITICAL - Tool has empty description: {:?}", value);
            panic!("Tool description cannot be empty");
        }

        let result = Self {
            r#type: ToolType::Function,
            function: serde_json::to_value(value).unwrap(),
        };

        // Verify the conversion worked
        if let Ok(json) = serde_json::to_value(&result) {
            info!(
                "ZAI: Converted tool JSON: {}",
                serde_json::to_string_pretty(&json).unwrap()
            );
        }

        result
    }
}

impl TryFrom<CompletionResponse> for completion::CompletionResponse<CompletionResponse> {
    type Error = CompletionError;

    fn try_from(value: CompletionResponse) -> Result<Self, Self::Error> {
        let choice = value
            .choices
            .first()
            .ok_or_else(|| CompletionError::ProviderError("No choices in response".to_string()))?;

        let message = choice
            .message
            .as_ref()
            .ok_or_else(|| CompletionError::ProviderError("No message in choice".to_string()))?;

        let content = match message {
            ZaiMessage::System { content, .. } => Some(content.clone()),
            ZaiMessage::User { content, .. } => Some(content.clone()),
            ZaiMessage::Assistant { content, .. } => content.clone(),
            ZaiMessage::ToolResult { content, .. } => Some(content.clone()),
        };

        let tool_calls = match message {
            ZaiMessage::Assistant { tool_calls, .. } => tool_calls
                .as_ref()
                .map(|calls| {
                    calls
                        .iter()
                        .map(|call| MessageToolCall {
                            id: call.id.clone(),
                            call_id: None,
                            function: ToolFunction {
                                name: call.function.name.clone(),
                                arguments: serde_json::from_str(
                                    call.function.arguments.as_str().unwrap_or_default(),
                                )
                                .unwrap_or_default(),
                            },
                        })
                        .map(AssistantContent::ToolCall)
                        .collect()
                })
                .unwrap_or_default(),
            _ => Vec::new(),
        };

        let mut content = if let Some(content) = content {
            vec![AssistantContent::Text(Text { text: content })]
        } else {
            Vec::new()
        };

        content.extend(tool_calls);

        let choice = rig::OneOrMany::many(content).map_err(|_| {
            CompletionError::ProviderError(
                "Response contained no message or tool call (empty)".to_string(),
            )
        })?;

        let usage = completion::Usage {
            input_tokens: value
                .usage
                .as_ref()
                .map(|u| u.prompt_tokens as u64)
                .unwrap_or(0),
            output_tokens: value
                .usage
                .as_ref()
                .map(|u| u.completion_tokens as u64)
                .unwrap_or(0),
            total_tokens: value
                .usage
                .as_ref()
                .map(|u| u.total_tokens as u64)
                .unwrap_or(0),
        };

        Ok(completion::CompletionResponse {
            choice,
            usage,
            raw_response: value,
        })
    }
}

#[derive(Clone)]
pub struct CompletionModel {
    pub client: Client,
    pub model: String,
}

impl CompletionModel {
    /// Create a new completion model
    pub fn new(client: Client, model: String) -> Self {
        Self { client, model }
    }
}

impl CompletionModel {
    fn create_completion_request(
        &self,
        completion_request: CompletionRequest,
    ) -> Result<serde_json::Value, CompletionError> {
        let messages: Vec<ZaiMessage> = completion_request
            .chat_history
            .into_iter()
            .try_fold(Vec::new(), |acc, msg| {
                let converted = message_to_zai_messages(msg)?;
                Ok::<Vec<ZaiMessage>, String>(acc.into_iter().chain(converted).collect())
            })
            .map_err(|e| {
                CompletionError::ProviderError(format!("Failed to convert messages: {e}"))
            })?;

        let mut request = serde_json::json!({
            "model": self.model,
            "messages": messages,
        });

        // Add temperature if specified
        if let Some(temp) = completion_request.temperature {
            request["temperature"] = serde_json::Value::Number(
                serde_json::Number::from_f64(temp).unwrap_or_else(|| serde_json::Number::from(0)),
            );
        }

        // Add max_tokens if specified
        if let Some(max_tokens) = completion_request.max_tokens {
            request["max_tokens"] = serde_json::Value::Number(serde_json::Number::from(max_tokens));
        }

        // Add tools if specified
        if !completion_request.tools.is_empty() {
            let tool_definitions: Vec<ZaiToolDefinition> = completion_request
                .tools
                .into_iter()
                .map(Into::into)
                .collect();

            // Debug logging for tools
            info!(
                "ZAI: Creating request with {} tools",
                tool_definitions.len()
            );
            for (i, tool) in tool_definitions.iter().enumerate() {
                let tool_json = serde_json::to_value(tool).unwrap();
                info!(
                    "ZAI: Tool {}: {}",
                    i,
                    serde_json::to_string_pretty(&tool_json).unwrap()
                );

                // Check for empty tool type
                if let Some(tool_type) = tool_json.get("type") {
                    if tool_type.as_str().unwrap_or("").is_empty() {
                        error!("ZAI: CRITICAL - Tool {} has empty type field!", i);
                    }
                } else {
                    error!("ZAI: CRITICAL - Tool {} missing type field!", i);
                }
            }

            request["tools"] = serde_json::Value::Array(
                tool_definitions
                    .into_iter()
                    .map(serde_json::to_value)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(CompletionError::JsonError)?,
            );

            request["tool_choice"] = serde_json::Value::String("auto".to_string());

            // Debug log the final request
            info!(
                "ZAI: Final request: {}",
                serde_json::to_string_pretty(&request).unwrap()
            );
        }

        // Add additional parameters
        if let Some(additional_params) = completion_request.additional_params {
            for (key, value) in additional_params
                .as_object()
                .unwrap_or(&serde_json::Map::new())
            {
                request[key] = value.clone();
            }
        }

        Ok(request)
    }
}

impl completion::CompletionModel for CompletionModel {
    type Response = CompletionResponse;
    type StreamingResponse = rig::providers::openai::StreamingCompletionResponse;

    async fn completion(
        &self,
        completion_request: CompletionRequest,
    ) -> Result<completion::CompletionResponse<Self::Response>, rig::completion::CompletionError>
    {
        let request = self.create_completion_request(completion_request)?;

        let response: CompletionResponse = self.client.post("chat/completions", &request).await?;

        response.try_into()
    }

    async fn stream(
        &self,
        completion_request: CompletionRequest,
    ) -> Result<
        rig::streaming::StreamingCompletionResponse<Self::StreamingResponse>,
        rig::completion::CompletionError,
    > {
        let mut request = self.create_completion_request(completion_request)?;
        request["stream"] = serde_json::Value::Bool(true);

        let builder = self
            .client
            .http_client
            .post(format!("{}/chat/completions", self.client.base_url))
            .bearer_auth(&self.client.api_key)
            .header("Content-Type", "application/json")
            .json(&request);

        rig::providers::openai::send_compatible_streaming_request(builder).await
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct StreamingDelta {
    content: Option<String>,
    tool_calls: Option<Vec<StreamingToolCall>>,
    reasoning_content: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct StreamingToolCall {
    index: u32,
    id: Option<String>,
    #[serde(rename = "type")]
    tool_type: Option<String>,
    function: Option<StreamingFunction>,
}

#[derive(Serialize, Deserialize, Debug)]
struct StreamingFunction {
    name: Option<String>,
    arguments: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct StreamingChoice {
    delta: StreamingDelta,
}

#[derive(Serialize, Deserialize, Debug)]
struct StreamingCompletionChunk {
    choices: Vec<StreamingChoice>,
    usage: Option<ZaiUsage>,
}

// Re-use OpenAI's streaming response structure
pub use rig::providers::openai::StreamingCompletionResponse;
use tracing::{error, info};

use crate::providers::zai::Client;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

impl From<Option<ZaiUsage>> for TokenUsage {
    fn from(_usage: Option<ZaiUsage>) -> Self {
        Self {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        }
    }
}

pub async fn send_compatible_streaming_response<S>(
    mut stream: S,
) -> Result<CompletionUsage, CompletionError>
where
    S: futures::Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Unpin,
{
    use futures::StreamExt;

    let mut final_usage: Option<CompletionUsage> = None;

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(CompletionError::HttpError)?;

        let chunk_str = std::str::from_utf8(&chunk).unwrap_or("");

        for line in chunk_str.lines() {
            let line = line.trim();

            if line.is_empty() || line == "data: [DONE]" {
                continue;
            }

            if let Some(data_str) = line.strip_prefix("data: ") {
                match serde_json::from_str::<StreamingCompletionChunk>(data_str) {
                    Ok(chunk) => {
                        if let Some(zai_usage) = chunk.usage {
                            final_usage = Some(CompletionUsage {
                                input_tokens: zai_usage.prompt_tokens as u64,
                                output_tokens: zai_usage.completion_tokens as u64,
                                total_tokens: zai_usage.total_tokens as u64,
                            });
                        }
                    }
                    Err(_) => {
                        // Failed to parse streaming chunk, continuing...
                    }
                }
            }
        }
    }

    Ok(final_usage.unwrap_or(CompletionUsage {
        input_tokens: 0,
        output_tokens: 0,
        total_tokens: 0,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_deserialize_vec_choice() {
        let data = json!({
            "choices": [
                {
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "Hello!"
                    },
                    "finish_reason": "stop"
                }
            ]
        });

        let _parsed: CompletionResponse = serde_json::from_value(data).unwrap();
    }

    #[test]
    fn test_deserialize_zai_response() {
        let data = json!({
            "choices": [
                {
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": null,
                        "tool_calls": [
                            {
                                "id": "call_123",
                                "type": "function",
                                "function": {
                                    "name": "get_current_time",
                                    "arguments": "{\"timezone\":\"America/New_York\"}"
                                }
                            }
                        ]
                    },
                    "finish_reason": "tool_calls"
                }
            ],
            "usage": {
                "prompt_tokens": 190,
                "completion_tokens": 75,
                "total_tokens": 265
            }
        });

        let _parsed: CompletionResponse = serde_json::from_value(data).unwrap();
    }

    #[test]
    fn test_serialize_deserialize_tool_call_message() {
        let message = ZaiMessage::Assistant {
            content: None,
            name: None,
            tool_calls: Some(vec![MessageToolCall {
                id: "call_123".to_string(),
                call_id: None,
                function: ToolFunction {
                    name: "get_current_time".to_string(),
                    arguments: serde_json::json!({"timezone": "America/New_York"}),
                },
            }]),
        };

        let serialized = serde_json::to_value(&message).unwrap();
        let _deserialized: ZaiMessage = serde_json::from_value(serialized).unwrap();
    }

    #[test]
    fn test_zai_tool_definition_serialization() {
        // Create a mock tool definition to test serialization
        let tool_def = rig::completion::ToolDefinition {
            name: "get_current_time".to_string(),
            description: "Get the current time".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "timezone": {
                        "type": "string",
                        "description": "Timezone to get time for"
                    }
                },
                "required": ["timezone"]
            }),
        };

        let zai_tool_def: ZaiToolDefinition = tool_def.into();
        let serialized = serde_json::to_value(&zai_tool_def).unwrap();

        println!(
            "Serialized ZAI tool definition: {}",
            serde_json::to_string_pretty(&serialized).unwrap()
        );

        // Check that the type field is present and not empty
        assert!(
            serialized.get("type").is_some(),
            "Type field should be present"
        );
        assert!(
            serialized["type"].as_str().is_some(),
            "Type should be a string"
        );
        assert_eq!(
            serialized["type"].as_str().unwrap(),
            "function",
            "Type should be 'function'"
        );

        // Test deserialization back
        let _deserialized: ZaiToolDefinition = serde_json::from_value(serialized).unwrap();
    }
}
