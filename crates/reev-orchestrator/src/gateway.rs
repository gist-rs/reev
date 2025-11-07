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
use reev_types::flow::{AtomicMode, DynamicFlowPlan, ExecutionResult, StepResult, WalletContext};
use reev_types::tools::ToolName;

use reev_db::config::DatabaseConfig;
use reev_db::writer::DatabaseWriter;
use regex::Regex;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::NamedTempFile;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tracing::{debug, error, info, instrument};

/// Simple user intent analysis for dynamic YML generation
#[derive(Debug, Clone)]
pub struct UserIntent {
    /// Type of user request
    pub intent_type: String, // swap, lend, withdraw, complex
    /// Extracted parameters from user request
    pub parameters: std::collections::HashMap<String, String>,
    /// Primary goal description
    pub primary_goal: String,
    /// Required tools to execute this intent
    pub required_tools: Vec<String>,
    /// Confidence in intent analysis
    pub confidence: f32,
}

/// Orchestrator Gateway for processing user prompts and generating flows
pub struct OrchestratorGateway {
    /// Solana environment for placeholder resolution
    _solana_env: Arc<Mutex<SolanaEnv>>,
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
    /// Create enhanced balance check step with detailed wallet context
    fn create_enhanced_balance_check_step(
        &self,
        context: &WalletContext,
        step_id: &str,
    ) -> Result<reev_types::flow::DynamicStep> {
        let usdc_balance = context
            .get_token_balance("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")
            .map(|b| b.balance as f64 / 1_000_000.0)
            .unwrap_or(0.0);

        let prompt_template = format!(
            "Check wallet {} balances and portfolio. Current SOL: {:.6}, USDC: {:.2}, Total: ${:.2}. \
             Calculate available SOL for operations and prepare for percentage-based strategy. \
             Wallet pubkey: {}. Available for strategy: {:.6} SOL (50% of balance).",
            context.owner,
            context.sol_balance_sol(),
            usdc_balance,
            context.total_value_usd,
            context.owner,
            context.sol_balance_sol() * 0.5
        );

        Ok(reev_types::flow::DynamicStep::new(
            format!("{step_id}_balance_check"),
            prompt_template,
            "Initial portfolio assessment and balance verification".to_string(),
        )
        .with_tool(ToolName::GetAccountBalance)
        .with_estimated_time(10)
        .with_recovery(reev_types::flow::RecoveryStrategy::Retry { attempts: 3 }))
    }

    /// Create enhanced calculation step with detailed strategy planning
    fn create_enhanced_calculation_step(
        &self,
        context: &WalletContext,
        target_multiplier: f64,
        step_id: &str,
    ) -> Result<reev_types::flow::DynamicStep> {
        let sol_balance = context.sol_balance_sol();
        let sol_price = context
            .get_token_price("So11111111111111111111111111111111111111112")
            .unwrap_or(150.0);

        let available_sol = sol_balance * 0.5; // 50% strategy
        let estimated_usdc_after_swap = available_sol * sol_price;
        let target_usdc = estimated_usdc_after_swap * target_multiplier;

        let prompt_template = format!(
            "Calculate multiplication strategy for wallet {}: \
             Available SOL: {:.6} (50% of balance = {:.6}), \
             Estimated USDC after swap: {:.2}, \
             Target USDC after multiplication: {:.2} ({}x), \
             Required yield: {:.2}% from lending to achieve target. \
             SOL price: ${:.2}. Wallet pubkey: {}",
            context.owner,
            sol_balance,
            available_sol,
            estimated_usdc_after_swap,
            target_usdc,
            target_multiplier,
            (target_multiplier - 1.0) * 100.0,
            sol_price,
            context.owner
        );

        Ok(reev_types::flow::DynamicStep::new(
            format!("{step_id}_calculation"),
            prompt_template,
            "Strategy calculation and parameter planning".to_string(),
        )
        .with_estimated_time(5)
        .with_critical(true))
    }

    /// Create enhanced swap step with transaction details
    fn create_enhanced_swap_step_with_details(
        &self,
        context: &WalletContext,
        swap_amount_sol: f64,
        step_id: &str,
    ) -> Result<reev_types::flow::DynamicStep> {
        let sol_price = context
            .get_token_price("So11111111111111111111111111111111111111112")
            .unwrap_or(150.0);

        let estimated_usdc = swap_amount_sol * sol_price;
        let slippage_tolerance = 0.03; // 3%

        let prompt_template = format!(
            "Execute Jupiter swap: {} SOL → USDC for wallet {}. \
             Expected output: {:.2} USDC (slippage: ±{:.1}%). \
             SOL price: ${:.2}. \
             Transaction details: source wallet {}, \
             input amount: {:.6} SOL ({} lamports), \
             minimum received: {:.2} USDC. \
             Monitor for price impact and execution success.",
            swap_amount_sol,
            context.owner,
            estimated_usdc,
            slippage_tolerance * 100.0,
            sol_price,
            context.owner,
            swap_amount_sol,
            (swap_amount_sol * 1_000_000_000.0) as u64,
            estimated_usdc * (1.0 - slippage_tolerance)
        );

        Ok(reev_types::flow::DynamicStep::new(
            format!("{step_id}_swap"),
            prompt_template,
            "Jupiter DEX swap execution with detailed parameters".to_string(),
        )
        .with_tool(ToolName::JupiterSwap)
        .with_estimated_time(30)
        .with_recovery(reev_types::flow::RecoveryStrategy::Retry { attempts: 2 })
        .with_critical(true))
    }

    /// Create enhanced lending step with position details
    fn create_enhanced_lend_step_with_details(
        &self,
        context: &WalletContext,
        lend_amount_usdc: f64,
        target_apy: f64,
        step_id: &str,
    ) -> Result<reev_types::flow::DynamicStep> {
        let current_apy_range = "5-12";
        let expected_daily_yield = lend_amount_usdc * (target_apy / 100.0) / 365.0;

        let prompt_template = format!(
            "Deposit {} USDC into Jupiter lending for wallet {}. \
             Target APY: {:.1}%, Market range: {}%. \
             Expected daily yield: ${:.4}. \
             Position details: wallet {}, deposit amount: {:.2} USDC ({} micro-USDC), \
             strategy: maximize yield within risk tolerance. \
             Monitor APY changes and position health.",
            lend_amount_usdc,
            context.owner,
            target_apy,
            current_apy_range,
            expected_daily_yield,
            context.owner,
            lend_amount_usdc,
            (lend_amount_usdc * 1_000_000.0) as u64
        );

        Ok(reev_types::flow::DynamicStep::new(
            format!("{step_id}_lend"),
            prompt_template,
            "Jupiter lending position creation with detailed parameters".to_string(),
        )
        .with_tool(ToolName::JupiterLendEarnDeposit)
        .with_estimated_time(45)
        .with_recovery(reev_types::flow::RecoveryStrategy::Retry { attempts: 2 })
        .with_critical(true))
    }

    /// Generate enhanced 300-series flow with rich context and step details
    pub fn generate_enhanced_300_flow(
        &self,
        prompt: &str,
        context: &WalletContext,
    ) -> Result<reev_types::flow::DynamicFlowPlan> {
        let flow_id = format!(
            "enhanced-300-{}-{}",
            chrono::Utc::now().timestamp(),
            uuid::Uuid::new_v4()
                .to_string()
                .chars()
                .take(8)
                .collect::<String>()
        );

        let mut flow = reev_types::flow::DynamicFlowPlan::new(
            flow_id.clone(),
            prompt.to_string(),
            context.clone(),
        )
        .with_atomic_mode(reev_types::flow::AtomicMode::Strict);

        // Parse strategy parameters
        let prompt_lower = prompt.to_lowercase();
        let target_multiplier = if prompt_lower.contains("1.5x") {
            1.5
        } else if prompt_lower.contains("2x") {
            2.0
        } else {
            1.5 // default
        };

        let sol_balance = context.sol_balance_sol();
        let swap_amount = if prompt_lower.contains("%") {
            if let Some(percent_str) = extract_percentage(prompt) {
                let percentage = percent_str.parse::<f64>().unwrap_or(50.0) / 100.0;
                sol_balance * percentage
            } else {
                sol_balance * 0.5
            }
        } else {
            sol_balance * 0.5
        };

        // Generate enhanced flow steps with detailed context
        flow = flow
            .with_step(self.create_enhanced_balance_check_step(context, "step1")?)
            .with_step(self.create_enhanced_calculation_step(
                context,
                target_multiplier,
                "step2",
            )?)
            .with_step(self.create_enhanced_swap_step_with_details(
                context,
                swap_amount,
                "step3",
            )?)
            .with_step(self.create_enhanced_lend_step_with_details(
                context,
                swap_amount * 150.0,
                8.5,
                "step4",
            )?)
            .with_step(create_positions_check_step_with_recovery(context)?);

        info!(
            flow_id = %flow.flow_id,
            total_steps = %flow.steps.len(),
            swap_amount = %swap_amount,
            target_multiplier = %target_multiplier,
            "Generated enhanced 300-series flow with detailed steps"
        );

        Ok(flow)
    }

    /// Generate simple dynamic flow plan for user requests (like 100/200 series)
    ///
    /// Note: This is for USER-FACING dynamic requests only.
    /// 300-series benchmarks are STATIC YML files for internal testing.
    ///
    /// User flows: Natural language → Simple dynamic YML → Existing static runner
    /// 300-series: Pre-defined comprehensive test cases → Runner for systematic testing
    fn generate_simple_dynamic_flow(
        &self,
        prompt: &str,
        context: &WalletContext,
        intent: &UserIntent,
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

        let flow = DynamicFlowPlan::new(flow_id.clone(), prompt.to_string(), context.clone())
            .with_atomic_mode(AtomicMode::Strict);

        // Simple flows: swap, lend, withdraw - like existing 100/200 series
        match intent.intent_type.as_str() {
            "swap" => {
                let sol_amount = intent
                    .parameters
                    .get("sol_amount")
                    .and_then(|v| v.parse::<f64>().ok())
                    .unwrap_or(1.0);

                Ok(flow
                    .with_step(create_account_balance_step_with_recovery(context)?)
                    .with_step(
                        self.create_enhanced_swap_step_with_details(context, sol_amount, "swap")?,
                    )
                    .with_step(create_positions_check_step_with_recovery(context)?))
            }
            "lend" => {
                let usdc_amount = intent
                    .parameters
                    .get("usdc_amount")
                    .and_then(|v| v.parse::<f64>().ok())
                    .unwrap_or(100.0);

                Ok(flow
                    .with_step(create_account_balance_step_with_recovery(context)?)
                    .with_step(self.create_enhanced_lend_step_with_details(
                        context,
                        usdc_amount,
                        8.5,
                        "lend",
                    )?)
                    .with_step(create_positions_check_step_with_recovery(context)?))
            }
            "complex" => {
                // Multi-step strategies
                let sol_amount = context.sol_balance_sol() * 0.5;
                Ok(flow
                    .with_step(create_account_balance_step_with_recovery(context)?)
                    .with_step(
                        self.create_enhanced_swap_step_with_details(
                            context, sol_amount, "complex",
                        )?,
                    )
                    .with_step(self.create_enhanced_lend_step_with_details(
                        context,
                        sol_amount * 150.0,
                        8.5,
                        "complex",
                    )?)
                    .with_step(create_positions_check_step_with_recovery(context)?))
            }
            _ => {
                // Default to simple flow
                let sol_amount = context.sol_balance_sol() * 1.0;
                Ok(flow
                    .with_step(create_account_balance_step_with_recovery(context)?)
                    .with_step(
                        self.create_enhanced_swap_step_with_details(
                            context, sol_amount, "default",
                        )?,
                    )
                    .with_step(create_positions_check_step_with_recovery(context)?))
            }
        }
    }

    /// Create emergency withdraw step for crisis situations
    #[allow(dead_code)]
    fn create_emergency_withdraw_step(
        &self,
        context: &WalletContext,
    ) -> Result<reev_types::flow::DynamicStep> {
        let prompt_template = format!(
            "Emergency withdrawal of all Jupiter lending positions for wallet {}. \
             Crisis protocol activated - prioritize capital preservation over yield. \
             Withdraw all positions to stable assets immediately. \
             Current portfolio value: ${:.2}, SOL balance: {:.6}.",
            context.owner,
            context.total_value_usd,
            context.sol_balance_sol()
        );

        Ok(reev_types::flow::DynamicStep::new(
            "emergency_withdraw".to_string(),
            prompt_template,
            "Emergency withdrawal from all lending positions".to_string(),
        )
        .with_tool(ToolName::JupiterLendEarnWithdraw)
        .with_estimated_time(30)
        .with_recovery(reev_types::flow::RecoveryStrategy::Retry { attempts: 3 })
        .with_critical(true))
    }

    /// Create emergency swap to stable assets step
    #[allow(dead_code)]
    fn create_emergency_swap_to_stable_step(
        &self,
        context: &WalletContext,
    ) -> Result<reev_types::flow::DynamicStep> {
        let prompt_template = format!(
            "Emergency swap all volatile assets to stablecoins for wallet {}. \
             Crisis mode - convert SOL to USDC or other stable assets. \
             Use minimal SOL for transaction fees only. \
             Current SOL: {:.6}, Portfolio value: ${:.2}.",
            context.owner,
            context.sol_balance_sol(),
            context.total_value_usd
        );

        Ok(reev_types::flow::DynamicStep::new(
            "emergency_swap_to_stable".to_string(),
            prompt_template,
            "Emergency swap to stable assets".to_string(),
        )
        .with_tool(ToolName::JupiterSwap)
        .with_estimated_time(25)
        .with_recovery(reev_types::flow::RecoveryStrategy::Retry { attempts: 3 })
        .with_critical(true))
    }

    /// Create multi-pool lending step for advanced strategies
    #[allow(dead_code)]
    fn create_multi_pool_lend_step(
        &self,
        context: &WalletContext,
        usdc_amount: f64,
    ) -> Result<reev_types::flow::DynamicStep> {
        let prompt_template = format!(
            "Advanced yield farming optimization for wallet {}: \
             Allocate {:.2} USDC across multiple Jupiter lending pools for optimal yield. \
             Strategy: 60% primary pool, 30% secondary pool, 10% experimental pool. \
             Target APY: 10-15% across diversified positions. \
             Monitor for impermanent loss and rebalance as needed. \
             Wallet: {}, Total portfolio: ${:.2}.",
            context.owner, usdc_amount, context.owner, context.total_value_usd
        );

        Ok(reev_types::flow::DynamicStep::new(
            "multi_pool_lend".to_string(),
            prompt_template,
            "Advanced multi-pool yield optimization".to_string(),
        )
        .with_tool(ToolName::JupiterLendEarnDeposit)
        .with_estimated_time(60)
        .with_recovery(reev_types::flow::RecoveryStrategy::Retry { attempts: 2 })
        .with_critical(true))
    }
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

        // Create database writer for ping-pong execution
        let db_config = DatabaseConfig::local("reev_orchestrator.db");
        let database = Arc::new(DatabaseWriter::new(db_config).await?);

        Ok(Self {
            _solana_env: solana_env,
            context_resolver: context_resolver.clone(),
            yml_generator: Arc::new(YmlGenerator::new()),
            generated_files: Arc::new(RwLock::new(Vec::new())),
            recovery_engine: Arc::new(RwLock::new(recovery_engine)),
            recovery_config,
            ping_pong_executor: Arc::new(RwLock::new(PingPongExecutor::new(
                300_000, // 5 minute timeout
                context_resolver,
                database,
            ))),
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

        // Create database writer for ping-pong execution
        let db_config = DatabaseConfig::local("reev_orchestrator.db");
        let database = Arc::new(DatabaseWriter::new(db_config).await?);

        Ok(Self {
            _solana_env: solana_env,
            context_resolver: context_resolver.clone(),
            yml_generator: Arc::new(YmlGenerator::new()),
            generated_files: Arc::new(RwLock::new(Vec::new())),
            recovery_engine: Arc::new(RwLock::new(recovery_engine)),
            recovery_config,
            ping_pong_executor: Arc::new(RwLock::new(PingPongExecutor::new(
                30000,
                context_resolver,
                database,
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

        // 3. Generate enhanced flow plan using real context
        let flow_plan = self
            .generate_enhanced_flow_plan(user_prompt, &context, None)
            .await?;
        debug!(
            "Generated enhanced flow plan with {} steps using real wallet data",
            flow_plan.steps.len()
        );

        // 4. Generate YML for bridge mode
        let yml_path = self.yml_generator.generate_yml(&flow_plan).await?;
        info!("Generated YML: {}", yml_path);

        Ok((flow_plan, yml_path))
    }

    /// Refine user prompt with wallet context
    pub fn refine_prompt(&self, prompt: &str, context: &WalletContext) -> String {
        // Enhance prompt with real wallet context for better flow generation
        let context_info = format!(
            "\n\nWALLET CONTEXT:\n- SOL Balance: {:.6} SOL\n- Total Portfolio Value: ${:.2}\n- Available Tokens: {}\n\n",
            context.sol_balance_sol(),
            context.total_value_usd,
            context.token_balances.len()
        );

        format!("{context_info}{prompt}")
    }

    /// Get a reference to the context resolver
    pub fn context_resolver(&self) -> &crate::context_resolver::ContextResolver {
        &self.context_resolver
    }

    /// Generate enhanced flow plan from refined prompt with real context data
    /// Simple intent analysis for user requests (like 100/200 series logic)
    pub async fn analyze_user_intent(
        &self,
        prompt: &str,
        context: &WalletContext,
    ) -> Result<UserIntent> {
        // For now, use rule-based analysis - ZAI agent integration will be added later
        debug!("[orchestrator] Using rule-based intent analysis (LLM integration to be added)");
        self.analyze_user_intent_rule_based(prompt, context).await
    }

    /// Fallback rule-based intent analysis
    async fn analyze_user_intent_rule_based(
        &self,
        prompt: &str,
        _context: &WalletContext,
    ) -> Result<UserIntent> {
        debug!("[orchestrator] Using rule-based intent analysis");

        let prompt_lower = prompt.to_lowercase();

        let (intent_type, primary_goal, parameters) = if prompt_lower.contains("multiply")
            || prompt_lower.contains("then")
            || (prompt_lower.contains("lend") && prompt_lower.contains("swap"))
        {
            (
                "complex".to_string(),
                "Multi-step DeFi strategy execution".to_string(),
                {
                    let mut params = std::collections::HashMap::new();
                    params.insert("strategy".to_string(), "swap_then_lend".to_string());

                    // Extract percentage if mentioned
                    if let Some(pct_match) = regex::Regex::new(r"(\d+)%").unwrap().captures(prompt)
                    {
                        if let Some(pct) = pct_match.get(1) {
                            params.insert("percentage".to_string(), pct.as_str().to_string());
                        }
                    }

                    params
                },
            )
        } else if prompt_lower.contains("lend")
            || prompt_lower.contains("deposit")
            || prompt_lower.contains("yield")
        {
            (
                "lend".to_string(),
                "Deposit funds for yield generation".to_string(),
                {
                    let mut params = std::collections::HashMap::new();
                    // Extract USDC amount if mentioned
                    if let Some(usdc_match) = regex::Regex::new(r"(\d+(?:\.\d+)?)\s*usdc")
                        .unwrap()
                        .captures(prompt)
                    {
                        if let Some(usdc) = usdc_match.get(1) {
                            params.insert("usdc_amount".to_string(), usdc.as_str().to_string());
                        }
                    }
                    params
                },
            )
        } else if prompt_lower.contains("swap")
            || prompt_lower.contains("exchange")
            || prompt_lower.contains("convert")
        {
            (
                "swap".to_string(),
                "Swap tokens between different assets".to_string(),
                {
                    let mut params = std::collections::HashMap::new();
                    if let Some(sol_match) = regex::Regex::new(r"(\d+(?:\.\d+)?)\s*sol")
                        .unwrap()
                        .captures(prompt)
                    {
                        if let Some(sol) = sol_match.get(1) {
                            params.insert("sol_amount".to_string(), sol.as_str().to_string());
                        }
                    }
                    params
                },
            )
        } else {
            (
                "check_positions".to_string(),
                "Check current positions and balances".to_string(),
                std::collections::HashMap::new(),
            )
        };

        let required_tools = match intent_type.as_str() {
            "lend" => vec![
                reev_types::ToolName::GetAccountBalance.to_string(),
                reev_types::ToolName::JupiterLendEarnDeposit.to_string(),
            ],
            "complex" => vec![
                reev_types::ToolName::GetAccountBalance.to_string(),
                reev_types::ToolName::JupiterSwap.to_string(),
                reev_types::ToolName::JupiterLendEarnDeposit.to_string(),
            ],
            "check_positions" => vec![
                reev_types::ToolName::GetAccountBalance.to_string(),
                reev_types::ToolName::GetJupiterLendEarnPosition.to_string(),
            ],
            _ => vec![
                reev_types::ToolName::GetAccountBalance.to_string(),
                reev_types::ToolName::JupiterSwap.to_string(),
            ],
        };

        Ok(UserIntent {
            intent_type,
            parameters,
            primary_goal,
            required_tools,
            confidence: 0.7, // Lower confidence for rule-based
        })
    }

    /// Generate simple dynamic YML from user intent (like 100/200 series)
    pub async fn generate_dynamic_yml(
        &self,
        intent: &UserIntent,
        prompt: &str,
        context: &WalletContext,
    ) -> Result<String> {
        let flow_plan = self.generate_simple_dynamic_flow(prompt, context, intent)?;
        let yml_path = self.yml_generator.generate_yml(&flow_plan).await?;
        Ok(yml_path)
    }

    pub async fn generate_enhanced_flow_plan(
        &self,
        prompt: &str,
        context: &WalletContext,
        _atomic_mode: Option<AtomicMode>,
    ) -> Result<DynamicFlowPlan> {
        // Simple intent analysis for user requests (like 100/200 series)
        let intent = self.analyze_user_intent(prompt, context).await?;

        info!(
            intent_type = %intent.intent_type,
            primary_goal = %intent.primary_goal,
            confidence = %intent.confidence,
            "Simple user intent analysis completed"
        );

        // Generate simple dynamic flow for user requests
        self.generate_simple_dynamic_flow(prompt, context, &intent)
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

    /// Determine if a flow should use database-based execution
    ///
    /// Checks if the flow has `flow_type: "dynamic"` to route to PingPongExecutor
    /// instead of file-based execution.
    ///
    /// # Arguments
    /// * `yml_path` - Path to the YML file to check
    ///
    /// # Returns
    /// * `bool` - true if should use database flow, false for file-based
    #[instrument(skip_all)]
    pub async fn should_use_database_flow(&self, yml_path: &PathBuf) -> Result<bool> {
        let content = tokio::fs::read_to_string(yml_path)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to read YML file: {e}"))?;

        // Parse YAML to check for flow_type field
        let yaml: serde_yaml::Value = serde_yaml::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse YAML: {e}"))?;

        // Check if flow_type is "dynamic"
        if let Some(flow_type) = yaml.get("flow_type").and_then(|v| v.as_str()) {
            let use_database = flow_type == "dynamic";
            debug!(
                "[Gateway] Flow type: {}, using database: {}",
                flow_type, use_database
            );
            return Ok(use_database);
        }

        // Default to file-based for backward compatibility
        debug!("[Gateway] No flow_type specified, using file-based execution");
        Ok(false)
    }

    /// Execute dynamic flow with consolidation
    ///
    /// This method routes dynamic flows to PingPongExecutor for database-based
    /// execution with automatic consolidation.
    ///
    /// # Arguments
    /// * `flow_plan` - Dynamic flow plan to execute
    /// * `agent_type` - Type of agent to use for execution
    ///
    /// # Returns
    /// * `Result<ExecutionResult>` - Execution result with consolidation info
    #[instrument(skip(self))]
    pub async fn execute_dynamic_flow_with_consolidation(
        &self,
        flow_plan: &DynamicFlowPlan,
        agent_type: &str,
    ) -> Result<ExecutionResult> {
        info!(
            "[Gateway] Starting dynamic flow with consolidation: {} steps",
            flow_plan.steps.len()
        );

        let mut executor = self.ping_pong_executor.write().await;
        executor
            .execute_flow_plan_with_ping_pong(flow_plan, agent_type)
            .await
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
    .with_tool(ToolName::JupiterSwap)
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
    .with_tool(ToolName::GetAccountBalance)
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
    .with_tool(ToolName::GetJupiterLendEarnPosition)
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
    .with_tool(ToolName::GetJupiterLendEarnPosition)
    .with_estimated_time(15)
    .with_recovery(reev_types::flow::RecoveryStrategy::Retry { attempts: 2 })
    .with_critical(false)) // Not critical for flow success
}

/// Create an enhanced swap step using real wallet context
pub fn create_enhanced_swap_step(
    context: &WalletContext,
    prompt: &str,
) -> Result<reev_types::flow::DynamicStep> {
    let sol_balance = context.sol_balance_sol();
    let sol_price = context
        .get_token_price("So11111111111111111111111111111111111111112")
        .unwrap_or(150.0);

    // Parse amount from prompt with real balance validation
    let prompt_lower = prompt.to_lowercase();
    let (swap_amount, _swap_reason) = if prompt_lower.contains("1 sol") {
        ("1".to_string(), "User requested 1 SOL swap".to_string())
    } else if prompt_lower.contains("0.5 sol") {
        ("0.5".to_string(), "User requested 0.5 SOL swap".to_string())
    } else if let Some(percent_str) = extract_percentage(prompt) {
        let percentage = percent_str.parse::<f64>().unwrap_or(50.0) / 100.0;
        let amount = (sol_balance * percentage).max(0.001).min(sol_balance);
        (
            format!("{amount:.6}"),
            format!("User requested {percent_str}% of SOL"),
        )
    } else {
        // Default to 50% with smart cap
        let default_amount = (sol_balance * 0.5).max(0.001).min(sol_balance);
        (
            format!("{default_amount:.6}"),
            "Default: 50% of SOL balance".to_string(),
        )
    };

    let _estimated_usdc = (swap_amount.parse::<f64>().unwrap_or(0.0) * sol_price).to_string();

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
    .with_tool(ToolName::JupiterSwap)
    .with_estimated_time(30))
}

/// Create an enhanced lending step using real wallet context
pub fn create_enhanced_lend_step(context: &WalletContext) -> Result<reev_types::flow::DynamicStep> {
    let usdc_balance = context
        .token_balances
        .get("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")
        .map(|balance| {
            let decimals = balance.decimals.unwrap_or(6) as f64;
            balance.balance as f64 / 10_f64.powi(decimals as i32)
        })
        .unwrap_or(0.0);

    let sol_balance_value = context.sol_balance_sol()
        * context
            .get_token_price("So11111111111111111111111111111111111111112")
            .unwrap_or(150.0);

    let prompt_template = format!(
        "Deposit available USDC into Jupiter lending for yield generation. \
         Wallet Context: {:.2} USDC available, {:.6} SOL (${:.2} value). \
         Total portfolio value: ${:.2}. \
         Strategy: Use maximum available USDC for optimal yield at current APY rates (5-12%). \
         If insufficient USDC, consider swapping SOL to USDC first.",
        usdc_balance,
        context.sol_balance_sol(),
        sol_balance_value,
        context.total_value_usd
    );

    Ok(reev_types::flow::DynamicStep::new(
        "enhanced_lend".to_string(),
        prompt_template,
        "Enhanced USDC lending using real wallet data".to_string(),
    )
    .with_tool(ToolName::JupiterLendEarnDeposit)
    .with_estimated_time(45)
    .with_recovery(reev_types::flow::RecoveryStrategy::Retry { attempts: 2 })
    .with_critical(true))
}

/// Extract percentage from prompt string
fn extract_percentage(prompt: &str) -> Option<String> {
    let re = Regex::new(r"(\d+\.?\d*)\s*%").ok()?;
    let caps = re.captures(prompt)?;
    Some(caps.get(1)?.as_str().to_string())
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
    .with_tool(ToolName::GetAccountBalance)
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
    .with_tool(ToolName::GetJupiterLendEarnPosition)
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
    .with_tool(ToolName::GetJupiterLendEarnPosition)
    .with_estimated_time(15)
    .with_critical(false)) // Not critical for flow success
}
