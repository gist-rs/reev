//! Rig Agent Integration for Phase 2 Tool Selection
//!
//! This module implements the RigAgent component that wraps rig framework
//! for LLM-driven tool selection and parameter extraction in Phase 2 of
//! Reev Core Architecture.

use anyhow::{anyhow, Result};
use reev_agent::enhanced::common::AgentTools;
use reev_types::flow::{StepResult, WalletContext};
use reqwest;
// Client from rig::providers::openai is not used, removed
use rig::tool::{Tool, ToolSet};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::string::String;
use std::sync::Arc;
use tracing::{debug, error, info, instrument};

use crate::execution::handlers::transfer::sol_transfer;

use crate::yml_schema::YmlStep;
use reev_types::tools::ToolName;

/// RigAgent for LLM-driven tool selection and parameter extraction
pub struct RigAgent {
    /// Model name for logging
    model_name: String,
    /// API key for the LLM service
    api_key: String,
    /// HTTP client for direct API calls
    http_client: reqwest::Client,
    /// Agent tools for executing blockchain operations
    agent_tools: Option<Arc<AgentTools>>,
}

impl RigAgent {
    /// Create a new RigAgent with the given model and tools
    pub async fn new(api_key: Option<String>, model_name: Option<String>) -> Result<Self> {
        let model_name = model_name.unwrap_or_else(|| "gpt-4".to_string());
        let api_key = api_key.ok_or_else(|| anyhow!("API key is required for RigAgent"))?;

        // Initialize tool set with Reev tools
        let _tool_set = Self::initialize_tool_set().await?; // Prefix with _ to suppress warning

        Ok(Self {
            model_name,
            api_key,
            http_client: reqwest::Client::new(),
            agent_tools: None,
        })
    }

    /// Create a new RigAgent with the given model and tools
    pub async fn new_with_tools(
        api_key: Option<String>,
        model_name: Option<String>,
        agent_tools: Arc<AgentTools>,
    ) -> Result<Self> {
        let model_name = model_name.unwrap_or_else(|| "gpt-4".to_string());
        let api_key = api_key.ok_or_else(|| anyhow!("API key is required for RigAgent"))?;

        // Initialize tool set with Reev tools
        let _tool_set = Self::initialize_tool_set().await?; // Prefix with _ to suppress warning

        Ok(Self {
            model_name,
            api_key,
            http_client: reqwest::Client::new(),
            agent_tools: Some(agent_tools),
        })
    }

    /// Execute a step using the rig agent for tool selection
    #[instrument(skip(self, step, wallet_context))]
    pub async fn execute_step_with_rig(
        &self,
        step: &YmlStep,
        wallet_context: &WalletContext,
    ) -> Result<StepResult> {
        info!("Executing step {} with rig agent", step.step_id);

        // Use the refined prompt if available, otherwise use the original prompt
        let prompt = if !step.refined_prompt.is_empty() {
            step.refined_prompt.clone()
        } else {
            step.prompt.clone()
        };

        // Create a context-aware prompt with wallet information
        let context_prompt = self.create_context_prompt(&prompt, wallet_context)?;

        // Get expected tools hints from the step
        let expected_tools = step.expected_tools.clone();

        // If we have expected tools, use them to guide the agent
        let response = if let Some(tools) = expected_tools {
            info!("Using expected tools to guide agent: {:?}", tools);
            self.prompt_with_expected_tools(&context_prompt, &tools)
                .await?
        } else {
            info!("No expected tools provided, using general agent prompt");
            self.prompt_agent(&context_prompt).await?
        };

        info!("Got response from agent: {}", response);

        // Extract tool calls from the response
        let tool_calls = self.extract_tool_calls(&response)?;

        // Execute the selected tools
        info!("Tool calls extracted: {:?}", tool_calls);
        let tool_results = self
            .execute_tools(tool_calls.clone(), wallet_context)
            .await?;
        info!("Tool execution results: {:?}", tool_results);

        // Create list of tool names that were executed
        let executed_tool_names: Vec<String> = tool_calls.keys().cloned().collect();

        // Create the step result
        let step_result = StepResult {
            step_id: step.step_id.clone(),
            success: true,
            error_message: None,
            tool_calls: executed_tool_names,
            output: json!({ "tool_results": tool_results }),
            execution_time_ms: 100, // This would be calculated in a real implementation
        };

        Ok(step_result)
    }

    /// Create a context-aware prompt with wallet information
    fn create_context_prompt(
        &self,
        prompt: &str,
        wallet_context: &WalletContext,
    ) -> Result<String> {
        let wallet_info = json!({
            "pubkey": wallet_context.owner,
            "sol_balance": wallet_context.sol_balance,
            "tokens": wallet_context.token_balances.values().collect::<Vec<_>>()
        });

        Ok(format!(
            "Given the following wallet context:\n{}\n\nPlease help with the following request: {}",
            serde_json::to_string_pretty(&wallet_info)?,
            prompt
        ))
    }

    /// Prompt the agent with expected tools hints
    async fn prompt_with_expected_tools(
        &self,
        prompt: &str,
        expected_tools: &[ToolName],
    ) -> Result<String> {
        let tools_hint = expected_tools
            .iter()
            .map(|tool| tool.to_string())
            .collect::<Vec<_>>()
            .join(", ");

        let guided_prompt = format!(
            "For this request, you should use one or more of these tools: {tools_hint}. {prompt}"
        );

        self.prompt_agent(&guided_prompt).await
    }

    /// Prompt the agent and get the response
    async fn prompt_agent(&self, prompt: &str) -> Result<String> {
        debug!("Prompting agent with: {}", prompt);

        // Create a structured prompt for the LLM
        let system_prompt = "You are an AI assistant for Solana DeFi operations. Based on the user's request, determine the appropriate tool to use and extract the necessary parameters. Respond with valid JSON in the following format:
{
  \"tool_calls\": [
    {
      \"name\": \"tool_name\",
      \"parameters\": {
        \"param1\": \"value1\",
        \"param2\": \"value2\"
      }
    }
  ]
}

Available tools:
- sol_transfer: Transfer SOL from one account to another. Parameters: recipient (string, required), amount (number in SOL, required), wallet (string, optional)
- jupiter_swap: Swap tokens using Jupiter. Parameters: input_mint (string, required, e.g., 'So11111111111111111111111111111111111111112' for SOL), output_mint (string, required, e.g., 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v' for USDC), input_amount (number, required, amount of tokens to swap, use decimal for partial amounts like 0.5 for half), wallet (string, optional)
- jupiter_lend_earn_deposit: Deposit tokens into Jupiter lending. Parameters: mint (string, required), amount (number, required), wallet (string, optional)
- get_account_balance: Get account balance. Parameters: account (string, required), mint (string, optional, defaults to SOL)

For token mint addresses:
- SOL: So11111111111111111111111111111111111111112
- USDC: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
- USDT: Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB

For swap operations, always determine the input and output mints based on the token names (SOL, USDC, etc.).
";

        // Prepare the request payload
        // Use the correct model name for ZAI API
        let model_name = if self.model_name == "glm-4.6-coding" {
            "glm-4.6"
        } else {
            &self.model_name
        };

        let request_payload = LLMRequest {
            model: model_name.to_string(),
            messages: vec![
                LLMMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                LLMMessage {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                },
            ],
            temperature: 0.3,
            max_tokens: 1000,
        };

        info!("Sending request to ZAI API with model: {}", model_name);
        info!("Prompt being sent to ZAI API: {}", prompt);

        // Make the API call
        let api_base = env::var("ZAI_API_BASE")
            .unwrap_or_else(|_| "https://api.z.ai/api/coding/paas/v4".to_string());
        let url = format!("{api_base}/chat/completions");

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_payload)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "API request failed with status: {}",
                response.status()
            ));
        }

        let response_body: LLMResponse = response.json().await?;

        // Extract the content from the response
        let content = response_body
            .choices
            .first()
            .map(|choice| choice.message.content.clone())
            .ok_or_else(|| anyhow!("No content in LLM response"))?;

        info!("LLM response: {}", content);
        Ok(content)
    }

    /// Extract tool calls from the agent response
    fn extract_tool_calls(&self, response: &str) -> Result<HashMap<String, serde_json::Value>> {
        // This is a simplified implementation
        // In a real implementation, we would parse the JSON response to extract tool calls
        info!("Extracting tool calls from response: {}", response);

        // Parse the response to extract tool calls
        self.parse_tool_calls_from_response(response)
    }

    /// Parse tool calls from LLM response
    fn parse_tool_calls_from_response(
        &self,
        response: &str,
    ) -> Result<HashMap<String, serde_json::Value>> {
        // Try to parse response as JSON first
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(response) {
            info!("Parsed JSON response successfully");
            if let Some(tool_calls) = json_value.get("tool_calls").and_then(|v| v.as_array()) {
                info!("Found {} tool calls in response", tool_calls.len());
                let mut tool_map = HashMap::new();
                for tool_call in tool_calls {
                    if let (Some(name), Some(params)) = (
                        tool_call.get("name").and_then(|n| n.as_str()),
                        tool_call.get("parameters"),
                    ) {
                        info!("Extracted tool call: {} with params: {}", name, params);
                        tool_map.insert(name.to_string(), params.clone());
                    } else {
                        info!("Tool call missing name or parameters: {:?}", tool_call);
                    }
                }
                return Ok(tool_map);
            } else {
                info!("No tool_calls found in JSON response");
            }
        } else {
            info!("Failed to parse response as JSON, trying text extraction");
        }

        // If JSON parsing fails, try to extract tool calls from text
        self.extract_tool_calls_from_text(response)
    }

    /// Extract tool calls from text response
    fn extract_tool_calls_from_text(
        &self,
        response: &str,
    ) -> Result<HashMap<String, serde_json::Value>> {
        // Simple regex-based extraction for tool calls
        // Format: "tool_name(parameters)" or "use tool_name with parameters: {...}"
        let mut tool_map = HashMap::new();

        // Pattern 1: tool_name(parameters)
        let re1 = regex::Regex::new(r"(\w+)\(([^)]+)\)").unwrap();
        for captures in re1.captures_iter(response) {
            if let (Some(tool_name), Some(params_str)) = (captures.get(1), captures.get(2)) {
                if let Ok(params) = serde_json::from_str::<serde_json::Value>(params_str.as_str()) {
                    tool_map.insert(tool_name.as_str().to_string(), params);
                }
            }
        }

        // Pattern 2: "use tool_name with parameters: {...}"
        let re2 = regex::Regex::new(r"use (\w+) with parameters: (\{[^}]+\})").unwrap();
        for captures in re2.captures_iter(response) {
            if let (Some(tool_name), Some(params_str)) = (captures.get(1), captures.get(2)) {
                if let Ok(params) = serde_json::from_str::<serde_json::Value>(params_str.as_str()) {
                    tool_map.insert(tool_name.as_str().to_string(), params);
                }
            }
        }

        Ok(tool_map)
    }

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
        let mut params_map = std::collections::HashMap::new();
        if let serde_json::Value::Object(map) = &params {
            for (key, value) in map {
                if let Some(str_value) = value.as_str() {
                    params_map.insert(key.clone(), str_value.to_string());
                } else {
                    params_map.insert(key.clone(), value.to_string());
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
        params: &std::collections::HashMap<String, String>,
        wallet_context: &WalletContext,
    ) -> Result<serde_json::Value> {
        let recipient = params
            .get("recipient")
            .ok_or_else(|| anyhow!("recipient parameter is required"))?;

        let amount_str = params
            .get("amount")
            .ok_or_else(|| anyhow!("amount parameter is required"))?;

        let amount: f64 = amount_str
            .parse()
            .map_err(|_| anyhow!("Invalid amount: {amount_str}"))?;

        let amount_lamports = (amount * 1_000_000_000.0) as u64;

        // Check if wallet has sufficient balance
        if wallet_context.sol_balance < amount_lamports {
            return Err(anyhow!(
                "Insufficient balance. Available: {} SOL, Required: {} SOL",
                wallet_context.sol_balance / 1_000_000_000,
                amount
            ));
        }

        // Use the existing AgentTools if available, otherwise create a new one
        // Create AgentTools for Jupiter swap execution
        // Create AgentTools for Jupiter Lend Earn Deposit execution
        let agent_tools = if let Some(ref tools) = self.agent_tools {
            Arc::clone(tools)
        } else {
            // Create AgentTools using the wallet context
            // Convert wallet owner string to keypair
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
            Arc::new(reev_agent::enhanced::common::AgentTools::new(key_map))
        };

        // Use the existing execute_direct_sol_transfer function from handlers
        // This will handle the actual blockchain transaction
        let transaction_result = sol_transfer::execute_direct_sol_transfer(
            &agent_tools,
            &format!("send {amount} sol to {recipient}"),
            &wallet_context.owner,
        )
        .await?;

        // Extract the transaction signature from the result
        let transaction_signature = if transaction_result.success {
            if let Some(output) = transaction_result.output.get("sol_transfer") {
                if let Some(sig) = output.get("transaction_signature") {
                    sig.as_str().unwrap_or("").to_string()
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        // If we got a signature, return it directly, otherwise return the error
        if transaction_signature.is_empty() && !transaction_result.success {
            return Err(anyhow!(
                "SOL transfer failed: {:?}",
                transaction_result
                    .error_message
                    .unwrap_or("Unknown error".to_string())
            ));
        }

        Ok(json!({
            "tool_name": "sol_transfer",
            "params": {
                "recipient": recipient,
                "amount": amount,
                "amount_lamports": amount_lamports,
                "wallet": wallet_context.owner
            },
            "transaction_signature": transaction_signature,
            "success": transaction_result.success
        }))
    }

    /// Execute Jupiter swap
    async fn execute_jupiter_swap(
        &self,
        params: &std::collections::HashMap<String, String>,
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
        let amount: f64 = amount_str
            .parse()
            .map_err(|_| anyhow!("Invalid amount: {amount_str}"))?;

        // Special handling for "all" amount to use full balance
        let is_all_amount = amount_str.to_lowercase() == "all";

        // Convert amount to lamports (1 SOL = 1,000,000,000 lamports)
        let amount_lamports = (amount * 1_000_000_000.0) as u64;

        // Create AgentTools for Jupiter swap execution
        let agent_tools = if let Some(ref tools) = self.agent_tools {
            Arc::clone(tools)
        } else {
            // Create AgentTools using the wallet context
            // Load the keypair and include both public key and private key
            let keypair = reev_lib::get_keypair().map_err(|e| {
                anyhow!(
                    "Failed to get keypair for wallet {}: {}",
                    wallet_context.owner,
                    e
                )
            })?;

            let mut key_map = std::collections::HashMap::new();
            key_map.insert("WALLET_PUBKEY".to_string(), wallet_context.owner.clone());
            key_map.insert("WALLET_KEYPAIR".to_string(), keypair.to_base58_string());
            Arc::new(reev_agent::enhanced::common::AgentTools::new(key_map))
        };

        // Use full balance if amount is "all", otherwise use specified amount
        let final_amount_lamports = if is_all_amount {
            // Use almost all SOL balance, keeping some for fees
            wallet_context.sol_balance - (100_000_000) // Reserve 0.1 SOL for fees
        } else {
            amount_lamports
        };

        let swap_args = reev_tools::tools::jupiter_swap::JupiterSwapArgs {
            user_pubkey: wallet_context.owner.clone(),
            input_mint: input_mint.to_string(),
            output_mint: output_mint.to_string(),
            amount: final_amount_lamports,
            slippage_bps: Some(100), // Default 1% slippage
        };

        let result = agent_tools
            .jupiter_swap_tool
            .call(swap_args)
            .await
            .map_err(|e| anyhow!("Jupiter swap execution failed: {e}"))?;

        // Parse the response to extract instructions and execute transaction
        info!("Jupiter swap tool returned result: {}", &result);
        if let Ok(response) = serde_json::from_str::<serde_json::Value>(&result) {
            debug!("Parsed response: {:#?}", response);
            if let Some(instructions) = response.get("instructions") {
                info!(
                    "Found {} instructions in Jupiter response",
                    instructions.as_array().unwrap_or(&vec![]).len()
                );
                debug!("Instructions value: {:#?}", instructions);

                // Convert instructions to RawInstruction format
                let raw_instructions: Result<Vec<reev_lib::agent::RawInstruction>> = instructions
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

                // Execute the transaction with the instructions
                match raw_instructions {
                    Ok(instructions) => {
                        let keypair = reev_lib::get_keypair()
                            .map_err(|e| anyhow!("Failed to load keypair: {e}"))?;
                        let user_pubkey = solana_sdk::signer::Signer::pubkey(&keypair);

                        match reev_lib::utils::execute_transaction(
                            instructions,
                            user_pubkey,
                            &keypair,
                        )
                        .await
                        {
                            Ok(signature) => {
                                info!(
                                    "Jupiter swap transaction executed with signature: {}",
                                    signature
                                );
                                Ok(json!({
                                    "tool_name": "jupiter_swap",
                                    "input_mint": input_mint,
                                    "output_mint": output_mint,
                                    "input_amount": amount,
                                    "input_amount_lamports": amount_lamports,
                                    "wallet": wallet_context.owner,
                                    "transaction_signature": signature,
                                    "success": true
                                }))
                            }
                            Err(e) => {
                                error!("Failed to execute Jupiter swap transaction: {}", e);
                                debug!("Transaction execution error details: {:#?}", e);
                                debug!("Failed at execute_transaction call");
                                Ok(json!({
                                    "tool_name": "jupiter_swap",
                                    "error": format!("Transaction execution failed: {e}"),
                                    "raw_response": result
                                }))
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to parse instructions: {}", e);
                        Ok(json!({
                            "tool_name": "jupiter_swap",
                            "error": format!("Failed to parse instructions: {e}"),
                            "raw_response": result
                        }))
                    }
                }
            } else {
                Ok(json!({
                    "tool_name": "jupiter_swap",
                    "error": "No instructions found in response",
                    "raw_response": result
                }))
            }
        } else {
            Ok(json!({
                "tool_name": "jupiter_swap",
                "error": "Failed to parse Jupiter response",
                "raw_response": result
            }))
        }
    }

    /// Execute Jupiter lend/earn deposit
    async fn execute_jupiter_lend_deposit(
        &self,
        params: &std::collections::HashMap<String, String>,
        wallet_context: &WalletContext,
    ) -> Result<serde_json::Value> {
        let mint = params
            .get("mint")
            .ok_or_else(|| anyhow!("mint parameter is required"))?;

        let amount_str = params
            .get("amount")
            .ok_or_else(|| anyhow!("amount parameter is required"))?;

        let amount: f64 = amount_str
            .parse()
            .map_err(|_| anyhow!("Invalid amount: {amount_str}"))?;

        // Create AgentTools for Jupiter Lend Earn Deposit execution
        let agent_tools = if let Some(ref tools) = self.agent_tools {
            Arc::clone(tools)
        } else {
            // Create AgentTools using the wallet context
            // Load the keypair and include both public key and private key
            let keypair = reev_lib::get_keypair().map_err(|e| {
                anyhow!(
                    "Failed to get keypair for wallet {}: {}",
                    wallet_context.owner,
                    e
                )
            })?;

            let mut key_map = std::collections::HashMap::new();
            key_map.insert("WALLET_PUBKEY".to_string(), wallet_context.owner.clone());
            key_map.insert("WALLET_KEYPAIR".to_string(), keypair.to_base58_string());
            Arc::new(reev_agent::enhanced::common::AgentTools::new(key_map))
        };

        // Execute Jupiter Lend Earn Deposit using AgentTools
        let deposit_args =
            reev_tools::tools::jupiter_lend_earn_deposit::JupiterLendEarnDepositArgs {
                user_pubkey: wallet_context.owner.clone(),
                asset_mint: mint.clone(),
                amount: (amount * 1_000_000_000.0) as u64, // Convert to lamports
            };

        let result = agent_tools
            .jupiter_lend_earn_deposit_tool
            .call(deposit_args)
            .await
            .map_err(|e| anyhow!("Jupiter Lend Earn Deposit execution failed: {e}"))?;

        // Parse the response to extract transaction signature
        let transaction_signature =
            if let Ok(instructions) = serde_json::from_str::<serde_json::Value>(&result) {
                if let Some(sig) = instructions.get("transactionSignature") {
                    sig.as_str().unwrap_or("").to_string()
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

        Ok(json!({
            "tool_name": "jupiter_lend_earn_deposit",
            "params": {
                "mint": mint,
                "amount": amount,
                "wallet": wallet_context.owner
            },
            "transaction_signature": transaction_signature,
            "success": true
        }))
    }

    /// Execute get account balance
    async fn execute_get_account_balance(
        &self,
        params: &std::collections::HashMap<String, String>,
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

    /// Initialize the tool set with Reev tools
    async fn initialize_tool_set() -> Result<ToolSet> {
        // Create a tool set with all Reev tools
        // For now, we'll create a minimal tool set as a placeholder
        // In a full implementation, we would add all Reev tools (SolTransfer, JupiterSwap, etc.)

        // Use the agent builder to create tools directly
        let tool_set = ToolSet::default();

        Ok(tool_set)
    }
}

/// LLM API request payload
#[derive(Debug, Serialize)]
struct LLMRequest {
    model: String,
    messages: Vec<LLMMessage>,
    temperature: f32,
    max_tokens: u32,
}

/// LLM API message
#[derive(Debug, Serialize)]
struct LLMMessage {
    role: String,
    content: String,
}

/// LLM API response
#[derive(Debug, Deserialize)]
struct LLMResponse {
    choices: Vec<LLMChoice>,
}

/// LLM API choice
#[derive(Debug, Deserialize)]
struct LLMChoice {
    message: LLMResponseMessage,
}

/// LLM API response message
#[derive(Debug, Deserialize)]
struct LLMResponseMessage {
    content: String,
}
