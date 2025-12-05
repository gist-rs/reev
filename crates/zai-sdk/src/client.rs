//! ZAI client implementation with builder pattern and generic types

use crate::error::{ZaiError, ZaiResult};
use crate::models::{CompletionRequest, CompletionResponse, GlmVariant, Message};
use crate::streaming::StreamingResponse;
use crate::traits::{
    DefaultRetryPolicy, ZaiAuthenticator, ZaiModelCapabilities, ZaiRequestHandler, ZaiRetryPolicy,
    ZaiStreamHandler, ZaiToolHandler,
};
use async_trait::async_trait;

use reqwest::Client as HttpClient;
use serde_json::json;

use std::time::Duration;
use tracing::{debug, info, trace, warn};

// Default API endpoints for different variants
const DEFAULT_STANDARD_API_URL: &str = "https://api.z.ai/api/paas/v4";
const DEFAULT_CODING_API_URL: &str = "https://api.z.ai/api/coding/paas/v4";

/// Builder for creating a ZAI client
pub struct ZaiClientBuilder<'a> {
    api_key: Option<String>,
    api_url: Option<&'a str>,
    variant: Option<GlmVariant>,
    timeout: Option<Duration>,
    http_client: Option<HttpClient>,
    retry_policy: Option<Box<dyn ZaiRetryPolicy + Send + Sync>>,
}

impl<'a> Default for ZaiClientBuilder<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> ZaiClientBuilder<'a> {
    /// Create a new ZAI client builder
    pub fn new() -> Self {
        Self {
            api_key: None,
            api_url: None,
            variant: None,
            timeout: None,
            http_client: None,
            retry_policy: None,
        }
    }

    /// Set the API key
    pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Set the API URL for the ZAI service
    /// This overrides the default URL for the selected variant
    pub fn api_url(mut self, api_url: &'a str) -> Self {
        self.api_url = Some(api_url);
        self
    }

    /// Set the GLM model variant
    /// This determines the default API URL if api_url is not specified
    pub fn variant(mut self, variant: GlmVariant) -> Self {
        self.variant = Some(variant);
        self
    }

    /// Set the timeout for API requests
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set a custom HTTP client
    pub fn http_client(mut self, client: HttpClient) -> Self {
        self.http_client = Some(client);
        self
    }

    /// Set a custom retry policy
    pub fn retry_policy(mut self, policy: Box<dyn ZaiRetryPolicy + Send + Sync>) -> Self {
        self.retry_policy = Some(policy);
        self
    }

    /// Build the ZAI client
    pub fn build(self) -> ZaiResult<ZaiClient> {
        // Get API key from parameter or environment variable
        let api_key = self
            .api_key
            .or_else(|| std::env::var("ZAI_API_KEY").ok())
            .ok_or_else(|| ZaiError::configuration("API key is required"))?;

        // Set default variant if not specified
        let variant = self.variant.unwrap_or(GlmVariant::Standard);

        // Determine API URL
        let base_url = if let Some(url) = self.api_url {
            url.to_string()
        } else {
            // Use default URL based on variant
            match variant {
                GlmVariant::Standard => DEFAULT_STANDARD_API_URL.to_string(),
                GlmVariant::Coding => DEFAULT_CODING_API_URL.to_string(),
            }
        };

        // Set timeout
        let timeout = self
            .timeout
            .unwrap_or_else(|| Duration::from_secs(variant.default_timeout_secs()));

        // Create HTTP client
        let http_client = self.http_client.unwrap_or_else(|| {
            HttpClient::builder()
                .timeout(timeout)
                .build()
                .map_err(|e| ZaiError::configuration(format!("Failed to create HTTP client: {e}")))
                .unwrap_or_default()
        });

        // Set retry policy
        let retry_policy = self
            .retry_policy
            .unwrap_or_else(|| Box::new(DefaultRetryPolicy::default()));

        Ok(ZaiClient {
            api_key: api_key.to_string(),
            base_url,
            variant,
            timeout,
            http_client,
            retry_policy,
        })
    }
}

/// ZAI client for interacting with GLM-4.6 models
pub struct ZaiClient {
    api_key: String,
    base_url: String,
    variant: GlmVariant,
    timeout: Duration,
    http_client: HttpClient,
    retry_policy: Box<dyn ZaiRetryPolicy + Send + Sync>,
}

impl std::fmt::Debug for ZaiClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ZaiClient")
            .field("base_url", &self.base_url)
            .field("variant", &self.variant)
            .field("timeout", &self.timeout)
            .field("api_key", &"***") // Don't log the actual API key
            .finish()
    }
}

impl ZaiClient {
    /// Create a new client builder
    pub fn builder() -> ZaiClientBuilder<'static> {
        ZaiClientBuilder::new()
    }

    /// Create a client from environment variables
    pub fn from_env() -> ZaiResult<Self> {
        Self::builder().build()
    }

    /// Get the model variant
    pub fn variant(&self) -> &GlmVariant {
        &self.variant
    }

    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Send a completion request
    pub async fn completion(&self, prompt: &str) -> ZaiResult<String> {
        let request = CompletionRequest::new(self.variant.api_model_name()).user(prompt);

        let response = self.send_request(request).await?;
        response
            .content()
            .ok_or_else(|| ZaiError::invalid_response("No content in response"))
            .map(|s| s.to_string())
    }

    /// Send a completion request with messages
    pub async fn completion_with_messages(&self, messages: Vec<Message>) -> ZaiResult<String> {
        let request = CompletionRequest::new(self.variant.api_model_name()).messages(messages);

        let response = self.send_request(request).await?;
        response
            .content()
            .ok_or_else(|| ZaiError::invalid_response("No content in response"))
            .map(|s| s.to_string())
    }

    /// Send a custom completion request
    pub async fn send_completion_request(
        &self,
        request: CompletionRequest,
    ) -> ZaiResult<CompletionResponse> {
        self.send_request(request).await
    }

    /// Stream a completion response
    pub async fn stream_completion(&self, prompt: &str) -> ZaiResult<StreamingResponse> {
        let request = CompletionRequest::new(self.variant.api_model_name()).user(prompt);

        self.stream_response(request).await
    }

    /// Send a request to the ZAI API with retry logic
    async fn send_with_retry<T, F, Fut>(&self, operation: F) -> ZaiResult<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = ZaiResult<T>>,
    {
        let mut last_error = None;
        let max_retries = self.retry_policy.max_retries();

        for attempt in 0..=max_retries {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    warn!("Request attempt {} failed: {}", attempt, error);

                    if !self.retry_policy.should_retry(&error, attempt) {
                        break;
                    }

                    // Store the error message as a string since ZaiError doesn't implement Clone
                    last_error = Some(ZaiError::other(error.to_string()));
                    let delay = self.retry_policy.retry_delay(attempt);
                    debug!("Retrying in {:?}", delay);
                    tokio::time::sleep(delay).await;
                }
            }
        }

        Err(last_error.unwrap_or_else(|| ZaiError::other("Request failed")))
    }

    /// Get the endpoint URL for a specific path
    fn endpoint_url(&self, path: &str) -> String {
        if path.is_empty() {
            self.base_url.clone()
        } else {
            format!(
                "{}/{}",
                self.base_url.trim_end_matches('/'),
                path.trim_start_matches('/')
            )
        }
    }

    /// Execute an HTTP POST request
    async fn post<T: serde::Serialize, R: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        request: &T,
    ) -> ZaiResult<R> {
        let url = self.endpoint_url(path);
        trace!("Sending POST request to: {}", url);

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(request)
            .send()
            .await
            .map_err(|e| ZaiError::api_request(format!("Failed to send request: {e}")))?;

        let status = response.status();
        if !status.is_success() {
            // Clone headers before consuming response
            let headers = response.headers().clone();
            let status_code = status.as_u16();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error response".to_string());

            return match status_code {
                401 | 403 => Err(ZaiError::authentication(format!(
                    "Authentication failed: {error_text}"
                ))),
                404 => Err(ZaiError::model_unavailable(format!(
                    "Model not found: {error_text}"
                ))),
                429 => {
                    let retry_after = headers
                        .get("Retry-After")
                        .and_then(|v| v.to_str().ok())
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(60);

                    Err(ZaiError::rate_limit(retry_after))
                }
                _ => Err(ZaiError::api_request(format!(
                    "API error {status_code}: {error_text}"
                ))),
            };
        }

        response
            .json()
            .await
            .map_err(|e| ZaiError::invalid_response(format!("Failed to parse response: {e}")))
    }

    /// Verify that the model is available and accessible
    pub async fn verify_model(&self, model_name: Option<&str>) -> ZaiResult<()> {
        let model = model_name.unwrap_or(self.variant.api_model_name());
        info!("Verifying model availability: {}", model);

        let test_request = json!({
            "model": model,
            "messages": [{"role": "user", "content": "test"}],
            "max_tokens": 1
        });

        let _: serde_json::Value = self.post("chat/completions", &test_request).await?;
        info!("Model verification successful: {}", model);
        Ok(())
    }
}

#[async_trait]
impl ZaiRequestHandler<CompletionRequest, CompletionResponse> for ZaiClient {
    async fn send_request(&self, request: CompletionRequest) -> ZaiResult<CompletionResponse> {
        self.send_with_retry(|| async {
            debug!("Sending completion request to model: {}", request.model);

            // Build the request body for ZAI API
            let mut request_body = json!({
                "model": request.model,
                "messages": request.messages,
                "stream": request.stream,
            });

            // Add optional parameters if provided
            if let Some(temp) = request.temperature {
                request_body["temperature"] = json!(temp);
            }

            if let Some(max_tokens) = request.max_tokens {
                request_body["max_tokens"] = json!(max_tokens);
            }

            // Add tools if provided
            if !request.tools.is_empty() {
                request_body["tools"] = json!(request.tools);
            }

            // Send the request
            let response: serde_json::Value = self.post("chat/completions", &request_body).await?;

            // Convert the response to our CompletionResponse type
            let completion_response: CompletionResponse = serde_json::from_value(response)
                .map_err(|e| ZaiError::invalid_response(format!("Invalid response format: {e}")))?;

            debug!("Received completion response");
            Ok(completion_response)
        })
        .await
    }
}

#[async_trait]
impl ZaiStreamHandler<CompletionRequest> for ZaiClient {
    async fn stream_response(&self, request: CompletionRequest) -> ZaiResult<StreamingResponse> {
        // For now, implement a simplified version that returns a non-streaming response
        // as a single chunk in a stream
        let mut non_streaming_request = request;
        non_streaming_request.stream = false;

        let response = self.send_request(non_streaming_request).await?;
        let content = response.content().unwrap_or("").to_string();

        // Create a simple stream with just one chunk
        let stream = futures::stream::iter(vec![Ok(content)]);

        Ok(Box::pin(stream))
    }
}

impl ZaiClient {
    /// Send a generic request to the API
    /// This method allows for arbitrary request and response types
    pub async fn send_generic_request<T, U>(&self, endpoint: &str, request: &T) -> ZaiResult<U>
    where
        T: serde::Serialize,
        U: serde::de::DeserializeOwned,
    {
        let url = self.endpoint_url(endpoint);
        trace!("Sending generic POST request to: {}", url);

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(request)
            .send()
            .await
            .map_err(|e| ZaiError::api_request(format!("Failed to send request: {e}")))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error response".to_string());

            return match status.as_u16() {
                401 | 403 => Err(ZaiError::authentication(format!(
                    "Authentication failed: {error_text}"
                ))),
                404 => Err(ZaiError::model_unavailable(format!(
                    "Endpoint not found: {error_text}"
                ))),
                429 => Err(ZaiError::rate_limit(60)),
                _ => Err(ZaiError::api_request(format!(
                    "API error {status}: {error_text}"
                ))),
            };
        }

        response
            .json()
            .await
            .map_err(|e| ZaiError::invalid_response(format!("Failed to parse response: {e}")))
    }

    /// Send a typed request to the chat completions endpoint
    /// This method allows for custom request types while still using the chat completions endpoint
    pub async fn send_typed_chat_request<T, U>(&self, request: &T) -> ZaiResult<U>
    where
        T: serde::Serialize,
        U: serde::de::DeserializeOwned,
    {
        self.send_generic_request("chat/completions", request).await
    }

    /// Send a raw JSON request to the chat completions endpoint
    /// This method provides maximum flexibility for custom request formats
    pub async fn send_raw_chat_request<U>(&self, request: &serde_json::Value) -> ZaiResult<U>
    where
        U: serde::de::DeserializeOwned,
    {
        self.send_generic_request("chat/completions", request).await
    }
}

#[async_trait]
impl ZaiAuthenticator for ZaiClient {
    async fn authenticate(&self) -> ZaiResult<String> {
        // For ZAI, authentication is simply verifying the API key works
        self.verify_model(None).await?;
        Ok(self.api_key.clone())
    }

    async fn is_authenticated(&self) -> ZaiResult<bool> {
        match self.verify_model(None).await {
            Ok(_) => Ok(true),
            Err(ZaiError::Authentication { .. }) => Ok(false),
            Err(_) => Ok(true), // Other errors don't indicate authentication issues
        }
    }

    async fn refresh_auth(&self) -> ZaiResult<()> {
        // ZAI uses simple token auth, no refresh needed
        Ok(())
    }
}

impl ZaiModelCapabilities for ZaiClient {
    fn supports_function_calling(&self) -> bool {
        matches!(self.variant, GlmVariant::Standard)
    }

    fn supports_streaming(&self) -> bool {
        true // Both variants support streaming
    }

    fn max_context_length(&self) -> usize {
        match self.variant {
            GlmVariant::Standard => 200000,
            GlmVariant::Coding => 200000, // Same context length for both
        }
    }

    fn max_output_tokens(&self) -> usize {
        match self.variant {
            GlmVariant::Standard => 32000,
            GlmVariant::Coding => 32000, // Same output tokens for both
        }
    }
}

#[async_trait]
impl ZaiToolHandler for ZaiClient {
    async fn execute_tool(
        &self,
        tool_name: &str,
        parameters: serde_json::Value,
    ) -> ZaiResult<serde_json::Value> {
        // This would be implemented by the consumer of the SDK
        // We provide a placeholder implementation that returns an error
        Err(ZaiError::invalid_parameters(format!(
            "Tool execution not implemented in SDK. Tool: {tool_name}, Parameters: {parameters}"
        )))
    }

    fn get_available_tools(&self) -> Vec<crate::models::ToolDefinition> {
        // This would be implemented by consumer of the SDK
        Vec::new()
    }

    fn validate_tool_parameters(
        &self,
        tool_name: &str,
        parameters: &serde_json::Value,
    ) -> ZaiResult<()> {
        // This would be implemented by consumer of the SDK
        Err(ZaiError::invalid_parameters(format!(
            "Tool parameter validation not implemented in SDK. Tool: {tool_name}, Parameters: {parameters}"
        )))
    }
}
