//! ZAI API client implementation
use serde::{Deserialize, Serialize};

use super::completion::CompletionModel as ZaiCompletionModel;
use rig::client::{
    impl_conversion_traits, CompletionClient, ProviderValue, VerifyClient, VerifyError,
};
use rig::{completion::CompletionError, prelude::ProviderClient};

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
        Self {
            base_url: base_url.into(),
            api_key: api_key.to_string(),
            http_client,
        }
    }

    /// Create a new ZAI client builder
    pub fn builder(api_key: &str) -> ClientBuilder<'_> {
        ClientBuilder::new(api_key)
    }

    /// Send a POST request to the ZAI API
    pub(crate) async fn post<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        request: &T,
    ) -> Result<R, CompletionError> {
        let url = format!("{}/{}", self.base_url, path);

        let response = self
            .http_client
            .post(&url)
            .bearer_auth(&self.api_key)
            .header("Content-Type", "application/json")
            .json(request)
            .send()
            .await
            .map_err(CompletionError::HttpError)?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error response".to_string());

            return Err(CompletionError::ProviderError(format!(
                "ZAI API error {status}: {text}"
            )));
        }

        response
            .json()
            .await
            .map_err(|e| CompletionError::ProviderError(e.to_string()))
    }

    /// Send a GET request to the ZAI API
    #[allow(unused)]
    pub(crate) async fn get<R: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
    ) -> Result<R, CompletionError> {
        let url = format!("{}/{}", self.base_url, path);

        let response = self
            .http_client
            .get(&url)
            .bearer_auth(&self.api_key)
            .send()
            .await
            .map_err(CompletionError::HttpError)?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error response".to_string());

            return Err(CompletionError::ProviderError(format!(
                "ZAI API error {status}: {text}"
            )));
        }

        response
            .json()
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
        let test_request = serde_json::json!({
            "model": model_name,
            "messages": [{"role": "user", "content": "test"}],
            "max_tokens": 1
        });

        let _: serde_json::Value =
            self.post("chat/completions", &test_request)
                .await
                .map_err(|e| match e {
                    CompletionError::HttpError(http_err) => VerifyError::HttpError(http_err),
                    CompletionError::ProviderError(provider_err) => {
                        // Check if it's a model not found error
                        if provider_err.to_lowercase().contains("model")
                            && provider_err.to_lowercase().contains("not found")
                        {
                            VerifyError::ProviderError(format!(
                                "Model '{model_name}' is not available"
                            ))
                        } else {
                            VerifyError::InvalidAuthentication
                        }
                    }
                    _ => VerifyError::ProviderError(e.to_string()),
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
