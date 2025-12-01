//! Simple types for benchmark runners
//!
//! This module provides simplified types for benchmark runners to avoid
//! complex dependencies on existing type structures.

use serde::{Deserialize, Serialize};

/// Simple flow structure for benchmark runners
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Flow {
    /// Unique identifier
    pub id: String,
    /// User prompt
    pub prompt: Option<String>,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Wallet information
    pub subject_wallet_info: Option<serde_json::Value>,
    /// Steps in the flow
    pub steps: Vec<FlowStep>,
    /// Ground truth for validation
    pub ground_truth: Option<serde_json::Value>,
}

/// Simple flow step structure for benchmark runners
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowStep {
    /// Step identifier
    pub step_id: String,
    /// Refined prompt for this step
    pub refined_prompt: String,
    /// Step context
    pub context: Option<String>,
    /// Whether step is critical
    pub critical: bool,
    /// Expected tools for this step
    pub expected_tools: Vec<String>,
}
