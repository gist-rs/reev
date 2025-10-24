//! Common helper module for shared agent functionality
//!
//! This module contains shared utilities and structures used by both
//! OpenAIAgent and ZAIAgent to eliminate code duplication.

use anyhow::Result;
use serde_json::json;
use std::collections::HashMap;
use tracing::{error, info, warn};

use crate::{context::integration::ContextIntegration, prompt::SYSTEM_PREAMBLE, LlmRequest};

use reev_tools::tools::{
    AccountBalanceTool, JupiterEarnTool, JupiterLendEarnDepositTool, JupiterLendEarnMintTool,
    JupiterLendEarnRedeemTool, JupiterLendEarnWithdrawTool, JupiterSwapTool, LendEarnTokensTool,
    SolTransferTool, SplTransferTool,
};

/// Enhanced tool logging macro for consistent OpenTelemetry tracking
#[macro_export]
macro_rules! log_tool_call {
    ($tool_name:expr, $args:expr) => {
        // Enhanced otel logging is enabled by default (can be disabled with REEV_ENHANCED_OTEL=0)
        if std::env::var("REEV_ENHANCED_OTEL").unwrap_or_else(|_| "1".to_string()) != "0" {
            tracing::info!("🔧 [{}] Enhanced otel logging ENABLED", $tool_name);

            // Record traditional otel span attributes for compatibility
            let span = tracing::Span::current();
            span.record("tool.name", $tool_name);
            span.record("tool.start_time", chrono::Utc::now().to_rfc3339());
            if let Some(args_json) = serde_json::to_value($args).ok() {
                if let Some(obj) = args_json.as_object() {
                    for (key, value) in obj {
                        if let Some(str_val) = value.as_str() {
                            span.record(&format!("tool.args.{}", key), str_val);
                        } else {
                            span.record(&format!("tool.args.{}", key), &value.to_string());
                        }
                    }
                }
            }

            // Also log to enhanced file-based system
            let input_params = serde_json::to_value($args)
                .unwrap_or_else(|_| serde_json::Value::Object(Default::default()));
            tracing::info!(
                "📝 [{}] Attempting to log to enhanced otel system",
                $tool_name
            );

            // Check if EnhancedOtelLogger is available before trying to log
            if let Ok(logger) = reev_flow::get_enhanced_otel_logger() {
                tracing::info!(
                    "🔍 [{}] EnhancedOtelLogger found with session_id: {}",
                    $tool_name,
                    logger.session_id()
                );
                reev_flow::log_enhanced_tool_call!(
                    $tool_name,
                    0, // Will be updated on completion
                    input_params,
                    serde_json::Value::Object(Default::default()),
                    reev_flow::enhanced_otel::ToolExecutionStatus::Success,
                    None::<&str>
                );
                tracing::info!("✅ [{}] Enhanced otel log call completed", $tool_name);
            } else {
                tracing::warn!(
                    "❌ [{}] EnhancedOtelLogger NOT AVAILABLE - tool calls will not be captured!",
                    $tool_name
                );
            }
        } else {
            tracing::info!("🚫 [{}] Enhanced otel logging DISABLED", $tool_name);
        }
        tracing::info!("[{}] Tool execution started", $tool_name);
    };
}

/// Enhanced tool completion logging macro
#[macro_export]
macro_rules! log_tool_completion {
    ($tool_name:expr, $execution_time_ms:expr, $result:expr, $success:expr) => {
        // Enhanced otel logging is enabled by default (can be disabled with REEV_ENHANCED_OTEL=0)
        if std::env::var("REEV_ENHANCED_OTEL").unwrap_or_else(|_| "1".to_string()) != "0" {
            // Record traditional otel span attributes for compatibility
            let span = tracing::Span::current();
            span.record("tool.execution_time_ms", $execution_time_ms);
            span.record("tool.completion_time", chrono::Utc::now().to_rfc3339());
            if $success {
                span.record("tool.status", "success");
                if let Some(result_json) = serde_json::to_value($result).ok() {
                    if let Some(obj) = result_json.as_object() {
                        for (key, value) in obj {
                            if let Some(str_val) = value.as_str() {
                                span.record(&format!("tool.result.{}", key), str_val);
                            } else {
                                span.record(&format!("tool.result.{}", key), &value.to_string());
                            }
                        }
                    }
                }
            } else {
                span.record("tool.status", "error");
                if let Some(error_msg) = $result.as_str() {
                    span.record("tool.error.message", error_msg);
                }
            }

            // Also log to enhanced file-based system
            let input_params = serde_json::json!({}); // Will be populated from earlier call
            if $success {
                reev_flow::log_enhanced_tool_success!(
                    $tool_name,
                    $execution_time_ms,
                    input_params,
                    $result
                );
            } else {
                let error_msg = if let Some(s) = $result.as_str() {
                    s
                } else {
                    "Unknown error"
                };
                reev_flow::log_enhanced_tool_error!(
                    $tool_name,
                    $execution_time_ms,
                    input_params,
                    error_msg
                );
            }
        }
        if $success {
            tracing::info!(
                "[{}] Tool execution completed in {}ms",
                $tool_name,
                $execution_time_ms
            );
        } else {
            tracing::error!(
                "[{}] Tool execution failed in {}ms: {}",
                $tool_name,
                $execution_time_ms,
                $result
            );
        }
    };
}

/// 🎯 Complete response format including transactions, summary, and signatures
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub transactions: Vec<serde_json::Value>,
    pub summary: String,
    pub signatures: Vec<String>,
}

/// 🧠 Enhanced prompt data with context information
#[derive(Debug, Clone)]
pub struct EnhancedPromptData {
    pub prompt: String,
    pub has_context: bool,
    pub recommended_depth: u32,
}

/// 🛠️ Tool collection for agent builders
pub struct AgentTools {
    pub sol_tool: SolTransferTool,
    pub spl_tool: SplTransferTool,
    pub jupiter_swap_tool: JupiterSwapTool,
    pub jupiter_lend_earn_deposit_tool: JupiterLendEarnDepositTool,
    pub jupiter_lend_earn_withdraw_tool: JupiterLendEarnWithdrawTool,
    pub jupiter_lend_earn_mint_tool: JupiterLendEarnMintTool,
    pub jupiter_lend_earn_redeem_tool: JupiterLendEarnRedeemTool,
    pub jupiter_earn_tool: JupiterEarnTool,
    pub balance_tool: AccountBalanceTool,
    pub lend_earn_tokens_tool: LendEarnTokensTool,
}

impl AgentTools {
    /// Create new tool collection with the provided key_map
    pub fn new(key_map: HashMap<String, String>) -> Self {
        Self {
            sol_tool: SolTransferTool {
                key_map: key_map.clone(),
            },
            spl_tool: SplTransferTool {
                key_map: key_map.clone(),
            },
            jupiter_swap_tool: JupiterSwapTool {
                key_map: key_map.clone(),
            },
            jupiter_lend_earn_deposit_tool: JupiterLendEarnDepositTool {
                key_map: key_map.clone(),
            },
            jupiter_lend_earn_withdraw_tool: JupiterLendEarnWithdrawTool {
                key_map: key_map.clone(),
            },
            jupiter_lend_earn_mint_tool: JupiterLendEarnMintTool {
                key_map: key_map.clone(),
            },
            jupiter_lend_earn_redeem_tool: JupiterLendEarnRedeemTool {
                key_map: key_map.clone(),
            },
            jupiter_earn_tool: JupiterEarnTool {
                key_map: key_map.clone(),
            },
            balance_tool: AccountBalanceTool {
                key_map: key_map.clone(),
            },
            lend_earn_tokens_tool: LendEarnTokensTool::new(key_map),
        }
    }
}

/// 🧠 Common agent initialization utilities
pub struct AgentHelper;

impl AgentHelper {
    /// Build enhanced context and prompt data
    pub fn build_enhanced_context(
        payload: &LlmRequest,
        key_map: &HashMap<String, String>,
    ) -> Result<(ContextIntegration, EnhancedPromptData, String)> {
        let context_config = ContextIntegration::config_for_benchmark_type(&payload.id);
        let context_integration = ContextIntegration::new(context_config);

        let enhanced_prompt_data = if let Some(account_states) = &payload.account_states {
            // Use real observation state when available (FIXES #002)
            info!("[AgentHelper] Using account_states from observation for real balances");
            context_integration.build_enhanced_prompt_from_observation(
                &payload.prompt,
                account_states,
                key_map,
                &payload.id,
            )
        } else {
            // Fallback to initial_state from YAML (old behavior)
            let initial_state = payload.initial_state.clone().unwrap_or_default();
            info!("[AgentHelper] Using initial_state from YAML (fallback behavior)");
            context_integration.build_enhanced_prompt(
                &payload.prompt,
                &initial_state,
                key_map,
                &payload.id,
            )
        };

        let enhanced_prompt = format!(
            "{SYSTEM_PREAMBLE}\n\n---\n{}\n---",
            enhanced_prompt_data.prompt
        );

        Ok((
            context_integration,
            EnhancedPromptData {
                prompt: enhanced_prompt_data.prompt,
                has_context: enhanced_prompt_data.has_context,
                recommended_depth: enhanced_prompt_data.recommended_depth,
            },
            enhanced_prompt,
        ))
    }

    /// Determine optimal conversation depth based on context
    pub fn determine_conversation_depth(
        context_integration: &ContextIntegration,
        enhanced_prompt_data: &EnhancedPromptData,
        initial_state: &[reev_lib::benchmark::InitialStateItem],
        key_map: &HashMap<String, String>,
        benchmark_id: &str,
    ) -> usize {
        if enhanced_prompt_data.has_context {
            // Context provided - use efficient direct action depth
            enhanced_prompt_data.recommended_depth as usize
        } else {
            // Discovery mode - use extended depth for exploration
            context_integration.determine_optimal_depth(initial_state, key_map, benchmark_id)
                as usize
        }
    }

    /// Log prompt information for debugging
    pub fn log_prompt_info(
        agent_name: &str,
        payload: &LlmRequest,
        enhanced_prompt_data: &EnhancedPromptData,
        enhanced_prompt: &str,
        conversation_depth: usize,
    ) {
        info!("[{agent_name}] === PROMPT BEING SENT TO LLM ===");
        info!("[{agent_name}] Benchmark ID: {}", payload.id);
        info!("[{agent_name}] Model: {}", payload.model_name);
        info!(
            "[{agent_name}] Enhanced Prompt Length: {} chars",
            enhanced_prompt.len()
        );
        info!(
            "[{agent_name}] Has Context: {}",
            enhanced_prompt_data.has_context
        );
        info!(
            "[{agent_name}] Recommended Depth: {}",
            enhanced_prompt_data.recommended_depth
        );
        info!("[{agent_name}] === END PROMPT LOGGING ===");

        info!("[{agent_name}] === DEPTH CALCULATION ===");
        info!("[{agent_name}] Benchmark ID: {}", payload.id);
        info!(
            "[{agent_name}] Has Context: {}",
            enhanced_prompt_data.has_context
        );
        info!(
            "[{agent_name}] Recommended Depth: {}",
            enhanced_prompt_data.recommended_depth
        );
        info!(
            "[{agent_name}] Final Conversation Depth: {}",
            conversation_depth
        );
        info!("[{agent_name}] Is Single Turn: {}", conversation_depth == 1);
        info!("[{agent_name}] === END DEPTH CALCULATION ===");
    }

    /// Enhance user request with stop instructions for single-turn operations
    pub fn enhance_user_request(
        user_request: &str,
        conversation_depth: usize,
        agent_name: &str,
    ) -> String {
        if conversation_depth == 1 {
            format!(
                "{user_request}\n\nURGENT - READ CAREFULLY\n\
1. Execute the requested operation using appropriate tools\n\
2. When tools return 'status: ready' and 'action: *_complete' - OPERATION IS COMPLETE!\n\
3. IMMEDIATELY STOP - Format and return the transaction instructions\n\
4. ABSOLUTELY NO MORE TOOL CALLS AFTER SUCCESS!\n\
5. EACH EXTRA TOOL CALL CAUSES MaxDepthError AND COMPLETE FAILURE\n\
6. YOUR ENTIRE MISSION IS: Execute ONCE, detect completion, and STOP!"
            )
        } else {
            info!(
                "[{agent_name}] Multi-turn mode - conversation_depth: {}",
                conversation_depth
            );
            user_request.to_string()
        }
    }

    /// Format comprehensive response with transactions and optional flows
    pub fn format_comprehensive_response(
        execution_result: ExecutionResult,
        tool_calls: Option<Vec<serde_json::Value>>,
        agent_name: &str,
    ) -> Result<String> {
        let mut comprehensive_response = json!({
            "transactions": execution_result.transactions,
            "summary": execution_result.summary,
            "signatures": execution_result.signatures
        });

        // Add tool calls data if available
        if let Some(tool_calls) = tool_calls {
            if !tool_calls.is_empty() {
                comprehensive_response["flows"] = json!(tool_calls);
                info!(
                    "[{agent_name}] Tool calls captured: {} tool calls",
                    tool_calls.len()
                );
            }
        }

        info!(
            "[{agent_name}] Comprehensive response with {} transactions, {} signatures",
            execution_result.transactions.len(),
            execution_result.signatures.len()
        );

        Ok(serde_json::to_string(&comprehensive_response)?)
    }

    /// Extract tool calls from OpenTelemetry traces
    pub fn extract_tool_calls_from_otel() -> Vec<serde_json::Value> {
        if let Some(otel_trace) = reev_lib::otel_extraction::extract_current_otel_trace() {
            let tool_calls = reev_lib::otel_extraction::parse_otel_trace_to_tools(otel_trace);
            // Convert ToolCallInfo to JSON values
            tool_calls
                .into_iter()
                .map(|tool_call| {
                    serde_json::json!({
                        "tool_name": tool_call.tool_name,
                        "tool_args": tool_call.tool_args,
                        "execution_time_ms": tool_call.execution_time_ms,
                        "result_status": tool_call.result_status,
                        "result_data": tool_call.result_data
                    })
                })
                .collect()
        } else {
            vec![]
        }
    }
}

/// 🧠 Extract tool execution results from agent response
pub async fn extract_execution_results(
    response_str: &str,
    agent_name: &str,
) -> Result<ExecutionResult> {
    info!("[{agent_name}] === EXECUTION RESULT EXTRACTION ===");
    info!("[{agent_name}] Extracting execution results from response");
    info!(
        "[{agent_name}] Debug - Raw response string (length: {}): {}",
        response_str.len(),
        if response_str.len() > 500 {
            format!("{}...", &response_str[..500])
        } else {
            response_str.to_string()
        }
    );

    // Check if response contains completion signals
    let has_completion_signals = response_str.contains("status") && response_str.contains("ready");
    let has_action_complete = response_str.contains("action") && response_str.contains("_complete");
    let has_final_response =
        response_str.contains("final_response") && response_str.contains("true");

    info!("[{agent_name}] Completion Signal Analysis:");
    info!(
        "[{agent_name}] - Has 'status: ready': {}",
        has_completion_signals
    );
    info!(
        "[{agent_name}] - Has 'action: *_complete': {}",
        has_action_complete
    );
    info!(
        "[{agent_name}] - Has 'final_response: true': {}",
        has_final_response
    );

    // 🧠 Extract JSON from mixed natural language and JSON responses
    let json_str = if response_str.starts_with("```json") && response_str.ends_with("```") {
        response_str
            .trim_start_matches("```json")
            .trim_end_matches("```")
            .trim()
    } else if response_str.starts_with("```") && response_str.ends_with("```") {
        response_str
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim()
    } else if let Some(start) = response_str.find("```json") {
        // Find JSON block in natural language text
        if let Some(end) = response_str[start..].find("```") {
            response_str[start..start + end]
                .trim_start_matches("```json")
                .trim()
        } else {
            response_str
        }
    } else if let Some(start) = response_str.find('{') {
        // Find first complete JSON object in text
        let mut brace_count = 0;
        let mut in_string = false;
        let mut escape_next = false;
        let mut json_end = start;

        for (i, ch) in response_str[start..].char_indices() {
            if escape_next {
                escape_next = false;
                continue;
            }

            match ch {
                '\\' if in_string => escape_next = true,
                '"' => in_string = !in_string,
                '{' if !in_string => brace_count += 1,
                '}' if !in_string => {
                    brace_count -= 1;
                    if brace_count == 0 {
                        json_end = start + i + 1;
                        break;
                    }
                }
                _ => {}
            }
        }

        &response_str[start..json_end]
    } else {
        response_str
    };

    info!("[{agent_name}] Extracted JSON string: {}", json_str);

    // 🧠 Parse the extracted JSON with GLM-specific fallback handling
    info!(
        "[{agent_name}] Attempting to parse JSON string: {}",
        json_str
    );
    let parsed = match parse_json_with_glm_fallback(json_str, agent_name) {
        Ok(parsed) => {
            info!("[{agent_name}] ✅ JSON parsing succeeded");
            parsed
        }
        Err(e) => {
            error!(
                "[{agent_name}] ❌ All JSON parsing attempts failed: {}. Using fallback response.",
                e
            );
            return Ok(ExecutionResult {
                transactions: vec![],
                summary: format!(
                    "JSON parsing failed: {}. Response may need manual review. Original response: {}",
                    e,
                    if response_str.len() > 200 {
                        format!("{}...", &response_str[..200])
                    } else {
                        response_str.to_string()
                    }
                ),
                signatures: vec![],
            });
        }
    };

    info!("[{agent_name}] Parsed JSON: {}", parsed);

    // 🎯 Extract transactions, summary, and signatures
    // Handle string-escaped JSON transactions
    let transactions = if let Some(transactions_array) = parsed["transactions"].as_array() {
        transactions_array
            .iter()
            .filter_map(|tx| {
                if let Some(tx_str) = tx.as_str() {
                    // Parse the string as JSON
                    serde_json::from_str::<serde_json::Value>(tx_str).ok()
                } else if tx.is_object() {
                    // Direct JSON object
                    Some(tx.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    } else {
        vec![]
    };

    let summary = parsed["summary"]
        .as_str()
        .unwrap_or("Operation completed")
        .to_string();

    let signatures: Vec<String> = parsed["signatures"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|s| s.as_str().map(|s| s.to_string()))
        .collect();

    info!(
        "[{agent_name}] Extracted {} transactions",
        transactions.len()
    );
    info!("[{agent_name}] Summary: {}", summary);
    info!("[{agent_name}] Extracted {} signatures", signatures.len());

    Ok(ExecutionResult {
        transactions,
        summary,
        signatures,
    })
}

/// 🛡️ GLM-specific JSON parsing with robust fallback strategies
fn parse_json_with_glm_fallback(json_str: &str, agent_name: &str) -> Result<serde_json::Value> {
    // Strategy 1: Try direct JSON parsing first
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json_str) {
        info!("[{agent_name}] ✅ Direct JSON parsing succeeded");
        return Ok(parsed);
    }

    warn!("[{agent_name}] Direct JSON parsing failed, trying GLM fallback strategies...");
    info!("[{agent_name}] JSON that failed to parse: {}", json_str);

    // Strategy 2: Handle GLM-specific format issues (trailing commas, missing quotes, etc.)
    let cleaned_json = clean_glm_json_response(json_str);
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&cleaned_json) {
        info!("[{agent_name}] ✅ GLM JSON cleaning succeeded");
        return Ok(parsed);
    }

    // Strategy 3: Try to extract JSON from mixed natural language (more aggressive)
    if let Some(extracted_json) = extract_json_from_mixed_response(json_str) {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&extracted_json) {
            info!("[{agent_name}] ✅ Mixed response JSON extraction succeeded");
            return Ok(parsed);
        }
    }

    // Strategy 4: Handle empty/malformed responses gracefully
    if json_str.trim().is_empty() {
        warn!("[{agent_name}] Empty response detected, creating fallback structure");
        info!("[{agent_name}] Response was empty or only whitespace");
        return Ok(serde_json::json!({
            "transactions": [],
            "summary": "Agent returned empty response",
            "signatures": []
        }));
    }

    Err(anyhow::anyhow!("All JSON parsing strategies failed"))
}

/// 🧹 Clean common GLM JSON formatting issues
fn clean_glm_json_response(json_str: &str) -> String {
    let mut cleaned = json_str.trim().to_string();

    // Remove trailing commas before closing brackets/braces
    if let Ok(re) = regex::Regex::new(r",\s*([}\]\]])") {
        cleaned = re.replace_all(&cleaned, "$1").to_string();
    }

    // Fix unquoted property names (common in GLM responses)
    if let Ok(re) = regex::Regex::new(r"(\w+):") {
        cleaned = re.replace_all(&cleaned, "\"$1\":").to_string();
    }

    // Fix single quotes instead of double quotes
    cleaned = cleaned.replace('\'', "\"");

    // Remove any leading/trailing non-JSON content
    if let Some(start) = cleaned.find('{') {
        cleaned = cleaned[start..].to_string();
    }

    // Ensure proper JSON structure
    if !cleaned.starts_with('{') {
        cleaned = format!("{{\"transactions\": [], \"summary\": \"{cleaned}\"}}");
    }

    cleaned
}

/// 🔍 Extract JSON from mixed natural language responses (more aggressive)
fn extract_json_from_mixed_response(response: &str) -> Option<String> {
    // Look for JSON-like patterns in the response
    let json_patterns = [
        r"\{[^{}]*transactions[^{}]*\}",
        r"\{[^{}]*summary[^{}]*\}",
        r"\{[^{}]*signatures[^{}]*\}",
        r"\{[^{}]*result[^{}]*\}",
    ];

    for pattern in &json_patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            if let Some(captures) = re.captures(response) {
                if let Some(matched) = captures.get(0) {
                    let json_str = matched.as_str().trim();
                    if json_str.starts_with('{') && json_str.ends_with('}') {
                        return Some(json_str.to_string());
                    }
                }
            }
        }
    }

    None
}
