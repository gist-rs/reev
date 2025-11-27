use crate::context::{ContextResolver, SolanaEnvironment};
use anyhow::{anyhow, Result};
use reev_agent::enhanced::common::AgentTools;
use reev_lib::agent::{RawAccountMeta, RawInstruction};
use reev_lib::constants;
use reev_lib::utils::{execute_transaction, get_keypair};
use reev_tools::tools::jupiter_swap::JupiterSwapArgs;
use reev_types::flow::StepResult;
use rig::tool::Tool;
use serde_json::json;
use solana_sdk::signer::Signer;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info, warn};

/// Execute a direct Jupiter swap operation without expected parameters
pub async fn execute_direct_jupiter_swap(
    agent_tools: &Arc<AgentTools>,
    wallet_owner: &str,
    prompt: &str,
) -> Result<StepResult> {
    info!(
        "Executing direct Jupiter swap operation with prompt: {}",
        prompt
    );

    // Get SOL and USDC mint addresses
    let sol_mint = constants::sol_mint();
    let usdc_mint = constants::usdc_mint();

    // Parse prompt to extract swap parameters
    let prompt_lower = prompt.to_lowercase();

    // Default values
    let mut input_mint = sol_mint.to_string();
    let mut output_mint = usdc_mint.to_string();
    let mut amount = 100_000_000u64; // Default: 0.1 SOL

    // Extract input and output tokens
    if prompt_lower.contains("sol") && prompt_lower.contains("usdc") {
        if prompt_lower.contains("for usdc") {
            // SOL -> USDC
            input_mint = sol_mint.to_string();
            output_mint = usdc_mint.to_string();
        } else if prompt_lower.contains("for sol") || prompt_lower.contains("to sol") {
            // USDC -> SOL
            input_mint = usdc_mint.to_string();
            output_mint = sol_mint.to_string();
        }
    }

    // Check for "all" indicator first before extracting amount with regex
    if prompt_lower.contains("all") || prompt_lower.contains("all ") {
        // For "all" SOL or "ALL" indicator, get the actual wallet balance
        // Create a context resolver with explicit RPC URL
        let context_resolver = ContextResolver::new(SolanaEnvironment {
            rpc_url: Some("https://api.mainnet-beta.solana.com".to_string()),
        });

        // Get the actual wallet balance for the user
        if let Ok(wallet_context) = context_resolver.resolve_wallet_context(wallet_owner).await {
            if input_mint == sol_mint.to_string() {
                // Reserve 0.01 SOL for gas fees
                let gas_reserve = 10_000_000u64; // 0.01 SOL
                amount = if wallet_context.sol_balance > gas_reserve {
                    wallet_context.sol_balance - gas_reserve
                } else {
                    // If balance is less than gas reserve, use half of the balance
                    wallet_context.sol_balance / 2
                };
            } else if input_mint == usdc_mint.to_string() {
                // For USDC, find the USDC token balance
                if let Some(usdc_balance) = wallet_context
                    .token_balances
                    .get(&constants::usdc_mint().to_string())
                {
                    amount = usdc_balance.balance;
                } else {
                    amount = 100_000_000u64; // Default: 100 USDC
                }
            }
        } else {
            // Fallback to default values if we can't get wallet context
            if input_mint == sol_mint.to_string() {
                amount = 5_000_000_000u64; // 5 SOL
            } else if input_mint == usdc_mint.to_string() {
                amount = 100_000_000u64; // 100 USDC
            }
        }
    } else {
        // Extract amount from patterns like "0.1 sol" or "10 usdc"
        let re = regex::Regex::new(r"(\d+\.?\d*)\s*(sol|usdc)").unwrap();
        if let Some(captures) = re.captures(&prompt_lower) {
            if let (Some(amount_str), Some(token)) = (captures.get(1), captures.get(2)) {
                let amount_value: f64 = amount_str.as_str().parse().unwrap_or(0.0);
                let token_type = token.as_str();

                // Convert to raw amount based on token type
                if token_type == "sol" {
                    amount = (amount_value * 1_000_000_000.0) as u64;
                } else if token_type == "usdc" {
                    amount = (amount_value * 1_000_000.0) as u64;
                }
            }
        } else {
            // Default amount if no pattern matched
            amount = 100_000_000u64; // Default: 1 SOL
        }
    }

    // Create swap args with parsed values
    let swap_args = JupiterSwapArgs {
        user_pubkey: wallet_owner.to_string(),
        input_mint,
        output_mint,
        amount,
        slippage_bps: Some(100), // Default 1% slippage
    };

    info!("Executing JupiterSwapTool with args: {:?}", swap_args);

    // Execute the jupiter swap tool directly
    let result = agent_tools
        .jupiter_swap_tool
        .call(swap_args)
        .await
        .map_err(|e| anyhow!("JupiterSwap execution failed: {e}"))?;

    handle_jupiter_swap_result(Ok(result)).await
}

/// Handle Jupiter swap operation
pub async fn handle_jupiter_swap(
    params: &HashMap<String, serde_json::Value>,
    tools: &Arc<AgentTools>,
) -> Result<serde_json::Value> {
    info!("Executing JupiterSwap with parameters: {:?}", params);

    // Convert parameters to expected format for JupiterSwapTool
    let swap_args = JupiterSwapArgs {
        user_pubkey: params
            .get("user_pubkey")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        input_mint: params
            .get("input_mint")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        output_mint: params
            .get("output_mint")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        amount: params.get("amount").and_then(|v| v.as_u64()).unwrap_or(0),
        slippage_bps: params
            .get("slippage_bps")
            .and_then(|v| v.as_u64())
            .map(|v| v as u16),
    };

    let result = tools
        .jupiter_swap_tool
        .call(swap_args)
        .await
        .map_err(|e| anyhow!("JupiterSwap execution failed: {e}"))?;

    process_jupiter_swap_result(result).await
}

/// Process the result of a Jupiter swap operation
pub async fn process_jupiter_swap_result(result: String) -> Result<serde_json::Value> {
    // Parse the JSON response to extract structured data
    if let Ok(response) = serde_json::from_str::<serde_json::Value>(&result) {
        // Check if that tool prepared instructions
        if let Some(instructions) = response.get("instructions").and_then(|v| v.as_array()) {
            // Convert instructions to RawInstruction format
            let raw_instructions: Result<Vec<RawInstruction>> = instructions
                .iter()
                .map(|inst| {
                    let program_id = inst
                        .get("program_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| anyhow!("Missing program_id"))?
                        .to_string();

                    let accounts = inst
                        .get("accounts")
                        .and_then(|v| v.as_array())
                        .ok_or_else(|| anyhow!("Missing accounts"))?
                        .iter()
                        .map(|acc| {
                            Ok(RawAccountMeta {
                                pubkey: acc
                                    .get("pubkey")
                                    .and_then(|v| v.as_str())
                                    .ok_or_else(|| anyhow!("Missing pubkey"))?
                                    .to_string(),
                                is_signer: acc
                                    .get("is_signer")
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or(false),
                                is_writable: acc
                                    .get("is_writable")
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or(false),
                            })
                        })
                        .collect::<Result<Vec<_>>>()?;

                    let data = inst
                        .get("data")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| anyhow!("Missing data"))?
                        .to_string();

                    Ok(RawInstruction {
                        program_id,
                        accounts,
                        data,
                    })
                })
                .collect();

            // Call transaction processing function
            process_transaction_with_instructions(raw_instructions, response, "JupiterSwap").await
        } else {
            warn!("No instructions found in response");
            Err(anyhow!("No instructions found in response"))
        }
    } else {
        warn!("Failed to parse Jupiter swap tool response");
        Err(anyhow!("Failed to parse Jupiter swap tool response"))
    }
}

/// Handle the result of a direct Jupiter swap operation
pub async fn handle_jupiter_swap_result(
    result: Result<String, anyhow::Error>,
) -> Result<StepResult> {
    match result {
        Ok(response_json) => {
            info!("JupiterSwapTool executed successfully");

            // Parse the JSON response to extract instructions
            if let Ok(response) = serde_json::from_str::<serde_json::Value>(&response_json) {
                if let Some(instructions) = response.get("instructions").and_then(|v| v.as_array())
                {
                    info!("Found {} instructions in response", instructions.len());

                    // Convert instructions to RawInstruction format
                    let raw_instructions: Result<Vec<RawInstruction>> = instructions
                        .iter()
                        .map(|inst| {
                            let program_id = inst
                                .get("program_id")
                                .and_then(|v| v.as_str())
                                .ok_or_else(|| anyhow!("Missing program_id"))?
                                .to_string();

                            let accounts = inst
                                .get("accounts")
                                .and_then(|v| v.as_array())
                                .ok_or_else(|| anyhow!("Missing accounts"))?
                                .iter()
                                .map(|acc| {
                                    Ok(RawAccountMeta {
                                        pubkey: acc
                                            .get("pubkey")
                                            .and_then(|v| v.as_str())
                                            .ok_or_else(|| anyhow!("Missing pubkey"))?
                                            .to_string(),
                                        is_signer: acc
                                            .get("is_signer")
                                            .and_then(|v| v.as_bool())
                                            .unwrap_or(false),
                                        is_writable: acc
                                            .get("is_writable")
                                            .and_then(|v| v.as_bool())
                                            .unwrap_or(false),
                                    })
                                })
                                .collect::<Result<Vec<_>>>()?;

                            let data = inst
                                .get("data")
                                .and_then(|v| v.as_str())
                                .ok_or_else(|| anyhow!("Missing data"))?
                                .to_string();

                            Ok(RawInstruction {
                                program_id,
                                accounts,
                                data,
                            })
                        })
                        .collect();

                    // Call transaction processing function for StepResult
                    process_transaction_with_instructions_step_result(
                        raw_instructions,
                        response,
                        "jupiter_swap",
                    )
                    .await
                } else {
                    warn!("No instructions found in response");
                    Ok(StepResult {
                        step_id: uuid::Uuid::new_v4().to_string(),
                        success: false,
                        error_message: Some("No instructions found in response".to_string()),
                        tool_calls: vec!["jupiter_swap".to_string()],
                        output: json!({
                            "jupiter_swap": {
                                "error": "No instructions found in response",
                                "raw_response": response_json
                            }
                        }),
                        execution_time_ms: 1000,
                    })
                }
            } else {
                warn!("Failed to parse Jupiter swap tool response");
                Ok(StepResult {
                    step_id: uuid::Uuid::new_v4().to_string(),
                    success: false,
                    error_message: Some("Failed to parse response".to_string()),
                    tool_calls: vec!["jupiter_swap".to_string()],
                    output: json!({
                        "jupiter_swap": {
                            "error": "Failed to parse response",
                            "raw_response": response_json
                        }
                    }),
                    execution_time_ms: 1000,
                })
            }
        }
        Err(e) => {
            error!("JupiterSwapTool execution failed: {}", e);
            Ok(StepResult {
                step_id: uuid::Uuid::new_v4().to_string(),
                success: false,
                error_message: Some(format!("Tool execution failed: {e}")),
                tool_calls: vec!["jupiter_swap".to_string()],
                output: json!({
                    "jupiter_swap": {
                        "error": format!("Tool execution failed: {e}"),
                    }
                }),
                execution_time_ms: 1000,
            })
        }
    }
}

/// Process raw instructions and execute the transaction
async fn process_transaction_with_instructions(
    raw_instructions_result: Result<Vec<RawInstruction>>,
    response: serde_json::Value,
    tool_name: &str,
) -> Result<serde_json::Value> {
    match raw_instructions_result {
        Ok(instructions) => {
            let instructions_count = instructions.len();
            info!("Successfully parsed {} instructions", instructions_count);

            // Get keypair for signing
            let keypair = get_keypair().map_err(|e| anyhow!("Failed to load keypair: {e}"))?;

            // Execute the transaction
            let user_pubkey = Signer::pubkey(&keypair);
            match execute_transaction(instructions, user_pubkey, &keypair).await {
                Ok(signature) => {
                    info!(
                        "Transaction executed successfully with signature: {}",
                        signature
                    );
                    Ok(json!({
                        "tool_name": tool_name,
                        "transaction_signature": signature,
                        "instructions_count": instructions_count,
                        "response": response
                    }))
                }
                Err(e) => {
                    error!("Transaction execution failed: {}", e);
                    Ok(json!({
                        "tool_name": tool_name,
                        "error": format!("Transaction execution failed: {e}"),
                        "response": response
                    }))
                }
            }
        }
        Err(e) => {
            error!("Failed to parse instructions: {}", e);
            Ok(json!({
                "tool_name": tool_name,
                "error": format!("Failed to parse instructions: {e}"),
                "response": response
            }))
        }
    }
}

/// Process raw instructions and execute the transaction for StepResult
async fn process_transaction_with_instructions_step_result(
    raw_instructions_result: Result<Vec<RawInstruction>>,
    response: serde_json::Value,
    tool_name: &str,
) -> Result<StepResult> {
    match raw_instructions_result {
        Ok(instructions) => {
            let instructions_count = instructions.len();
            info!("Successfully parsed {} instructions", instructions_count);

            // Get keypair for signing
            let keypair = get_keypair().map_err(|e| anyhow!("Failed to load keypair: {e}"))?;

            // Execute the transaction
            let user_pubkey = Signer::pubkey(&keypair);
            match execute_transaction(instructions, user_pubkey, &keypair).await {
                Ok(signature) => {
                    info!(
                        "Transaction executed successfully with signature: {}",
                        signature
                    );
                    Ok(StepResult {
                        step_id: uuid::Uuid::new_v4().to_string(),
                        success: true,
                        error_message: None,
                        tool_calls: vec![tool_name.to_string()],
                        output: json!({
                            "jupiter_swap": {
                                "transaction_signature": signature,
                                "instructions_count": instructions_count,
                                "full_response": response
                            }
                        }),
                        execution_time_ms: 1000,
                    })
                }
                Err(e) => {
                    error!("Transaction execution failed: {}", e);
                    Ok(StepResult {
                        step_id: uuid::Uuid::new_v4().to_string(),
                        success: false,
                        error_message: Some(format!("Transaction execution failed: {e}")),
                        tool_calls: vec![tool_name.to_string()],
                        output: json!({
                            tool_name: {
                                "error": format!("Transaction execution failed: {e}"),
                                "raw_response": response
                            }
                        }),
                        execution_time_ms: 1000,
                    })
                }
            }
        }
        Err(e) => {
            error!("Failed to parse instructions: {}", e);
            Ok(StepResult {
                step_id: uuid::Uuid::new_v4().to_string(),
                success: false,
                error_message: Some(format!("Failed to parse instructions: {e}")),
                tool_calls: vec![tool_name.to_string()],
                output: json!({
                    tool_name: {
                        "error": format!("Failed to parse instructions: {e}"),
                        "raw_response": response
                    }
                }),
                execution_time_ms: 1000,
            })
        }
    }
}
