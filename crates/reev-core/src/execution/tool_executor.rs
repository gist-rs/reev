//! Executor for AI agent tools

use std::sync::Arc;

use anyhow::{anyhow, Result};
use solana_sdk::signature::Signer; // Import Signer trait
use tracing::{error, info, instrument};

use crate::execution::types::recovery_config::RecoveryConfig;
use crate::yml_schema::YmlStep;
// YmlToolCall is no longer used in V3 implementation

// use reev_lib::agent::RawInstruction; // Not used here
// use reev_lib::utils::{execute_transaction, get_keypair}; // Not used here
use reev_types::flow::{StepResult, WalletContext};

// Import context resolver and AgentTools
use reev_agent::enhanced::common::AgentTools;

// Import RigAgent for tool selection
use crate::execution::rig_agent::RigAgent;

/// Executor for AI agent tools
pub struct ToolExecutor {
    agent_tools: Option<Arc<AgentTools>>,
    rig_agent: Option<Arc<RigAgent>>,
    api_key: Option<String>,
    _model_name: String,
}

impl Default for ToolExecutor {
    fn default() -> Self {
        Self::new().expect("Failed to create default ToolExecutor")
    }
}

impl ToolExecutor {
    /// Create a new tool executor
    pub fn new() -> Result<Self> {
        let model_name =
            std::env::var("GLM_MODEL").unwrap_or_else(|_| "glm-4.6-coding".to_string());
        let api_key = std::env::var("ZAI_API_KEY").ok();

        info!("Creating ToolExecutor with model: {}", model_name);

        Ok(Self {
            agent_tools: None,
            rig_agent: None,
            api_key,
            _model_name: model_name,
        })
    }

    /// Set recovery configuration
    pub fn with_recovery_config(self, _config: RecoveryConfig) -> Self {
        // Recovery config would be stored here if needed
        self
    }

    /// Enable rig agent for tool selection
    pub async fn enable_rig_agent(mut self) -> Result<Self> {
        info!("Enabling rig agent for tool selection");
        match self.initialize_rig_agent().await {
            Ok(agent) => {
                self.rig_agent = Some(agent);
                info!("RigAgent successfully enabled");
                Ok(self)
            }
            Err(e) => {
                error!("Failed to initialize RigAgent: {}", e);
                // Return error instead of continuing without RigAgent
                Err(anyhow!("Failed to initialize RigAgent: {e}"))
            }
        }
    }

    /// Set custom tool executor
    pub fn with_tool_executor(self, _tool_executor: ToolExecutor) -> Self {
        self
    }

    /// Execute a step with actual tool execution
    #[instrument(skip(self, step, wallet_context))]
    pub async fn execute_step(
        &self,
        step: &YmlStep,
        wallet_context: &WalletContext,
    ) -> Result<StepResult> {
        info!("Executing step: {}", step.prompt);

        // Per V3 plan, always use RigAgent for tool selection
        // This should never be None since we initialize it in the constructor
        let rig_agent = self.rig_agent.as_ref().ok_or_else(|| {
            anyhow!("RigAgent not initialized - this should not happen with the V3 implementation")
        })?;

        info!("Using rig agent for tool selection based on refined prompt");
        rig_agent.execute_step_with_rig(step, wallet_context).await
    }

    /// Initialize RigAgent for tool selection
    async fn initialize_rig_agent(&self) -> Result<Arc<RigAgent>> {
        info!("Initializing RigAgent for tool selection");

        // Create a default wallet pubkey if agent_tools is None
        let wallet_pubkey = if let Some(ref tools) = self.agent_tools {
            // Get wallet pubkey from the sol_tool's key_map
            tools
                .sol_tool
                .key_map
                .get("WALLET_PUBKEY")
                .cloned()
                .unwrap_or_default()
        } else {
            // Load the default keypair from ~/.config/solana/id.json
            let keypair = reev_lib::get_keypair()
                .map_err(|e| anyhow!("Failed to load default keypair: {e}"))?;
            keypair.pubkey().to_string()
        };

        // Initialize AgentTools if not already set
        let agent_tools = if let Some(ref tools) = self.agent_tools {
            Arc::clone(tools)
        } else {
            // Load the keypair again to get the private key
            let keypair = reev_lib::get_keypair()
                .map_err(|e| anyhow!("Failed to load default keypair: {e}"))?;

            let mut key_map = std::collections::HashMap::new();
            key_map.insert("WALLET_PUBKEY".to_string(), wallet_pubkey.clone());
            key_map.insert("WALLET_KEYPAIR".to_string(), keypair.to_base58_string());
            Arc::new(AgentTools::new(key_map))
        };

        // Initialize RigAgent with AgentTools
        let rig_agent = Arc::new(
            RigAgent::new_with_tools(
                self.api_key.clone(),
                Some("glm-4.6-coding".to_string()),
                agent_tools,
            )
            .await?,
        );

        Ok(rig_agent)
    }

    /// Execute a step with wallet context and previous step history
    #[instrument(skip(self, step, wallet_context, previous_results))]
    pub async fn execute_step_with_history(
        &self,
        step: &YmlStep,
        wallet_context: &WalletContext,
        previous_results: &[StepResult],
    ) -> Result<StepResult> {
        info!("Executing step {} with previous history", step.prompt);

        // Per V3 plan, always use RigAgent for tool selection
        // This should never be None since we initialize it in the constructor
        let rig_agent = self.rig_agent.as_ref().ok_or_else(|| {
            anyhow!("RigAgent not initialized - this should not happen with the V3 implementation")
        })?;

        info!("Using rig agent with history for tool selection based on refined prompt");
        rig_agent
            .execute_step_with_rig_and_history(step, wallet_context, previous_results)
            .await
    }
}
