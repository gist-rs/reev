use anyhow::Result;
use rig::{completion::Prompt, prelude::*, providers::openai::Client};
use serde_json::json;
use std::collections::HashMap;
use tracing::{info, warn};

use crate::{
    context::integration::ContextIntegration,
    flow::{create_flow_infrastructure, GlobalFlowTracker},
    prompt::SYSTEM_PREAMBLE,
    tools::{
        AccountBalanceTool, JupiterEarnTool, JupiterLendEarnDepositTool, JupiterLendEarnMintTool,
        JupiterLendEarnRedeemTool, JupiterLendEarnWithdrawTool, JupiterSwapTool,
        LendEarnTokensTool, PositionInfoTool, SolTransferTool, SplTransferTool,
    },
    LlmRequest,
};

/// 🎯 Complete response format including transactions, summary, and signatures
#[derive(Debug, Clone)]
struct ExecutionResult {
    transactions: Vec<serde_json::Value>,
    summary: String,
    signatures: Vec<String>,
}

/// 🤖 Enhanced OpenAI Agent with Superior Multi-Turn Capabilities
///
/// This agent leverages the Rig framework's multi-turn conversation to enable
/// step-by-step reasoning, adaptive execution, and superior decision making
/// that demonstrates AI capabilities beyond deterministic approaches.
pub struct OpenAIAgent;

impl OpenAIAgent {
    /// 🧠 Run enhanced OpenAI agent with intelligent multi-step execution
    ///
    /// Uses the Rig framework's multi-turn conversation system to break down
    /// complex DeFi operations into manageable steps, validate each step, and
    /// adapt strategy based on results, showcasing superior AI intelligence.
    pub async fn run(
        model_name: &str,
        payload: LlmRequest,
        key_map: HashMap<String, String>,
    ) -> Result<String> {
        info!("[OpenAIAgent] Running enhanced multi-turn agent with model: {model_name}");

        // 🧠 Build enhanced context with account information
        let context_config = ContextIntegration::config_for_benchmark_type(&payload.id);
        let context_integration = ContextIntegration::new(context_config);
        let initial_state = payload.initial_state.clone().unwrap_or_default();

        let enhanced_prompt_data = context_integration.build_enhanced_prompt(
            &payload.prompt,
            &initial_state,
            &key_map,
            &payload.id,
        );

        let enhanced_prompt = format!(
            "{SYSTEM_PREAMBLE}\n\n---\n{}\n---",
            enhanced_prompt_data.prompt
        );

        // 🤖 MULTI-TURN CONVERSATION: Enable step-by-step reasoning
        let user_request = enhanced_prompt_data.prompt.clone();

        // 🚀 Initialize OpenAI client
        let client = Client::builder("")
            .base_url("http://localhost:1234/v1")
            .build()?;

        // 🧠 ADAPTIVE CONVERSATION DEPTH: Use context-aware depth optimization
        let conversation_depth = if enhanced_prompt_data.has_context {
            // Context provided - use efficient direct action depth
            enhanced_prompt_data.recommended_depth as usize
        } else {
            // Discovery mode - use extended depth for exploration
            context_integration.determine_optimal_depth(&initial_state, &key_map, &payload.id)
                as usize
        };

        info!(
            "[OpenAIAgent] Using conversation depth: {} for request",
            conversation_depth
        );

        // 🛠️ Create flow tracking infrastructure
        let _flow_tracker = create_flow_infrastructure();

        // 🛠️ Instantiate tools with context-aware key_map
        let sol_tool = SolTransferTool {
            key_map: key_map.clone(),
        };
        let spl_tool = SplTransferTool {
            key_map: key_map.clone(),
        };
        let jupiter_swap_tool = JupiterSwapTool {
            key_map: key_map.clone(),
        };
        let jupiter_lend_earn_deposit_tool = JupiterLendEarnDepositTool {
            key_map: key_map.clone(),
        };
        let jupiter_lend_earn_withdraw_tool = JupiterLendEarnWithdrawTool {
            key_map: key_map.clone(),
        };
        let jupiter_lend_earn_mint_tool = JupiterLendEarnMintTool {
            key_map: key_map.clone(),
        };
        let jupiter_lend_earn_redeem_tool = JupiterLendEarnRedeemTool {
            key_map: key_map.clone(),
        };
        let jupiter_positions_tool = JupiterEarnTool {
            key_map: key_map.clone(),
        };
        let jupiter_earn_tool = JupiterEarnTool {
            key_map: key_map.clone(),
        };

        // 🔍 DISCOVERY TOOLS: Enable prerequisite validation when context is insufficient
        let balance_tool = AccountBalanceTool {
            key_map: key_map.clone(),
        };
        let position_tool = PositionInfoTool {
            key_map: key_map.clone(),
        };
        let lend_earn_tokens_tool = LendEarnTokensTool::new(key_map.clone());

        // 🧠 Build enhanced multi-turn agent
        let agent = client
            .completion_model(model_name)
            .completions_api()
            .into_agent_builder()
            .preamble(&enhanced_prompt)
            .tool(sol_tool)
            .tool(spl_tool)
            .tool(jupiter_swap_tool)
            .tool(jupiter_lend_earn_deposit_tool)
            .tool(jupiter_lend_earn_withdraw_tool)
            .tool(jupiter_lend_earn_mint_tool)
            .tool(jupiter_lend_earn_redeem_tool)
            .tool(jupiter_positions_tool)
            .tool(jupiter_earn_tool)
            .tool(balance_tool)
            .tool(position_tool)
            .tool(lend_earn_tokens_tool)
            .build();

        // Add explicit stop instruction to the user request for simple operations
        let enhanced_user_request = if conversation_depth == 1 {
            format!("{user_request}\n\n🚨 CRITICAL INSTRUCTIONS:\n1. Execute the requested operation using appropriate tools\n2. When tools return 'status: ready' and 'action: *_complete', IMMEDIATELY STOP\n3. Format the transaction response using the provided instructions\n4. DO NOT make additional tool calls after completion\n5. FAILURE TO STOP WILL CAUSE MaxDepthError!")
        } else {
            user_request.to_string()
        };

        let response = agent
            .prompt(&enhanced_user_request)
            .multi_turn(conversation_depth)
            .await?;

        let response_str = response.to_string();
        info!(
            "[OpenAIAgent] Raw response from enhanced multi-turn agent: {}",
            response_str
        );

        // 🎯 EXTRACT TOOL EXECUTION RESULTS FROM CONVERSATION
        let execution_result = extract_execution_results(&response_str).await?;

        // 🎯 EXTRACT FLOW DATA FROM GLOBAL TRACKER
        let flow_data = GlobalFlowTracker::get_flow_data();

        // 🎯 FORMAT COMPREHENSIVE RESPONSE WITH FLOWS
        let mut comprehensive_response = json!({
            "transactions": execution_result.transactions,
            "summary": execution_result.summary,
            "signatures": execution_result.signatures
        });

        // Add flow data if available
        if let Some(flows) = flow_data {
            comprehensive_response["flows"] = json!(flows);
            info!(
                "[OpenAIAgent] Flow data captured: {} tool calls",
                flows.total_tool_calls
            );
        }

        info!(
            "[OpenAIAgent] Comprehensive response with {} transactions, {} signatures",
            execution_result.transactions.len(),
            execution_result.signatures.len()
        );

        Ok(serde_json::to_string(&comprehensive_response)?)
    }
}

/// 🧠 Extract tool execution results from agent response
async fn extract_execution_results(response_str: &str) -> Result<ExecutionResult> {
    info!("[OpenAIAgent] Extracting execution results from response");
    info!(
        "[OpenAIAgent] Debug - Raw response string: {}",
        response_str
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
                '"' if !in_string => in_string = true,
                '"' if in_string => in_string = false,
                '\\' if in_string => escape_next = true,
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

    info!("[OpenAIAgent] Debug - Extracted JSON string: {}", json_str);

    // Try to parse as JSON first
    match serde_json::from_str::<serde_json::Value>(json_str) {
        Ok(json_value) => {
            // Check if response already contains our expected format
            if let (Some(transactions), Some(summary), Some(signatures)) = (
                json_value.get("transactions"),
                json_value.get("summary"),
                json_value.get("signatures"),
            ) {
                info!("[OpenAIAgent] Response already in comprehensive format");
                info!(
                    "[OpenAIAgent] Debug - Transactions found: {:?}",
                    transactions
                );
                info!("[OpenAIAgent] Debug - Summary: {:?}", summary);
                info!("[OpenAIAgent] Debug - Signatures: {:?}", signatures);

                // 🎯 CHECK FOR DIRECT INSTRUCTION OBJECTS IN TRANSACTIONS ARRAY
                // Handle case where transactions array contains instruction objects directly
                let direct_instructions: Vec<serde_json::Value> = transactions
                    .as_array()
                    .unwrap_or(&vec![])
                    .iter()
                    .filter_map(|tx| {
                        // Check if this is a direct instruction object (has program_id, accounts, data)
                        if tx.get("program_id").is_some()
                            && tx.get("accounts").is_some()
                            && tx.get("data").is_some()
                        {
                            info!(
                                "[OpenAIAgent] Found direct instruction object in transactions array"
                            );
                            Some(tx.clone())
                        } else {
                            None
                        }
                    })
                    .collect();

                if !direct_instructions.is_empty() {
                    info!(
                        "[OpenAIAgent] Processing {} direct instruction objects",
                        direct_instructions.len()
                    );
                    info!(
                        "[OpenAIAgent] Debug - Direct instruction objects found: {:?}",
                        direct_instructions
                    );
                    return Ok(ExecutionResult {
                        transactions: direct_instructions,
                        summary: summary.as_str().unwrap_or("").to_string(),
                        signatures: signatures
                            .as_array()
                            .unwrap_or(&vec![])
                            .iter()
                            .filter_map(|s| s.as_str())
                            .map(|s| s.to_string())
                            .collect(),
                    });
                }

                // 🎯 CHECK FOR WRAPPED INSTRUCTION OBJECTS (jupiter_redeem format)
                // Handle case where transactions array contains objects with "instructions" field
                let wrapped_instructions: Vec<serde_json::Value> = transactions
                    .as_array()
                    .unwrap_or(&vec![])
                    .iter()
                    .filter_map(|tx| {
                        // Check if this is a wrapped instruction object with "instructions" field
                        if tx.get("instructions").is_some() {
                            info!(
                                "[OpenAIAgent] Found wrapped instruction object with instructions field"
                            );
                            // Extract the instructions array from the wrapped object
                            tx.get("instructions").map(|instructions| {
                                if instructions.is_array() {
                                    // Get the array of instruction objects
                                    instructions.as_array().unwrap_or(&vec![]).to_vec()
                                } else {
                                    // Handle single instruction object
                                    vec![instructions.clone()]
                                }
                            })
                        } else {
                            None
                        }
                    })
                    .flatten()
                    .collect();

                if !wrapped_instructions.is_empty() {
                    info!(
                        "[OpenAIAgent] Processing {} wrapped instruction objects",
                        wrapped_instructions.len()
                    );
                    info!(
                        "[OpenAIAgent] Debug - Wrapped instruction objects found: {:?}",
                        wrapped_instructions
                    );
                    return Ok(ExecutionResult {
                        transactions: wrapped_instructions,
                        summary: summary.as_str().unwrap_or("").to_string(),
                        signatures: signatures
                            .as_array()
                            .unwrap_or(&vec![])
                            .iter()
                            .filter_map(|s| s.as_str())
                            .map(|s| s.to_string())
                            .collect(),
                    });
                }

                // 🧠 HANDLE MARKDOWN JSON CODE BLOCKS - EXTRACT FROM WRAPPER
                let final_instructions: Vec<serde_json::Value> = transactions
                    .as_array()
                    .unwrap_or(&vec![])
                    .iter()
                    .filter_map(|tx| {
                        let tx_str = tx.as_str().unwrap_or_default();

                        // The transaction string contains escaped JSON
                        // It could be:
                        // 1. An array of instruction objects: "[{\"program_id\":...}]"
                        // 2. An object with instructions field: "{\"instructions\":[...]}"
                        match serde_json::from_str::<serde_json::Value>(tx_str) {
                            Ok(tx_obj) => {
                                info!("[OpenAIAgent] Successfully parsed escaped JSON transaction object");

                                // Check if this is an array of instruction objects (direct format)
                                if tx_obj.is_array() {
                                    info!("[OpenAIAgent] Found direct instruction array format");
                                    Some(tx_obj.as_array().unwrap_or(&vec![]).to_vec())
                                }
                                // Check if this is an object with "instructions" field (wrapped format)
                                else if let Some(instructions) = tx_obj.get("instructions") {
                                    info!("[OpenAIAgent] Found wrapped instruction object with instructions field");
                                    if instructions.is_array() {
                                        Some(instructions.as_array().unwrap_or(&vec![]).to_vec())
                                    } else {
                                        Some(vec![instructions.clone()])
                                    }
                                }
                                // Handle single instruction object
                                else {
                                    info!("[OpenAIAgent] Found single instruction object format");
                                    Some(vec![tx_obj])
                                }
                            }
                            Err(parse_error) => {
                                warn!(
                                    "[OpenAIAgent] Failed to parse escaped JSON transaction object: {}",
                                    parse_error
                                );
                                None
                            }
                        }
                    })
                    .flatten()
                    .collect();

                info!(
                    "[OpenAIAgent] Debug - Final instructions from escaped JSON: {:?}",
                    final_instructions
                );

                info!(
                    "[OpenAIAgent] Debug - Returning execution result with {} instructions",
                    final_instructions.len()
                );
                return Ok(ExecutionResult {
                    transactions: final_instructions,
                    summary: summary.as_str().unwrap_or("").to_string(),
                    signatures: signatures
                        .as_array()
                        .unwrap_or(&vec![])
                        .iter()
                        .filter_map(|s| s.as_str())
                        .map(|s| s.to_string())
                        .collect(),
                });
            }

            // 🎯 CHECK FOR STRUCTURED TOOL RESPONSES (JupiterSwapResponse, etc.)
            if let (
                Some(instructions),
                Some(transaction_count),
                Some(estimated_signatures),
                Some(operation_type),
            ) = (
                json_value.get("instructions"),
                json_value.get("transaction_count"),
                json_value.get("estimated_signatures"),
                json_value.get("operation_type"),
            ) {
                info!(
                    "[OpenAIAgent] Found structured tool response for {}",
                    operation_type
                );
                let tx_count = transaction_count.as_u64().unwrap_or(1) as usize;
                let signatures: Vec<String> = estimated_signatures
                    .as_array()
                    .unwrap_or(&vec![])
                    .iter()
                    .filter_map(|s| s.as_str())
                    .map(|s| s.to_string())
                    .collect();

                info!(
                    "[OpenAIAgent] Debug - Extracted {} instructions from structured response",
                    tx_count
                );
                return Ok(ExecutionResult {
                    transactions: instructions.as_array().unwrap_or(&vec![]).to_vec(),
                    summary: format!(
                        "Successfully executed {} {} operation(s)",
                        tx_count,
                        operation_type.as_str().unwrap_or("transaction")
                    ),
                    signatures,
                });
            }

            // Check if response contains tool calls or instruction data
            if let Some(instruction_field) = json_value.get("instruction") {
                info!("[OpenAIAgent] Found instruction field in response");
                Ok(ExecutionResult {
                    transactions: vec![instruction_field.clone()],
                    summary: format!("Executed {} transaction(s)", 1),
                    signatures: vec![], // Would need to be populated during actual execution
                })
            } else {
                // Wrap natural language response
                info!("[OpenAIAgent] Wrapping natural language response");
                Ok(ExecutionResult {
                    transactions: vec![],
                    summary: response_str.trim().to_string(),
                    signatures: vec![],
                })
            }
        }
        Err(parse_error) => {
            warn!(
                "[OpenAIAgent] Failed to parse JSON as structured response: {}",
                parse_error
            );
            info!("[OpenAIAgent] Debug - Falling back to natural language parsing");
            // Pure natural language response
            Ok(ExecutionResult {
                transactions: vec![],
                summary: response_str.trim().to_string(),
                signatures: vec![],
            })
        }
    }
}
