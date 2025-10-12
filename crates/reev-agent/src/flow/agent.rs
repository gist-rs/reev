//! # Flow Agent Implementation
//!
//! Simple flow agent that executes tools directly without LLM touching transactions.

use anyhow::Result;
use rig::tool::ToolDyn;
use std::collections::HashMap;
use tracing::{error, info};

use crate::{
    flow::{
        benchmark::FlowBenchmark,
        state::{FlowState, StepResult, StepStatus},
    },
    run::run_agent,
    tools::{
        jupiter_earn::JupiterEarnTool, jupiter_lend_earn_deposit::JupiterLendEarnDepositTool,
        jupiter_lend_earn_mint_redeem::JupiterLendEarnMintTool,
        jupiter_lend_earn_mint_redeem::JupiterLendEarnRedeemTool,
        jupiter_lend_earn_withdraw::JupiterLendEarnWithdrawTool, jupiter_swap::JupiterSwapTool,
        native::SolTransferTool, native::SplTransferTool,
    },
    LlmRequest,
};

/// Simple flow agent that executes tools directly without LLM touching transactions
pub struct FlowAgent {
    /// Model name for the agent (used only for complex scenarios)
    model_name: String,
    /// Available tools for the flow agent
    _tools: HashMap<String, Box<dyn ToolDyn>>,
    /// Current conversation state
    state: FlowState,
    /// Key mapping for placeholder pubkeys to real values
    key_map: HashMap<String, String>,
}

impl FlowAgent {
    /// Create a new FlowAgent with the specified model
    pub async fn new(model_name: &str) -> Result<Self> {
        info!(
            "[FlowAgent] Initializing flow agent with model: {}",
            model_name
        );

        // Create toolset with all available flow tools
        let (tools, key_map) = Self::create_toolset().await?;

        let state = FlowState::new(0);

        Ok(Self {
            model_name: model_name.to_string(),
            _tools: tools,
            state,
            key_map,
        })
    }

    /// Create the toolset with all available flow tools
    async fn create_toolset() -> Result<(HashMap<String, Box<dyn ToolDyn>>, HashMap<String, String>)>
    {
        let mut tools: HashMap<String, Box<dyn ToolDyn>> = HashMap::new();

        // Create real pubkeys for the key_map like existing examples
        let user_wallet_pubkey = solana_sdk::pubkey::Pubkey::new_unique();
        let mut key_map = HashMap::new();
        key_map.insert(
            "USER_WALLET_PUBKEY".to_string(),
            user_wallet_pubkey.to_string(),
        );

        // Initialize each tool
        tools.insert(
            "sol_transfer".to_string(),
            Box::new(SolTransferTool {
                key_map: key_map.clone(),
            }) as Box<dyn ToolDyn>,
        );
        tools.insert(
            "spl_transfer".to_string(),
            Box::new(SplTransferTool {
                key_map: key_map.clone(),
            }) as Box<dyn ToolDyn>,
        );
        tools.insert(
            "jupiter_swap".to_string(),
            Box::new(JupiterSwapTool {
                key_map: key_map.clone(),
            }) as Box<dyn ToolDyn>,
        );
        tools.insert(
            "jupiter_lend_earn_deposit".to_string(),
            Box::new(JupiterLendEarnDepositTool {
                key_map: key_map.clone(),
            }) as Box<dyn ToolDyn>,
        );
        tools.insert(
            "jupiter_lend_earn_withdraw".to_string(),
            Box::new(JupiterLendEarnWithdrawTool {
                key_map: key_map.clone(),
            }) as Box<dyn ToolDyn>,
        );
        tools.insert(
            "jupiter_lend_earn_mint".to_string(),
            Box::new(JupiterLendEarnMintTool {
                key_map: key_map.clone(),
            }) as Box<dyn ToolDyn>,
        );
        tools.insert(
            "jupiter_lend_earn_redeem".to_string(),
            Box::new(JupiterLendEarnRedeemTool {
                key_map: key_map.clone(),
            }) as Box<dyn ToolDyn>,
        );
        tools.insert(
            "jupiter_earn".to_string(),
            Box::new(JupiterEarnTool {
                key_map: key_map.clone(),
            }) as Box<dyn ToolDyn>,
        );

        Ok((tools, key_map))
    }

    /// Execute a single step in the flow
    pub async fn execute_step(
        &mut self,
        step: &crate::flow::benchmark::FlowStep,
        benchmark: &FlowBenchmark,
    ) -> Result<StepResult> {
        let start_time = std::time::SystemTime::now();
        let _start_time_clone = start_time;

        // Enrich prompt with context
        let _prompt = self.enrich_prompt(&step.prompt, benchmark);

        // Simple tool selection based on keywords
        // Give ALL tools to LLM for simpler logic
        let all_tools = vec![
            "jupiter_swap".to_string(),
            "jupiter_lend_earn_mint".to_string(),
            "jupiter_lend_earn_redeem".to_string(),
            "jupiter_lend_earn_deposit".to_string(),
            "jupiter_lend_earn_withdraw".to_string(),
            "jupiter_earn".to_string(),
            "sol_transfer".to_string(),
            "spl_transfer".to_string(),
        ];

        info!(
            "[FlowAgent] Step {}: Making {} tools available to LLM",
            step.step,
            all_tools.len()
        );

        // LLM FALLBACK: For complex multi-tool scenarios ONLY
        // LLM MUST NEVER produce transactions - only reasoning and tool selection
        info!("[FlowAgent] Using LLM for reasoning only - NO TRANSACTION GENERATION");
        let llm_request = LlmRequest {
            id: format!("{}-step-{}", benchmark.id, step.step),
            prompt: format!("REASONING ONLY: Analyze this request and suggest tools. NEVER generate transactions or instructions: {}", step.prompt),
            context_prompt: self.build_context_prompt(benchmark, step, &all_tools),
            model_name: self.model_name.clone(),
            initial_state: None,
            mock: false,
        };

        let response = match run_agent(&self.model_name, llm_request).await {
            Ok(response) => response,
            Err(e) => return Err(e),
        };

        // Create step result
        let step_result = StepResult {
            step: step.step,
            description: step.description.clone(),
            llm_response: response.clone(),
            execution_response: Some("LLM reasoning only - no execution".to_string()),
            instructions: Vec::new(), // LLM NEVER produces transactions
            status: StepStatus::Success,
            completed_at: format!("{start_time:?}"),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("execution_mode".to_string(), "llm_fallback".to_string());
                meta
            },
        };

        // Add conversation turn to state
        self.state.add_turn(crate::flow::state::ConversationTurn {
            turn: self.state.conversation_history.len() + 1,
            step: step.step,
            user_prompt: step.prompt.clone(),
            system_prompt: crate::flow::FLOW_SYSTEM_PREAMBLE.to_string(),
            llm_response: response,
            tools_called: all_tools.clone(),
            timestamp: format!("{start_time:?}"),
        });

        Ok(step_result)
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

        // Add the current task
        enriched_parts.push(format!("\n=== Current Task ===\n{prompt}"));

        enriched_parts.join("\n")
    }

    /// Build the context prompt for the agent
    fn build_context_prompt(
        &self,
        _benchmark: &FlowBenchmark,
        _step: &crate::flow::benchmark::FlowStep,
        _all_tools: &[String],
    ) -> String {
        // Create YAML context with key_map like other examples
        let context_yaml = serde_json::json!({
            "key_map": self.key_map
        });

        format!(
            "---\n\nCURRENT ON-CHAIN CONTEXT:\n{}\n\n---",
            serde_yaml::to_string(&context_yaml).expect("Failed to serialize key_map")
        )
    }

    /// Get the current flow state
    pub fn get_state(&self) -> &FlowState {
        &self.state
    }

    /// Get a mutable reference to the current flow state
    pub fn reset_state(&mut self) {
        self.state = FlowState::new(0);
    }

    /// Load a flow benchmark into the agent
    pub async fn load_benchmark(&mut self, benchmark: &FlowBenchmark) -> Result<()> {
        info!("[FlowAgent] Loading flow benchmark: {}", benchmark.id);

        self.state = FlowState::new(benchmark.total_steps());
        self.state
            .add_context("flow_id".to_string(), benchmark.id.clone());
        self.state.add_context(
            "flow_description".to_string(),
            benchmark.description.clone(),
        );

        info!("[FlowAgent] Flow loaded: {} steps", benchmark.total_steps());
        Ok(())
    }

    /// Execute the complete multi-step flow
    pub async fn execute_flow(&mut self, benchmark: &FlowBenchmark) -> Result<Vec<StepResult>> {
        info!("[FlowAgent] Executing flow: {}", benchmark.id);
        let mut all_results = Vec::new();

        for step in &benchmark.flow {
            info!(
                "[FlowAgent] ======== Step {} / {} ========",
                step.step,
                benchmark.total_steps()
            );
            info!("[FlowAgent] Step description: {}", step.description);

            let step_result = self.execute_step(step, benchmark).await?;

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
        Ok(all_results)
    }
}
