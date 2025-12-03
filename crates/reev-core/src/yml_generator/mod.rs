//! YML Generator for Phase 1 of V3 Plan
//!
//! This module implements the V3 plan YML generation component in Phase 1.
//! It uses refined prompts from the LanguageRefiner to generate structured YML flows
//! with appropriate expected_tools hints for the rig agent. This implementation
//! follows the V3 plan where RigAgent handles tool selection based on refined prompts.

use reev_types::tools::ToolName;

mod flow_templates;
// operation_parser module has been completely removed in V3 architecture
mod step_builders;
mod unified_flow_builder;

// operation_parser exports removed - module has been completely deleted
pub use unified_flow_builder::UnifiedFlowBuilder;

use anyhow::Result;
use reev_types::flow::WalletContext;
use tracing::{info, instrument};

use crate::prompt_processor::RefinedPrompt;
use crate::yml_schema::YmlFlow;

/// YML generator for creating structured flows from refined prompts
///
/// This implementation follows the V3 plan with a simplified YmlGenerator
/// that generates flows directly without operation parsing.
pub struct YmlGenerator {
    // Stateless in V3 architecture - no fields needed
}

impl Default for YmlGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl YmlGenerator {
    /// Create a new YML generator
    pub fn new() -> Self {
        Self {} // Stateless in V3 architecture
    }

    /// Create a YML generator with custom error tolerance
    pub fn with_error_tolerance(_error_tolerance: f64) -> Self {
        // In V3, error tolerance is handled at the ground truth level
        // This method is kept for backward compatibility
        Self::new()
    }

    /// Generate a YML flow from a refined prompt and wallet context
    #[instrument(skip(self, refined_prompt, wallet_context))]
    pub async fn generate_flow(
        &self,
        refined_prompt: &RefinedPrompt,
        wallet_context: &WalletContext,
    ) -> Result<YmlFlow> {
        info!(
            "Generating YML flow from refined prompt: {}",
            refined_prompt.refined
        );

        // Create a flow with potentially multiple steps based on refined prompt
        // Following V3 plan, each operation should be a separate step
        let flow_id = uuid::Uuid::new_v4().to_string();

        // Create wallet info from context
        let wallet_info = crate::yml_schema::YmlWalletInfo::new(
            wallet_context.owner.clone(),
            wallet_context.sol_balance,
        )
        .with_total_value(wallet_context.total_value_usd);

        // Add tokens to wallet info
        let mut final_wallet_info = wallet_info;
        for token in wallet_context.token_balances.values() {
            final_wallet_info = final_wallet_info.with_token(token.clone());
        }

        // Parse the refined prompt to extract individual operations
        let operations = extract_operations_from_prompt(&refined_prompt.refined);

        // If no operations found, create a single step with the refined prompt
        let steps = if operations.is_empty() {
            let expected_tools =
                determine_expected_tools(&refined_prompt.refined).unwrap_or_default();

            vec![crate::yml_schema::YmlStep::new(
                uuid::Uuid::new_v4().to_string(),
                refined_prompt.refined.clone(),
                format!("Executing: {}", refined_prompt.original),
            )
            .with_refined_prompt(refined_prompt.refined.clone())
            .with_expected_tools(expected_tools)]
        } else {
            // Create a separate step for each operation
            operations
                .into_iter()
                .enumerate()
                .map(|(i, operation)| {
                    let expected_tools = determine_expected_tools(&operation).unwrap_or_default();
                    crate::yml_schema::YmlStep::new(
                        uuid::Uuid::new_v4().to_string(),
                        operation.clone(),
                        format!("Step {}: {}", i + 1, operation),
                    )
                    .with_refined_prompt(operation.clone())
                    .with_expected_tools(expected_tools)
                })
                .collect()
        };

        // Create the flow
        let flow = YmlFlow::new(flow_id, refined_prompt.original.clone(), final_wallet_info)
            .with_steps(steps)
            .with_refined_prompt(refined_prompt.refined.clone());

        Ok(flow)
    }

    /// Generate a YML flow from a structured refined prompt and wallet context
    /// This method uses the structured response directly with extracted parameters
    #[instrument(skip(self, structured_prompt, wallet_context))]
    pub async fn generate_flow_from_structured_prompt(
        &self,
        structured_prompt: &crate::prompt_processor::StructuredRefinedPrompt,
        wallet_context: &WalletContext,
    ) -> Result<YmlFlow> {
        info!(
            "Generating YML flow from structured prompt: {}",
            structured_prompt.refined_prompt
        );

        // Create a flow with potentially multiple steps based on structured prompt
        // Following V3 plan, each operation should be a separate step
        let flow_id = uuid::Uuid::new_v4().to_string();

        // Create wallet info from context
        let wallet_info = crate::yml_schema::YmlWalletInfo::new(
            wallet_context.owner.clone(),
            wallet_context.sol_balance,
        )
        .with_total_value(wallet_context.total_value_usd);

        // Add tokens to wallet info
        let mut final_wallet_info = wallet_info;
        for token in wallet_context.token_balances.values() {
            final_wallet_info = final_wallet_info.with_token(token.clone());
        }

        // Create a single step with the refined prompt and extracted parameters
        // For structured prompts, we use the refined_prompt directly
        let expected_tools =
            determine_expected_tools(&structured_prompt.refined_prompt).unwrap_or_default();

        // Create YML step with expected tool calls based on the structured response
        let mut step = crate::yml_schema::YmlStep::new(
            uuid::Uuid::new_v4().to_string(),
            structured_prompt.refined_prompt.clone(),
            format!("Executing: {}", structured_prompt.original_prompt),
        )
        .with_refined_prompt(structured_prompt.refined_prompt.clone())
        .with_expected_tools(expected_tools)
        .with_structured_prompt(structured_prompt.clone());

        // Add expected tool calls based on the structured response parameters
        match structured_prompt.action {
            crate::prompt_processor::PromptAction::Swap => {
                if let Some(amount) = &structured_prompt.parameters.amount {
                    if let Some(input_mint) = &structured_prompt.parameters.input_mint {
                        if let Some(output_mint) = &structured_prompt.parameters.output_mint {
                            let tool_call = crate::yml_schema::YmlToolCall::new(
                                reev_types::tools::ToolName::JupiterSwap,
                                true,
                            )
                            .with_parameter_str("input_mint".to_string(), input_mint.to_string())
                            .with_parameter_str("output_mint".to_string(), output_mint.to_string())
                            .with_parameter_str("input_amount".to_string(), amount.clone());

                            step = step.with_tool_call(tool_call);
                        }
                    }
                }
            }
            crate::prompt_processor::PromptAction::Transfer => {
                if let Some(amount) = &structured_prompt.parameters.amount {
                    if let Some(input_mint) = &structured_prompt.parameters.input_mint {
                        // Select tool based on token type
                        let tool_name =
                            if input_mint == "So11111111111111111111111111111111111111112" {
                                reev_types::tools::ToolName::SolTransfer
                            } else {
                                reev_types::tools::ToolName::SplTransfer
                            };

                        let mut tool_call = crate::yml_schema::YmlToolCall::new(tool_name, true)
                            .with_parameter_str("amount".to_string(), amount.clone())
                            .with_parameter_str("mint_address".to_string(), input_mint.to_string());

                        // Add recipient if available
                        if let Some(recipient) = &structured_prompt.target_pubkey {
                            tool_call = tool_call
                                .with_parameter_str("recipient".to_string(), recipient.to_string());
                        }

                        step = step.with_tool_call(tool_call);
                    }
                }
            }
            crate::prompt_processor::PromptAction::Lend => {
                if let Some(amount) = &structured_prompt.parameters.amount {
                    if let Some(input_mint) = &structured_prompt.parameters.input_mint {
                        let tool_call = crate::yml_schema::YmlToolCall::new(
                            reev_types::tools::ToolName::JupiterLendEarnDeposit,
                            true,
                        )
                        .with_parameter_str("amount".to_string(), amount.clone())
                        .with_parameter_str("mint".to_string(), input_mint.to_string());

                        step = step.with_tool_call(tool_call);
                    }
                }
            }
            crate::prompt_processor::PromptAction::Earn => {
                if let Some(input_mint) = &structured_prompt.parameters.input_mint {
                    let tool_call = crate::yml_schema::YmlToolCall::new(
                        reev_types::tools::ToolName::JupiterLendEarnDeposit,
                        true,
                    )
                    .with_parameter_str("mint".to_string(), input_mint.to_string());

                    step = step.with_tool_call(tool_call);
                }
            }
            crate::prompt_processor::PromptAction::Borrow => {
                if let Some(amount) = &structured_prompt.parameters.amount {
                    if let Some(input_mint) = &structured_prompt.parameters.input_mint {
                        let tool_call = crate::yml_schema::YmlToolCall::new(
                            reev_types::tools::ToolName::JupiterLendEarnWithdraw,
                            true,
                        )
                        .with_parameter_str("amount".to_string(), amount.clone())
                        .with_parameter_str("mint".to_string(), input_mint.to_string());

                        step = step.with_tool_call(tool_call);
                    }
                }
            }
            crate::prompt_processor::PromptAction::Unknown => {
                // For unknown actions, we don't add any specific tool calls
            }
        }

        let steps = vec![step];

        // Create the flow
        let mut flow = YmlFlow::new(
            flow_id,
            structured_prompt.original_prompt.clone(),
            final_wallet_info,
        )
        .with_steps(steps)
        .with_refined_prompt(structured_prompt.refined_prompt.clone());

        // Generate comprehensive ground truth for benchmarking
        let ground_truth = generate_comprehensive_ground_truth(
            &flow,
            &structured_prompt.refined_prompt,
            wallet_context,
        );
        flow = flow.with_ground_truth(ground_truth);

        info!(
            "Generated YML flow with {} steps, ID: {}",
            flow.steps.len(),
            flow.flow_id
        );
        Ok(flow)
    }
}

/// Determine expected tools based on the refined prompt
fn determine_expected_tools(refined_prompt: &str) -> Option<Vec<ToolName>> {
    let prompt_lower = refined_prompt.to_lowercase();

    // Check for transfer operations
    if prompt_lower.contains("transfer") || prompt_lower.contains("send") {
        // Check if it's an SPL token transfer
        if prompt_lower.contains("usdc")
            || prompt_lower.contains("usdt")
            || prompt_lower.contains("ray")
            || prompt_lower.contains("srm")
        {
            return Some(vec![ToolName::SplTransfer]);
        }
        // Default to SOL transfer
        return Some(vec![ToolName::SolTransfer]);
    }

    // Check for swap operations
    if prompt_lower.contains("swap") {
        return Some(vec![ToolName::JupiterSwap]);
    }

    // Check for lend operations
    if prompt_lower.contains("lend") || prompt_lower.contains("deposit") {
        return Some(vec![ToolName::JupiterLendEarnDeposit]);
    }

    // Check for transfer/send operations
    if prompt_lower.contains("transfer") || prompt_lower.contains("send") {
        return Some(vec![ToolName::SolTransfer]);
    }

    // Check for balance operations
    if prompt_lower.contains("balance") || prompt_lower.contains("get") {
        return Some(vec![ToolName::GetAccountBalance]);
    }

    // Default to no expected tools if no pattern matches
    None
}

/// Generate comprehensive ground truth for benchmarking
fn generate_comprehensive_ground_truth(
    flow: &crate::yml_schema::YmlFlow,
    refined_prompt: &str,
    wallet_context: &reev_types::flow::WalletContext,
) -> crate::yml_schema::YmlGroundTruth {
    let prompt_lower = refined_prompt.to_lowercase();

    // Start with basic ground truth
    let mut ground_truth = crate::yml_schema::YmlGroundTruth::new().with_min_score(0.7); // Default minimum score

    // Add final state assertions based on operation type
    if prompt_lower.contains("swap") {
        ground_truth = add_swap_assertions(ground_truth, &wallet_context.owner);
    } else if prompt_lower.contains("lend") || prompt_lower.contains("deposit") {
        ground_truth = add_lend_assertions(ground_truth, &wallet_context.owner);
    } else if prompt_lower.contains("transfer") || prompt_lower.contains("send") {
        ground_truth = add_transfer_assertions(ground_truth, &wallet_context.owner, refined_prompt);
    }

    // Add success criteria
    ground_truth = add_success_criteria(ground_truth, refined_prompt);

    // Add expected data structures
    ground_truth = add_expected_data_structures(ground_truth);

    // Add flow complexity expectations
    ground_truth = add_flow_complexity_expectations(ground_truth, flow);

    // Add OpenTelemetry tracking expectations
    ground_truth = add_otel_tracking_expectations(ground_truth, flow);

    // Add recovery expectations
    ground_truth = add_recovery_expectations(ground_truth, flow);

    ground_truth
}

/// Add assertions for swap operations
fn add_swap_assertions(
    mut ground_truth: crate::yml_schema::YmlGroundTruth,
    _owner: &str,
) -> crate::yml_schema::YmlGroundTruth {
    // Account for swap amount + gas reserve + transaction fees
    ground_truth = ground_truth.with_assertion(
        crate::yml_schema::YmlAssertion::new("SolBalanceChange".to_string())
            .with_pubkey(_owner.to_string())
            .with_expected_change_lte(-2_500_000_000.0), // 2.5 SOL max (swap + fees)
    );

    // Add expected tool call
    ground_truth = ground_truth.with_tool_call(crate::yml_schema::YmlToolCall::new(
        reev_types::tools::ToolName::JupiterSwap,
        true, // critical
    ));

    ground_truth
}

/// Add assertions for lend operations
fn add_lend_assertions(
    mut ground_truth: crate::yml_schema::YmlGroundTruth,
    _owner: &str,
) -> crate::yml_schema::YmlGroundTruth {
    // Add expected tool call
    ground_truth = ground_truth.with_tool_call(crate::yml_schema::YmlToolCall::new(
        reev_types::tools::ToolName::JupiterLendEarnDeposit,
        true, // critical
    ));

    ground_truth
}

/// Add assertions for transfer operations
fn add_transfer_assertions(
    mut ground_truth: crate::yml_schema::YmlGroundTruth,
    _owner: &str,
    refined_prompt: &str,
) -> crate::yml_schema::YmlGroundTruth {
    let prompt_lower = refined_prompt.to_lowercase();
    let is_spl_transfer = prompt_lower.contains("usdc")
        || prompt_lower.contains("usdt")
        || prompt_lower.contains("ray")
        || prompt_lower.contains("srm");

    if is_spl_transfer {
        // Add SPL token balance change assertion
        let _token_symbol = extract_token_symbol_from_prompt(refined_prompt);
        ground_truth = ground_truth.with_assertion(
            crate::yml_schema::YmlAssertion::new("TokenBalanceChange".to_string())
                .with_pubkey(_owner.to_string())
                .with_expected_change_lte(-1_100_000.0), // 1.1 USDC/USDT max (transfer + fees)
        );

        // Add expected tool call for SPL transfer
        ground_truth = ground_truth.with_tool_call(crate::yml_schema::YmlToolCall::new(
            ToolName::SplTransfer,
            true, // critical
        ));
    } else {
        // Account for transfer amount + transaction fees
        ground_truth = ground_truth.with_assertion(
            crate::yml_schema::YmlAssertion::new("SolBalanceChange".to_string())
                .with_pubkey(_owner.to_string())
                .with_expected_change_lte(-1_100_000_000.0), // 1.1 SOL max (transfer + fees)
        );

        // Add expected tool call for SOL transfer
        ground_truth = ground_truth.with_tool_call(crate::yml_schema::YmlToolCall::new(
            ToolName::SolTransfer,
            true, // critical
        ));
    }

    ground_truth
}

/// Helper function to extract token symbol from prompt
fn extract_token_symbol_from_prompt(prompt: &str) -> String {
    let prompt_lower = prompt.to_lowercase();
    if prompt_lower.contains("usdc") {
        "Usdc".to_string()
    } else if prompt_lower.contains("usdt") {
        "Usdt".to_string()
    } else if prompt_lower.contains("ray") {
        "Ray".to_string()
    } else if prompt_lower.contains("srm") {
        "Srm".to_string()
    } else {
        "Unknown".to_string()
    }
}

/// Add success criteria based on operation type
fn add_success_criteria(
    mut ground_truth: crate::yml_schema::YmlGroundTruth,
    refined_prompt: &str,
) -> crate::yml_schema::YmlGroundTruth {
    let prompt_lower = refined_prompt.to_lowercase();

    if prompt_lower.contains("swap") {
        ground_truth = ground_truth.with_success_criterion(
            crate::yml_schema::YmlSuccessCriterion::new("token_exchange".to_string())
                .with_description("Successfully exchange tokens at fair rate".to_string())
                .with_required(true)
                .with_weight(0.6),
        );
    } else if prompt_lower.contains("lend") || prompt_lower.contains("deposit") {
        ground_truth = ground_truth.with_success_criterion(
            crate::yml_schema::YmlSuccessCriterion::new("yield_generation".to_string())
                .with_description("Successfully deposit assets for yield".to_string())
                .with_required(true)
                .with_weight(0.6),
        );
    } else if prompt_lower.contains("transfer") || prompt_lower.contains("send") {
        ground_truth = ground_truth.with_success_criterion(
            crate::yml_schema::YmlSuccessCriterion::new("asset_transfer".to_string())
                .with_description("Successfully transfer assets to recipient".to_string())
                .with_required(true)
                .with_weight(0.6),
        );
    }

    // Add common success criteria
    ground_truth = ground_truth.with_success_criterion(
        crate::yml_schema::YmlSuccessCriterion::new("error_handling".to_string())
            .with_description("Handle errors gracefully and provide feedback".to_string())
            .with_required(true)
            .with_weight(0.2),
    );

    ground_truth
}

/// Add expected data structures for validation
fn add_expected_data_structures(
    mut ground_truth: crate::yml_schema::YmlGroundTruth,
) -> crate::yml_schema::YmlGroundTruth {
    // Add wallet context data structure expectation
    ground_truth = ground_truth.with_data_structure(
        crate::yml_schema::YmlDataStructure::new(
            "$.result.data.wallet_context".to_string(),
            "object".to_string(),
        )
        .with_required_fields(vec![
            "sol_balance".to_string(),
            "total_portfolio_value".to_string(),
        ])
        .with_weight(0.2),
    );

    // Add execution result data structure expectation
    ground_truth = ground_truth.with_data_structure(
        crate::yml_schema::YmlDataStructure::new(
            "$.result.data.execution_result".to_string(),
            "object".to_string(),
        )
        .with_required_fields(vec![
            "success".to_string(),
            "transaction_signature".to_string(),
        ])
        .with_weight(0.2),
    );

    ground_truth
}

/// Add flow complexity expectations based on flow characteristics
fn add_flow_complexity_expectations(
    mut ground_truth: crate::yml_schema::YmlGroundTruth,
    flow: &crate::yml_schema::YmlFlow,
) -> crate::yml_schema::YmlGroundTruth {
    // Add step count expectation
    ground_truth = ground_truth.with_flow_complexity(
        crate::yml_schema::YmlFlowComplexity::new("step_count".to_string())
            .with_description("Flow should have appropriate number of steps".to_string())
            .with_required(true)
            .with_min_steps(flow.steps.len() as u32)
            .with_weight(0.2),
    );

    // Add multi-step execution expectation for multi-step flows
    if flow.steps.len() > 1 {
        ground_truth = ground_truth.with_flow_complexity(
            crate::yml_schema::YmlFlowComplexity::new("multi_step_execution".to_string())
                .with_description("Should execute multiple steps in correct order".to_string())
                .with_required(true)
                .with_min_steps(2)
                .with_weight(0.3),
        );
    }

    ground_truth
}

/// Add OpenTelemetry tracking expectations
fn add_otel_tracking_expectations(
    mut ground_truth: crate::yml_schema::YmlGroundTruth,
    flow: &crate::yml_schema::YmlFlow,
) -> crate::yml_schema::YmlGroundTruth {
    // Collect expected tools from all steps
    let mut expected_tools = Vec::new();
    for step in &flow.steps {
        if let Some(tools) = &step.expected_tools {
            for tool in tools {
                expected_tools.push(format!("{tool:?}"));
            }
        }
    }

    // Add tool call tracking expectation
    ground_truth = ground_truth.with_otel_tracking(
        crate::yml_schema::YmlOtelTracking::new("tool_call_logging".to_string())
            .with_description("OpenTelemetry should track all tool calls".to_string())
            .with_required(true)
            .with_required_tools(expected_tools)
            .with_weight(0.3),
    );

    // Add execution tracing expectation
    ground_truth = ground_truth.with_otel_tracking(
        crate::yml_schema::YmlOtelTracking::new("execution_tracing".to_string())
            .with_description("Flow execution should be traceable end-to-end".to_string())
            .with_required(true)
            .with_required_spans(vec![
                "prompt_processing".to_string(),
                "context_resolution".to_string(),
                "flow_execution".to_string(),
            ])
            .with_weight(0.3),
    );

    ground_truth
}

/// Add recovery expectations based on flow characteristics
fn add_recovery_expectations(
    mut ground_truth: crate::yml_schema::YmlGroundTruth,
    flow: &crate::yml_schema::YmlFlow,
) -> crate::yml_schema::YmlGroundTruth {
    // Add atomic execution expectation for all flows
    ground_truth = ground_truth.with_recovery_expectation(
        crate::yml_schema::YmlRecoveryExpectation::new("atomic_execution".to_string())
            .with_description("Flow should execute atomically or fail gracefully".to_string())
            .with_required(true)
            .with_weight(0.4),
    );

    // Add multi-step specific recovery expectations
    if flow.steps.len() > 1 {
        ground_truth = ground_truth.with_recovery_expectation(
            crate::yml_schema::YmlRecoveryExpectation::new("partial_success_handling".to_string())
                .with_description("Should handle partial successes gracefully".to_string())
                .with_required(true)
                .with_weight(0.3),
        );
    }

    ground_truth
}

/// Extract individual operations from a multi-step prompt
fn extract_operations_from_prompt(refined_prompt: &str) -> Vec<String> {
    let prompt_lower = refined_prompt.to_lowercase();
    let mut operations = Vec::new();

    // If this is a multi-step prompt with "then" or "and"
    if prompt_lower.contains(" then ") || prompt_lower.contains(" and ") {
        // Split by "then" first (preferred over "and")
        if prompt_lower.contains(" then ") {
            let parts: Vec<&str> = refined_prompt.split(" then ").collect();

            for part in parts {
                if !part.trim().is_empty() {
                    // Clean up the operation string
                    let mut operation = part.trim().to_string();

                    // Remove any trailing quotes that might have been added
                    if operation.ends_with('"') {
                        operation.pop();
                    }

                    // Check if this is a valid operation (contains action words)
                    if operation.to_lowercase().contains("swap")
                        || operation.to_lowercase().contains("lend")
                        || operation.to_lowercase().contains("transfer")
                        || operation.to_lowercase().contains("send")
                    {
                        operations.push(operation);
                    }
                }
            }
        }
        // Try splitting by "and" if "then" wasn't found
        else if prompt_lower.contains(" and ") {
            let parts: Vec<&str> = refined_prompt.split(" and ").collect();

            for part in parts {
                if !part.trim().is_empty() {
                    // Clean up the operation string
                    let mut operation = part.trim().to_string();

                    // Remove any trailing quotes that might have been added
                    if operation.ends_with('"') {
                        operation.pop();
                    }

                    // Check if this is a valid operation (contains action words)
                    if operation.to_lowercase().contains("swap")
                        || operation.to_lowercase().contains("lend")
                        || operation.to_lowercase().contains("transfer")
                        || operation.to_lowercase().contains("send")
                    {
                        operations.push(operation);
                    }
                }
            }
        }
    }

    // Return the operations if we found multiple valid ones
    if operations.len() > 1 {
        return operations;
    }

    // If no multi-step pattern found, return empty to use single step approach
    Vec::new()
}
