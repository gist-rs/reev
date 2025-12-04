//! Prompt Processor for V3 Plan
//!
//! This module implements language refinement functionality using structured processing.
//! It uses StructuredProcessor to refine user prompts with max amount calculation.

use anyhow::{anyhow, Result};
use tracing::{debug, info, instrument};

// Import modules
pub mod max_amount_calculator;
pub mod structured_processor;
pub mod types;
pub mod validation;

// Re-export types
pub use max_amount_calculator::{MaxAmountCalculation, MaxAmountCalculator};
pub use structured_processor::StructuredProcessor;
pub use types::{
    PromptAction, PromptParameters, StructuredRefineRequest, StructuredRefineResponse,
    StructuredRefinedPrompt, ValidationResult,
};
pub use validation::validate_structured_response_with_max_amounts;

/// Structured refined prompt for backward compatibility
#[derive(Debug, Clone)]
pub struct RefinedPrompt {
    /// Original prompt
    pub original: String,
    /// Refined prompt
    pub refined: String,
    /// Confidence in this refinement
    pub confidence: f32,
    /// Usable amount for transfers (when "all" keyword was used)
    pub usable_amount: Option<f64>,
}

impl RefinedPrompt {
    /// Get the confidence value
    pub fn get_confidence(&self) -> f32 {
        self.confidence
    }

    /// Create a new refined prompt with default values for testing
    ///
    /// This constructor should only be used in tests.
    /// Production code should use the full struct initialization.
    pub fn new_for_test(original: String, refined: String) -> Self {
        Self {
            original,
            refined,
            confidence: 0.8, // Default confidence for testing
            usable_amount: None,
        }
    }

    /// Create a new refined prompt with usable amount for testing
    ///
    /// This constructor should only be used in tests.
    /// Production code should use the full struct initialization.
    pub fn new_for_test_with_amount(original: String, refined: String, usable_amount: f64) -> Self {
        Self {
            original,
            refined,
            confidence: 0.8, // Default confidence for testing
            usable_amount: Some(usable_amount),
        }
    }
}

/// Prompt processor for refining user prompts with structured processing
pub struct PromptProcessor {
    /// Structured processor for handling prompts
    structured_processor: StructuredProcessor,
}

impl Default for PromptProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl PromptProcessor {
    /// Create a new prompt processor
    pub fn new() -> Self {
        Self {
            structured_processor: StructuredProcessor::new(),
        }
    }

    /// Create a prompt processor with custom configuration
    pub fn with_config(api_key: String, model_name: String) -> Self {
        Self {
            structured_processor: StructuredProcessor::with_config(api_key, model_name),
        }
    }

    /// Process a user prompt using structured response
    #[instrument(skip(self))]
    pub async fn process_prompt(
        &self,
        prompt: &str,
        owner_wallet_address: &str,
    ) -> Result<RefinedPrompt> {
        info!("Processing prompt: {}", prompt);

        // Use structured processing
        let structured_result = self
            .structured_processor
            .process_prompt_structured(prompt, owner_wallet_address)
            .await?;

        // Convert to backward compatible format
        let refined_prompt = RefinedPrompt {
            original: structured_result.original_prompt,
            refined: structured_result.refined_prompt,
            confidence: structured_result.confidence,
            usable_amount: structured_result.usable_amount,
        };

        info!("Processed prompt successfully");
        debug!(
            "Original: {} -> Refined: {}",
            refined_prompt.original, refined_prompt.refined
        );

        Ok(refined_prompt)
    }

    /// Process a prompt with structured response
    #[instrument(skip(self))]
    pub async fn process_prompt_structured(
        &self,
        prompt: &str,
        owner_wallet_address: &str,
    ) -> Result<StructuredRefinedPrompt> {
        info!("Processing prompt with structured response: {}", prompt);

        // Use structured processing
        let result = self
            .structured_processor
            .process_prompt_structured(prompt, owner_wallet_address)
            .await?;

        info!("Processed prompt with structured response successfully");
        debug!(
            "Original: {} -> Refined: {}",
            result.original_prompt, result.refined_prompt
        );

        Ok(result)
    }
}

/// Create wallet context for the given address
pub async fn create_wallet_context(
    wallet_address: &str,
) -> Result<reev_types::flow::WalletContext> {
    // Re-export from the utils module
    crate::utils::create_wallet_context(wallet_address)
        .await
        .map_err(|e| anyhow!("Failed to create wallet context: {e}"))
}
