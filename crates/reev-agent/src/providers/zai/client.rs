//! ZAI API client implementation
// serde imports no longer needed after migration to zai-sdk

use super::completion::CompletionModel as ZaiCompletionModel;
use rig::client::{
    impl_conversion_traits, CompletionClient, ProviderValue, VerifyClient, VerifyError,
};
use rig::{completion::CompletionError, prelude::ProviderClient};

// Import zai-sdk
use std::sync::Arc;
use zai_sdk::{GlmVariant, ZaiClient};

const ZAI_API_BASE_URL: &str = "https://api.z.ai/api/coding/paas/v4";

/// ZAI client builder
pub struct ClientBuilder<'a> {
    api_key: &'a str,
    base_url: Option<&'a str>,
    http_client: reqwest::Client,
}

impl<'a> ClientBuilder<'a> {
    /// Create a new ZAI client builder
    pub fn new(api_key: &'a str) -> Self {
        Self {
            api_key,
            base_url: None,
            http_client: reqwest::Client::new(),
        }
    }

    /// Set a custom base URL for the ZAI API
    pub fn base_url(mut self, base_url: &'a str) -> Self {
        self.base_url = Some(base_url);
        self
    }

    /// Set a custom HTTP client
    pub fn custom_client(mut self, client: reqwest::Client) -> Self {
        self.http_client = client;
        self
    }

    /// Build the ZAI client
    pub fn build(self) -> Client {
        Client::new(
            self.base_url.unwrap_or(ZAI_API_BASE_URL),
            self.api_key,
            self.http_client,
        )
    }
}

/// ZAI client
#[derive(Clone)]
pub struct Client {
    pub base_url: String,
    pub api_key: String,
    pub http_client: reqwest::Client,
    // Internal zai-sdk client wrapped in Arc for shared access
    pub(crate) zai_client: Arc<ZaiClient>,
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("base_url", &self.base_url)
            .field("api_key", &"***")
            .finish()
    }
}

impl Client {
    /// Create a new ZAI client
    pub fn new(base_url: impl Into<String>, api_key: &str, http_client: reqwest::Client) -> Self {
        let base_url_str = base_url.into();

        // Create zai-sdk client
        let variant = if base_url_str.contains("coding") {
            GlmVariant::Coding
        } else {
            GlmVariant::Standard
        };

        let zai_client = Arc::new(
            ZaiClient::builder()
                .variant(variant)
                .api_key(api_key)
                .build()
                .expect("Failed to create ZAI client"),
        );

        Self {
            base_url: base_url_str,
            api_key: api_key.to_string(),
            http_client,
            zai_client,
        }
    }

    /// Create a new ZAI client builder
    pub fn builder(api_key: &str) -> ClientBuilder<'_> {
        ClientBuilder::new(api_key)
    }

    /// Send a completion request using zai-sdk
    pub async fn completion_with_zai_sdk(&self, prompt: &str) -> Result<String, CompletionError> {
        self.zai_client
            .completion(prompt)
            .await
            .map_err(|e| CompletionError::ProviderError(e.to_string()))
    }
}

impl ProviderClient for Client {
    fn from_env() -> Self {
        let api_key =
            std::env::var("ZAI_API_KEY").expect("ZAI_API_KEY environment variable not set");
        Self::new(ZAI_API_BASE_URL, &api_key, reqwest::Client::new())
    }

    fn from_val(input: ProviderValue) -> Self {
        let ProviderValue::Simple(api_key) = input else {
            panic!("Incorrect provider value type")
        };
        Self::new(ZAI_API_BASE_URL, &api_key, reqwest::Client::new())
    }
}

impl CompletionClient for Client {
    type CompletionModel = ZaiCompletionModel;

    fn completion_model(&self, model: &str) -> Self::CompletionModel {
        ZaiCompletionModel::new(self.clone(), model.to_string())
    }
}

impl VerifyClient for Client {
    async fn verify(&self) -> Result<(), VerifyError> {
        // ZAI doesn't have a dedicated models endpoint, so we'll try a minimal completion
        self.verify_model("glm-4.6").await
    }
}

impl Client {
    /// Verify a specific model is available and accessible
    pub async fn verify_model(&self, model_name: &str) -> Result<(), VerifyError> {
        // Use zai-sdk for model verification
        self.zai_client
            .verify_model(Some(model_name))
            .await
            .map_err(|e| match e {
                zai_sdk::ZaiError::Authentication { .. } => VerifyError::InvalidAuthentication,
                zai_sdk::ZaiError::ModelUnavailable { .. } => {
                    VerifyError::ProviderError(format!("Model '{model_name}' is not available"))
                }
                zai_sdk::ZaiError::RateLimit { .. } => {
                    VerifyError::ProviderError("Rate limit exceeded".to_string())
                }
                zai_sdk::ZaiError::ApiRequest { message } => VerifyError::ProviderError(message),
                zai_sdk::ZaiError::Network { .. } => {
                    VerifyError::ProviderError("Network error".to_string())
                }
                zai_sdk::ZaiError::InvalidResponse { message } => {
                    VerifyError::ProviderError(message)
                }
                _ => VerifyError::ProviderError(format!("Verification error: {e}")),
            })?;
        Ok(())
    }
}

// Implement conversion traits for ZAI client
impl_conversion_traits!(
    AsEmbeddings,
    AsTranscription,
    AsImageGeneration,
    AsAudioGeneration for Client
);
