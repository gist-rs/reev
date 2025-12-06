//! Jupiter Lend/Earn Deposit Tool Implementation
//!
//! This module contains implementation of the Jupiter lend/earn deposit tool.

use anyhow::{anyhow, Result};
use reev_agent::enhanced::common::AgentTools;
use reev_types::flow::WalletContext;
use rig::tool::Tool;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info};

use super::tool_params::JupiterLendParams;
use super::tool_results::{JupiterLendResult, ToolResult};

/// Execute Jupiter lend/earn deposit
pub async fn execute_jupiter_lend_deposit(
    params: &HashMap<String, String>,
    wallet_context: &WalletContext,
    agent_tools: Arc<AgentTools>,
) -> Result<ToolResult> {
    // Parse parameters into typed struct
    let lend_params = JupiterLendParams::from_hashmap(params)?;
    let mint = &lend_params.mint;
    let amount_str = &lend_params.amount;

    debug!(
        "DEBUG: execute_jupiter_lend_deposit received amount_str: {}",
        amount_str
    );

    let amount: f64 = amount_str
        .parse()
        .map_err(|_| anyhow!("Invalid amount: {amount_str}"))?;

    debug!(
        "DEBUG: execute_jupiter_lend_deposit parsed amount as f64: {}",
        amount
    );
    debug!(
        "DEBUG: execute_jupiter_lend_deposit casting to u64: {}",
        amount as u64
    );

    // Check if the amount is already in lamports or needs conversion
    let amount_lamports = if amount > 1_000_000.0 {
        // Amount is likely already in lamports (for USDC/USDT)
        debug!("DEBUG: Amount appears to be in lamports: {}", amount as u64);
        amount as u64
    } else {
        // Amount is likely in human-readable format, convert to lamports
        debug!(
            "DEBUG: Converting amount to lamports: {}",
            amount * 1_000_000.0
        );
        (amount * 1_000_000.0) as u64
    };

    debug!("DEBUG: Final amount for Jupiter lend: {}", amount_lamports);

    // Execute Jupiter Lend Earn Deposit using AgentTools
    // Note: The amount is already in the correct units (smallest denomination)
    // as provided by the LLM, so we don't need to multiply by 1_000_000
    let deposit_args = reev_tools::tools::jupiter_lend_earn_deposit::JupiterLendEarnDepositArgs {
        user_pubkey: wallet_context.owner.clone(),
        asset_mint: mint.clone(),
        amount: amount_lamports,
    };

    let result = agent_tools
        .jupiter_lend_earn_deposit_tool
        .call(deposit_args)
        .await
        .map_err(|e| anyhow!("Jupiter Lend Earn Deposit execution failed: {e}"))?;

    // Parse the response to extract instructions and execute transaction
    info!("Jupiter lend deposit tool returned result: {}", &result);
    if let Ok(response) = serde_json::from_str::<serde_json::Value>(&result) {
        debug!("Parsed response: {:#?}", response);

        // The response is a serialized Vec<RawInstruction>, let's convert it
        let raw_instructions: Result<Vec<reev_lib::agent::RawInstruction>> = response
            .as_array()
            .unwrap_or(&vec![])
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
                        Ok(reev_lib::agent::RawAccountMeta {
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

                Ok(reev_lib::agent::RawInstruction {
                    program_id,
                    accounts,
                    data,
                })
            })
            .collect();

        // Execute transaction with the instructions
        match raw_instructions {
            Ok(instructions) => {
                info!(
                    "DEBUG: About to execute Jupiter lend transaction with {} instructions",
                    instructions.len()
                );
                let keypair =
                    reev_lib::get_keypair().map_err(|e| anyhow!("Failed to load keypair: {e}"))?;
                let user_pubkey = solana_sdk::signer::Signer::pubkey(&keypair);

                match reev_lib::utils::execute_transaction(instructions, user_pubkey, &keypair)
                    .await
                {
                    Ok(signature) => {
                        info!(
                            "Jupiter lend deposit transaction executed with signature: {}",
                            signature
                        );
                        Ok(ToolResult::JupiterLend(JupiterLendResult {
                            tool_name: "jupiter_lend_earn_deposit".to_string(),
                            mint: mint.clone(),
                            amount: amount_lamports,
                            wallet: wallet_context.owner.clone(),
                            transaction_signature: Some(signature),
                            success: true,
                            error: None,
                        }))
                    }
                    Err(e) => {
                        error!("Failed to execute Jupiter lend deposit transaction: {}", e);
                        debug!("Transaction execution error details: {:#?}", e);

                        Ok(ToolResult::JupiterLend(JupiterLendResult {
                            tool_name: "jupiter_lend_earn_deposit".to_string(),
                            mint: mint.clone(),
                            amount: amount as u64,
                            wallet: wallet_context.owner.clone(),
                            transaction_signature: None,
                            success: false,
                            error: Some(format!("Transaction execution failed: {e}")),
                        }))
                    }
                }
            }
            Err(e) => {
                error!("Failed to parse Jupiter lend deposit instructions: {}", e);
                Ok(ToolResult::JupiterLend(JupiterLendResult {
                    tool_name: "jupiter_lend_earn_deposit".to_string(),
                    mint: mint.clone(),
                    amount: amount as u64,
                    wallet: wallet_context.owner.clone(),
                    transaction_signature: None,
                    success: false,
                    error: Some(format!("Failed to parse instructions: {e}")),
                }))
            }
        }
    } else {
        error!("Failed to parse Jupiter lend deposit response as JSON");
        Ok(ToolResult::JupiterLend(JupiterLendResult {
            tool_name: "jupiter_lend_earn_deposit".to_string(),
            mint: mint.clone(),
            amount: amount as u64,
            wallet: wallet_context.owner.clone(),
            transaction_signature: None,
            success: false,
            error: Some("Failed to parse response as JSON".to_string()),
        }))
    }
}
