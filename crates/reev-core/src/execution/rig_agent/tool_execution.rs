//! Tool Execution for RigAgent
//!
//! This module contains methods for executing individual blockchain operations.

use anyhow::{anyhow, Result};
use reev_agent::enhanced::common::AgentTools;
use reev_protocols::native::handle_sol_transfer;
use reev_types::flow::WalletContext;
use serde_json::json;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tracing::{debug, error, info};

use super::prompting::HttpProvider;
use rig::tool::Tool;

/// Trait for tool execution operations
#[allow(async_fn_in_trait)]
pub trait ToolExecutor {
    /// Execute the selected tools
    async fn execute_tools(
        &self,
        tool_calls: HashMap<String, serde_json::Value>,
        wallet_context: &WalletContext,
    ) -> Result<Vec<serde_json::Value>>;

    /// Execute a single tool
    async fn execute_single_tool(
        &self,
        tool_name: &str,
        params: serde_json::Value,
        wallet_context: &WalletContext,
    ) -> Result<serde_json::Value>;

    /// Execute SOL transfer
    async fn execute_sol_transfer(
        &self,
        params: &HashMap<String, String>,
        wallet_context: &WalletContext,
    ) -> Result<serde_json::Value>;

    /// Execute Jupiter swap
    async fn execute_jupiter_swap(
        &self,
        params: &HashMap<String, String>,
        wallet_context: &WalletContext,
    ) -> Result<serde_json::Value>;

    /// Execute Jupiter lend/earn deposit
    async fn execute_jupiter_lend_deposit(
        &self,
        params: &HashMap<String, String>,
        wallet_context: &WalletContext,
    ) -> Result<serde_json::Value>;

    /// Execute get account balance
    async fn execute_get_account_balance(
        &self,
        params: &HashMap<String, String>,
        wallet_context: &WalletContext,
    ) -> Result<serde_json::Value>;
}

/// Trait for accessing agent tools and HTTP client
pub trait AgentProvider {
    fn agent_tools(&self) -> Option<Arc<AgentTools>>;
}

/// Implementation for any struct with agent_tools field
impl<T> ToolExecutor for T
where
    T: AgentProvider + HttpProvider,
{
    /// Execute the selected tools
    async fn execute_tools(
        &self,
        tool_calls: HashMap<String, serde_json::Value>,
        wallet_context: &WalletContext,
    ) -> Result<Vec<serde_json::Value>> {
        let mut results = Vec::new();

        for (tool_name, params) in tool_calls {
            let result = self
                .execute_single_tool(&tool_name, params, wallet_context)
                .await?;
            results.push(result);
        }

        Ok(results)
    }

    /// Execute a single tool
    async fn execute_single_tool(
        &self,
        tool_name: &str,
        params: serde_json::Value,
        wallet_context: &WalletContext,
    ) -> Result<serde_json::Value> {
        // Execute the tool using the agent's tool_set
        debug!("Executing tool {} with params: {}", tool_name, params);

        // Convert the parameters to a string map
        let mut params_map = HashMap::new();
        if let serde_json::Value::Object(map) = &params {
            for (key, value) in map {
                if let Some(str_value) = value.as_str() {
                    params_map.insert(key.clone(), str_value.to_string());
                } else {
                    // Handle numeric values more carefully to avoid scientific notation issues
                    match value {
                        serde_json::Value::Number(n) => {
                            if let Some(u) = n.as_u64() {
                                // For u64 values, use directly to avoid scientific notation
                                params_map.insert(key.clone(), u.to_string());
                            } else if let Some(i) = n.as_i64() {
                                // For i64 values, use directly to avoid scientific notation
                                params_map.insert(key.clone(), i.to_string());
                            } else if let Some(f) = n.as_f64() {
                                // For floating point values, format without scientific notation
                                // Check if it's an integer value first to preserve precision
                                if f.fract() == 0.0 && f.abs() < (i64::MAX as f64) {
                                    params_map.insert(key.clone(), (f as i64).to_string());
                                } else {
                                    params_map.insert(key.clone(), f.to_string());
                                }
                            } else {
                                params_map.insert(key.clone(), value.to_string());
                            }
                        }
                        _ => {
                            params_map.insert(key.clone(), value.to_string());
                        }
                    }
                }
            }
        }

        // Execute the tool based on its name
        match tool_name {
            "sol_transfer" => self.execute_sol_transfer(&params_map, wallet_context).await,
            "jupiter_swap" => self.execute_jupiter_swap(&params_map, wallet_context).await,
            "jupiter_lend_earn_deposit" => {
                self.execute_jupiter_lend_deposit(&params_map, wallet_context)
                    .await
            }
            "get_account_balance" => {
                self.execute_get_account_balance(&params_map, wallet_context)
                    .await
            }
            _ => Ok(json!({
                "tool_name": tool_name,
                "params": params,
                "error": format!("Unknown tool: {tool_name}")
            })),
        }
    }

    /// Execute SOL transfer
    async fn execute_sol_transfer(
        &self,
        params: &HashMap<String, String>,
        wallet_context: &WalletContext,
    ) -> Result<serde_json::Value> {
        let recipient = params
            .get("recipient")
            .ok_or_else(|| anyhow!("recipient parameter is required"))?;

        let amount_str = params
            .get("amount")
            .ok_or_else(|| anyhow!("amount parameter is required"))?;

        // Handle "all" keyword case
        let amount: f64 = if amount_str.to_lowercase() == "all" {
            // Calculate transfer amount as wallet balance minus gas reserve
            // Reserve 0.05 SOL for transaction fees (5,000,000 lamports)
            let gas_reserve = 5_000_000u64;

            // Ensure we don't try to transfer more than available
            let available_balance = if wallet_context.sol_balance > gas_reserve {
                wallet_context.sol_balance - gas_reserve
            } else {
                // If balance is less than or equal to gas reserve, transfer half
                wallet_context.sol_balance / 2
            };

            available_balance as f64 / 1_000_000_000.0
        } else {
            amount_str
                .parse()
                .map_err(|_| anyhow!("Invalid amount: {amount_str}"))?
        };

        // Parse recipient and sender pubkeys
        let recipient_pubkey =
            Pubkey::from_str(recipient).map_err(|e| anyhow!("Invalid recipient address: {e}"))?;
        let sender_pubkey = Pubkey::from_str(&wallet_context.owner)
            .map_err(|e| anyhow!("Invalid sender address: {e}"))?;

        // Convert amount to lamports for the protocol handler
        let amount_lamports = (amount * 1_000_000_000.0) as u64;

        // For "all" keyword, pass u64::MAX to let the protocol handler calculate the amount
        let transfer_amount = if amount_str.to_lowercase() == "all" {
            u64::MAX
        } else {
            amount_lamports
        };

        // Use the newer protocol handler from reev-protocols
        let instructions =
            handle_sol_transfer(sender_pubkey, recipient_pubkey, transfer_amount).await?;

        // Get the default keypair for signing
        let keypair = reev_lib::get_keypair().map_err(|e| anyhow!("Failed to get keypair: {e}"))?;

        // Execute the transaction
        let transaction_signature =
            reev_lib::execute_transaction(instructions, sender_pubkey, &keypair)
                .await
                .map_err(|e| anyhow!("Failed to execute transaction: {e}"))?;

        // Log the successful transaction
        info!(
            "SOL transfer executed with signature: {}",
            transaction_signature
        );

        Ok(json!({
            "tool_name": "sol_transfer",
            "params": {
                "recipient": recipient,
                "amount": amount,
                "amount_lamports": amount_lamports,
                "wallet": wallet_context.owner
            },
            "transaction_signature": transaction_signature,
            "success": true
        }))
    }

    /// Execute Jupiter swap
    async fn execute_jupiter_swap(
        &self,
        params: &HashMap<String, String>,
        wallet_context: &WalletContext,
    ) -> Result<serde_json::Value> {
        let input_mint = params
            .get("input_mint")
            .ok_or_else(|| anyhow!("input_mint parameter is required"))?;

        let output_mint = params
            .get("output_mint")
            .ok_or_else(|| anyhow!("output_mint parameter is required"))?;

        let amount_str = params
            .get("input_amount")
            .or_else(|| params.get("amount"))
            .ok_or_else(|| anyhow!("input_amount parameter is required"))?;

        // Check if amount is "all" before parsing to float
        let is_all_amount = amount_str.to_lowercase() == "all";

        let amount: f64 = if is_all_amount {
            // For "all", use the SOL balance directly
            wallet_context.sol_balance as f64 / 1_000_000_000.0
        } else {
            amount_str
                .parse()
                .map_err(|_| anyhow!("Invalid amount: {amount_str}"))?
        };

        // Convert amount to lamports (1 SOL = 1,000,000,000 lamports)
        let amount_lamports = (amount * 1_000_000_000.0) as u64;

        // Parse the mint addresses
        let input_mint_pubkey =
            Pubkey::from_str(input_mint).map_err(|e| anyhow!("Invalid input mint: {e}"))?;
        let output_mint_pubkey =
            Pubkey::from_str(output_mint).map_err(|e| anyhow!("Invalid output mint: {e}"))?;

        // Parse user pubkey
        let user_pubkey = Pubkey::from_str(&wallet_context.owner)
            .map_err(|e| anyhow!("Invalid user pubkey: {e}"))?;

        // Use full balance if amount is "all", otherwise use specified amount
        let final_amount_lamports = if is_all_amount {
            // Reserve 0.01 SOL for gas fees
            wallet_context
                .sol_balance
                .saturating_sub(reev_lib::constants::amounts::tokens::sol::JUPITER_SWAP_FEE_RESERVE)
        // Reserve 0.01 SOL for fees
        } else {
            amount_lamports
        };

        // Use the newer protocol handler from reev-protocols
        let instructions = reev_protocols::jupiter::swap::handle_jupiter_swap(
            user_pubkey,
            input_mint_pubkey,
            output_mint_pubkey,
            final_amount_lamports,
            100, // Default 1% slippage
        )
        .await
        .map_err(|e| anyhow!("Failed to prepare swap transaction: {e}"))?;

        // Get the default keypair for signing
        let keypair = reev_lib::get_keypair().map_err(|e| anyhow!("Failed to get keypair: {e}"))?;

        // Execute the transaction
        let transaction_signature =
            reev_lib::execute_transaction(instructions, user_pubkey, &keypair)
                .await
                .map_err(|e| anyhow!("Failed to execute transaction: {e}"))?;

        info!(
            "Jupiter swap executed with signature: {}",
            transaction_signature
        );

        Ok(json!({
            "tool_name": "jupiter_swap",
            "input_mint": input_mint,
            "output_mint": output_mint,
            "amount": final_amount_lamports,
            "wallet": wallet_context.owner,
            "transaction_signature": transaction_signature,
            "success": true
        }))
    }

    /// Execute Jupiter lend/earn deposit
    async fn execute_jupiter_lend_deposit(
        &self,
        params: &HashMap<String, String>,
        wallet_context: &WalletContext,
    ) -> Result<serde_json::Value> {
        let mint = params
            .get("mint")
            .ok_or_else(|| anyhow!("mint parameter is required"))?;

        let amount_str = params
            .get("amount")
            .ok_or_else(|| anyhow!("amount parameter is required"))?;

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

        // Create AgentTools for Jupiter Lend Earn Deposit execution
        let agent_tools = self.get_or_create_agent_tools(wallet_context)?;

        // Execute Jupiter Lend Earn Deposit using AgentTools
        // Note: The amount is already in the correct units (smallest denomination)
        // as provided by the LLM, so we don't need to multiply by 1_000_000
        let deposit_args =
            reev_tools::tools::jupiter_lend_earn_deposit::JupiterLendEarnDepositArgs {
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
                    let keypair = reev_lib::get_keypair()
                        .map_err(|e| anyhow!("Failed to load keypair: {e}"))?;
                    let user_pubkey = solana_sdk::signer::Signer::pubkey(&keypair);

                    match reev_lib::utils::execute_transaction(instructions, user_pubkey, &keypair)
                        .await
                    {
                        Ok(signature) => {
                            info!(
                                "Jupiter lend deposit transaction executed with signature: {}",
                                signature
                            );
                            Ok(json!({
                                "tool_name": "jupiter_lend_earn_deposit",
                                "params": {
                                    "mint": mint,
                                    "amount": amount_lamports,
                                    "wallet": wallet_context.owner
                                },
                                "transaction_signature": signature,
                                "success": true
                            }))
                        }
                        Err(e) => {
                            error!("Failed to execute Jupiter lend deposit transaction: {}", e);
                            debug!("Transaction execution error details: {:#?}", e);

                            Ok(json!({
                                "tool_name": "jupiter_lend_earn_deposit",
                                "params": {
                                    "mint": mint,
                                    "amount": amount,
                                    "wallet": wallet_context.owner
                                },
                                "error": format!("Transaction execution failed: {}", e),
                                "success": false
                            }))
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to parse Jupiter lend deposit instructions: {}", e);
                    Ok(json!({
                        "tool_name": "jupiter_lend_earn_deposit",
                        "params": {
                            "mint": mint,
                            "amount": amount,
                            "wallet": wallet_context.owner
                        },
                        "error": format!("Failed to parse instructions: {}", e),
                        "success": false
                    }))
                }
            }
        } else {
            error!("Failed to parse Jupiter lend deposit response as JSON");
            Ok(json!({
                "tool_name": "jupiter_lend_earn_deposit",
                "params": {
                    "mint": mint,
                    "amount": amount,
                    "wallet": wallet_context.owner
                },
                "error": "Failed to parse response as JSON",
                "success": false
            }))
        }
    }

    /// Execute get account balance
    async fn execute_get_account_balance(
        &self,
        params: &HashMap<String, String>,
        _wallet_context: &WalletContext,
    ) -> Result<serde_json::Value> {
        let account = params
            .get("account")
            .ok_or_else(|| anyhow!("account parameter is required"))?;

        let default_mint = "So11111111111111111111111111111111111111112".to_string();
        let mint = params.get("mint").unwrap_or(&default_mint);

        // Mock balance for now
        // In a real implementation, this would query the blockchain
        let balance = match mint.as_str() {
            "So11111111111111111111111111111111111111112" => {
                // Mock SOL balance
                rand::random::<u64>() % 10_000_000_000
            }
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" => {
                // Mock USDC balance
                rand::random::<u64>() % 1_000_000_000
            }
            _ => {
                // Mock other token balance
                rand::random::<u64>() % 1_000_000_000
            }
        };

        Ok(json!({
            "tool_name": "get_account_balance",
            "params": {
                "account": account,
                "mint": mint
            },
            "balance": balance,
            "success": true
        }))
    }
}

/// Helper trait for getting or creating agent tools
pub trait AgentToolHelper {
    fn get_or_create_agent_tools(&self, wallet_context: &WalletContext) -> Result<Arc<AgentTools>>;
}

/// Implementation for any struct with agent_tools and HttpProvider capabilities
impl<T> AgentToolHelper for T
where
    T: AgentProvider + HttpProvider,
{
    fn get_or_create_agent_tools(&self, wallet_context: &WalletContext) -> Result<Arc<AgentTools>> {
        // Use the existing AgentTools if available
        if let Some(ref tools) = self.agent_tools() {
            return Ok(Arc::clone(tools));
        }

        // Create new AgentTools using the wallet context
        let keypair = reev_lib::get_keypair().map_err(|e| {
            anyhow!(
                "Failed to get keypair for wallet {}: {}",
                wallet_context.owner,
                e
            )
        })?;

        // Include both public key and private key base58 in key_map
        let mut key_map = std::collections::HashMap::new();
        key_map.insert("WALLET_PUBKEY".to_string(), wallet_context.owner.clone());
        key_map.insert("WALLET_KEYPAIR".to_string(), keypair.to_base58_string());
        Ok(Arc::new(AgentTools::new(key_map)))
    }
}

// Required imports for the HttpProvider trait
// HttpProvider is already imported above
