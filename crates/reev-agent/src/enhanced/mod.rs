//! 🧠 Enhanced AI Agent Modules
//!
//! This module provides superior AI agent capabilities that demonstrate
//! intelligence beyond deterministic approaches. It includes:
//!
//! - **Enhanced Context**: Rich financial intelligence and multi-step workflow understanding
//! - **Common Helpers**: Shared utilities for agent functionality
//! - **Gemini Agent**: Advanced reasoning with Google's Gemini models
//! - **OpenAI Agent**: Multi-turn conversation with OpenAI-compatible models
//! - **GLM Agent**: Tool-based agent using GLM's OpenAI-compatible API
//!
//! These agents showcase superior AI capabilities including:
//! - Multi-step DeFi workflow orchestration
//! - Adaptive strategy selection and execution
//! - Intelligent error recovery and retry mechanisms
//! - Context-aware decision making that exceeds deterministic patterns

pub mod common;
pub mod enhanced_context;
// pub mod glm_coding_agent; // Removed - now using ZAIAgent for GLM models
// pub mod gemini; // Not implemented yet
pub mod openai;
pub mod zai_agent;

// Re-export main components for easier access
pub use common::{extract_execution_results, AgentHelper, AgentTools, ExecutionResult};
pub use enhanced_context::{EnhancedContextAgent, RequestAnalysis};
// pub use glm_coding_agent::GlmCodingAgent; // Removed - now using ZAIAgent for GLM models
// pub use gemini::GeminiAgent; // Not implemented yet
pub use openai::OpenAIAgent;
pub use zai_agent::ZAIAgent;
