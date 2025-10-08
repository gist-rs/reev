//! 🧠 Enhanced AI Agent Modules
//!
//! This module provides superior AI agent capabilities that demonstrate
//! intelligence beyond deterministic approaches. It includes:
//!
//! - **Enhanced Context**: Rich financial intelligence and multi-step workflow understanding
//! - **Gemini Agent**: Advanced reasoning with Google's Gemini models
//! - **OpenAI Agent**: Multi-turn conversation with OpenAI-compatible models
//!
//! These agents showcase superior AI capabilities including:
//! - Multi-step DeFi workflow orchestration
//! - Adaptive strategy selection and execution
//! - Intelligent error recovery and retry mechanisms
//! - Context-aware decision making that exceeds deterministic patterns

pub mod enhanced_context;
pub mod gemini;
pub mod openai;

// Re-export main components for easier access
pub use enhanced_context::{EnhancedContextAgent, RequestAnalysis};
pub use gemini::GeminiAgent;
pub use openai::OpenAIAgent;
