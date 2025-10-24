//! ZAI provider for the GLM-4.6 model with tool calling support
//!
//! # Example
//! ```
//! use reev_agent::providers::zai;
//!
//! let client = zai::Client::new("ZAI_API_KEY");
//!
//! let glm_model = client.completion_model(zai::GLM_4_6);
//! ```

pub mod client;
pub mod completion;

pub use client::Client;

pub const GLM_4_6: &str = "glm-4.6";
