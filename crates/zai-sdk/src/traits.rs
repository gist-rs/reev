//! Generic traits for ZAI SDK request and response handling

use async_trait::async_trait;
use std::pin::Pin;

/// Trait for handling ZAI requests with generic request and response types
#[async_trait]
pub trait ZaiRequestHandler<T, U> {
    /// Send a request to the ZAI API and return a response
    async fn send_request(&self, request: T) -> crate::error::ZaiResult<U>;
}

/// Trait for handling ZAI responses with generic response types
#[async_trait]
pub trait ZaiResponseHandler<T> {
    /// Process a response from the ZAI API
    fn process_response(&self, response: serde_json::Value) -> crate::error::ZaiResult<T>;

    /// Validate that the response contains expected fields
    fn validate_response(&self, response: &serde_json::Value) -> crate::error::ZaiResult<()>;
}

/// Trait for streaming responses from the ZAI API
#[async_trait]
pub trait ZaiStreamHandler<T> {
    /// Stream a response from the ZAI API
    async fn stream_response(
        &self,
        request: T,
    ) -> crate::error::ZaiResult<
        Pin<Box<dyn futures::Stream<Item = crate::error::ZaiResult<String>> + Send>>,
    >;
}

/// Trait for converting between different request/response types
pub trait ZaiConverter<T, U> {
    /// Convert from type T to type U
    fn convert_from(input: T) -> crate::error::ZaiResult<U>;

    /// Convert from type U to type T
    fn convert_to(input: U) -> crate::error::ZaiResult<T>;
}

/// Trait for building ZAI requests with a fluent API
pub trait ZaiRequestBuilder<T> {
    /// The final request type
    type Output;

    /// Build the final request
    fn build(self) -> crate::error::ZaiResult<Self::Output>;

    /// Add a parameter to the request
    fn with_parameter(self, key: &str, value: serde_json::Value) -> Self;

    /// Add multiple parameters to the request
    fn with_parameters<I>(self, parameters: I) -> Self
    where
        I: IntoIterator<Item = (String, serde_json::Value)>;
}

/// Trait for tool calling capabilities
#[async_trait]
pub trait ZaiToolHandler {
    /// Execute a tool call
    async fn execute_tool(
        &self,
        tool_name: &str,
        parameters: serde_json::Value,
    ) -> crate::error::ZaiResult<serde_json::Value>;

    /// Get available tools
    fn get_available_tools(&self) -> Vec<crate::models::ToolDefinition>;

    /// Validate tool parameters
    fn validate_tool_parameters(
        &self,
        tool_name: &str,
        parameters: &serde_json::Value,
    ) -> crate::error::ZaiResult<()>;
}

/// Trait for model capabilities
pub trait ZaiModelCapabilities {
    /// Check if the model supports function calling
    fn supports_function_calling(&self) -> bool;

    /// Check if the model supports streaming
    fn supports_streaming(&self) -> bool;

    /// Get the maximum context length for the model
    fn max_context_length(&self) -> usize;

    /// Get the maximum output tokens for the model
    fn max_output_tokens(&self) -> usize;
}

/// Trait for authenticating with the ZAI API
#[async_trait]
pub trait ZaiAuthenticator {
    /// Authenticate with the API
    async fn authenticate(&self) -> crate::error::ZaiResult<String>;

    /// Check if the current authentication is still valid
    async fn is_authenticated(&self) -> crate::error::ZaiResult<bool>;

    /// Refresh the authentication token if needed
    async fn refresh_auth(&self) -> crate::error::ZaiResult<()>;
}

/// Trait for retrying failed requests
#[async_trait]
pub trait ZaiRetryPolicy {
    /// Determine if a request should be retried
    fn should_retry(&self, error: &crate::error::ZaiError, attempt: u32) -> bool;

    /// Get the delay before the next retry
    fn retry_delay(&self, attempt: u32) -> std::time::Duration;

    /// Get the maximum number of retry attempts
    fn max_retries(&self) -> u32;
}

/// Trait for rate limiting
#[async_trait]
pub trait ZaiRateLimiter {
    /// Check if a request is allowed
    async fn is_request_allowed(&self) -> crate::error::ZaiResult<bool>;

    /// Wait until a request is allowed
    async fn wait_until_allowed(&self) -> crate::error::ZaiResult<()>;

    /// Get the remaining requests in the current window
    async fn remaining_requests(&self) -> crate::error::ZaiResult<u32>;
}

/// Trait for telemetry and monitoring
pub trait ZaiTelemetry {
    /// Record a request start
    fn record_request_start(&self, model: &str, request_type: &str);

    /// Record a request completion
    fn record_request_completion(
        &self,
        model: &str,
        request_type: &str,
        duration: std::time::Duration,
        success: bool,
        tokens_used: Option<u32>,
    );

    /// Record an error
    fn record_error(&self, error: &crate::error::ZaiError);
}

/// Default implementation of retry policy
#[derive(Debug, Clone)]
pub struct DefaultRetryPolicy {
    max_retries: u32,
    base_delay: std::time::Duration,
    max_delay: std::time::Duration,
    backoff_multiplier: f64,
}

impl DefaultRetryPolicy {
    /// Create a new default retry policy
    pub fn new() -> Self {
        Self {
            max_retries: 3,
            base_delay: std::time::Duration::from_millis(100),
            max_delay: std::time::Duration::from_secs(30),
            backoff_multiplier: 2.0,
        }
    }

    /// Set the maximum number of retries
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Set the base delay
    pub fn with_base_delay(mut self, base_delay: std::time::Duration) -> Self {
        self.base_delay = base_delay;
        self
    }

    /// Set the maximum delay
    pub fn with_max_delay(mut self, max_delay: std::time::Duration) -> Self {
        self.max_delay = max_delay;
        self
    }

    /// Set the backoff multiplier
    pub fn with_backoff_multiplier(mut self, backoff_multiplier: f64) -> Self {
        self.backoff_multiplier = backoff_multiplier;
        self
    }
}

impl Default for DefaultRetryPolicy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ZaiRetryPolicy for DefaultRetryPolicy {
    fn should_retry(&self, error: &crate::error::ZaiError, attempt: u32) -> bool {
        if attempt >= self.max_retries {
            return false;
        }

        matches!(
            error,
            crate::error::ZaiError::Network { .. }
                | crate::error::ZaiError::Timeout { .. }
                | crate::error::ZaiError::RateLimit { .. }
                | crate::error::ZaiError::ApiRequest { .. }
        )
    }

    fn retry_delay(&self, attempt: u32) -> std::time::Duration {
        let multiplier = self.backoff_multiplier.powi(attempt as i32);
        // Convert duration to milliseconds, apply multiplier, then convert back
        let base_ms = self.base_delay.as_millis() as u64;
        let multiplied_ms = (base_ms as f64 * multiplier) as u64;
        let delay = std::time::Duration::from_millis(multiplied_ms);
        std::cmp::min(delay, self.max_delay)
    }

    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}
