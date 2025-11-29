//! Context Creation and Conversion for RigAgent
//!
//! This module contains methods for creating and converting YML context for the RigAgent.

use anyhow::Result;
use reev_types::flow::{StepResult, WalletContext};

use crate::execution::context_builder::{YmlContextBuilder, YmlOperationContext};
use crate::yml_schema::YmlStep;

/// Trait for context operations
pub trait ContextProvider {
    /// Create a context-aware prompt with wallet information and previous step history
    fn create_yml_context(
        &self,
        step: &YmlStep,
        wallet_context: &WalletContext,
        previous_results: &[StepResult],
    ) -> Result<YmlOperationContext>;

    /// Convert YML context to prompt format for LLM
    fn yml_context_to_prompt(&self, context: &YmlOperationContext, prompt: &str) -> Result<String>;
}

/// Default implementation of ContextProvider for any struct
impl<T> ContextProvider for T {
    /// Create a context-aware prompt with wallet information and previous step history
    fn create_yml_context(
        &self,
        step: &YmlStep,
        wallet_context: &WalletContext,
        previous_results: &[StepResult],
    ) -> Result<YmlOperationContext> {
        // Create a builder with wallet context
        let mut builder =
            YmlContextBuilder::new(wallet_context.clone()).with_previous_results(previous_results);

        // Add operation type from expected tools if available
        if let Some(expected_tools) = &step.expected_tools {
            if !expected_tools.is_empty() {
                // Convert ToolName to string for operation type
                if let Some(first_tool) = expected_tools.first() {
                    let tool_str = format!("{first_tool:?}");
                    let operation_type = tool_str.to_lowercase();
                    builder = builder.with_operation_type(&operation_type);
                }

                // Add constraints based on expected tools
                for tool in expected_tools {
                    let tool_str = format!("{tool:?}");
                    builder = builder.with_constraint(&format!(
                        "Consider using {tool_str} as it's expected for this operation"
                    ));
                }
            }
        }

        // Add step information if available
        if let Some(step_id) = step
            .step_id
            .split('_')
            .next_back()
            .and_then(|s| s.parse::<usize>().ok())
        {
            // Try to extract step number from step_id
            builder = builder.with_step_info(step_id, step_id + 1);
        }

        // Build the context
        let context = builder.build();

        // Log the generated context
        tracing::info!("Generated YML context for step {}", step.step_id);
        tracing::trace!("YML context: {:?}", context);

        Ok(context)
    }

    /// Convert YML context to prompt format for LLM
    fn yml_context_to_prompt(&self, context: &YmlOperationContext, prompt: &str) -> Result<String> {
        let mut full_prompt = String::new();

        // Convert AI context to prompt format
        let context_prompt = context.ai_context.to_prompt_format();
        full_prompt.push_str(&context_prompt);
        full_prompt.push('\n');

        // Debug logging for pubkey
        tracing::info!(
            "DEBUG: Context prompt includes pubkey: {}",
            context_prompt.contains("pubkey")
        );

        // Add constraints if any
        if !context.metadata.constraints.is_empty() {
            full_prompt.push_str("\nConstraints:\n");
            for constraint in &context.metadata.constraints {
                full_prompt.push_str(&format!("- {constraint}\n"));
            }
        }

        // Add the user prompt
        full_prompt.push_str(&format!(
            "\nPlease help with the following request: {prompt}\n"
        ));

        // Add specific instructions for multi-step flows
        if !context.ai_context.previous_results.is_empty() {
            full_prompt.push_str("\nIMPORTANT: For this step, please use the actual amounts from previous steps when determining parameters. For example, if this is a lend step after a swap, use the actual amount received from the swap, not an estimated amount.");
            full_prompt.push_str("\n\nCRITICAL: For lend operations after a swap, only use the amount received from the swap itself, not the total token balance which might include pre-existing amounts. The amount should already be in the smallest denomination (e.g., for USDC, 1 USDC = 1,000,000 units).");
            full_prompt.push_str("\n\nEXPLICIT INSTRUCTION: When the prompt says 'lend 95% of available USDC', you must calculate 95% of the ACTUAL USDC balance shown in the wallet context above, not any other value. Do not use percentages of old balances or estimated values.");
        }

        Ok(full_prompt)
    }
}
