//! Ping-Pong Executor for Orchestrator-Agent Coordination
//!
//! This module implements the critical missing coordination mechanism between
//! the orchestrator and agents. It executes flow plans step-by-step with
//! verification, enabling proper multi-step flow completion and partial scoring.
//!
//! **Issue #17: OTEL Integration at Orchestrator Level**
//! This executor now owns OTEL session initialization and logging for unified
//! tracing across all agent types (OpenAI, ZAI, future).
//!
//! **Issue #19: Pubkey Resolution in Dynamic Flow Execution**
//! This executor now properly prepares key_map for user wallet resolution,
//! enabling tools to resolve placeholder addresses correctly.

use crate::context_resolver::ContextResolver;
use anyhow::Result;
use reev_agent::{run_agent, LlmRequest};
use reev_flow::{
    get_enhanced_otel_logger, init_enhanced_otel_logging_with_session, log_prompt_event,
    log_step_complete, log_tool_call,
};
use reev_types::tools::ToolName;

// Re-export constants from reev-lib
use reev_lib::constants::{sol_mint, usdc_mint};
use reev_types::flow::{DynamicFlowPlan, DynamicStep, StepResult};

use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info, instrument, warn};
use uuid::Uuid;

/// Ping-pong executor that coordinates step-by-step execution
pub struct PingPongExecutor {
    /// Maximum time per step (milliseconds)
    step_timeout_ms: u64,
    /// OTEL session ID for unified tracing
    otel_session_id: Option<String>,
    /// Context resolver for placeholder resolution
    context_resolver: Arc<ContextResolver>,
}

impl PingPongExecutor {
    /// Create new ping-pong executor
    pub fn new(step_timeout_ms: u64, context_resolver: Arc<ContextResolver>) -> Self {
        Self {
            step_timeout_ms,
            otel_session_id: None,
            context_resolver,
        }
    }

    /// Execute flow plan with step-by-step ping-pong coordination
    #[instrument(skip(self, flow_plan, agent_type), fields(
        flow_id = %flow_plan.flow_id,
        total_steps = %flow_plan.steps.len()
    ))]
    pub async fn execute_flow_plan(
        &mut self,
        flow_plan: &DynamicFlowPlan,
        agent_type: &str,
    ) -> Result<Vec<StepResult>> {
        let flow_start_time = std::time::Instant::now();

        // Initialize OTEL session for this flow execution
        let otel_session_id = format!(
            "orchestrator-flow-{}-{}",
            flow_plan.flow_id,
            chrono::Utc::now().timestamp_millis()
        );

        // Try to initialize OTEL logging, but handle case where it's already initialized
        match init_enhanced_otel_logging_with_session(otel_session_id.clone()) {
            Ok(session_id) => {
                self.otel_session_id = Some(session_id);
                info!(
                    "[PingPongExecutor] ✅ OTEL logging initialized with session: {}",
                    self.otel_session_id.as_ref().unwrap()
                );
            }
            Err(e) => {
                // Logger already initialized - try to get existing logger
                match get_enhanced_otel_logger() {
                    Ok(logger) => {
                        self.otel_session_id = Some(logger.session_id().to_string());
                        warn!(
                            "[PingPongExecutor] ⚠️ Using existing OTEL session: {}",
                            self.otel_session_id.as_ref().unwrap()
                        );
                    }
                    Err(_) => {
                        // No logger available and can't initialize - continue without OTEL
                        warn!("[PingPongExecutor] ⚠️ No OTEL logging available: {}", e);
                    }
                }
            }
        }

        info!(
            "[PingPongExecutor] Starting step-by-step execution: {} steps with OTEL session {}",
            flow_plan.steps.len(),
            otel_session_id
        );

        // Log flow start to OTEL if available
        if self.otel_session_id.is_some() {
            log_prompt_event!(
                [
                    reev_types::ToolName::JupiterSwap.to_string(),
                    reev_types::ToolName::JupiterLendEarnDeposit.to_string(),
                    reev_types::ToolName::GetAccountBalance.to_string(),
                    reev_types::ToolName::GetJupiterLendEarnPosition.to_string()
                ],
                format!(
                    "Orchestrator executing {} steps with {}",
                    flow_plan.steps.len(),
                    agent_type
                ),
                flow_plan.user_prompt.clone()
            );
        }

        let mut step_results = Vec::new();
        let mut completed_steps = 0;

        for (step_index, step) in flow_plan.steps.iter().enumerate() {
            info!(
                "[PingPongExecutor] Executing step {}/{}: {}",
                step_index + 1,
                flow_plan.steps.len(),
                step.step_id
            );

            // Execute current step with agent
            match self
                .execute_single_step(step, agent_type, &step_results, &flow_plan.context.owner)
                .await
            {
                Ok(step_result) => {
                    completed_steps += 1;
                    info!(
                        "[PingPongExecutor] ✅ Step {} completed successfully",
                        step.step_id
                    );

                    // Store result for next steps
                    step_results.push(step_result);

                    // Check if flow should continue based on step success and criticality
                    let last_result = step_results.last().unwrap();
                    if !last_result.success && step.critical {
                        // Critical step failed - terminate flow
                        warn!(
                            "[PingPongExecutor] Flow terminated after step {} (critical failure)",
                            step.step_id
                        );
                        break;
                    }
                    // Continue to next step for non-critical failures or successful steps
                }
                Err(step_error) => {
                    error!(
                        "[PingPongExecutor] ❌ Step {} failed: {}",
                        step.step_id, step_error
                    );

                    // Create failed step result
                    let failed_result = StepResult {
                        step_id: step.step_id.clone(),
                        success: false,
                        duration_ms: self.step_timeout_ms,
                        tool_calls: vec![],
                        output: None,
                        error_message: Some(step_error.to_string()),
                        recovery_attempts: 0,
                    };

                    step_results.push(failed_result);

                    // Check if this is a critical failure
                    if step.critical {
                        error!(
                            "[PingPongExecutor] Critical step {} failed, aborting flow",
                            step.step_id
                        );
                        break;
                    } else {
                        warn!(
                            "[PingPongExecutor] Non-critical step {} failed, continuing flow",
                            step.step_id
                        );
                        // Continue to next step for non-critical failures
                    }
                }
            }
        }

        let completion_rate = completed_steps as f64 / flow_plan.steps.len() as f64;
        info!(
            "[PingPongExecutor] Flow completed: {}/{} steps ({:.1}% completion)",
            completed_steps,
            flow_plan.steps.len(),
            completion_rate * 100.0
        );

        // Log flow completion to OTEL if available
        if self.otel_session_id.is_some() {
            log_step_complete!(
                flow_plan.flow_id,
                flow_start_time.elapsed().as_millis() as u64, // flow_time_ms
                0 // step_time_ms (not applicable for flow completion)
            );

            // Write OTEL summary
            if let Ok(logger) = get_enhanced_otel_logger() {
                logger.write_summary().unwrap_or_else(|e| {
                    warn!("Failed to write OTEL summary: {}", e);
                });
            }
        }

        Ok(step_results)
    }

    /// Execute a single step with agent coordination
    async fn execute_single_step(
        &self,
        step: &DynamicStep,
        agent_type: &str,
        previous_results: &[StepResult],
        wallet_pubkey: &str,
    ) -> Result<StepResult> {
        let _start_time = std::time::Instant::now();

        // Create context for this step
        let step_context = self.create_step_context(step, previous_results).await?;

        // Execute with timeout
        let step_result = tokio::time::timeout(
            std::time::Duration::from_millis(self.step_timeout_ms),
            self.execute_agent_step(step, agent_type, &step_context, wallet_pubkey),
        )
        .await
        .map_err(|_| {
            anyhow::anyhow!(
                "Step {} timed out after {}ms",
                step.step_id,
                self.step_timeout_ms
            )
        })?;

        step_result
    }

    /// Create context for current step execution
    async fn create_step_context(
        &self,
        step: &DynamicStep,
        previous_results: &[StepResult],
    ) -> Result<String> {
        let mut context = format!(
            "Executing step: {}\nDescription: {}\n\n",
            step.step_id, step.description
        );

        if !previous_results.is_empty() {
            context.push_str("Previous step results:\n");
            for (i, result) in previous_results.iter().enumerate() {
                context.push_str(&format!(
                    "  Step {}: {} - {}\n",
                    i + 1,
                    result.step_id,
                    if result.success { "SUCCESS" } else { "FAILED" }
                ));
                if let Some(data) = &result.output {
                    context.push_str(&format!("    Data: {data}\n"));
                }
            }
            context.push('\n');
        }

        context.push_str(&format!(
            "Current task: {}\nPlease execute this step and report results.",
            step.prompt_template
        ));

        Ok(context)
    }

    /// Execute agent for a single step
    async fn execute_agent_step(
        &self,
        step: &DynamicStep,
        agent_type: &str,
        context: &str,
        wallet_pubkey: &str,
    ) -> Result<StepResult> {
        let start_time = std::time::Instant::now();

        // Create prompt for agent execution
        let prompt = format!(
            "{}\n\nContext: {}\n\nExecute this specific step and return results.",
            step.prompt_template, context
        );

        info!(
            "[PingPongExecutor] Executing step {} with {} agent",
            step.step_id, agent_type
        );

        // Log step start to OTEL if available
        if self.otel_session_id.is_some() {
            log_prompt_event!(
                ToolName::vec_to_string(&step.required_tools),
                format!("Orchestrator executing step: {}", step.description),
                prompt.clone()
            );
        }

        // Create LlmRequest for real agent execution
        // Resolve wallet pubkey using context resolver (handles placeholders like USER_WALLET_PUBKEY)
        let _resolved_wallet_pubkey = match self
            .context_resolver
            .resolve_placeholder(wallet_pubkey)
            .await
        {
            Ok(resolved) => resolved,
            Err(e) => {
                error!("[PingPongExecutor] Failed to resolve wallet pubkey: {}", e);
                wallet_pubkey.to_string() // Fallback to original
            }
        };

        // Resolve wallet pubkey using context resolver (handles placeholders like USER_WALLET_PUBKEY)
        let resolved_wallet_pubkey = match self
            .context_resolver
            .resolve_placeholder(wallet_pubkey)
            .await
        {
            Ok(resolved) => resolved,
            Err(e) => {
                error!("[PingPongExecutor] Failed to resolve wallet pubkey: {}", e);
                wallet_pubkey.to_string() // Fallback to original
            }
        };

        // Create LlmRequest for real agent execution
        let request = LlmRequest {
            id: Uuid::new_v4().to_string(),
            session_id: self
                .otel_session_id
                .clone()
                .unwrap_or_else(|| format!("orchestrator-{}", step.step_id)),
            prompt: prompt.clone(),
            context_prompt: context.to_string(),
            model_name: agent_type.to_string(),
            mock: false, // Use real execution
            initial_state: None,
            allowed_tools: Some(ToolName::vec_to_string(&step.required_tools)), // Pass allowed tools
            account_states: None,
            key_map: Some(self.create_key_map_with_wallet(&resolved_wallet_pubkey)), // Provide key mapping with resolved wallet
        };

        // Execute real agent
        match run_agent(agent_type, request).await {
            Ok(response) => {
                let duration_ms = start_time.elapsed().as_millis() as u64;

                // Check if response contains error indicators
                let has_error = response.to_lowercase().contains("error")
                    || response.to_lowercase().contains("failed")
                    || response.to_lowercase().contains("404")
                    || response.to_lowercase().contains("bad request")
                    || response.to_lowercase().contains("not found");

                // Parse enhanced tool calls from response with parameters
                let enhanced_tool_calls = self.parse_tool_calls_from_response(&response)?;

                // Update duration_ms for each tool call based on actual execution time
                let mut enhanced_tool_calls_with_duration = enhanced_tool_calls;
                for tool_call in &mut enhanced_tool_calls_with_duration {
                    tool_call.duration_ms = duration_ms;
                }

                // Convert to strings for StepResult compatibility
                let tool_calls = enhanced_tool_calls_with_duration
                    .iter()
                    .map(|v| v.tool_name.clone())
                    .collect::<Vec<_>>();

                // Log enhanced tool execution to OTEL if available
                if self.otel_session_id.is_some() {
                    for tool_call in &enhanced_tool_calls_with_duration {
                        log_tool_call!(
                            &tool_call.tool_name,
                            tool_call.params.clone().unwrap_or(serde_json::json!({}))
                        );
                    }
                }

                // Store enhanced tool calls in session for flow diagram generation
                if let Some(session_id) = &self.otel_session_id {
                    if let Err(e) = self
                        .store_enhanced_tool_calls(session_id, &enhanced_tool_calls_with_duration)
                        .await
                    {
                        warn!(
                            "[PingPongExecutor] Failed to store enhanced tool calls: {}",
                            e
                        );
                    }
                }

                Ok(StepResult {
                    step_id: step.step_id.clone(),
                    success: !has_error,
                    duration_ms,
                    tool_calls,
                    output: Some(response),
                    error_message: if has_error {
                        Some("Agent execution failed".to_string())
                    } else {
                        None
                    },
                    recovery_attempts: 0,
                })
            }
            Err(error) => {
                let duration_ms = start_time.elapsed().as_millis() as u64;

                error!(
                    "[PingPongExecutor] ❌ Agent execution failed for step {}: {}",
                    step.step_id, error
                );

                // Log failed execution to OTEL if available
                if self.otel_session_id.is_some() {
                    let failed_tool = step
                        .required_tools
                        .first()
                        .cloned()
                        .unwrap_or(ToolName::ExecuteTransaction);
                    log_tool_call!(
                        failed_tool.to_string(),
                        serde_json::json!({"error": error.to_string()})
                    );
                }

                Ok(StepResult {
                    step_id: step.step_id.clone(),
                    success: false,
                    duration_ms,
                    tool_calls: vec![],
                    output: None,
                    error_message: Some(error.to_string()),
                    recovery_attempts: 0,
                })
            }
        }
    }

    /// Parse tool calls from agent response (simplified implementation)
    fn parse_tool_calls_from_response(
        &self,
        response: &str,
    ) -> Result<Vec<reev_types::ToolCallSummary>> {
        // Enhanced parser to capture tool calls with parameters
        let mut tool_calls = Vec::new();
        let current_time = chrono::Utc::now();

        // Look for tool call indicators in response using type-safe enum
        use reev_types::ToolName;

        // Enhanced tool call detection with parameter extraction
        if response.contains(ToolName::JupiterSwap.to_string().as_str()) {
            let params = self.extract_swap_parameters(response);
            tool_calls.push(reev_types::ToolCallSummary {
                tool_name: ToolName::JupiterSwap.to_string(),
                timestamp: current_time,
                duration_ms: 0, // Will be set by caller
                success: true,
                error: None,
                params: Some(params),
                result_data: None, // Will be populated after execution
                tool_args: None,   // Will be populated with raw agent response
            });
        }

        if response.contains(ToolName::JupiterLendEarnDeposit.to_string().as_str()) {
            let params = self.extract_lend_parameters(response);
            tool_calls.push(reev_types::ToolCallSummary {
                tool_name: ToolName::JupiterLendEarnDeposit.to_string(),
                timestamp: current_time,
                duration_ms: 0,
                success: true,
                error: None,
                params: Some(params),
                result_data: None,
                tool_args: None, // Will be populated with raw agent response
            });
        }

        if response.contains(ToolName::GetJupiterLendEarnPosition.to_string().as_str()) {
            tool_calls.push(reev_types::ToolCallSummary {
                tool_name: ToolName::GetJupiterLendEarnPosition.to_string(),
                timestamp: current_time,
                duration_ms: 0,
                success: true,
                error: None,
                params: Some(serde_json::json!({})),
                result_data: None,
                tool_args: None, // Will be populated with raw agent response
            });
        }

        if response.contains(ToolName::GetAccountBalance.to_string().as_str()) {
            tool_calls.push(reev_types::ToolCallSummary {
                tool_name: ToolName::GetAccountBalance.to_string(),
                timestamp: current_time,
                duration_ms: 0,
                success: true,
                error: None,
                params: Some(serde_json::json!({})),
                result_data: None,
                tool_args: None, // Will be populated with raw agent response
            });
        }

        if response.contains(ToolName::GetJupiterLendEarnTokens.to_string().as_str()) {
            tool_calls.push(reev_types::ToolCallSummary {
                tool_name: ToolName::GetJupiterLendEarnTokens.to_string(),
                timestamp: current_time,
                duration_ms: 0,
                success: true,
                error: None,
                params: Some(serde_json::json!({})),
                result_data: None,
                tool_args: None, // Will be populated with raw agent response
            });
        }

        Ok(tool_calls)
    }

    /// Extract Jupiter swap parameters from response text
    fn extract_swap_parameters(&self, response: &str) -> serde_json::Value {
        // Look for patterns like "2.0 SOL", "50% of SOL", etc.
        let mut params = serde_json::json!({});

        // Extract amount patterns
        if let Some(amount_match) = regex::Regex::new(r"(\d+\.?\d*)\s*(SOL|USDC)")
            .ok()
            .and_then(|re| re.captures(response))
        {
            let amount = amount_match.get(1).unwrap().as_str();
            let token = amount_match.get(2).unwrap().as_str();
            params["amount"] = serde_json::Value::String(amount.to_string());
            params["token"] = serde_json::Value::String(token.to_string());
        }

        // Extract percentage patterns
        if let Some(percentage_match) = regex::Regex::new(r"(\d+\.?\d*)%\s*of\s*(SOL|USDC)")
            .ok()
            .and_then(|re| re.captures(response))
        {
            let percentage = percentage_match.get(1).unwrap().as_str();
            let token = percentage_match.get(2).unwrap().as_str();
            params["percentage"] = serde_json::Value::String(percentage.to_string());
            params["of_token"] = serde_json::Value::String(token.to_string());
        }

        params
    }

    /// Extract Jupiter lend parameters from response text
    fn extract_lend_parameters(&self, response: &str) -> serde_json::Value {
        let mut params = serde_json::json!({});

        // Extract APY patterns
        if let Some(apy_match) = regex::Regex::new(r"(\d+\.?\d*)%\s*APY")
            .ok()
            .and_then(|re| re.captures(response))
        {
            let apy = apy_match.get(1).unwrap().as_str();
            params["apy"] = serde_json::Value::String(apy.to_string());
        }

        // Extract yield target patterns
        if let Some(yield_match) = regex::Regex::new(r"yield\s+target\s+(\d+\.?\d*)x")
            .ok()
            .and_then(|re| re.captures(response))
        {
            let yield_target = yield_match.get(1).unwrap().as_str();
            params["yield_target"] = serde_json::Value::String(yield_target.to_string());
        }

        // Extract amount patterns for lend
        if let Some(amount_match) = regex::Regex::new(r"deposit\s+(\d+\.?\d*)\s*(SOL|USDC)")
            .ok()
            .and_then(|re| re.captures(response))
        {
            let amount = amount_match.get(1).unwrap().as_str();
            let token = amount_match.get(2).unwrap().as_str();
            params["deposit_amount"] = serde_json::Value::String(amount.to_string());
            params["deposit_token"] = serde_json::Value::String(token.to_string());
        }

        params
    }

    /// Create key_map for dynamic flow execution with actual wallet context
    /// This resolves the Issue #19 pubkey resolution problem by providing proper key mapping
    /// that includes the actual user wallet from the dynamic flow request
    fn create_key_map_with_wallet(&self, wallet_pubkey: &str) -> HashMap<String, String> {
        let mut key_map = HashMap::new();

        // Use actual wallet pubkey from the dynamic flow request
        key_map.insert("USER_WALLET_PUBKEY".to_string(), wallet_pubkey.to_string());
        key_map.insert(
            "RECIPIENT_WALLET_PUBKEY".to_string(),
            "11111111111111111111111111111112".to_string(),
        );

        // Common token mints
        key_map.insert("USDC_MINT".to_string(), usdc_mint().to_string());
        key_map.insert("SOL_MINT".to_string(), sol_mint().to_string());
        key_map.insert("WSOL_MINT".to_string(), sol_mint().to_string());

        // User token accounts (placeholder addresses for testing)
        key_map.insert(
            "USER_USDC_ATA".to_string(),
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        );
        key_map.insert(
            "USER_JUSDC_ATA".to_string(),
            "Esm1YjKsx9LGYhjuTzpJgUvK3Ttzr3M4hZqAsvRbK2zr".to_string(),
        );
        key_map.insert(
            "USER_JSOL_ATA".to_string(),
            "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN".to_string(),
        );

        info!(
            "[PingPongExecutor] Created key_map with wallet {} for pubkey resolution",
            wallet_pubkey
        );

        key_map
    }

    /// Store enhanced tool calls in session for flow diagram generation
    async fn store_enhanced_tool_calls(
        &self,
        session_id: &str,
        tool_calls: &[reev_types::ToolCallSummary],
    ) -> Result<()> {
        // Store using OTEL session infrastructure
        if let Ok(logger) = reev_flow::get_enhanced_otel_logger() {
            for tool_call in tool_calls {
                let enhanced_tool_call = reev_flow::EnhancedToolCall {
                    timestamp: tool_call.timestamp,
                    session_id: session_id.to_string(),
                    reev_runner_version: env!("CARGO_PKG_VERSION").to_string(),
                    reev_agent_version: env!("CARGO_PKG_VERSION").to_string(),
                    event_type: reev_flow::EventType::ToolInput,
                    prompt: None,
                    tool_input: Some(reev_flow::ToolInputInfo {
                        tool_name: tool_call.tool_name.clone(),
                        tool_args: tool_call.params.clone().unwrap_or(serde_json::json!({})),
                    }),
                    tool_output: Some(reev_flow::ToolOutputInfo {
                        success: tool_call.success,
                        results: tool_call
                            .result_data
                            .clone()
                            .unwrap_or(serde_json::json!({})),
                        error_message: tool_call.error.clone(),
                    }),
                    timing: reev_flow::TimingInfo {
                        flow_timeuse_ms: 0, // Not applicable for individual tool calls
                        step_timeuse_ms: tool_call.duration_ms,
                    },
                    metadata: serde_json::json!({
                        "enhanced_tracking": true,
                        "orchestrator_generated": true
                    }),
                };

                logger.log_tool_call(enhanced_tool_call)?;
            }
        }

        info!(
            "[PingPongExecutor] Stored {} enhanced tool calls for session {}",
            tool_calls.len(),
            session_id
        );

        Ok(())
    }
}
