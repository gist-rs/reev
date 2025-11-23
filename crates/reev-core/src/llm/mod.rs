//! LLM Integration Module for reev-core
//!
//! This module provides integration with LLM services for generating
//! structured YML flows from user prompts.

pub mod glm_client;
pub mod prompt_templates;

// Re-export for convenience
pub use glm_client::GLMClient;
pub use prompt_templates::FlowPromptTemplate;

// Mock implementation is only available for testing
#[cfg(test)]
pub mod mock_llm;
#[cfg(test)]
pub use mock_llm::MockGLMClient;
