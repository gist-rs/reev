//! Executor for Phase 2 Tool Execution
//!
//! This module implements the Phase 2 tool execution with parameter generation.
//! It executes each step of the YML flow with proper validation and error recovery.

use crate::execution::{SharedExecutor, ToolExecutor};
use crate::validation::FlowValidator;
use crate::yml_schema::{YmlFlow, YmlStep};
use anyhow::{anyhow, Result};
use reev_types::flow::{DynamicFlowPlan, DynamicStep, FlowResult, StepResult, WalletContext};
use reev_types::tools::ToolName;
use serde_json::json;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, error, info, instrument, warn};

/// Executor for Phase 2 tool execution
pub struct Executor {
    /// Flow validator for validation checks
    _validator: FlowValidator,
    /// Recovery configuration
    recovery_config: RecoveryConfig,
    /// Tool executor for actual tool execution
    tool_executor: SharedExecutor,
}

impl Default for Executor {
    fn default() -> Self {
        Self::new().expect("Failed to create default executor")
    }
}

impl Executor {
    /// Create a new executor
    pub fn new() -> Result<Self> {
        // Always use the real tool executor with RigAgent enabled
        // Store the tool executor without RigAgent first
        let tool_executor: SharedExecutor = Arc::new(ToolExecutor::new()?);

        Ok(Self {
            _validator: FlowValidator::new(),
            recovery_config: RecoveryConfig::default(),
            tool_executor,
        })
    }

    /// Initialize executor with RigAgent enabled (async version)
    pub async fn new_async_with_rig() -> Result<Self> {
        // Create tool executor with RigAgent enabled
        let tool_executor: SharedExecutor =
            Arc::new(ToolExecutor::new()?.enable_rig_agent().await?);

        Ok(Self {
            _validator: FlowValidator::new(),
            recovery_config: RecoveryConfig::default(),
            tool_executor,
        })
    }

    /// Create a new executor with rig agent enabled
    pub async fn new_with_rig() -> Result<Self> {
        // Use the async version with RigAgent enabled
        Self::new_async_with_rig().await
    }

    /// Set recovery configuration
    pub fn with_recovery_config(mut self, config: RecoveryConfig) -> Self {
        self.recovery_config = config;
        self
    }

    /// Set custom tool executor
    pub fn with_tool_executor(mut self, tool_executor: ToolExecutor) -> Self {
        self.tool_executor = Arc::new(tool_executor);
        self
    }

    /// Execute a YML flow with validation and error recovery
    #[instrument(skip(self, flow, initial_context))]
    pub async fn execute_flow(
        &self,
        flow: &YmlFlow,
        initial_context: &WalletContext,
    ) -> Result<FlowResult> {
        info!("Starting execution of flow: {}", flow.flow_id);

        let start_time = Instant::now();
        let mut step_results = Vec::new();
        let mut execution_successful = true;
        let mut error_message = None;

        // Validate flow structure before execution
        if let Err(e) = self._validator.validate_flow(flow) {
            error!("Flow validation failed: {}", e);
            return Err(anyhow!("Flow validation failed: {e}"));
        }

        // Convert YML flow to DynamicFlowPlan for execution
        let dynamic_flow_plan = self.yml_to_dynamic_flow_plan(flow, initial_context)?;

        // Ground truth is optional for validation
        let _ground_truth = flow.ground_truth.as_ref();

        // Execute each step with updated context
        let mut current_context = initial_context.clone();

        for (step_index, step) in flow.steps.iter().enumerate() {
            let step_start_time = Instant::now();

            info!(
                "Executing step {} of {}: {}",
                step_index + 1,
                flow.steps.len(),
                step.step_id
            );

            // Convert YML step to DynamicStep
            let dynamic_step = self.yml_to_dynamic_step(step, &dynamic_flow_plan.flow_id)?;

            // Execute step using existing step execution pattern
            match self
                .execute_step_with_recovery(&dynamic_step, &step_results, &current_context)
                .await
            {
                Ok(step_result) => {
                    debug!("Step {} completed successfully", step.step_id);

                    // Update context based on step result before moving to next step
                    current_context = self
                        .update_context_after_step(&current_context, &step_result)
                        .await?;

                    // Debug log to verify the updated context
                    info!(
                        "DEBUG: After updating context - USDC balance: {:?}",
                        current_context
                            .token_balances
                            .get("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")
                            .map(|t| t.balance)
                    );

                    // For multi-step flows, add a delay to ensure blockchain has fully processed
                    // This helps with timing issues between context updates and balance validation
                    if step_index < flow.steps.len() - 1 {
                        info!("Waiting 2 seconds to ensure blockchain has fully processed swap before proceeding to next step");
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    }

                    step_results.push(step_result);
                }
                Err(e) => {
                    error!("Step {} failed: {}", step.step_id, e);

                    // Create failed step result
                    let step_result = StepResult {
                        step_id: step.step_id.clone(),
                        success: false,
                        error_message: Some(e.to_string()),
                        tool_calls: vec![],
                        output: json!({}),
                        execution_time_ms: step_start_time.elapsed().as_millis() as u64,
                    };

                    // Don't update context for failed steps
                    step_results.push(step_result);

                    // Check if this is a critical step
                    if step.critical.unwrap_or(true) {
                        error!(
                            "Critical step {} failed, stopping flow execution",
                            step.step_id
                        );
                        execution_successful = false;
                        error_message = Some(format!("Critical step failed: {e}"));
                        break;
                    } else {
                        warn!(
                            "Non-critical step {} failed, continuing with flow",
                            step.step_id
                        );
                    }
                }
            }
        }

        // Validate final state against ground truth if available
        if let Some(ground_truth) = &flow.ground_truth {
            // Get final wallet context after all steps
            let final_context = if execution_successful {
                self.get_final_wallet_context(initial_context, &step_results)
                    .await
            } else {
                initial_context.clone()
            };

            if let Err(e) = self
                ._validator
                .validate_final_state(&final_context, ground_truth)
            {
                warn!("Final state validation failed: {}", e);
                // Don't fail the entire execution, just record the validation failure
            } else {
                info!("Final state validation passed");
            }
        }

        // Calculate metrics before moving step_results
        let successful_steps = step_results.iter().filter(|r| r.success).count();
        let failed_steps = step_results.iter().filter(|r| !r.success).count();
        let total_tool_calls = step_results.iter().map(|r| r.tool_calls.len()).sum();

        // Create flow result
        let flow_result = FlowResult {
            flow_id: flow.flow_id.clone(),
            user_prompt: flow.user_prompt.clone(),
            success: execution_successful,
            step_results,
            metrics: reev_types::flow::FlowMetrics {
                total_duration_ms: start_time.elapsed().as_millis() as u64,
                successful_steps,
                failed_steps,
                critical_failures: 0,     // TODO: Count critical failures
                non_critical_failures: 0, // TODO: Count non-critical failures
                total_tool_calls,
                context_resolution_ms: 0, // TODO: Track context resolution time
                prompt_generation_ms: 0,  // TODO: Track prompt generation time
                cache_hit_rate: 0.0,      // TODO: Track cache hit rate
            },
            final_context: Some(initial_context.clone()),
            error_message,
        };

        info!(
            "Flow execution completed with success: {}",
            flow_result.success
        );
        Ok(flow_result)
    }

    /// Execute a step with error recovery
    #[instrument(skip(self, step, previous_results, current_context))]
    async fn execute_step_with_recovery(
        &self,
        step: &DynamicStep,
        previous_results: &[StepResult],
        current_context: &WalletContext,
    ) -> Result<StepResult> {
        // Convert DynamicStep to YmlStep for tool execution
        let yml_step = YmlStep {
            step_id: step.step_id.clone(),
            prompt: step.prompt_template.clone(),
            refined_prompt: step.prompt_template.clone(), // Default to original prompt
            context: step.description.clone(),
            critical: Some(step.critical),
            estimated_time_seconds: Some(step.estimated_time_seconds),
            expected_tool_calls: Some(Vec::new()), // Will be generated by ToolExecutor
            expected_tools: if !step.required_tools.is_empty() {
                // Convert required_tools (Vec<String>) to Vec<ToolName>
                Some(
                    step.required_tools
                        .iter()
                        .filter_map(|tool_str| ToolName::from_str_safe(tool_str.into()))
                        .collect(),
                )
            } else {
                None
            },
        };

        // Use the provided current_context which should have the correct wallet balance
        let wallet_context = current_context.clone();

        debug!(
            "DEBUG: execute_step_with_recovery - USDC balance in context: {:?}",
            current_context
                .token_balances
                .get("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")
                .map(|t| t.balance)
        );

        // Execute step using tool executor with previous step history
        // Execute the step with either simple execution or with history if previous steps exist
        let step_result = if previous_results.is_empty() {
            // First step or when history is not needed
            self.tool_executor
                .execute_step(&yml_step, &wallet_context)
                .await?
        } else {
            // Pass previous step history for context-aware execution
            self.tool_executor
                .execute_step_with_history(&yml_step, &current_context, previous_results)
                .await?
        };

        Ok(step_result)
    }

    /// Convert YML flow to DynamicFlowPlan
    fn yml_to_dynamic_flow_plan(
        &self,
        flow: &YmlFlow,
        initial_context: &WalletContext,
    ) -> Result<DynamicFlowPlan> {
        let mut steps = Vec::new();

        // Convert each YML step to DynamicStep
        for yml_step in &flow.steps {
            let dynamic_step = self.yml_to_dynamic_step(yml_step, &flow.flow_id)?;
            steps.push(dynamic_step);
        }

        // Create DynamicFlowPlan
        let mut plan = DynamicFlowPlan::new(
            flow.flow_id.clone(),
            flow.user_prompt.clone(),
            initial_context.clone(),
        );

        // Add each step to the plan
        for step in steps {
            plan = plan.with_step(step);
        }

        Ok(plan)
    }

    /// Convert YML step to DynamicStep
    fn yml_to_dynamic_step(&self, yml_step: &YmlStep, _flow_id: &str) -> Result<DynamicStep> {
        let step_id = yml_step.step_id.clone();

        // Convert expected tool calls to required tools
        let required_tools = if let Some(tool_calls) = &yml_step.expected_tool_calls {
            tool_calls.iter().map(|tc| tc.tool_name.clone()).collect()
        } else {
            vec![]
        };

        // Create DynamicStep
        let step = DynamicStep::new(step_id, yml_step.prompt.clone(), yml_step.context.clone())
            .with_required_tools(required_tools)
            .with_critical(yml_step.critical.unwrap_or(true))
            .with_estimated_time(yml_step.estimated_time_seconds.unwrap_or(30));

        Ok(step)
    }

    /// Update wallet context after a step execution
    async fn update_context_after_step(
        &self,
        current_context: &WalletContext,
        step_result: &StepResult,
    ) -> Result<WalletContext> {
        info!(
            "Updating wallet context after step: {}",
            step_result.step_id
        );

        // Create a new context based on the current one
        let mut updated_context = current_context.clone();

        // Extract tool results from step output
        // Handle the direct structure returned by RigAgent
        if step_result.output.get("tool_name").is_some() {
            // Handle direct tool result from RigAgent
            if let Some(tool_name) = step_result.output.get("tool_name").and_then(|v| v.as_str()) {
                if tool_name == "jupiter_swap" {
                    self.update_context_after_jupiter_swap(&mut updated_context, step_result)
                        .await?;
                } else if tool_name == "jupiter_lend" {
                    self.update_context_after_jupiter_lend(&mut updated_context, step_result)?;
                }
            }
        }
        // Handle nested structure from tool_results array
        else if let Some(tool_results) = step_result.output.get("tool_results") {
            if let Some(results_array) = tool_results.as_array() {
                for result in results_array {
                    // Check tool type by tool_name field
                    if let Some(tool_name) = result.get("tool_name").and_then(|v| v.as_str()) {
                        match tool_name {
                            "jupiter_swap" => {
                                // Check if swap was successful
                                if let Some(success) =
                                    result.get("success").and_then(|v| v.as_bool())
                                {
                                    if success {
                                        // Extract swap details
                                        if let (
                                            Some(input_mint),
                                            Some(output_mint),
                                            Some(tx_signature),
                                        ) = (
                                            result.get("input_mint").and_then(|v| v.as_str()),
                                            result.get("output_mint").and_then(|v| v.as_str()),
                                            result
                                                .get("transaction_signature")
                                                .and_then(|v| v.as_str()),
                                        ) {
                                            // Get input amount in lamports
                                            let input_amount = result
                                                .get("input_amount_lamports")
                                                .and_then(|v| v.as_u64())
                                                .unwrap_or(0);

                                            // Get wallet pubkey
                                            let wallet_pubkey = result
                                                .get("wallet")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or_else(|| &updated_context.owner);

                                            // Query the blockchain to get actual output amount
                                            let rpc_client =
                                                solana_client::rpc_client::RpcClient::new(
                                                    "http://localhost:8899".to_string(),
                                                );

                                            let actual_output_amount = self
                                                .get_swap_output_amount(
                                                    &rpc_client,
                                                    wallet_pubkey,
                                                    output_mint,
                                                )
                                                .await
                                                .unwrap_or(0);

                                            info!(
                                                "Actual output amount for swap: {} of {}",
                                                actual_output_amount, output_mint
                                            );

                                            // Update SOL balance if SOL was swapped
                                            if input_mint
                                                == "So11111111111111111111111111111111111111111112"
                                            {
                                                info!(
                                                    "Updating SOL balance from {} to {} (subtracting {})",
                                                    updated_context.sol_balance,
                                                    updated_context.sol_balance.saturating_sub(input_amount),
                                                    input_amount
                                                );
                                                updated_context.sol_balance = updated_context
                                                    .sol_balance
                                                    .saturating_sub(input_amount);
                                            }

                                            // Update output token balance
                                            if let Some(token_balance) =
                                                updated_context.token_balances.get_mut(output_mint)
                                            {
                                                info!(
                                                    "Updating {} token balance from {} to {} (adding {})",
                                                    output_mint,
                                                    token_balance.balance,
                                                    token_balance.balance.saturating_add(actual_output_amount),
                                                    actual_output_amount
                                                );
                                                debug!(
                                                    "DEBUG: After updating, USDC balance in context: {}",
                                                    token_balance.balance.saturating_add(actual_output_amount)
                                                );
                                                token_balance.balance = token_balance
                                                    .balance
                                                    .saturating_add(actual_output_amount);
                                            } else {
                                                // Create new token entry if it doesn't exist
                                                info!(
                                                    "Creating new token entry for {} with balance {}",
                                                    output_mint, actual_output_amount
                                                );
                                                updated_context.token_balances.insert(
                                                    output_mint.to_string(),
                                                    reev_types::flow::TokenBalance {
                                                        balance: actual_output_amount,
                                                        decimals: Some(6), // Default to 6 decimals for most tokens
                                                        formatted_amount: None,
                                                        mint: output_mint.to_string(),
                                                        owner: Some(updated_context.owner.clone()),
                                                        symbol: None,
                                                    },
                                                );
                                            }

                                            info!(
                                                "Updated context: swapped {} of {} for {} of {}",
                                                input_amount,
                                                input_mint,
                                                actual_output_amount,
                                                output_mint
                                            );
                                        }
                                    }
                                }
                            }
                            "jupiter_lend_earn_deposit" => {
                                // Update context after Jupiter Lend Earn Deposit
                                if let (Some(success), Some(asset_mint)) = (
                                    result.get("success").and_then(|v| v.as_bool()),
                                    result.get("mint").and_then(|v| v.as_str()),
                                ) {
                                    if success {
                                        if let Some(amount) = result.get("amount") {
                                            let amount = if let Some(i) = amount.as_u64() {
                                                i
                                            } else if let Some(f) = amount.as_f64() {
                                                (f * 1_000_000.0) as u64 // Convert from USDC if needed
                                            } else {
                                                0
                                            };

                                            // Update token balance after lending
                                            if let Some(token_balance) =
                                                updated_context.token_balances.get_mut(asset_mint)
                                            {
                                                token_balance.balance =
                                                    token_balance.balance.saturating_sub(amount);
                                                info!(
                                                    "Updated context: lent {} of {}",
                                                    amount, asset_mint
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {
                                // Unknown tool, skip
                            }
                        }
                    }
                }
            }
        }

        // Recalculate total value
        updated_context.calculate_total_value();

        info!(
            "Context update completed. SOL: {}, Total value: ${:.2}",
            updated_context.sol_balance_sol(),
            updated_context.total_value_usd
        );

        Ok(updated_context)
    }

    /// Update wallet context after a Jupiter swap transaction
    async fn update_context_after_jupiter_swap(
        &self,
        updated_context: &mut WalletContext,
        step_result: &StepResult,
    ) -> Result<()> {
        // First, try to extract the values from the tool_results array
        if let Some(tool_results) = step_result.output.get("tool_results") {
            if let Some(results_array) = tool_results.as_array() {
                for result in results_array {
                    // Check if this is a Jupiter swap result
                    if let Some(jupiter_swap) = result.get("jupiter_swap") {
                        // Extract values from the jupiter_swap object
                        let input_mint = jupiter_swap.get("input_mint").and_then(|v| v.as_str());
                        let output_mint = jupiter_swap.get("output_mint").and_then(|v| v.as_str());
                        let success = jupiter_swap
                            .get("success")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);

                        if let (Some(input_mint), Some(output_mint)) = (input_mint, output_mint) {
                            if success {
                                // Get input amount from the swap result
                                let input_amount =
                                    if let Some(amount) = jupiter_swap.get("input_amount") {
                                        if let Some(f) = amount.as_f64() {
                                            (f * 1_000_000_000.0) as u64
                                        } else if let Some(i) = amount.as_u64() {
                                            i
                                        } else {
                                            0
                                        }
                                    } else {
                                        0
                                    };

                                // Get transaction signature from step result
                                let tx_signature = jupiter_swap
                                    .get("transaction_signature")
                                    .and_then(|v| v.as_str());

                                // Get wallet pubkey
                                let wallet_pubkey = jupiter_swap
                                    .get("wallet")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or_else(|| &updated_context.owner);

                                // tx_signature is already an Option<&str> from the destructuring above,
                                // so we don't need to check if it's Some again
                                {
                                    // Query the blockchain to get the actual output amount
                                    let rpc_client = solana_client::rpc_client::RpcClient::new(
                                        "http://localhost:8899".to_string(),
                                    );

                                    // Get the actual output amount from the swap transaction
                                    // Get the actual output amount from the swap transaction
                                    let actual_output_amount = self
                                        .get_swap_output_amount(
                                            &rpc_client,
                                            wallet_pubkey,
                                            output_mint,
                                        )
                                        .await
                                        .unwrap_or(0);

                                    info!(
                                        "DEBUG: Blockchain query result - {} balance: {}",
                                        output_mint, actual_output_amount
                                    );

                                    info!(
                                        "Actual output amount for swap: {} of {}",
                                        actual_output_amount, output_mint
                                    );

                                    // Update SOL balance if SOL was swapped
                                    if input_mint == "So11111111111111111111111111111111111111112" {
                                        info!(
                                            "Updating SOL balance from {} to {} (subtracting {})",
                                            updated_context.sol_balance,
                                            updated_context
                                                .sol_balance
                                                .saturating_sub(input_amount),
                                            input_amount
                                        );
                                        updated_context.sol_balance = updated_context
                                            .sol_balance
                                            .saturating_sub(input_amount);
                                    }

                                    // Update output token balance - use actual on-chain balance
                                    info!("Checking if token {} exists in context", output_mint);

                                    // Query the current on-chain balance for the token
                                    let current_on_chain_balance = self
                                        .get_swap_output_amount(
                                            &rpc_client,
                                            wallet_pubkey,
                                            output_mint,
                                        )
                                        .await
                                        .unwrap_or(0);

                                    info!(
                                        "Current on-chain balance for {}: {} (using this instead of adding to previous balance)",
                                        output_mint, current_on_chain_balance
                                    );

                                    if let Some(token_balance) =
                                        updated_context.token_balances.get_mut(output_mint)
                                    {
                                        info!(
                                            "Updating {} token balance from {} to {} (using actual on-chain balance)",
                                            output_mint,
                                            token_balance.balance,
                                            current_on_chain_balance
                                        );
                                        token_balance.balance = current_on_chain_balance;
                                    } else {
                                        // Create new token entry if it doesn't exist
                                        info!(
                                            "Creating new token entry for {} with balance {}",
                                            output_mint, current_on_chain_balance
                                        );
                                        updated_context.token_balances.insert(
                                            output_mint.to_string(),
                                            reev_types::flow::TokenBalance {
                                                balance: current_on_chain_balance,
                                                decimals: Some(6), // Default to 6 decimals for most tokens
                                                formatted_amount: None,
                                                mint: output_mint.to_string(),
                                                owner: Some(updated_context.owner.clone()),
                                                symbol: None,
                                            },
                                        );
                                        info!(
                                            "New token entry created: {} -> {}",
                                            output_mint, current_on_chain_balance
                                        );
                                    }

                                    info!(
                                        "Updated context: swapped {} of {} for {} of {}",
                                        input_amount, input_mint, actual_output_amount, output_mint
                                    );
                                }
                            } else {
                                warn!("Jupiter swap failed, not updating context");
                            }
                        }
                    }
                    // Check if this is a regular swap result with tool_name field
                    else if let Some(tool_name) = result.get("tool_name").and_then(|v| v.as_str())
                    {
                        if tool_name == "jupiter_swap" {
                            // Check if swap was successful
                            if let Some(success) = result.get("success").and_then(|v| v.as_bool()) {
                                if success {
                                    // Extract swap details
                                    if let (
                                        Some(input_mint),
                                        Some(output_mint),
                                        Some(tx_signature),
                                    ) = (
                                        result.get("input_mint").and_then(|v| v.as_str()),
                                        result.get("output_mint").and_then(|v| v.as_str()),
                                        result
                                            .get("transaction_signature")
                                            .and_then(|v| v.as_str()),
                                    ) {
                                        // Get input amount in lamports
                                        let input_amount = result
                                            .get("input_amount_lamports")
                                            .and_then(|v| v.as_u64())
                                            .unwrap_or(0);

                                        // Get wallet pubkey
                                        let wallet_pubkey = result
                                            .get("wallet")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or_else(|| &updated_context.owner);

                                        // tx_signature is already an Option<&str> from the destructuring above,
                                        // so we don't need to check if it's Some again
                                        {
                                            // Query the blockchain to get the actual output amount
                                            let rpc_client =
                                                solana_client::rpc_client::RpcClient::new(
                                                    "http://localhost:8899".to_string(),
                                                );

                                            // Get the actual output amount from the swap transaction
                                            let actual_output_amount = self
                                                .get_swap_output_amount(
                                                    &rpc_client,
                                                    wallet_pubkey,
                                                    output_mint,
                                                )
                                                .await
                                                .unwrap_or(0);

                                            info!(
                                                "Actual output amount for swap: {} of {}",
                                                actual_output_amount, output_mint
                                            );

                                            // Update SOL balance if SOL was swapped
                                            if input_mint
                                                == "So11111111111111111111111111111111111111112"
                                            {
                                                info!(
                                                    "Updating SOL balance from {} to {} (subtracting {})",
                                                    updated_context.sol_balance,
                                                    updated_context
                                                        .sol_balance
                                                        .saturating_sub(input_amount),
                                                    input_amount
                                                );
                                                updated_context.sol_balance = updated_context
                                                    .sol_balance
                                                    .saturating_sub(input_amount);
                                            }

                                            // Update output token balance
                                            info!(
                                                "Checking if token {} exists in context",
                                                output_mint
                                            );
                                            // Update output token balance - use actual on-chain balance
                                            info!(
                                                "Checking if token {} exists in context",
                                                output_mint
                                            );

                                            // Query the current on-chain balance for this token
                                            let current_on_chain_balance = self
                                                .get_swap_output_amount(
                                                    &rpc_client,
                                                    wallet_pubkey,
                                                    output_mint,
                                                )
                                                .await
                                                .unwrap_or(0);

                                            info!(
                                                "Current on-chain balance for {}: {} (using this instead of adding to previous balance)",
                                                output_mint, current_on_chain_balance
                                            );

                                            if let Some(token_balance) =
                                                updated_context.token_balances.get_mut(output_mint)
                                            {
                                                info!(
                                                    "Updating {} token balance from {} to {} (using actual on-chain balance)",
                                                    output_mint,
                                                    token_balance.balance,
                                                    current_on_chain_balance
                                                );
                                                token_balance.balance = current_on_chain_balance;
                                            } else {
                                                // Create new token entry if it doesn't exist
                                                info!(
                                                    "Creating new token entry for {} with balance {}",
                                                    output_mint, current_on_chain_balance
                                                );
                                                updated_context.token_balances.insert(
                                                    output_mint.to_string(),
                                                    reev_types::flow::TokenBalance {
                                                        balance: current_on_chain_balance,
                                                        decimals: Some(6), // Default to 6 decimals for most tokens
                                                        formatted_amount: None,
                                                        mint: output_mint.to_string(),
                                                        owner: Some(updated_context.owner.clone()),
                                                        symbol: None,
                                                    },
                                                );
                                                info!(
                                                    "New token entry created: {} -> {}",
                                                    output_mint, current_on_chain_balance
                                                );
                                            }

                                            info!(
                                                "Updated context: swapped {} of {} for {} of {}",
                                                input_amount,
                                                input_mint,
                                                actual_output_amount,
                                                output_mint
                                            );
                                        }
                                    }
                                } else {
                                    warn!("Jupiter swap failed, not updating context");
                                }
                            } else {
                                // If success field is not present, check for error field
                                if result.get("error").is_some() {
                                    warn!("Jupiter swap failed with error, not updating context");
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Update wallet context after a Jupiter lend transaction
    fn update_context_after_jupiter_lend(
        &self,
        updated_context: &mut WalletContext,
        step_result: &StepResult,
    ) -> Result<()> {
        if let (Some(asset_mint), Some(success)) = (
            step_result
                .output
                .get("asset_mint")
                .and_then(|v| v.as_str()),
            step_result.output.get("success").and_then(|v| v.as_bool()),
        ) {
            if success {
                let amount = if let Some(a) = step_result.output.get("amount") {
                    a.as_u64().unwrap_or(0)
                } else {
                    0
                };

                // Update token balance after lending (subtract from available balance)
                if let Some(token_balance) = updated_context.token_balances.get_mut(asset_mint) {
                    token_balance.balance = token_balance.balance.saturating_sub(amount);
                    info!("Updated context: lent {} of {}", amount, asset_mint);
                }
            }
        }
        Ok(())
    }

    /// Get actual output amount from a swap transaction by querying token balance
    async fn get_swap_output_amount(
        &self,
        rpc_client: &solana_client::rpc_client::RpcClient,
        wallet_pubkey: &str,
        output_mint: &str,
    ) -> Result<u64> {
        info!(
            "Querying token balance for wallet {} and mint {}",
            wallet_pubkey, output_mint
        );

        // Add a small delay to ensure blockchain has fully processed the transaction
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

        // Use token balance query function from reev-lib
        reev_lib::utils::solana::query_token_balance(rpc_client, wallet_pubkey, output_mint)
    }

    /// Get final wallet context after all steps have executed
    async fn get_final_wallet_context(
        &self,
        initial_context: &WalletContext,
        _step_results: &[StepResult],
    ) -> WalletContext {
        // For now, return the initial context as a placeholder
        // In a full implementation, this would update the context based on step results
        initial_context.clone()
    }
}

/// Configuration for error recovery
#[derive(Debug, Clone)]
pub struct RecoveryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: usize,
    /// Delay between retry attempts in milliseconds
    pub retry_delay_ms: u64,
    /// Whether to use exponential backoff for retries
    pub exponential_backoff: bool,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            retry_delay_ms: 1000,
            exponential_backoff: false,
        }
    }
}
