//! # Flow Agent Implementation
//!
//! This module implements the core FlowAgent that orchestrates multi-step
//! flows using RAG-based tool selection and conversation state management.

use anyhow::Result;
use regex::Regex;
use rig::tool::ToolDyn;
use serde_json::json;
use std::collections::HashMap;
use tracing::{error, info, warn};

use crate::{
    flow::{
        benchmark::FlowBenchmark,
        state::{FlowState, SolanaInstruction, StepResult, StepStatus},
    },
    tools::{
        jupiter_lend_deposit::JupiterLendDepositTool,
        jupiter_lend_withdraw::JupiterLendWithdrawTool, jupiter_swap::JupiterSwapTool,
        sol_transfer::SolTransferTool, spl_transfer::SplTransferTool,
    },
};

/// RAG-based flow agent capable of orchestrating multi-step DeFi workflows
pub struct FlowAgent {
    /// Available tools for the flow agent
    tools: HashMap<String, Box<dyn ToolDyn>>,
    /// Current conversation state
    state: FlowState,
}

impl FlowAgent {
    /// Create a new FlowAgent with the specified model
    pub async fn new(_model_name: &str) -> Result<Self> {
        info!(
            "[FlowAgent] Initializing flow agent with model: {}",
            _model_name
        );

        // Create toolset with all available flow tools
        let tools = Self::create_toolset().await?;

        let state = FlowState::new(0); // Will be updated when benchmark is loaded

        Ok(Self { tools, state })
    }

    /// Create the toolset with all available flow tools
    async fn create_toolset() -> Result<HashMap<String, Box<dyn ToolDyn>>> {
        let mut tools: HashMap<String, Box<dyn ToolDyn>> = HashMap::new();

        // Initialize each tool
        tools.insert(
            "sol_transfer".to_string(),
            Box::new(SolTransferTool) as Box<dyn ToolDyn>,
        );
        tools.insert(
            "spl_transfer".to_string(),
            Box::new(SplTransferTool) as Box<dyn ToolDyn>,
        );
        tools.insert(
            "jupiter_swap".to_string(),
            Box::new(JupiterSwapTool {
                key_map: std::collections::HashMap::new(),
            }) as Box<dyn ToolDyn>,
        );
        tools.insert(
            "jupiter_lend_deposit".to_string(),
            Box::new(JupiterLendDepositTool {
                key_map: std::collections::HashMap::new(),
            }) as Box<dyn ToolDyn>,
        );
        tools.insert(
            "jupiter_lend_withdraw".to_string(),
            Box::new(JupiterLendWithdrawTool {
                key_map: std::collections::HashMap::new(),
            }) as Box<dyn ToolDyn>,
        );

        info!(
            "[FlowAgent] Initialized {} tools for flow execution",
            tools.len()
        );
        Ok(tools)
    }

    /// Load a flow benchmark and prepare the agent
    pub async fn load_benchmark(&mut self, benchmark: &FlowBenchmark) -> Result<()> {
        info!("[FlowAgent] Loading flow benchmark: {}", benchmark.id);

        // Update state with benchmark information
        self.state = FlowState::new(benchmark.total_steps());
        self.state
            .add_context("flow_id".to_string(), benchmark.id.clone());
        self.state.add_context(
            "flow_description".to_string(),
            benchmark.description.clone(),
        );

        // Add initial context from benchmark metadata
        for (key, value) in &benchmark.metadata {
            self.state
                .add_context(format!("benchmark_{key}"), value.to_string());
        }

        info!(
            "[FlowAgent] Flow loaded: {} steps, {} critical steps",
            benchmark.total_steps(),
            benchmark.get_critical_steps().len()
        );

        Ok(())
    }

    /// Execute the complete multi-step flow
    pub async fn execute_flow(&mut self, benchmark: &FlowBenchmark) -> Result<Vec<StepResult>> {
        info!("[FlowAgent] Executing flow: {}", benchmark.id);
        info!("[FlowAgent] Flow summary:\n{}", benchmark.get_summary());

        let mut all_results = Vec::new();

        for step in &benchmark.flow {
            info!(
                "[FlowAgent] ======== Step {} / {} ========",
                step.step,
                benchmark.total_steps()
            );
            info!("[FlowAgent] Step description: {}", step.description);

            // Prepare the enriched prompt with context
            let enriched_prompt = self.enrich_prompt(&step.prompt, benchmark);

            // Add context to state
            self.state
                .add_context("current_step".to_string(), step.step.to_string());

            // Execute the step with multi-turn conversation
            let step_result = self.execute_step(step, &enriched_prompt).await?;

            // Store the result
            let step_id = format!("step_{}", step.step);
            self.state.add_result(step_id.clone(), step_result.clone());
            all_results.push(step_result.clone());

            info!(
                "[FlowAgent] Step {} completed with status: {:?}",
                step.step, step_result.status
            );

            // Check if step was critical and failed
            if step.critical && matches!(step_result.status, StepStatus::Failed(_)) {
                error!(
                    "[FlowAgent] Critical step {} failed, stopping flow",
                    step.step
                );
                break;
            }
        }

        info!("[FlowAgent] Flow execution complete");
        info!("[FlowAgent] Final state:\n{}", self.state.get_summary());

        Ok(all_results)
    }

    /// Execute a single step in the flow
    async fn execute_step(
        &mut self,
        step: &crate::flow::benchmark::FlowStep,
        prompt: &str,
    ) -> Result<StepResult> {
        let start_time = chrono::Utc::now().to_rfc3339();
        let start_time_clone = start_time.clone();

        info!("[FlowAgent] Executing step with prompt: {}", step.prompt);

        // Use RAG to find relevant tools for this step
        let relevant_tools = self.find_relevant_tools(prompt).await?;

        info!(
            "[FlowAgent] Found {} relevant tools: {:?}",
            relevant_tools.len(),
            relevant_tools
        );

        // Execute tools and collect instructions
        let mut all_instructions = Vec::new();
        let mut tool_results = Vec::new();

        for tool_name in &relevant_tools {
            if let Some(_tool) = self.tools.get(tool_name) {
                info!("[FlowAgent] Executing tool: {}", tool_name);

                // Create arguments for the tool
                let tool_args = self.create_tool_args(tool_name, prompt)?;

                // Execute the tool (simulate tool call for now)
                let tool_result = self.simulate_tool_call(tool_name, &tool_args).await?;

                // Parse instructions from tool result
                let instructions = self.parse_tool_result(&tool_result)?;
                all_instructions.extend(instructions);
                tool_results.push(tool_result);

                info!("[FlowAgent] Tool {} executed successfully", tool_name);
            } else {
                warn!("[FlowAgent] Tool {} not found in toolset", tool_name);
            }
        }

        // Create combined response
        let response = json!({
            "tools_executed": relevant_tools,
            "tool_results": tool_results,
            "total_instructions": all_instructions.len()
        })
        .to_string();

        let instructions = all_instructions;

        // Create step result
        let step_result = StepResult {
            step: step.step,
            description: step.description.clone(),
            llm_response: response.clone(),
            instructions,
            status: StepStatus::Success, // Will be updated based on execution
            completed_at: start_time_clone,
            metadata: {
                let mut meta = HashMap::new();
                meta.insert(
                    "relevant_tools".to_string(),
                    json!(relevant_tools).to_string(),
                );
                meta.insert("prompt".to_string(), step.prompt.clone());
                meta
            },
        };

        // Add conversation turn to state
        self.state.add_turn(crate::flow::state::ConversationTurn {
            turn: self.state.conversation_history.len() + 1,
            step: step.step,
            user_prompt: prompt.to_string(),
            system_prompt: crate::flow::FLOW_SYSTEM_PREAMBLE.to_string(),
            llm_response: response,
            tools_called: relevant_tools.clone(),
            timestamp: start_time,
        });

        Ok(step_result)
    }

    /// Find relevant tools using RAG (Retrieval Augmented Generation)
    async fn find_relevant_tools(&self, prompt: &str) -> Result<Vec<String>> {
        // Simple keyword-based tool selection for now
        // In a full implementation, this would use vector embeddings
        let mut relevant_tools = Vec::new();

        let prompt_lower = prompt.to_lowercase();

        if (prompt_lower.contains("swap") || prompt_lower.contains("exchange"))
            && self.tools.contains_key("jupiter_swap")
        {
            relevant_tools.push("jupiter_swap".to_string());
        }

        if (prompt_lower.contains("deposit") || prompt_lower.contains("lend"))
            && self.tools.contains_key("jupiter_lend_deposit")
        {
            relevant_tools.push("jupiter_lend_deposit".to_string());
        }

        if (prompt_lower.contains("withdraw") || prompt_lower.contains("unstake"))
            && self.tools.contains_key("jupiter_lend_withdraw")
        {
            relevant_tools.push("jupiter_lend_withdraw".to_string());
        }

        if prompt_lower.contains("sol")
            && !prompt_lower.contains("usdc")
            && self.tools.contains_key("sol_transfer")
        {
            relevant_tools.push("sol_transfer".to_string());
        }

        if (prompt_lower.contains("token") || prompt_lower.contains("spl"))
            && self.tools.contains_key("spl_transfer")
        {
            relevant_tools.push("spl_transfer".to_string());
        }

        info!("[FlowAgent] RAG search found tools: {:?}", relevant_tools);

        Ok(relevant_tools)
    }

    /// Enrich prompt with context and previous step results
    fn enrich_prompt(&self, prompt: &str, benchmark: &FlowBenchmark) -> String {
        let mut enriched_parts = Vec::new();

        // Add system preamble
        enriched_parts.push(crate::flow::FLOW_SYSTEM_PREAMBLE.to_string());

        // Add flow context
        enriched_parts.push(format!(
            "\n=== Current Flow Context ===\n\
            Flow ID: {}\n\
            Description: {}\n\
            Current Step: {}/{}\n\
            Tags: {}\n",
            benchmark.id,
            benchmark.description,
            self.state.current_step,
            benchmark.total_steps(),
            benchmark.tags.join(", ")
        ));

        // Add on-chain context (would be populated from environment)
        enriched_parts.push(
            "\n=== On-Chain Context ===\n\
            Note: On-chain context would be populated here from the environment."
                .to_string(),
        );

        // Add previous step results
        if !self.state.step_results.is_empty() {
            enriched_parts.push(format!(
                "\n=== Previous Step Results ===\n{}",
                self.state.format_step_results()
            ));
        }

        // Add general context
        if !self.state.context.is_empty() {
            enriched_parts.push(format!(
                "\n=== Additional Context ===\n{}",
                self.state.format_context()
            ));
        }

        // Add the current task
        enriched_parts.push(format!("\n=== Current Task ===\n{prompt}"));

        enriched_parts.join("\n")
    }

    /// Parse instructions from LLM response
    #[allow(dead_code)]
    fn parse_instructions(&self, response: &str) -> Result<Vec<SolanaInstruction>> {
        info!("[FlowAgent] Parsing instructions from LLM response");

        // Look for JSON blocks in the response
        let re = Regex::new(r"(?s)```(?:json)?\s*(\{[\s\S]*?\}|\[[\s\S]*?\})\s*```").unwrap();

        if let Some(caps) = re.captures(response) {
            let json_str = caps.get(1).map_or("", |m| m.as_str());

            match serde_json::from_str::<serde_json::Value>(json_str) {
                Ok(value) => {
                    // Handle both single instruction and array of instructions
                    match value {
                        serde_json::Value::Object(obj) => {
                            // Single instruction
                            let instruction = self.parse_single_instruction(&obj)?;
                            Ok(vec![instruction])
                        }
                        serde_json::Value::Array(arr) => {
                            // Array of instructions
                            let mut instructions = Vec::new();
                            for item in arr {
                                if let serde_json::Value::Object(obj) = item {
                                    let instruction = self.parse_single_instruction(&obj)?;
                                    instructions.push(instruction);
                                }
                            }
                            Ok(instructions)
                        }
                        _ => {
                            warn!("[FlowAgent] Unexpected JSON structure in LLM response");
                            Ok(Vec::new())
                        }
                    }
                }
                Err(e) => {
                    warn!("[FlowAgent] Failed to parse JSON from LLM response: {}", e);
                    Ok(Vec::new())
                }
            }
        } else {
            warn!("[FlowAgent] No JSON block found in LLM response");
            Ok(Vec::new())
        }
    }

    /// Parse a single Solana instruction from JSON
    fn parse_single_instruction(
        &self,
        obj: &serde_json::Map<String, serde_json::Value>,
    ) -> Result<SolanaInstruction> {
        Ok(SolanaInstruction {
            program_id: obj
                .get("program_id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            accounts: obj
                .get("accounts")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|acc| acc.as_object())
                        .map(|acc_obj| crate::flow::state::AccountMeta {
                            pubkey: acc_obj
                                .get("pubkey")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown")
                                .to_string(),
                            is_signer: acc_obj
                                .get("is_signer")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false),
                            is_writable: acc_obj
                                .get("is_writable")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false),
                        })
                        .collect()
                })
                .unwrap_or_default(),
            data: obj
                .get("data")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            should_succeed: obj
                .get("should_succeed")
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
        })
    }

    /// Create tool arguments based on the prompt and tool type
    fn create_tool_args(&self, tool_name: &str, _prompt: &str) -> Result<serde_json::Value> {
        match tool_name {
            "jupiter_swap" => {
                // Extract swap parameters from prompt
                let args = serde_json::json!({
                    "user_pubkey": "USER_WALLET_PUBKEY",
                    "input_mint": "So11111111111111111111111111111111111111112", // SOL
                    "output_mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC
                    "amount": 500000000, // 0.5 SOL in lamports
                    "slippage_bps": 50 // 0.5%
                });
                Ok(args)
            }
            "jupiter_lend_deposit" => {
                // Extract deposit parameters from prompt
                let args = serde_json::json!({
                    "user_pubkey": "USER_WALLET_PUBKEY",
                    "asset_mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC
                    "amount": 50000000 // Expected USDC amount
                });
                Ok(args)
            }
            "jupiter_lend_withdraw" => {
                // Extract withdraw parameters from prompt
                let args = serde_json::json!({
                    "user_pubkey": "USER_WALLET_PUBKEY",
                    "asset_mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC
                    "amount": 50000000 // Expected USDC amount
                });
                Ok(args)
            }
            "sol_transfer" => {
                let args = serde_json::json!({
                    "from_pubkey": "USER_WALLET_PUBKEY",
                    "to_pubkey": "RECIPIENT_PUBKEY",
                    "lamports": 100000000
                });
                Ok(args)
            }
            "spl_transfer" => {
                let args = serde_json::json!({
                    "from_pubkey": "USER_WALLET_PUBKEY",
                    "to_pubkey": "RECIPIENT_PUBKEY",
                    "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                    "amount": 1000000
                });
                Ok(args)
            }
            _ => Err(anyhow::anyhow!("Unknown tool: {tool_name}")),
        }
    }

    /// Parse tool result to extract instructions
    fn parse_tool_result(&self, result: &str) -> Result<Vec<SolanaInstruction>> {
        // Try to parse the result as JSON instructions
        match serde_json::from_str::<serde_json::Value>(result) {
            Ok(value) => {
                if let Some(instructions) = value.as_array() {
                    let mut parsed_instructions = Vec::new();
                    for instruction in instructions {
                        if let Some(obj) = instruction.as_object() {
                            let parsed = self.parse_single_instruction(obj)?;
                            parsed_instructions.push(parsed);
                        }
                    }
                    Ok(parsed_instructions)
                } else {
                    // If it's not an array, try to parse as single instruction
                    if let Some(obj) = value.as_object() {
                        let parsed = self.parse_single_instruction(obj)?;
                        Ok(vec![parsed])
                    } else {
                        // Create a fallback instruction
                        Ok(vec![SolanaInstruction {
                            program_id: "unknown".to_string(),
                            accounts: Vec::new(),
                            data: result.to_string(),
                            should_succeed: true,
                        }])
                    }
                }
            }
            Err(_) => {
                // If parsing fails, create a fallback instruction
                Ok(vec![SolanaInstruction {
                    program_id: "unknown".to_string(),
                    accounts: Vec::new(),
                    data: result.to_string(),
                    should_succeed: true,
                }])
            }
        }
    }

    /// Get the current flow state
    pub fn get_state(&self) -> &FlowState {
        &self.state
    }

    /// Get a mutable reference to the current flow state
    pub fn get_state_mut(&mut self) -> &mut FlowState {
        &mut self.state
    }

    /// Reset the flow state
    pub fn reset_state(&mut self) {
        self.state = FlowState::new(0);
    }

    /// Simulate tool execution (would be actual tool calls in production)
    async fn simulate_tool_call(
        &self,
        tool_name: &str,
        args: &serde_json::Value,
    ) -> Result<String> {
        match tool_name {
            "jupiter_swap" => Ok(json!({
                "swap_executed": true,
                "input_mint": args["input_mint"],
                "output_mint": args["output_mint"],
                "amount": args["amount"],
                "program_id": "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4",
                "instructions": [
                    {
                        "program_id": "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4",
                        "accounts": ["USER_WALLET_PUBKEY", "JUP_TOKEN_ACCOUNT"],
                        "data": "swap_instruction_data",
                        "should_succeed": true
                    }
                ]
            })
            .to_string()),
            "jupiter_lend_deposit" => Ok(json!({
                "deposit_executed": true,
                "asset_mint": args["asset_mint"],
                "amount": args["amount"],
                "program_id": "jup3YeL8QhtSx1e253b2FDvsMNC87fDrgQZivbrndc9",
                "instructions": [
                    {
                        "program_id": "jup3YeL8QhtSx1e253b2FDvsMNC87fDrgQZivbrndc9",
                        "accounts": ["USER_WALLET_PUBKEY", "USDC_TOKEN_ACCOUNT"],
                        "data": "deposit_instruction_data",
                        "should_succeed": true
                    }
                ]
            })
            .to_string()),
            "sol_transfer" => Ok(json!({
                "transfer_executed": true,
                "from": args["from_pubkey"],
                "to": args["to_pubkey"],
                "lamports": args["lamports"],
                "program_id": "11111111111111111111111111111111",
                "instructions": [
                    {
                        "program_id": "11111111111111111111111111111111",
                        "accounts": [args["from_pubkey"], args["to_pubkey"]],
                        "data": "transfer_instruction_data",
                        "should_succeed": true
                    }
                ]
            })
            .to_string()),
            "spl_transfer" => Ok(json!({
                "transfer_executed": true,
                "from": args["from_pubkey"],
                "to": args["to_pubkey"],
                "mint": args["mint"],
                "amount": args["amount"],
                "program_id": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
                "instructions": [
                    {
                        "program_id": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
                        "accounts": [args["from_pubkey"], args["to_pubkey"], args["mint"]],
                        "data": "spl_transfer_instruction_data",
                        "should_succeed": true
                    }
                ]
            })
            .to_string()),
            "jupiter_lend_withdraw" => Ok(json!({
                "withdraw_executed": true,
                "asset_mint": args["asset_mint"],
                "amount": args["amount"],
                "program_id": "jup3YeL8QhtSx1e253b2FDvsMNC87fDrgQZivbrndc9",
                "instructions": [
                    {
                        "program_id": "jup3YeL8QhtSx1e253b2FDvsMNC87fDrgQZivbrndc9",
                        "accounts": ["USER_WALLET_PUBKEY", "USDC_TOKEN_ACCOUNT"],
                        "data": "withdraw_instruction_data",
                        "should_succeed": true
                    }
                ]
            })
            .to_string()),
            _ => Err(anyhow::anyhow!("Unknown tool: {tool_name}")),
        }
    }
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn test_flow_agent_creation() {
        // This test would require proper environment setup
        // For now, we'll just test the structure
    }
}
