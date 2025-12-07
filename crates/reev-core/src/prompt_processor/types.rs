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
    /// Sequence of operations for multi-step prompts
    pub operation_sequence: Vec<Operation>,
}

/// Operation in a multi-step sequence
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Operation {
    /// Action type for this operation
    pub action: PromptAction,
    /// The wallet performing this action
    pub subject_pubkey: Option<String>,
    /// The destination address (for transfers/operations to others)
    pub target_pubkey: Option<String>,
    /// Extracted parameters for this operation
    pub parameters: PromptParameters,
    /// Confidence in this extraction (0.0-1.0)
    pub confidence: f32,
    /// Usable amount for this operation
    pub usable_amount: Option<f64>,
}

/// Action types that can be detected in prompts
#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Hash, Eq, strum::Display, strum::EnumString,
)]
pub enum PromptAction {
    Transfer,
    Swap,
    Lend,
    Earn,
    Borrow,
    Unknown,
}

/// Parameters extracted from prompt
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PromptParameters {
    /// Amount to transfer/swap/lend
    pub amount: Option<String>,
    /// Input token mint address
    pub input_mint: Option<String>,
    /// Output token mint address
    pub output_mint: Option<String>,
    /// Additional flexible parameters
    #[serde(default)]
    pub additional: HashMap<String, serde_json::Value>,
    /// Typed additional parameters for transfer operations
    pub transfer_params: Option<TransferAdditionalParams>,
    /// Typed additional parameters for swap operations
    pub swap_params: Option<SwapAdditionalParams>,
}

/// Additional parameters for transfer operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct TransferAdditionalParams {
    /// Recipient pubkey address
    pub recipient: Option<String>,
    /// Target pubkey address (alternative to recipient)
    pub target_pubkey: Option<String>,
    /// Recipient pubkey address (alternative to recipient)
    pub recipient_pubkey: Option<String>,
    /// Subject pubkey address
    pub subject_pubkey: Option<String>,
    /// User pubkey address (alternative to subject_pubkey)
    pub user_pubkey: Option<String>,
}

/// Additional parameters for swap operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SwapAdditionalParams {
    /// Input token mint address
    pub input_mint: Option<String>,
    /// Output token mint address
    pub output_mint: Option<String>,
    /// Amount of input token to swap
    pub input_amount: Option<u64>,
}

/// Result of validation
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationResult {
    Valid,
    Invalid(Vec<String>),
}

impl StructuredRefinedPrompt {
    /// Create a builder for StructuredRefinedPrompt
    pub fn builder() -> StructuredRefinedPromptBuilder {
        StructuredRefinedPromptBuilder::new()
    }

    /// Create a new structured refined prompt with minimal arguments
    pub fn with_defaults(
        refined_prompt: String,
        action: PromptAction,
        original_prompt: String,
    ) -> Self {
        Self {
            refined_prompt,
            action,
            subject_pubkey: None,
            target_pubkey: None,
            parameters: PromptParameters::default(),
            confidence: 0.8,
            original_prompt,
            usable_amount: None,
            operation_sequence: Vec::new(),
        }
    }

    /// Create a new structured refined prompt with minimal arguments and usable amount
    pub fn with_defaults_and_amount(
        refined_prompt: String,
        action: PromptAction,
        original_prompt: String,
        usable_amount: Option<f64>,
    ) -> Self {
        Self {
            refined_prompt,
            action,
            subject_pubkey: None,
            target_pubkey: None,
            parameters: PromptParameters::default(),
            confidence: 0.8,
            original_prompt,
            usable_amount,
            operation_sequence: Vec::new(),
        }
    }
}

/// Request to LLM for structured response
#[derive(Debug, Serialize, Deserialize)]
pub struct StructuredRefineRequest {
    /// The prompt to refine
    pub prompt: String,
    /// Owner wallet address (if available)
    pub owner_wallet_address: Option<String>,
    /// Maximum amount that can be transferred (for "all" keyword)
    pub max_amount: Option<f64>,
    /// YML-formatted max amounts for all action types
    pub max_amounts_yml: Option<String>,
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

        Ok(StructuredRefinedPrompt::builder()
            .refined_prompt(self.refined_prompt)
            .action(action)
            .subject_pubkey(self.subject_pubkey)
            .target_pubkey(self.target_pubkey)
            .parameters(self.parameters)
            .confidence(self.confidence as f32)
            .original_prompt(original_prompt)
            .usable_amount(usable_amount)
            .build())
    }
}

/// Builder for StructuredRefinedPrompt
pub struct StructuredRefinedPromptBuilder {
    refined_prompt: Option<String>,
    action: Option<PromptAction>,
    subject_pubkey: Option<String>,
    target_pubkey: Option<String>,
    parameters: Option<PromptParameters>,
    confidence: Option<f32>,
    original_prompt: Option<String>,
    usable_amount: Option<f64>,
    operation_sequence: Option<Vec<Operation>>,
}

impl Default for StructuredRefinedPromptBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl StructuredRefinedPromptBuilder {
    pub fn new() -> Self {
        Self {
            refined_prompt: None,
            action: None,
            subject_pubkey: None,
            target_pubkey: None,
            parameters: None,
            confidence: None,
            original_prompt: None,
            usable_amount: None,
            operation_sequence: None,
        }
    }

    pub fn refined_prompt(mut self, refined_prompt: String) -> Self {
        self.refined_prompt = Some(refined_prompt);
        self
    }

    pub fn action(mut self, action: PromptAction) -> Self {
        self.action = Some(action);
        self
    }

    pub fn subject_pubkey(mut self, subject_pubkey: Option<String>) -> Self {
        self.subject_pubkey = subject_pubkey;
        self
    }

    pub fn target_pubkey(mut self, target_pubkey: Option<String>) -> Self {
        self.target_pubkey = target_pubkey;
        self
    }

    pub fn parameters(mut self, parameters: PromptParameters) -> Self {
        self.parameters = Some(parameters);
        self
    }

    pub fn confidence(mut self, confidence: f32) -> Self {
        self.confidence = Some(confidence);
        self
    }

    pub fn original_prompt(mut self, original_prompt: String) -> Self {
        self.original_prompt = Some(original_prompt);
        self
    }

    pub fn usable_amount(mut self, usable_amount: Option<f64>) -> Self {
        self.usable_amount = usable_amount;
        self
    }

    pub fn operation_sequence(mut self, operation_sequence: Vec<Operation>) -> Self {
        self.operation_sequence = Some(operation_sequence);
        self
    }

    pub fn build(self) -> StructuredRefinedPrompt {
        StructuredRefinedPrompt {
            refined_prompt: self.refined_prompt.unwrap_or_default(),
            action: self.action.unwrap_or(PromptAction::Unknown),
            subject_pubkey: self.subject_pubkey,
            target_pubkey: self.target_pubkey,
            parameters: self.parameters.unwrap_or_default(),
            confidence: self.confidence.unwrap_or(0.8),
            original_prompt: self.original_prompt.unwrap_or_default(),
            usable_amount: self.usable_amount,
            operation_sequence: self.operation_sequence.unwrap_or_default(),
        }
    }
}
