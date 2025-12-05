//! ZAI SDK - Generic SDK for GLM-4.6 Models
//!
//! This crate provides a unified interface for interacting with ZAI's GLM-4.6 models
//! including standard, coding, and future variants. It uses a builder pattern with
//! generic type parameters for flexible request and response handling.
//!
//! # Quick Start
//!
//! ```rust
//! use zai_sdk::{ZaiClient, GlmVariant};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let client = ZaiClient::builder()
//!         .variant(GlmVariant::Standard)
//!         .api_key("your-api-key")
//!         .build()?;
//!
//!     // In a real example, you would make an API call here:
//!     // let response = client
//!     //     .completion("What is the capital of France?")
//!     //     .await?;
//!     // println!("Response: {}", response);
//!     Ok(())
//! }
//! ```
//!
//! # Generic Request/Response Handling
//!
//! The SDK provides flexible request/response handling with generic types:
//!
//! ```rust
//! use zai_sdk::{ZaiClient, GlmVariant};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Serialize)]
//! struct CustomRequest {
//!     model: String,
//!     messages: Vec<serde_json::Value>,
//!     custom_param: bool,
//! }
//!
//! #[derive(Deserialize)]
//! struct CustomResponse {
//!     custom_field: String,
//!     // ... other fields
//! }
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let client = ZaiClient::builder()
//!         .variant(GlmVariant::Standard)
//!         .api_key("your-api-key")
//!         .build()?;
//!
//!     let request = CustomRequest {
//!         model: "glm-4.6".to_string(),
//!         messages: vec![],
//!         custom_param: true,
//!     };
//!     // In a real example, you would make an API call here:
//!     // let response: CustomResponse = client
//!     //     .send_generic_request("custom/endpoint", &request)
//!     //     .await?;
//!     // println!("Response: {}", response.custom_field);
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod error;
pub mod models;
pub mod streaming;
pub mod traits;

// Re-export commonly used items
pub use client::{ZaiClient, ZaiClientBuilder};
pub use error::{ZaiError, ZaiResult};
pub use models::{CompletionRequest, CompletionResponse, GlmVariant, Message};
pub use traits::{
    ZaiAuthenticator, ZaiModelCapabilities, ZaiRequestHandler, ZaiResponseHandler, ZaiRetryPolicy,
    ZaiStreamHandler, ZaiToolHandler,
};

/// Default model variant when not specified
pub const DEFAULT_MODEL: &str = "glm-4.6";

/// Default base URL for ZAI API
pub const DEFAULT_BASE_URL: &str = "https://api.z.ai/api/paas/v4";

/// Default timeout for API requests (30 seconds)
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Streaming response type alias for convenience
pub use streaming::StreamingResponse;
