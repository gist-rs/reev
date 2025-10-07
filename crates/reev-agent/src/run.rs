use anyhow::Result;
use rig::{
    completion::Prompt,
    prelude::*,
    providers::{gemini, openai::Client},
};
use serde::Deserialize;
use serde_json::json;
use spl_associated_token_account;
use spl_token;
use std::collections::HashMap;
use tracing::info;

use crate::{
    prompt::SYSTEM_PREAMBLE,
    tools::{
        JupiterEarnTool, JupiterLendDepositTool, JupiterLendWithdrawTool, JupiterMintTool,
        JupiterRedeemTool, JupiterSwapTool, SolTransferTool, SplTransferTool,
    },
    LlmRequest,
};

/// A minimal struct for deserializing the `key_map` from the `context_prompt` YAML.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct AgentContext {
    key_map: HashMap<String, String>,
}

/// Dispatches the request to the appropriate agent based on the model name.
/// It first parses the on-chain context to provide it to the tools that need it.
pub async fn run_agent(model_name: &str, payload: LlmRequest) -> Result<String> {
    // If mock is enabled, use deterministic agent instead
    if payload.mock {
        info!("[run_agent] Mock mode enabled, routing to deterministic agent");
        let response = crate::run_deterministic_agent(payload).await?;
        return Ok(serde_json::to_string(&response.0)?);
    }
    // Parse the context_prompt to extract the key_map, which is needed by the JupiterSwapTool
    // to correctly identify mock mints.
    let yaml_str = payload
        .context_prompt
        .trim_start_matches("---\n\nCURRENT ON-CHAIN CONTEXT:\n")
        .trim_end_matches("\n\n\n---")
        .trim();

    // If parsing fails, default to an empty map. This allows the agent to function
    // even with a malformed or missing context, though tools needing it may fail.
    let context: AgentContext = serde_yaml::from_str(yaml_str).unwrap_or(AgentContext {
        key_map: HashMap::new(),
    });
    let key_map = context.key_map;

    if model_name.starts_with("gemini") {
        info!("[reev-agent] Using Gemini agent for model: {model_name}");
        run_gemini_agent(model_name, payload, key_map).await
    } else {
        info!("[reev-agent] Using OpenAI compat agent for model: {model_name}");
        run_openai_compatible_agent(model_name, payload, key_map).await
    }
}

/// Runs the AI agent logic using a Google Gemini model.
async fn run_gemini_agent(
    model_name: &str,
    payload: LlmRequest,
    key_map: HashMap<String, String>,
) -> Result<String> {
    let client = gemini::Client::from_env();

    let gen_cfg = gemini::completion::gemini_api_types::GenerationConfig {
        temperature: Some(0.0),
        ..Default::default()
    };
    let cfg =
        gemini::completion::gemini_api_types::AdditionalParameters::default().with_config(gen_cfg);

    // Instantiate the JupiterSwapTool with the context-aware key_map.
    let jupiter_swap_tool = JupiterSwapTool {
        key_map: key_map.clone(),
    };
    let jupiter_lend_deposit_tool = JupiterLendDepositTool {
        key_map: key_map.clone(),
    };
    let jupiter_lend_withdraw_tool = JupiterLendWithdrawTool {
        key_map: key_map.clone(),
    };
    let jupiter_mint_tool = JupiterMintTool {
        key_map: key_map.clone(),
    };
    let jupiter_redeem_tool = JupiterRedeemTool {
        key_map: key_map.clone(),
    };
    let jupiter_positions_tool = JupiterEarnTool {
        key_map: key_map.clone(),
    };
    let jupiter_earnings_tool = JupiterEarnTool {
        key_map: key_map.clone(),
    };

    let sol_tool = SolTransferTool {
        key_map: key_map.clone(),
    };
    let spl_tool = SplTransferTool { key_map };

    let agent = client
        .agent(model_name)
        .preamble(SYSTEM_PREAMBLE)
        .additional_params(serde_json::to_value(cfg)?)
        .tool(sol_tool)
        .tool(spl_tool)
        .tool(jupiter_swap_tool)
        .tool(jupiter_lend_deposit_tool)
        .tool(jupiter_lend_withdraw_tool)
        .tool(jupiter_mint_tool)
        .tool(jupiter_redeem_tool)
        .tool(jupiter_positions_tool)
        .tool(jupiter_earnings_tool)
        .build();

    let full_prompt = format!(
        "{}\n\nUSER REQUEST: {}",
        payload.context_prompt, payload.prompt
    );

    let response = agent.prompt(&full_prompt).await?;

    info!(
        "[reev-agent] Raw response from rig: {}",
        response.to_string()
    );

    // The `rig` agent returns a JSON string from the tool call. This might be a raw
    // array of instructions, or a JSON object containing an `instruction` field. We
    // handle both cases to support different model behaviors.
    let tool_call_response: serde_json::Value = serde_json::from_str(&response.to_string())?;
    let instruction = if let Some(instruction_field) = tool_call_response.get("instruction") {
        instruction_field
    } else {
        &tool_call_response
    };

    Ok(serde_json::to_string(instruction)?)
}

/// Runs the AI agent logic using a local lmstudio model locally.
async fn run_openai_compatible_agent(
    model_name: &str,
    payload: LlmRequest,
    key_map: HashMap<String, String>,
) -> Result<String> {
    let client = Client::builder("")
        .base_url("http://localhost:1234/v1")
        .build()?;

    // Instantiate the JupiterSwapTool with the context-aware key_map.
    let jupiter_swap_tool = JupiterSwapTool {
        key_map: key_map.clone(),
    };
    let jupiter_lend_deposit_tool = JupiterLendDepositTool {
        key_map: key_map.clone(),
    };
    let jupiter_lend_withdraw_tool = JupiterLendWithdrawTool {
        key_map: key_map.clone(),
    };
    let jupiter_mint_tool = JupiterMintTool {
        key_map: key_map.clone(),
    };
    let jupiter_redeem_tool = JupiterRedeemTool {
        key_map: key_map.clone(),
    };
    let jupiter_positions_tool = JupiterEarnTool {
        key_map: key_map.clone(),
    };
    let jupiter_earnings_tool = JupiterEarnTool {
        key_map: key_map.clone(),
    };

    let sol_tool = SolTransferTool {
        key_map: key_map.clone(),
    };
    let spl_tool = SplTransferTool {
        key_map: key_map.clone(),
    };

    let agent = client
        .completion_model(model_name)
        .completions_api()
        .into_agent_builder()
        .preamble(SYSTEM_PREAMBLE)
        .tool(sol_tool)
        .tool(spl_tool)
        .tool(jupiter_swap_tool)
        .tool(jupiter_lend_deposit_tool)
        .tool(jupiter_lend_withdraw_tool)
        .tool(jupiter_mint_tool)
        .tool(jupiter_redeem_tool)
        .tool(jupiter_positions_tool)
        .tool(jupiter_earnings_tool)
        .build();

    let full_prompt = format!(
        "{}\n\nUSER REQUEST: {}",
        payload.context_prompt, payload.prompt
    );

    let response = agent.prompt(&full_prompt).multi_turn(3).await?;

    info!(
        "[reev-agent] Raw response from rig: {}",
        response.to_string()
    );

    // The `rig` agent returns a JSON string from the tool call. This might be a raw
    // array of instructions, or a JSON object containing an `instruction` field. We
    // handle both cases to support different model behaviors.
    let tool_call_response: serde_json::Value = serde_json::from_str(&response.to_string())?;

    // Check if this is a tool call response that needs to be executed
    if let Some(method) = tool_call_response.get("method").and_then(|m| m.as_str()) {
        info!("[reev-agent] Detected tool call: {}", method);

        // Execute the appropriate tool based on the method
        match method {
            "sol_transfer" | "spl_transfer" => {
                // For transfer tools, we need to execute them to get the instructions
                if let Some(params) = tool_call_response.get("params") {
                    let tool_result = execute_native_transfer(method, params, &key_map).await?;
                    return Ok(tool_result);
                }
            }
            "jupiter_swap" => {
                if let Some(params) = tool_call_response.get("params") {
                    let tool_result = execute_jupiter_swap(params, &key_map).await?;
                    return Ok(tool_result);
                }
            }
            "jupiter_lend_deposit" | "jupiter_lend_withdraw" => {
                if let Some(params) = tool_call_response.get("params") {
                    let tool_result = execute_jupiter_lend(method, params, &key_map).await?;
                    return Ok(tool_result);
                }
            }
            "jupiter_mint" => {
                if let Some(params) = tool_call_response.get("params") {
                    let tool_result = execute_jupiter_mint(params, &key_map).await?;
                    return Ok(tool_result);
                }
            }
            "jupiter_redeem" => {
                if let Some(params) = tool_call_response.get("params") {
                    let tool_result = execute_jupiter_redeem(params, &key_map).await?;
                    return Ok(tool_result);
                }
            }
            "jupiter_earn" => {
                // For jupiter_earn, this might be an API-based benchmark
                // Return the raw response since it doesn't contain instructions
                return Ok(response.to_string());
            }
            _ => {
                info!("[reev-agent] Unknown tool method: {}", method);
            }
        }
    }

    // Check if this is a Jupiter tool response with instructions field
    if let Some(instructions) = tool_call_response
        .get("instructions")
        .and_then(|i| i.as_array())
    {
        info!(
            "[reev-agent] Detected Jupiter tool response with {} instructions",
            instructions.len()
        );
        return Ok(serde_json::to_string(instructions)?);
    }

    // Check if this is a tool response without method field (direct tool call result)
    // Look for fields that indicate a transfer operation
    if tool_call_response.get("amount").is_some() && tool_call_response.get("operation").is_some() {
        info!("[reev-agent] Detected transfer operation in response");
        let tool_result = execute_native_transfer("sol", &tool_call_response, &key_map).await?;
        return Ok(tool_result);
    }

    // If not a tool call or tool execution failed, return the response as-is
    let instruction = if let Some(instruction_field) = tool_call_response.get("instruction") {
        instruction_field
    } else {
        &tool_call_response
    };

    Ok(serde_json::to_string(instruction)?)
}

/// Helper function to execute native transfer tools
async fn execute_native_transfer(
    method: &str,
    params: &serde_json::Value,
    key_map: &HashMap<String, String>,
) -> Result<String> {
    use solana_sdk::pubkey::Pubkey;
    use solana_system_interface::instruction as system_instruction;
    use std::str::FromStr;

    let user_pubkey = key_map.get("USER_WALLET_PUBKEY").unwrap();
    let recipient_pubkey = params
        .get("recipient_pubkey")
        .and_then(|p| p.as_str())
        .unwrap();
    let amount = params.get("amount").and_then(|a| a.as_u64()).unwrap_or(0);

    match method {
        "sol" => {
            // Generate real SOL transfer instruction
            let instruction = system_instruction::transfer(
                &Pubkey::from_str(user_pubkey)?,
                &Pubkey::from_str(recipient_pubkey)?,
                amount,
            );

            let raw_instruction = json!({
                "program_id": "11111111111111111111111111111111",
                "accounts": [
                    {
                        "pubkey": user_pubkey,
                        "is_signer": true,
                        "is_writable": true
                    },
                    {
                        "pubkey": recipient_pubkey,
                        "is_signer": false,
                        "is_writable": true
                    }
                ],
                "data": bs58::encode(instruction.data).into_string()
            });
            Ok(serde_json::to_string(&[raw_instruction])?)
        }
        "spl" => {
            // Generate real SPL transfer instruction
            let mint_address = params
                .get("mint_address")
                .and_then(|m| m.as_str())
                .unwrap_or("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"); // USDC default

            let source = spl_associated_token_account::get_associated_token_address(
                &Pubkey::from_str(user_pubkey)?,
                &Pubkey::from_str(mint_address)?,
            );
            let destination = spl_associated_token_account::get_associated_token_address(
                &Pubkey::from_str(recipient_pubkey)?,
                &Pubkey::from_str(mint_address)?,
            );

            let instruction = spl_token::instruction::transfer(
                &spl_token::id(),
                &source,
                &destination,
                &Pubkey::from_str(user_pubkey)?,
                &[],
                amount,
            )?;

            let raw_instruction = json!({
                "program_id": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
                "accounts": [
                    {
                        "pubkey": source.to_string(),
                        "is_signer": false,
                        "is_writable": true
                    },
                    {
                        "pubkey": destination.to_string(),
                        "is_signer": false,
                        "is_writable": true
                    },
                    {
                        "pubkey": user_pubkey,
                        "is_signer": true,
                        "is_writable": false
                    }
                ],
                "data": bs58::encode(instruction.data).into_string()
            });
            Ok(serde_json::to_string(&[raw_instruction])?)
        }
        _ => {
            // Default to SOL transfer
            let instruction = system_instruction::transfer(
                &Pubkey::from_str(user_pubkey)?,
                &Pubkey::from_str(recipient_pubkey)?,
                amount,
            );

            let raw_instruction = json!({
                "program_id": "11111111111111111111111111111111",
                "accounts": [
                    {
                        "pubkey": user_pubkey,
                        "is_signer": true,
                        "is_writable": true
                    },
                    {
                        "pubkey": recipient_pubkey,
                        "is_signer": false,
                        "is_writable": true
                    }
                ],
                "data": bs58::encode(instruction.data).into_string()
            });
            Ok(serde_json::to_string(&[raw_instruction])?)
        }
    }
}

/// Helper function to execute Jupiter swap tool
async fn execute_jupiter_swap(
    _params: &serde_json::Value,
    key_map: &HashMap<String, String>,
) -> Result<String> {
    // For now, return dummy instructions
    let instructions = json!([
        {
            "program_id": "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4",
            "accounts": [
                {"pubkey": key_map.get("USER_WALLET_PUBKEY").unwrap_or(&"unknown".to_string()), "is_signer": true, "is_writable": true},
                {"pubkey": "So11111111111111111111111111111111111111112", "is_signer": false, "is_writable": false},
                {"pubkey": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "is_signer": false, "is_writable": false}
            ],
            "data": "11111111111111111111111111111111"
        }
    ]);
    Ok(serde_json::to_string(&instructions)?)
}

/// Helper function to execute Jupiter lend tools
async fn execute_jupiter_lend(
    _method: &str,
    _params: &serde_json::Value,
    key_map: &HashMap<String, String>,
) -> Result<String> {
    // For now, return dummy instructions
    let instructions = json!([
        {
            "program_id": "jup3YeL8QhtSx1e253b2FDvsMNC87fDrgQZivbrndc9",
            "accounts": [
                {"pubkey": key_map.get("USER_WALLET_PUBKEY").unwrap_or(&"unknown".to_string()), "is_signer": true, "is_writable": true},
                {"pubkey": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "is_signer": false, "is_writable": false}
            ],
            "data": "11111111111111111111111111111111"
        }
    ]);
    Ok(serde_json::to_string(&instructions)?)
}

/// Helper function to execute Jupiter mint tool
async fn execute_jupiter_mint(
    params: &serde_json::Value,
    key_map: &HashMap<String, String>,
) -> Result<String> {
    use jup_sdk::api::get_mint_instructions;

    let asset = params
        .get("asset")
        .and_then(|a| a.as_str())
        .unwrap_or("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")
        .to_string();
    let signer = key_map
        .get("USER_WALLET_PUBKEY")
        .unwrap_or(&"unknown".to_string())
        .clone();
    let shares = params
        .get("shares")
        .and_then(|s| s.as_u64())
        .unwrap_or(1000000);

    let response = get_mint_instructions(asset.to_string(), signer.clone(), shares)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get mint instructions: {e}"))?;

    // Convert InstructionData to JSON string
    let instructions_json = convert_instructions_to_json(&response.instructions)?;
    Ok(instructions_json)
}

/// Helper function to execute Jupiter redeem tool
async fn execute_jupiter_redeem(
    params: &serde_json::Value,
    key_map: &HashMap<String, String>,
) -> Result<String> {
    use jup_sdk::api::get_redeem_instructions;

    let asset = params
        .get("asset")
        .and_then(|a| a.as_str())
        .unwrap_or("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")
        .to_string();
    let signer = key_map
        .get("USER_WALLET_PUBKEY")
        .unwrap_or(&"unknown".to_string())
        .clone();
    let shares = params
        .get("shares")
        .and_then(|s| s.as_u64())
        .unwrap_or(1000000);

    let response = get_redeem_instructions(asset.to_string(), signer.clone(), shares)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get redeem instructions: {e}"))?;

    // Convert InstructionData to JSON string
    let instructions_json = convert_instructions_to_json(&response.instructions)?;
    Ok(instructions_json)
}

/// Helper function to convert Jupiter InstructionData to JSON string
fn convert_instructions_to_json(
    instructions: &[jup_sdk::models::InstructionData],
) -> Result<String> {
    let converted: Vec<serde_json::Value> = instructions
        .iter()
        .map(|inst| {
            serde_json::json!({
                "program_id": inst.program_id,
                "accounts": inst.accounts.iter().map(|acc| {
                    serde_json::json!({
                        "pubkey": acc.pubkey,
                        "is_signer": acc.is_signer,
                        "is_writable": acc.is_writable
                    })
                }).collect::<Vec<_>>(),
                "data": inst.data
            })
        })
        .collect();

    serde_json::to_string(&converted)
        .map_err(|e| anyhow::anyhow!("Failed to serialize instructions: {e}"))
}
