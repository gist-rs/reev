//! Data models for ZAI SDK

use serde::{Deserialize, Serialize};

/// GLM model variants supported by ZAI
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GlmVariant {
    /// Standard GLM-4.6 variant using the general API endpoint
    Standard,
    /// GLM-4.6-coding variant using the coding-specific API endpoint
    Coding,
}

impl GlmVariant {
    /// Get the API endpoint for this variant
    pub fn endpoint(&self) -> &'static str {
        match self {
            GlmVariant::Standard => "https://api.z.ai/api/paas/v4",
            GlmVariant::Coding => "https://api.z.ai/api/coding/paas/v4",
        }
    }

    /// Get the model name for API requests
    pub fn api_model_name(&self) -> &'static str {
        // Both variants use "glm-4.6" at their respective endpoints
        "glm-4.6"
    }

    /// Get the display name for the variant
    pub fn display_name(&self) -> &'static str {
        match self {
            GlmVariant::Standard => "glm-4.6",
            GlmVariant::Coding => "glm-4.6-coding",
        }
    }

    /// Get the default timeout for this variant (in seconds)
    pub fn default_timeout_secs(&self) -> u64 {
        match self {
            GlmVariant::Standard => 30,
            GlmVariant::Coding => 60, // Coding tasks may take longer
        }
    }
}

impl std::fmt::Display for GlmVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl From<&str> for GlmVariant {
    fn from(s: &str) -> Self {
        match s {
            "glm-4.6" => GlmVariant::Standard,
            "glm-4.6-coding" => GlmVariant::Coding,
            _ => GlmVariant::Standard, // Default to Standard for unknown variants
        }
    }
}

/// Message role in chat completion
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

/// Chat message in completion request
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    /// The role of the message author
    pub role: MessageRole,
    /// The content of the message
    pub content: String,
    /// Optional name for the message author
    pub name: Option<String>,
}

impl Message {
    /// Create a new system message
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::System,
            content: content.into(),
            name: None,
        }
    }

    /// Create a new user message
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: content.into(),
            name: None,
        }
    }

    /// Create a new assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: content.into(),
            name: None,
        }
    }

    /// Create a new tool message
    pub fn tool(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Tool,
            content: content.into(),
            name: None,
        }
    }

    /// Set the name of the message author
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

/// Tool definition for function calling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// The type of the tool (always "function")
    #[serde(rename = "type")]
    pub tool_type: String,
    /// The function definition
    pub function: FunctionDefinition,
}

impl ToolDefinition {
    /// Create a new tool definition
    pub fn function(
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> FunctionDefinitionBuilder {
        FunctionDefinitionBuilder::new(name, description)
    }
}

/// Function definition for tool calling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    /// The name of the function
    pub name: String,
    /// The description of the function
    pub description: String,
    /// The parameters schema
    pub parameters: serde_json::Value,
}

/// Builder for creating function definitions
pub struct FunctionDefinitionBuilder {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

impl FunctionDefinitionBuilder {
    /// Create a new function definition builder
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters: serde_json::json!({"type": "object", "properties": {}}),
        }
    }

    /// Add a parameter to the function
    pub fn parameter(
        mut self,
        name: impl Into<String>,
        param_type: impl Into<String>,
        description: impl Into<String>,
        required: bool,
    ) -> Self {
        let param_name = name.into();
        let properties = self
            .parameters
            .get_mut("properties")
            .and_then(|p| p.as_object_mut())
            .unwrap();

        properties.insert(
            param_name.clone(),
            serde_json::json!({
                "type": param_type.into(),
                "description": description.into()
            }),
        );

        if required {
            // Check if required array exists, create it if needed
            if self.parameters.get("required").is_none() {
                self.parameters["required"] = serde_json::json!([]);
            }

            // Get mutable reference to the required array
            let required_params = self
                .parameters
                .get_mut("required")
                .and_then(|r| r.as_array_mut())
                .unwrap();

            required_params.push(serde_json::Value::String(param_name));
        }

        self
    }

    /// Build the function definition
    pub fn build(self) -> FunctionDefinition {
        FunctionDefinition {
            name: self.name,
            description: self.description,
            parameters: self.parameters,
        }
    }
}

impl From<FunctionDefinition> for ToolDefinition {
    fn from(function: FunctionDefinition) -> Self {
        Self {
            tool_type: "function".to_string(),
            function,
        }
    }
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Number of tokens in the prompt
    pub prompt_tokens: u32,
    /// Number of tokens in the completion
    pub completion_tokens: u32,
    /// Total number of tokens used
    pub total_tokens: u32,
}

/// Choice in a completion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    /// The index of the choice
    pub index: u32,
    /// The message content
    pub message: Message,
    /// The reason the completion finished
    pub finish_reason: Option<String>,
}

/// Completion request
#[derive(Debug, Clone)]
pub struct CompletionRequest {
    /// The model to use
    pub model: String,
    /// The messages to send
    pub messages: Vec<Message>,
    /// The tools to use
    pub tools: Vec<ToolDefinition>,
    /// The temperature for sampling (0.0 to 2.0)
    pub temperature: Option<f32>,
    /// The maximum number of tokens to generate
    pub max_tokens: Option<u32>,
    /// Whether to stream the response
    pub stream: bool,
}

impl CompletionRequest {
    /// Create a new completion request
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            messages: Vec::new(),
            tools: Vec::new(),
            temperature: None,
            max_tokens: None,
            stream: false,
        }
    }

    /// Add a system message
    pub fn system(mut self, content: impl Into<String>) -> Self {
        self.messages.push(Message::system(content));
        self
    }

    /// Add a user message
    pub fn user(mut self, content: impl Into<String>) -> Self {
        self.messages.push(Message::user(content));
        self
    }

    /// Add an assistant message
    pub fn assistant(mut self, content: impl Into<String>) -> Self {
        self.messages.push(Message::assistant(content));
        self
    }

    /// Add a tool message
    /// Add a tool message
    pub fn tool_message(mut self, content: impl Into<String>) -> Self {
        self.messages.push(Message::tool(content));
        self
    }

    /// Add a message
    pub fn message(mut self, message: Message) -> Self {
        self.messages.push(message);
        self
    }

    /// Add messages
    pub fn messages(mut self, messages: impl IntoIterator<Item = Message>) -> Self {
        self.messages.extend(messages);
        self
    }

    /// Add a tool
    pub fn tool(mut self, tool: ToolDefinition) -> Self {
        self.tools.push(tool);
        self
    }

    /// Add tools
    pub fn tools(mut self, tools: impl IntoIterator<Item = ToolDefinition>) -> Self {
        self.tools.extend(tools);
        self
    }

    /// Set the temperature
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set the maximum number of tokens
    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Enable streaming
    pub fn stream(mut self, stream: bool) -> Self {
        self.stream = stream;
        self
    }
}

/// Completion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    /// The ID of completion
    pub id: String,
    /// The object type (always "chat.completion")
    #[serde(default = "default_object")]
    pub object: String,
    /// The creation timestamp
    pub created: u64,
    /// The model used
    pub model: String,
    /// The choices in response
    pub choices: Vec<Choice>,
    /// Token usage information
    pub usage: Option<TokenUsage>,
}

/// Default value for object field
fn default_object() -> String {
    "chat.completion".to_string()
}

impl CompletionResponse {
    /// Get the first choice's message content
    pub fn content(&self) -> Option<&str> {
        self.choices
            .first()
            .map(|choice| &choice.message.content)
            .map(|s| s.as_str())
    }

    /// Get all choices
    pub fn get_choices(&self) -> &[Choice] {
        &self.choices
    }

    /// Get the token usage
    pub fn usage(&self) -> Option<&TokenUsage> {
        self.usage.as_ref()
    }
}
