//! Dynamic Flow Types
//!
//! This module contains types for dynamic flow orchestration, including
//! wallet context, flow plans, and related structures.

use super::tools::ToolName;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Wallet context containing balance, prices, and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletContext {
    /// Wallet owner public key
    pub owner: String,
    /// SOL balance in lamports
    pub sol_balance: u64,
    /// Token balances by mint address
    pub token_balances: HashMap<String, TokenBalance>,
    /// Token prices by mint address (USD)
    pub token_prices: HashMap<String, f64>,
    /// Total portfolio value in USD
    pub total_value_usd: f64,
}

impl WalletContext {
    /// Create a new wallet context
    pub fn new(owner: String) -> Self {
        Self {
            owner,
            sol_balance: 0,
            token_balances: HashMap::new(),
            token_prices: HashMap::new(),
            total_value_usd: 0.0,
        }
    }

    /// Get SOL balance in SOL units
    pub fn sol_balance_sol(&self) -> f64 {
        self.sol_balance as f64 / 1_000_000_000.0
    }

    /// Get token balance in human-readable format
    pub fn get_token_balance(&self, mint: &str) -> Option<&TokenBalance> {
        self.token_balances.get(mint)
    }

    /// Get token price in USD
    pub fn get_token_price(&self, mint: &str) -> Option<f64> {
        self.token_prices.get(mint).copied()
    }

    /// Add token balance
    pub fn add_token_balance(&mut self, mint: String, balance: TokenBalance) {
        self.token_balances.insert(mint, balance);
    }

    /// Add token price
    pub fn add_token_price(&mut self, mint: String, price: f64) {
        self.token_prices.insert(mint, price);
    }

    /// Calculate total value from all assets
    pub fn calculate_total_value(&mut self) {
        let sol_price = self
            .get_token_price("So11111111111111111111111111111111111111112")
            .unwrap_or(150.0); // Default SOL price

        let sol_value = self.sol_balance_sol() * sol_price;

        let mut token_value = 0.0;
        for (mint, balance) in &self.token_balances {
            if let Some(price) = self.get_token_price(mint) {
                let decimals = balance.decimals.unwrap_or(0) as f64;
                let amount = balance.balance as f64 / 10_f64.powi(decimals as i32);
                token_value += amount * price;
            }
        }

        self.total_value_usd = sol_value + token_value;
    }
}

/// Dynamic flow step with atomic behavior support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicStep {
    /// Unique step identifier
    pub step_id: String,
    /// Whether step is critical (failure = flow failure)
    pub critical: bool,
    /// Step prompt template
    pub prompt_template: String,
    /// Step description for users
    pub description: String,
    /// Required tools for this step
    pub required_tools: Vec<ToolName>,
    /// Recovery strategy if step fails
    pub recovery_strategy: Option<RecoveryStrategy>,
    /// Estimated execution time in seconds
    pub estimated_time_seconds: u64,
}

impl DynamicStep {
    /// Create a new dynamic step
    pub fn new(step_id: String, prompt_template: String, description: String) -> Self {
        Self {
            step_id,
            critical: true, // Critical by default for atomic behavior
            prompt_template,
            description,
            required_tools: Vec::new(),
            recovery_strategy: None,
            estimated_time_seconds: 30,
        }
    }

    /// Set criticality and return self for chaining
    pub fn with_critical(mut self, critical: bool) -> Self {
        self.critical = critical;
        self
    }

    /// Add required tool and return self for chaining
    pub fn with_tool(mut self, tool: ToolName) -> Self {
        self.required_tools.push(tool);
        self
    }

    /// Set recovery strategy and return self for chaining
    pub fn with_recovery(mut self, strategy: RecoveryStrategy) -> Self {
        self.recovery_strategy = Some(strategy);
        self
    }

    /// Set estimated time and return self for chaining
    pub fn with_estimated_time(mut self, seconds: u64) -> Self {
        self.estimated_time_seconds = seconds;
        self
    }
}

/// Recovery strategy for failed steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryStrategy {
    /// Retry the step with specified attempts
    Retry { attempts: usize },
    /// Execute alternative flow
    AlternativeFlow { flow_id: String },
    /// Request user fulfillment
    UserFulfillment { questions: Vec<String> },
}

/// Dynamic flow plan containing steps and context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicFlowPlan {
    /// Unique flow identifier
    pub flow_id: String,
    /// User prompt that generated this flow
    pub user_prompt: String,
    /// Flow steps in execution order
    pub steps: Vec<DynamicStep>,
    /// Wallet context at time of flow creation
    pub context: WalletContext,
    /// Flow metadata
    pub metadata: FlowMetadata,
    /// Atomic mode for flow execution
    pub atomic_mode: AtomicMode,
}

impl DynamicFlowPlan {
    /// Create a new dynamic flow plan
    pub fn new(flow_id: String, user_prompt: String, context: WalletContext) -> Self {
        Self {
            flow_id,
            user_prompt,
            steps: Vec::new(),
            context,
            metadata: FlowMetadata::new(),
            atomic_mode: AtomicMode::Strict,
        }
    }

    /// Add step and return self for chaining
    pub fn with_step(mut self, step: DynamicStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Set atomic mode and return self for chaining
    pub fn with_atomic_mode(mut self, mode: AtomicMode) -> Self {
        self.atomic_mode = mode;
        self
    }

    /// Get critical steps
    pub fn critical_steps(&self) -> Vec<&DynamicStep> {
        self.steps.iter().filter(|step| step.critical).collect()
    }

    /// Get non-critical steps
    pub fn non_critical_steps(&self) -> Vec<&DynamicStep> {
        self.steps.iter().filter(|step| !step.critical).collect()
    }

    /// Get total estimated execution time
    pub fn estimated_time_seconds(&self) -> u64 {
        self.steps
            .iter()
            .map(|step| step.estimated_time_seconds)
            .sum()
    }
}

/// Flow metadata for tracking and optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowMetadata {
    /// Flow creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Flow category (swap, lend, etc.)
    pub category: String,
    /// Flow complexity score
    pub complexity_score: u8,
    /// Flow tags
    pub tags: Vec<String>,
    /// Flow version for template evolution
    pub version: String,
}

impl Default for FlowMetadata {
    fn default() -> Self {
        Self::new()
    }
}

impl FlowMetadata {
    /// Create new flow metadata
    pub fn new() -> Self {
        Self {
            created_at: chrono::Utc::now(),
            category: "general".to_string(),
            complexity_score: 1,
            tags: Vec::new(),
            version: "1.0".to_string(),
        }
    }

    /// Set category and return self for chaining
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = category.into();
        self
    }

    /// Add tag and return self for chaining
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Set complexity score and return self for chaining
    pub fn with_complexity(mut self, score: u8) -> Self {
        self.complexity_score = score;
        self
    }
}

/// Atomic execution mode for flow control
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AtomicMode {
    /// Strict - any critical failure fails the entire flow
    #[default]
    Strict,
    /// Lenient - mark failures but continue execution
    Lenient,
    /// Conditional - some steps marked as non-critical
    Conditional,
}

impl AtomicMode {
    /// Get string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            AtomicMode::Strict => "strict",
            AtomicMode::Lenient => "lenient",
            AtomicMode::Conditional => "conditional",
        }
    }
}

/// Flow execution result for tracking and evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowResult {
    /// Flow identifier
    pub flow_id: String,
    /// User prompt
    pub user_prompt: String,
    /// Overall success status
    pub success: bool,
    /// Step execution results
    pub step_results: Vec<StepResult>,
    /// Execution metrics
    pub metrics: FlowMetrics,
    /// Final context after execution
    pub final_context: Option<WalletContext>,
    /// Error message if flow failed
    pub error_message: Option<String>,
}

/// Individual step execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    /// Step identifier
    pub step_id: String,
    /// Whether step succeeded
    pub success: bool,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
    /// Tool calls made during step
    pub tool_calls: Vec<String>,
    /// Step output
    pub output: Option<String>,
    /// Error message if step failed
    pub error_message: Option<String>,
    /// Recovery attempts made
    pub recovery_attempts: usize,
}

/// Flow execution metrics for evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowMetrics {
    /// Total execution time in milliseconds
    pub total_duration_ms: u64,
    /// Number of successful steps
    pub successful_steps: usize,
    /// Number of failed steps
    pub failed_steps: usize,
    /// Number of critical failures
    pub critical_failures: usize,
    /// Number of non-critical failures
    pub non_critical_failures: usize,
    /// Total tool calls made
    pub total_tool_calls: usize,
    /// Context resolution time in milliseconds
    pub context_resolution_ms: u64,
    /// Prompt generation time in milliseconds
    pub prompt_generation_ms: u64,
    /// Cache hit rate (0.0 to 1.0)
    pub cache_hit_rate: f64,
}

impl Default for FlowMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl FlowMetrics {
    /// Create new flow metrics
    pub fn new() -> Self {
        Self {
            total_duration_ms: 0,
            successful_steps: 0,
            failed_steps: 0,
            critical_failures: 0,
            non_critical_failures: 0,
            total_tool_calls: 0,
            context_resolution_ms: 0,
            prompt_generation_ms: 0,
            cache_hit_rate: 0.0,
        }
    }

    /// Get success rate (0.0 to 1.0)
    pub fn success_rate(&self) -> f64 {
        let total_steps = self.successful_steps + self.failed_steps;
        if total_steps == 0 {
            0.0
        } else {
            self.successful_steps as f64 / total_steps as f64
        }
    }

    /// Check if flow completed successfully
    pub fn is_successful(&self) -> bool {
        self.critical_failures == 0 && self.successful_steps > 0
    }
}

/// Prompt context for agent enhancement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptContext {
    /// Wallet context
    pub wallet_context: WalletContext,
    /// Current flow state
    pub flow_state: Option<FlowState>,
    /// Previous step results
    pub previous_results: Vec<StepResult>,
    /// Context generation timestamp
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

/// Current flow execution state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowState {
    /// Current step index
    pub current_step_index: usize,
    /// Total number of steps
    pub total_steps: usize,
    /// Flow execution mode
    pub atomic_mode: AtomicMode,
    /// Whether previous step succeeded
    pub previous_step_success: bool,
}

/// Benchmark source enum for supporting both static and dynamic flows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BenchmarkSource {
    /// Static YML file
    StaticFile { path: String },
    /// Dynamic flow from natural language
    DynamicFlow { prompt: String, wallet: String },
    /// Hybrid mode with both sources
    Hybrid {
        path: Option<String>,
        prompt: Option<String>,
    },
}

impl BenchmarkSource {
    /// Get prompt if available
    pub fn get_prompt(&self) -> Option<&str> {
        match self {
            BenchmarkSource::DynamicFlow { prompt, .. } => Some(prompt),
            BenchmarkSource::Hybrid {
                prompt: Some(p), ..
            } => Some(p),
            _ => None,
        }
    }

    /// Get wallet if available
    pub fn get_wallet(&self) -> Option<&str> {
        match self {
            BenchmarkSource::DynamicFlow { wallet, .. } => Some(wallet),
            BenchmarkSource::Hybrid {
                prompt: Some(_), ..
            } => {
                // For hybrid mode, wallet should be provided separately
                None
            }
            _ => None,
        }
    }

    /// Check if this is a dynamic source
    pub fn is_dynamic(&self) -> bool {
        matches!(
            self,
            BenchmarkSource::DynamicFlow { .. } | BenchmarkSource::Hybrid { .. }
        )
    }
}

/// Re-export TokenBalance from benchmark module for convenience
pub use crate::benchmark::TokenBalance;
