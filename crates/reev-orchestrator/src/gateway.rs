//! Orchestrator Gateway
//!
//! This module handles user prompt refinement and flow planning,
//! acting as the entry point for dynamic flow generation.

use crate::context_resolver::ContextResolver;
use crate::execution::PingPongExecutor;
use crate::generators::YmlGenerator;
use crate::recovery::engine::RecoveryMetrics;
use crate::recovery::{RecoveryConfig, RecoveryEngine};
use crate::Result;
use reev_lib::solana_env::environment::SolanaEnv;
use reev_types::flow::{AtomicMode, DynamicFlowPlan, StepResult, WalletContext};
use std::sync::Arc;
use tempfile::NamedTempFile;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tracing::{debug, error, info, instrument};

/// Orchestrator Gateway for processing user prompts and generating flows
pub struct OrchestratorGateway {
    /// Solana environment for placeholder resolution
    solana_env: Arc<Mutex<SolanaEnv>>,
    /// Context resolver for wallet and price information
    context_resolver: Arc<ContextResolver>,
    /// YML generator for creating benchmark files
    yml_generator: Arc<YmlGenerator>,
    /// Generated files tracker for cleanup
    generated_files: Arc<RwLock<Vec<NamedTempFile>>>,
    /// Recovery engine for Phase 3 recovery mechanisms
    recovery_engine: Arc<RwLock<RecoveryEngine>>,
    /// Recovery configuration
    #[allow(dead_code)]
    recovery_config: RecoveryConfig,
    /// Ping-pong executor for step-by-step coordination
    ping_pong_executor: Arc<RwLock<PingPongExecutor>>,
}

impl OrchestratorGateway {
    /// Create a new orchestrator gateway with default recovery configuration
    pub async fn new() -> Result<Self> {
        let recovery_config = RecoveryConfig::default();
        let recovery_engine = RecoveryEngine::new(recovery_config.clone());

        // Create Solana environment (uses hardcoded surfpool URL)
        let solana_env =
            Arc::new(Mutex::new(SolanaEnv::new().map_err(|e| {
                anyhow::anyhow!("Failed to create SolanaEnv: {e}")
            })?));

        // Create context resolver with Solana environment
        let context_resolver = Arc::new(ContextResolver::with_solana_env(solana_env.clone()));

        Ok(Self {
            solana_env,
            context_resolver: context_resolver.clone(),
            yml_generator: Arc::new(YmlGenerator::new()),
            generated_files: Arc::new(RwLock::new(Vec::new())),
            recovery_engine: Arc::new(RwLock::new(recovery_engine)),
            recovery_config,
            ping_pong_executor: Arc::new(RwLock::new(PingPongExecutor::new(
                30000,
                context_resolver,
            ))), // 30s timeout
        })
    }

    /// Create a new orchestrator gateway with custom recovery configuration
    pub async fn with_recovery_config(recovery_config: RecoveryConfig) -> Result<Self> {
        let recovery_engine = RecoveryEngine::new(recovery_config.clone());

        // Create Solana environment (uses hardcoded surfpool URL)
        let solana_env =
            Arc::new(Mutex::new(SolanaEnv::new().map_err(|e| {
                anyhow::anyhow!("Failed to create SolanaEnv: {e}")
            })?));

        // Create context resolver with Solana environment
        let context_resolver = Arc::new(ContextResolver::with_solana_env(solana_env.clone()));

        Ok(Self {
            solana_env,
            context_resolver: context_resolver.clone(),
            yml_generator: Arc::new(YmlGenerator::new()),
            generated_files: Arc::new(RwLock::new(Vec::new())),
            recovery_engine: Arc::new(RwLock::new(recovery_engine)),
            recovery_config,
            ping_pong_executor: Arc::new(RwLock::new(PingPongExecutor::new(
                30000,
                context_resolver,
            ))), // 30s timeout
        })
    }

    /// Process user request and generate dynamic flow
    #[instrument(skip(self))]
    pub async fn process_user_request(
        &self,
        user_prompt: &str,
        wallet_pubkey: &str,
    ) -> Result<(DynamicFlowPlan, String)> {
        info!("Processing user request: {}", user_prompt);

        // 1. Resolve wallet context
        let context = self
            .context_resolver
            .resolve_wallet_context(wallet_pubkey)
            .await?;
        debug!(
            "Resolved wallet context: {} SOL, ${:.2} total",
            context.sol_balance_sol(),
            context.total_value_usd
        );

        // 2. Refine user prompt with context
        let refined_prompt = self.refine_prompt(user_prompt, &context);
        debug!("Refined prompt: {}", refined_prompt);

        // 3. Generate flow plan (use original prompt, not refined)
        let flow_plan = self.generate_flow_plan(user_prompt, &context, None)?;
        debug!("Generated flow plan with {} steps", flow_plan.steps.len());

        // 4. Generate YML for bridge mode
        let yml_path = self.yml_generator.generate_yml(&flow_plan).await?;
        info!("Generated YML: {}", yml_path);

        Ok((flow_plan, yml_path))
    }

    /// Refine user prompt with wallet context
    pub fn refine_prompt(&self, prompt: &str, _context: &WalletContext) -> String {
        // Return original prompt unchanged for now - refinement will be done in step generation
        prompt.to_string()
    }

    /// Get a reference to the context resolver
    pub fn context_resolver(&self) -> &crate::context_resolver::ContextResolver {
        &self.context_resolver
    }

    /// Generate flow plan from refined prompt with Phase 3 recovery support
    pub fn generate_flow_plan(
        &self,
        prompt: &str,
        context: &WalletContext,
        atomic_mode: Option<AtomicMode>,
    ) -> Result<DynamicFlowPlan> {
        let flow_id = format!(
            "dynamic-{}-{}",
            chrono::Utc::now().timestamp(),
            uuid::Uuid::new_v4()
                .to_string()
                .chars()
                .take(8)
                .collect::<String>()
        );

        // Set atomic mode based on parameter or default to Strict
        let atomic_mode_for_logging = atomic_mode.unwrap_or(AtomicMode::Strict);
        let mut flow = DynamicFlowPlan::new(flow_id.clone(), prompt.to_string(), context.clone())
            .with_atomic_mode(atomic_mode_for_logging);

        // Parse intent and generate steps with recovery strategies
        let prompt_lower = prompt.to_lowercase();
        let has_swap = prompt_lower.contains("swap") || prompt_lower.contains("sol");
        let has_lend = prompt_lower.contains("lend") || prompt_lower.contains("yield");
        let has_multiply = prompt_lower.contains("multiply") || prompt_lower.contains("1.5x");
        let has_sol_percentage = prompt_lower.contains("%") && prompt_lower.contains("sol");

        // Check for complex flows first
        if has_swap && (has_lend || has_multiply) {
            // Complete multiplication strategy flow with recovery strategies
            // 1. Check current balances
            // 2. Swap SOL to USDC
            // 3. Lend USDC for yield
            // 4. Check final positions
            flow = flow
                .with_step(create_account_balance_step_with_recovery(context)?)
                .with_step(create_swap_step_with_recovery(context, prompt)?)
                .with_step(create_lend_step_with_recovery(context)?)
                .with_step(create_positions_check_step_with_recovery(context)?);
        } else if has_swap {
            // Single swap flow with recovery strategy
            flow = flow.with_step(create_swap_step_with_recovery(context, prompt)?);
        } else if has_lend {
            // Single lend flow with recovery strategy
            flow = flow.with_step(create_lend_step_with_recovery(context)?);
        } else if has_sol_percentage {
            // Percentage-based flow - assume swap with recovery strategy
            flow = flow.with_step(create_swap_step_with_recovery(context, prompt)?);
        } else {
            return Err(anyhow::anyhow!("Unsupported flow type: {prompt}"));
        }

        debug!(
            flow_id = %flow.flow_id,
            total_steps = %flow.steps.len(),
            atomic_mode = %atomic_mode_for_logging.as_str(),
            "Generated flow plan with recovery support"
        );

        Ok(flow)
    }

    /// Execute flow with Phase 3 recovery mechanisms
    #[instrument(skip(self, step_executor))]
    pub async fn execute_flow_with_recovery<F, Fut>(
        &self,
        flow_plan: DynamicFlowPlan,
        step_executor: F,
    ) -> Result<reev_types::flow::FlowResult>
    where
        F: FnMut(&reev_types::flow::DynamicStep, &Vec<reev_types::flow::StepResult>) -> Fut,
        Fut: std::future::Future<Output = anyhow::Result<reev_types::flow::StepResult>> + Send,
    {
        info!(
            flow_id = %flow_plan.flow_id,
            total_steps = %flow_plan.steps.len(),
            atomic_mode = %flow_plan.atomic_mode.as_str(),
            "Starting flow execution with Phase 3 recovery"
        );

        let mut recovery_engine = self.recovery_engine.write().await;
        let flow_result = recovery_engine
            .execute_flow_with_recovery(flow_plan, step_executor)
            .await;

        info!(
            flow_id = %flow_result.flow_id,
            success = %flow_result.success,
            successful_steps = %flow_result.metrics.successful_steps,
            failed_steps = %flow_result.metrics.failed_steps,
            "Flow execution with recovery completed"
        );

        Ok(flow_result)
    }

    /// Get recovery metrics for monitoring
    pub async fn get_recovery_metrics(&self) -> RecoveryMetrics {
        let recovery_engine = self.recovery_engine.read().await;
        recovery_engine.get_metrics().clone()
    }

    /// Reset recovery metrics
    pub async fn reset_recovery_metrics(&self) {
        let mut recovery_engine = self.recovery_engine.write().await;
        recovery_engine.reset_metrics();
    }

    /// Clean up generated temporary files
    #[instrument(skip(self))]
    pub async fn cleanup(&self) -> Result<()> {
        let mut files = self.generated_files.write().await;
        for file in files.drain(..) {
            if let Err(e) = file.close() {
                error!("Failed to cleanup temp file: {}", e);
            }
        }
        Ok(())
    }

    /// Execute flow plan with ping-pong coordination
    #[instrument(skip(self))]
    pub async fn execute_flow_with_ping_pong(
        &self,
        flow_plan: &DynamicFlowPlan,
        agent_type: &str,
    ) -> Result<Vec<StepResult>> {
        info!(
            "[Gateway] Starting ping-pong execution: {} steps",
            flow_plan.steps.len()
        );

        let mut executor = self.ping_pong_executor.write().await;
        executor.execute_flow_plan(flow_plan, agent_type).await
    }
}

/// Create a swap step based on context with recovery strategy
pub fn create_swap_step_with_recovery(
    context: &WalletContext,
    prompt: &str,
) -> Result<reev_types::flow::DynamicStep> {
    let sol_balance = context.sol_balance_sol();
    // Extract amount from prompt or use 50% default
    let prompt_lower = prompt.to_lowercase();
    let swap_amount = if prompt_lower.contains("1 sol") {
        "1".to_string()
    } else if prompt_lower.contains("0.5 sol") {
        "0.5".to_string()
    } else if prompt_lower.contains("25%") {
        (sol_balance * 0.25).to_string()
    } else if prompt_lower.contains("50%") {
        (sol_balance * 0.5).to_string()
    } else if prompt_lower.contains("100%") {
        sol_balance.to_string()
    } else {
        (sol_balance * 0.5).to_string() // Default 50%
    };

    let prompt_template = format!(
        "Swap {} SOL from wallet {} to USDC using Jupiter DEX. \
         Current SOL price: ${:.2}, USDC available for lending at 5-8% APY.",
        swap_amount,
        context.owner,
        context
            .get_token_price("So11111111111111111111111111111111111111112")
            .unwrap_or(150.0)
    );

    Ok(reev_types::flow::DynamicStep::new(
        "swap_1".to_string(),
        prompt_template,
        "Swap SOL to USDC using Jupiter".to_string(),
    )
    .with_tool("sol_tool")
    .with_estimated_time(30)
    .with_recovery(reev_types::flow::RecoveryStrategy::Retry { attempts: 3 }))
}

/// Create an account balance check step with recovery strategy
pub fn create_account_balance_step_with_recovery(
    context: &WalletContext,
) -> Result<reev_types::flow::DynamicStep> {
    let prompt_template = format!(
        "Check current wallet balances and positions for wallet {}. \
         Current SOL balance: {:.6}, Total portfolio value: ${:.2}. \
         This information is needed to plan the multiplication strategy.",
        context.owner,
        context.sol_balance_sol(),
        context.total_value_usd
    );

    Ok(reev_types::flow::DynamicStep::new(
        "balance_check".to_string(),
        prompt_template,
        "Check current wallet balances and positions".to_string(),
    )
    .with_tool("account_balance")
    .with_estimated_time(10)
    .with_recovery(reev_types::flow::RecoveryStrategy::Retry { attempts: 2 })
    .with_critical(false)) // Not critical for flow success
}

/// Create a lend step based on context with recovery strategy
pub fn create_lend_step_with_recovery(
    _context: &WalletContext,
) -> Result<reev_types::flow::DynamicStep> {
    let prompt_template =
        "Depositing USDC from the previous swap into Jupiter lending to earn yield. \
         Using the maximum available USDC balance for optimal returns."
            .to_string();

    Ok(reev_types::flow::DynamicStep::new(
        "lend_1".to_string(),
        prompt_template,
        "Deposit USDC into Jupiter lending".to_string(),
    )
    .with_tool("jupiter_earn_tool")
    .with_estimated_time(45)
    .with_recovery(reev_types::flow::RecoveryStrategy::Retry { attempts: 2 }))
    // Note: Lending step uses default critical behavior (true) for consistency
}

/// Create a positions check step with recovery strategy
pub fn create_positions_check_step_with_recovery(
    _context: &WalletContext,
) -> Result<reev_types::flow::DynamicStep> {
    let prompt_template =
        "Check final lending positions to verify the multiplication strategy results. \
         Confirm the USDC deposit was successful and track the expected yield generation."
            .to_string();

    Ok(reev_types::flow::DynamicStep::new(
        "positions_check".to_string(),
        prompt_template,
        "Check final lending positions".to_string(),
    )
    .with_tool("jupiter_positions")
    .with_estimated_time(15)
    .with_recovery(reev_types::flow::RecoveryStrategy::Retry { attempts: 2 })
    .with_critical(false)) // Not critical for flow success
}

/// Create a swap step based on context (legacy, non-recovery)
pub fn create_swap_step(
    context: &WalletContext,
    prompt: &str,
) -> Result<reev_types::flow::DynamicStep> {
    let sol_balance = context.sol_balance_sol();
    // Extract amount from prompt or use 50% default
    let prompt_lower = prompt.to_lowercase();
    let swap_amount = if prompt_lower.contains("1 sol") {
        "1".to_string()
    } else if prompt_lower.contains("0.5 sol") {
        "0.5".to_string()
    } else if prompt_lower.contains("25%") {
        (sol_balance * 0.25).to_string()
    } else if prompt_lower.contains("50%") {
        (sol_balance * 0.5).to_string()
    } else if prompt_lower.contains("100%") {
        sol_balance.to_string()
    } else {
        (sol_balance * 0.5).to_string() // Default 50%
    };

    let prompt_template = format!(
        "Swap {} SOL from wallet {} to USDC using Jupiter DEX. \
         Current SOL price: ${:.2}, USDC available for lending at 5-8% APY.",
        swap_amount,
        context.owner,
        context
            .get_token_price("So11111111111111111111111111111111111111112")
            .unwrap_or(150.0)
    );

    Ok(reev_types::flow::DynamicStep::new(
        "swap_1".to_string(),
        prompt_template,
        "Swap SOL to USDC using Jupiter".to_string(),
    )
    .with_tool("sol_tool")
    .with_estimated_time(30))
}

/// Create an account balance check step (legacy, non-recovery)
pub fn create_account_balance_step(
    context: &WalletContext,
) -> Result<reev_types::flow::DynamicStep> {
    let prompt_template = format!(
        "Check current wallet balances and positions for wallet {}. \
         Current SOL balance: {:.6}, Total portfolio value: ${:.2}.",
        context.owner,
        context.sol_balance_sol(),
        context.total_value_usd
    );

    Ok(reev_types::flow::DynamicStep::new(
        "balance_check".to_string(),
        prompt_template,
        "Check current wallet balances and positions".to_string(),
    )
    .with_tool("account_balance")
    .with_estimated_time(10)
    .with_critical(false)) // Not critical for flow success
}

/// Create a lend step based on context (legacy, non-recovery)
pub fn create_lend_step(_context: &WalletContext) -> Result<reev_types::flow::DynamicStep> {
    let prompt_template =
        "Depositing USDC from the previous swap into Jupiter lending to earn yield. \
         Using the maximum available USDC balance for optimal returns."
            .to_string();

    Ok(reev_types::flow::DynamicStep::new(
        "lend_1".to_string(),
        prompt_template,
        "Deposit USDC into Jupiter lending".to_string(),
    )
    .with_tool("jupiter_earn_tool")
    .with_estimated_time(45))
}

/// Create a positions check step (legacy, non-recovery)
pub fn create_positions_check_step(
    _context: &WalletContext,
) -> Result<reev_types::flow::DynamicStep> {
    let prompt_template =
        "Check final lending positions to verify the strategy results.".to_string();

    Ok(reev_types::flow::DynamicStep::new(
        "positions_check".to_string(),
        prompt_template,
        "Check final lending positions".to_string(),
    )
    .with_tool("jupiter_positions")
    .with_estimated_time(15)
    .with_critical(false)) // Not critical for flow success
}
