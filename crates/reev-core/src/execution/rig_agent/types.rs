//! LLM API Types for RigAgent
//!
//! This module contains the types used for interacting with the LLM API.

use serde::{Deserialize, Serialize};

/// LLM API request payload
#[derive(Debug, Serialize)]
pub struct LLMRequest {
    pub model: String,
    pub messages: Vec<LLMMessage>,
    pub temperature: f32,
    pub max_tokens: u32,
}

/// LLM API message
#[derive(Debug, Serialize)]
pub struct LLMMessage {
    pub role: String,
    pub content: String,
}

/// LLM API response
#[derive(Debug, Deserialize)]
pub struct LLMResponse {
    pub choices: Vec<LLMChoice>,
}

/// LLM API choice
#[derive(Debug, Deserialize)]
pub struct LLMChoice {
    pub message: LLMResponseMessage,
}

/// LLM API response message
#[derive(Debug, Deserialize)]
pub struct LLMResponseMessage {
    pub content: String,
}
