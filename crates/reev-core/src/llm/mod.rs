//! LLM Integration Module for reev-core
//!
//! This module provides integration with LLM services for generating
//! structured YML flows from user prompts.

pub mod glm_client;
pub mod mock;
pub mod prompt_templates;

// Re-export for convenience
pub use glm_client::GLMClient;
pub use mock::MockGLMClient;
pub use prompt_templates::FlowPromptTemplate;
