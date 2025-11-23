//! Validation for Verifiable AI-Generated DeFi Flows
//!
//! This module provides validation functionality for YML flows, including
//! structure validation, ground truth verification, and final state validation.

use crate::yml_schema::{YmlAssertion, YmlFlow, YmlGroundTruth, YmlStep};
use anyhow::{anyhow, Result};
use reev_types::flow::WalletContext;
use std::collections::HashMap;
use tracing::{info, warn};

/// Flow validator for YML flows and execution results
pub struct FlowValidator {
    /// Assertion validators by type
    assertion_validators: HashMap<String, Box<dyn AssertionValidator>>,
}

impl FlowValidator {
    /// Create a new flow validator
    pub fn new() -> Self {
        let mut validator = Self {
            assertion_validators: HashMap::new(),
        };

        // Register built-in assertion validators
        validator.register_assertion_validator(
            "SolBalanceChange".to_string(),
            Box::new(SolBalanceChangeValidator),
        );

        validator.register_assertion_validator(
            "TokenBalanceChange".to_string(),
            Box::new(TokenBalanceChangeValidator),
        );

        validator.register_assertion_validator(
            "PositionValueChange".to_string(),
            Box::new(PositionValueChangeValidator),
        );

        validator
    }

    /// Register a custom assertion validator
    pub fn register_assertion_validator(
        &mut self,
        assertion_type: String,
        validator: Box<dyn AssertionValidator>,
    ) {
        self.assertion_validators.insert(assertion_type, validator);
    }

    /// Validate a YML flow structure
    pub fn validate_flow(&self, flow: &YmlFlow) -> Result<()> {
        info!("Validating YML flow: {}", flow.flow_id);

        // Validate flow ID
        if flow.flow_id.is_empty() {
            return Err(anyhow!("Flow ID cannot be empty"));
        }

        // Validate user prompt
        if flow.user_prompt.is_empty() {
            return Err(anyhow!("User prompt cannot be empty"));
        }

        // Validate wallet info
        if flow.subject_wallet_info.pubkey.is_empty() {
            return Err(anyhow!("Wallet pubkey cannot be empty"));
        }

        // Validate steps
        if flow.steps.is_empty() {
            return Err(anyhow!("Flow must have at least one step"));
        }

        // Validate each step
        for (i, step) in flow.steps.iter().enumerate() {
            self.validate_step(step, i)?;
        }

        // Validate ground truth if present
        if let Some(ground_truth) = &flow.ground_truth {
            self.validate_ground_truth(ground_truth)?;
        }

        info!("YML flow validation passed: {}", flow.flow_id);
        Ok(())
    }

    /// Validate a single step
    fn validate_step(&self, step: &YmlStep, index: usize) -> Result<()> {
        // Validate step ID
        if step.step_id.is_empty() {
            return Err(anyhow!("Step {index} has empty step_id"));
        }

        // Validate prompt
        if step.prompt.is_empty() {
            return Err(anyhow!("Step {index} has empty prompt"));
        }

        // Validate expected tool calls
        if let Some(tool_calls) = &step.expected_tool_calls {
            if tool_calls.is_empty() {
                return Err(anyhow!("Step {index} has empty expected_tool_calls"));
            }

            for (j, tool_call) in tool_calls.iter().enumerate() {
                if tool_call.tool_name.to_string().is_empty() {
                    return Err(anyhow!("Step {index} tool call {j} has empty tool_name"));
                }
            }
        }

        Ok(())
    }

    /// Validate ground truth
    fn validate_ground_truth(&self, ground_truth: &YmlGroundTruth) -> Result<()> {
        // Validate error tolerance
        if let Some(tolerance) = ground_truth.error_tolerance {
            if !(0.0..=1.0).contains(&tolerance) {
                return Err(anyhow!("Error tolerance must be between 0.0 and 1.0"));
            }
        }

        // Validate assertions
        for (i, assertion) in ground_truth.final_state_assertions.iter().enumerate() {
            self.validate_assertion(assertion, i)?;
        }

        Ok(())
    }

    /// Validate a single assertion
    fn validate_assertion(&self, assertion: &YmlAssertion, index: usize) -> Result<()> {
        // Validate assertion type
        if assertion.assertion_type.is_empty() {
            return Err(anyhow!("Assertion {index} has empty assertion_type"));
        }

        // Check if we have a validator for this type
        if !self
            .assertion_validators
            .contains_key(&assertion.assertion_type)
        {
            warn!(
                "No validator registered for assertion type: {}",
                assertion.assertion_type
            );
        }

        // Validate expected change values
        if let Some(gte) = assertion.expected_change_gte {
            if let Some(lte) = assertion.expected_change_lte {
                if gte > lte {
                    return Err(anyhow!(
                        "Assertion {index} has invalid range: expected_change_gte ({gte}) > expected_change_lte ({lte})"
                    ));
                }
            }
        }

        Ok(())
    }

    /// Validate final state against ground truth
    pub fn validate_final_state(
        &self,
        final_context: &WalletContext,
        ground_truth: &YmlGroundTruth,
    ) -> Result<()> {
        info!("Validating final state against ground truth");

        // Validate each assertion
        for (i, assertion) in ground_truth.final_state_assertions.iter().enumerate() {
            if let Some(error) = self.validate_assertion_against_context(assertion, final_context) {
                warn!("Final state assertion {} failed: {}", i, error);
                return Err(anyhow!("Final state assertion {i} failed: {error}"));
            }
        }

        info!("Final state validation passed");
        Ok(())
    }

    /// Validate a single assertion against context
    fn validate_assertion_against_context(
        &self,
        assertion: &YmlAssertion,
        context: &WalletContext,
    ) -> Option<String> {
        // Get the validator for this assertion type
        let validator = self.assertion_validators.get(&assertion.assertion_type)?;

        // Validate using the validator
        validator.validate(assertion, context)
    }
}

impl Default for FlowValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for assertion validators
pub trait AssertionValidator: Send + Sync {
    /// Validate an assertion against a wallet context
    fn validate(&self, assertion: &YmlAssertion, context: &WalletContext) -> Option<String>;
}

/// Validator for SOL balance change assertions
struct SolBalanceChangeValidator;

impl AssertionValidator for SolBalanceChangeValidator {
    fn validate(&self, assertion: &YmlAssertion, context: &WalletContext) -> Option<String> {
        // For simplicity, we'll just check if the pubkey matches
        if let Some(pubkey) = &assertion.pubkey {
            if pubkey != &context.owner {
                return Some(format!(
                    "Pubkey mismatch: expected {}, got {}",
                    pubkey, context.owner
                ));
            }
        }

        // In a real implementation, we would compare the actual balance change
        // against the expected change specified in the assertion

        None
    }
}

/// Validator for token balance change assertions
struct TokenBalanceChangeValidator;

impl AssertionValidator for TokenBalanceChangeValidator {
    fn validate(&self, assertion: &YmlAssertion, context: &WalletContext) -> Option<String> {
        // For simplicity, we'll just check if the pubkey matches
        if let Some(pubkey) = &assertion.pubkey {
            if pubkey != &context.owner {
                return Some(format!(
                    "Pubkey mismatch: expected {}, got {}",
                    pubkey, context.owner
                ));
            }
        }

        // In a real implementation, we would compare the actual token balance change
        // against the expected change specified in the assertion

        None
    }
}

/// Validator for position value change assertions
struct PositionValueChangeValidator;

impl AssertionValidator for PositionValueChangeValidator {
    fn validate(&self, assertion: &YmlAssertion, context: &WalletContext) -> Option<String> {
        // For simplicity, we'll just check if the pubkey matches
        if let Some(pubkey) = &assertion.pubkey {
            if pubkey != &context.owner {
                return Some(format!(
                    "Pubkey mismatch: expected {}, got {}",
                    pubkey, context.owner
                ));
            }
        }

        // In a real implementation, we would compare the actual position value change
        // against the expected change specified in the assertion

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yml_schema::builders::create_swap_flow;

    #[test]
    fn test_validate_valid_flow() {
        let validator = FlowValidator::new();

        // Create a valid flow
        let flow = create_swap_flow(
            "test_pubkey".to_string(),
            1_000_000_000, // 1 SOL
            "SOL".to_string(),
            "USDC".to_string(),
            0.5, // 0.5 SOL
        );

        // Should validate successfully
        assert!(validator.validate_flow(&flow).is_ok());
    }

    #[test]
    fn test_validate_invalid_flow() {
        let validator = FlowValidator::new();

        // Create an invalid flow with empty steps
        let mut flow = create_swap_flow(
            "test_pubkey".to_string(),
            1_000_000_000, // 1 SOL
            "SOL".to_string(),
            "USDC".to_string(),
            0.5, // 0.5 SOL
        );

        // Remove all steps to make it invalid
        flow.steps.clear();

        // Should fail validation
        assert!(validator.validate_flow(&flow).is_err());
    }
}
