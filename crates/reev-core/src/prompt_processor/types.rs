//! Types for structured LLM response system

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Structured refined prompt with extracted action and parameters
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StructuredRefinedPrompt {
    /// Refined version of the original prompt
    pub refined_prompt: String,
    /// Detected action type
    pub action: PromptAction,
    /// The wallet performing the action
    pub subject_pubkey: Option<String>,
    /// The destination address (for transfers/operations to others)
    pub target_pubkey: Option<String>,
    /// Extracted parameters
    pub parameters: PromptParameters,
    /// Confidence in this extraction (0.0-1.0)
    pub confidence: f32,
    /// Original prompt text
    pub original_prompt: String,
    /// Usable amount for transfers (when "all" keyword was used)
    pub usable_amount: Option<f64>,
}

/// Action types that can be detected in prompts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PromptAction {
    Transfer,
    Swap,
    Lend,
    Earn,
    Borrow,
    Unknown,
}

/// Parameters extracted from the prompt
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PromptParameters {
    /// Amount to transfer/swap/lend
    pub amount: Option<String>,
    /// Input token mint address
    pub input_mint: Option<String>,
    /// Output token mint address
    pub output_mint: Option<String>,
    /// Additional flexible parameters
    pub additional: HashMap<String, serde_json::Value>,
}

/// Result of validation
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationResult {
    Valid,
    Invalid(Vec<String>),
}

impl StructuredRefinedPrompt {
    /// Create a new structured refined prompt using a builder pattern
    pub fn new(
        refined_prompt: String,
        action: PromptAction,
        subject_pubkey: Option<String>,
        target_pubkey: Option<String>,
        parameters: PromptParameters,
        confidence: f32,
        original_prompt: String,
        usable_amount: Option<f64>,
    ) -> Self {
        Self {
            refined_prompt,
            action,
            subject_pubkey,
            target_pubkey,
            parameters,
            confidence,
            original_prompt,
            usable_amount,
        }
    }

    /// Create a new structured refined prompt with minimal arguments
    pub fn with_defaults(
        refined_prompt: String,
        action: PromptAction,
        original_prompt: String,
        usable_amount: Option<f64>,
    ) -> Self {
        Self {
            refined_prompt: refined_prompt.clone(),
            action,
            subject_pubkey: None,
            target_pubkey: None,
            parameters: PromptParameters::default(),
            confidence: 0.8,
            original_prompt,
            usable_amount,
        }
    }
}

/// Request to LLM for structured response
#[derive(Debug, Serialize)]
pub struct StructuredRefineRequest {
    /// The prompt to refine
    pub prompt: String,
    /// Owner wallet address (if available)
    pub owner_wallet_address: Option<String>,
    /// Maximum amount that can be transferred (for "all" keyword)
    pub max_amount: Option<f64>,
}

/// Response from LLM with structured data
#[derive(Debug, Deserialize)]
pub struct StructuredRefineResponse {
    /// Refined version of the original prompt
    pub refined_prompt: String,
    /// Detected action type
    pub action: String,
    /// The wallet performing the action
    pub subject_pubkey: Option<String>,
    /// The destination address (for transfers/operations to others)
    pub target_pubkey: Option<String>,
    /// Extracted parameters
    pub parameters: PromptParameters,
    /// Confidence in this extraction (0.0-1.0)
    pub confidence: f64,
}

impl StructuredRefineResponse {
    /// Convert to StructuredRefinedPrompt
    pub fn to_structured_prompt(
        self,
        original_prompt: String,
        usable_amount: Option<f64>,
    ) -> Result<StructuredRefinedPrompt, String> {
        // Parse action string to enum
        let action = match self.action.to_lowercase().as_str() {
            "transfer" => PromptAction::Transfer,
            "swap" => PromptAction::Swap,
            "lend" => PromptAction::Lend,
            "earn" => PromptAction::Earn,
            "borrow" => PromptAction::Borrow,
            _ => PromptAction::Unknown,
        };

        Ok(StructuredRefinedPrompt::new(
            self.refined_prompt,
            action,
            self.subject_pubkey,
            self.target_pubkey,
            self.parameters,
            self.confidence as f32,
            original_prompt,
            usable_amount,
        ))
    }
}
