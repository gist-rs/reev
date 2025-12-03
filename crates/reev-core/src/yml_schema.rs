//! YML Schema for Verifiable AI-Generated DeFi Flows
//!
//! This module defines the YML structures used for both runtime guardrails
//! and evaluation criteria in the verifiable AI-generated DeFi flows architecture.

use reev_types::benchmark::TokenBalance;
use reev_types::tools::ToolName;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
// use uuid::Uuid; // Not needed as we're using uuid directly in the calls

/// YML Flow structure representing a complete DeFi operation plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YmlFlow {
    /// Unique flow identifier (UUID v7 for time-sortable IDs)
    pub flow_id: String,
    /// Original user prompt (may contain typos, unclear language)
    pub user_prompt: String,
    /// LLM-refined prompt (clear, unambiguous language)
    pub refined_prompt: String,
    /// Subject wallet information
    pub subject_wallet_info: YmlWalletInfo,
    /// Flow steps in execution order
    pub steps: Vec<YmlStep>,
    /// Ground truth for validation and guardrails
    pub ground_truth: Option<YmlGroundTruth>,
    /// Flow metadata
    pub metadata: FlowMetadata,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl YmlFlow {
    /// Create a new YML flow
    pub fn new(flow_id: String, user_prompt: String, subject_wallet_info: YmlWalletInfo) -> Self {
        Self {
            flow_id,
            user_prompt: user_prompt.clone(),
            refined_prompt: user_prompt.clone(), // Default to original prompt
            subject_wallet_info,
            steps: Vec::new(),
            ground_truth: None,
            metadata: FlowMetadata::new(),
            created_at: chrono::Utc::now(),
        }
    }

    /// Add step and return self for chaining
    pub fn with_step(mut self, step: YmlStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Add multiple steps and return self for chaining
    pub fn with_steps(mut self, steps: Vec<YmlStep>) -> Self {
        self.steps = steps;
        self
    }

    /// Set ground truth and return self for chaining
    pub fn with_ground_truth(mut self, ground_truth: YmlGroundTruth) -> Self {
        self.ground_truth = Some(ground_truth);
        self
    }

    /// Set metadata and return self for chaining
    pub fn with_metadata(mut self, metadata: FlowMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Set refined prompt and return self for chaining
    pub fn with_refined_prompt(mut self, refined_prompt: String) -> Self {
        self.refined_prompt = refined_prompt;
        self
    }

    /// Validate the flow structure
    pub fn validate(&self) -> Result<(), String> {
        if self.steps.is_empty() {
            return Err("Flow must have at least one step".to_string());
        }

        for (i, step) in self.steps.iter().enumerate() {
            if step.step_id.is_empty() {
                return Err(format!("Step {i} has empty step_id"));
            }
            if step.prompt.is_empty() {
                return Err(format!("Step {i} has empty prompt"));
            }
        }

        Ok(())
    }
}

/// Wallet information for the subject of the flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YmlWalletInfo {
    /// Wallet public key
    pub pubkey: String,
    /// SOL balance in lamports
    pub lamports: u64,
    /// Token balances
    pub tokens: Vec<TokenBalance>,
    /// Total portfolio value in USD
    pub total_value_usd: Option<f64>,
}

impl YmlWalletInfo {
    /// Create a new wallet info
    pub fn new(pubkey: String, lamports: u64) -> Self {
        Self {
            pubkey,
            lamports,
            tokens: Vec::new(),
            total_value_usd: None,
        }
    }

    /// Add token and return self for chaining
    pub fn with_token(mut self, token: TokenBalance) -> Self {
        self.tokens.push(token);
        self
    }

    /// Set total value and return self for chaining
    pub fn with_total_value(mut self, value: f64) -> Self {
        self.total_value_usd = Some(value);
        self
    }

    /// Get SOL balance in SOL units
    pub fn sol_balance_sol(&self) -> f64 {
        self.lamports as f64 / 1_000_000_000.0
    }
}

/// Individual flow step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YmlStep {
    /// Unique step identifier
    pub step_id: String,
    /// Original user prompt (may contain typos, unclear language)
    pub prompt: String,
    /// LLM-refined prompt (clear, unambiguous language)
    pub refined_prompt: String,
    /// Additional context for this step
    pub context: String,
    /// Structured prompt data from the prompt processor
    pub structured_prompt: Option<crate::prompt_processor::StructuredRefinedPrompt>,
    /// Expected tool calls for this step
    pub expected_tool_calls: Option<Vec<YmlToolCall>>,
    /// Expected tools list (hints for rig agent)
    pub expected_tools: Option<Vec<ToolName>>,
    /// Whether this step is critical (failure = flow failure)
    pub critical: Option<bool>,
    /// Estimated execution time in seconds
    pub estimated_time_seconds: Option<u64>,
}

impl YmlStep {
    /// Create a new step
    pub fn new(step_id: String, prompt: String, context: String) -> Self {
        Self {
            step_id,
            prompt: prompt.clone(),
            refined_prompt: prompt.clone(), // Default to original prompt
            context,
            structured_prompt: None,
            expected_tool_calls: None,
            expected_tools: None,
            critical: Some(true), // Critical by default
            estimated_time_seconds: None,
        }
    }

    /// Add tool call and return self for chaining
    pub fn with_tool_call(mut self, tool_call: YmlToolCall) -> Self {
        self.expected_tool_calls
            .get_or_insert_with(Vec::new)
            .push(tool_call);
        self
    }

    /// Set criticality and return self for chaining
    pub fn with_critical(mut self, critical: bool) -> Self {
        self.critical = Some(critical);
        self
    }

    /// Set estimated time and return self for chaining
    pub fn with_estimated_time(mut self, seconds: u64) -> Self {
        self.estimated_time_seconds = Some(seconds);
        self
    }

    /// Set refined prompt and return self for chaining
    pub fn with_refined_prompt(mut self, refined_prompt: String) -> Self {
        self.refined_prompt = refined_prompt;
        self
    }

    /// Set structured prompt data and return self for chaining
    pub fn with_structured_prompt(
        mut self,
        structured_prompt: crate::prompt_processor::StructuredRefinedPrompt,
    ) -> Self {
        self.structured_prompt = Some(structured_prompt);
        self
    }

    /// Set expected tools and return self for chaining
    pub fn with_expected_tools(mut self, tools: Vec<ToolName>) -> Self {
        self.expected_tools = Some(tools);
        self
    }
}

/// Expected tool call within a step
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct YmlToolCall {
    /// Name of the tool to call
    pub tool_name: ToolName,
    /// Whether this tool call is critical
    pub critical: bool,
    /// Expected parameters (simplified validation)
    pub expected_parameters: Option<HashMap<String, serde_json::Value>>,
}

/// Expected tool list for rig agent guidance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YmlExpectedTool {
    /// Name of the expected tool
    pub name: String,
    /// How likely this tool is to be needed (0.0-1.0)
    pub likelihood: f32,
    /// Brief description of why this tool is expected
    pub reason: Option<String>,
}

impl YmlExpectedTool {
    /// Create a new expected tool
    pub fn new(name: String, likelihood: f32) -> Self {
        Self {
            name,
            likelihood,
            reason: None,
        }
    }

    /// Set reason and return self for chaining
    pub fn with_reason(mut self, reason: String) -> Self {
        self.reason = Some(reason);
        self
    }
}

impl YmlToolCall {
    /// Create a new tool call
    pub fn new(tool_name: ToolName, critical: bool) -> Self {
        Self {
            tool_name,
            critical,
            expected_parameters: None,
        }
    }

    /// Add expected parameter and return self for chaining
    pub fn with_parameter(mut self, key: String, value: serde_json::Value) -> Self {
        self.expected_parameters
            .get_or_insert_with(HashMap::new)
            .insert(key, value);
        self
    }

    /// Add expected parameter from string and return self for chaining
    pub fn with_parameter_str(mut self, key: String, value: String) -> Self {
        self.expected_parameters
            .get_or_insert_with(HashMap::new)
            .insert(key, serde_json::Value::String(value));
        self
    }

    /// Set critical and return self for chaining
    pub fn with_critical(mut self, critical: bool) -> Self {
        self.critical = critical;
        self
    }
}

/// Ground truth for validation and guardrails
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YmlGroundTruth {
    /// Minimum score required for benchmark success
    pub min_score: Option<f64>,
    /// Final state assertions
    pub final_state_assertions: Vec<YmlAssertion>,
    /// Expected tool calls (redundant with step expected_tool_calls for convenience)
    pub expected_tool_calls: Option<Vec<YmlToolCall>>,
    /// Success criteria for benchmark evaluation
    pub success_criteria: Option<Vec<YmlSuccessCriterion>>,
    /// Expected data structure for validation
    pub expected_data_structure: Option<Vec<YmlDataStructure>>,
    /// Expected flow complexity metrics
    pub expected_flow_complexity: Option<Vec<YmlFlowComplexity>>,
    /// Expected OpenTelemetry tracking
    pub expected_otel_tracking: Option<Vec<YmlOtelTracking>>,
    /// Expected recovery behavior
    pub recovery_expectations: Option<Vec<YmlRecoveryExpectation>>,
    /// Error tolerance for slippage and rate issues (default 1%)
    pub error_tolerance: Option<f64>,
}

impl Default for YmlGroundTruth {
    fn default() -> Self {
        Self::new()
    }
}

impl YmlGroundTruth {
    /// Create a new ground truth
    pub fn new() -> Self {
        Self {
            min_score: Some(0.7), // Default minimum score
            final_state_assertions: Vec::new(),
            expected_tool_calls: None,
            success_criteria: None,
            expected_data_structure: None,
            expected_flow_complexity: None,
            expected_otel_tracking: None,
            recovery_expectations: None,
            error_tolerance: Some(0.01), // 1% default tolerance
        }
    }

    /// Set minimum score and return self for chaining
    pub fn with_min_score(mut self, score: f64) -> Self {
        self.min_score = Some(score);
        self
    }

    /// Add assertion and return self for chaining
    pub fn with_assertion(mut self, assertion: YmlAssertion) -> Self {
        self.final_state_assertions.push(assertion);
        self
    }

    /// Add tool call and return self for chaining
    pub fn with_tool_call(mut self, tool_call: YmlToolCall) -> Self {
        self.expected_tool_calls
            .get_or_insert_with(Vec::new)
            .push(tool_call);
        self
    }

    /// Add success criterion and return self for chaining
    pub fn with_success_criterion(mut self, criterion: YmlSuccessCriterion) -> Self {
        self.success_criteria
            .get_or_insert_with(Vec::new)
            .push(criterion);
        self
    }

    /// Add data structure expectation and return self for chaining
    pub fn with_data_structure(mut self, structure: YmlDataStructure) -> Self {
        self.expected_data_structure
            .get_or_insert_with(Vec::new)
            .push(structure);
        self
    }

    /// Add flow complexity expectation and return self for chaining
    pub fn with_flow_complexity(mut self, complexity: YmlFlowComplexity) -> Self {
        self.expected_flow_complexity
            .get_or_insert_with(Vec::new)
            .push(complexity);
        self
    }

    /// Add otel tracking expectation and return self for chaining
    pub fn with_otel_tracking(mut self, tracking: YmlOtelTracking) -> Self {
        self.expected_otel_tracking
            .get_or_insert_with(Vec::new)
            .push(tracking);
        self
    }

    /// Add recovery expectation and return self for chaining
    pub fn with_recovery_expectation(mut self, recovery: YmlRecoveryExpectation) -> Self {
        self.recovery_expectations
            .get_or_insert_with(Vec::new)
            .push(recovery);
        self
    }

    /// Set error tolerance and return self for chaining
    pub fn with_error_tolerance(mut self, tolerance: f64) -> Self {
        self.error_tolerance = Some(tolerance);
        self
    }

    /// Merge this ground truth with another, combining assertions and tool calls
    pub fn merge(mut self, other: YmlGroundTruth) -> Self {
        // Merge final state assertions
        self.final_state_assertions
            .extend(other.final_state_assertions);

        // Merge expected tool calls
        let merged_tool_calls = match (self.expected_tool_calls, other.expected_tool_calls) {
            (Some(mut self_calls), Some(other_calls)) => {
                self_calls.extend(other_calls);
                Some(self_calls)
            }
            (Some(self_calls), None) => Some(self_calls),
            (None, Some(other_calls)) => Some(other_calls),
            (None, None) => None,
        };
        self.expected_tool_calls = merged_tool_calls;

        // Use the smallest error tolerance (more strict)
        self.error_tolerance = match (self.error_tolerance, other.error_tolerance) {
            (Some(self_tolerance), Some(other_tolerance)) => {
                Some(self_tolerance.min(other_tolerance))
            }
            (Some(self_tolerance), None) => Some(self_tolerance),
            (None, Some(other_tolerance)) => Some(other_tolerance),
            (None, None) => Some(0.01), // Default tolerance
        };

        self
    }
}

/// Assertion for validating final state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YmlAssertion {
    /// Type of assertion
    pub assertion_type: String,
    /// Public key for account-specific assertions
    pub pubkey: Option<String>,
    /// Expected change value
    pub expected_change_gte: Option<f64>,
    /// Expected change value (upper bound)
    pub expected_change_lte: Option<f64>,
    /// Custom assertion parameters
    pub parameters: Option<HashMap<String, serde_json::Value>>,
}

impl YmlAssertion {
    /// Create a new assertion
    pub fn new(assertion_type: String) -> Self {
        Self {
            assertion_type,
            pubkey: None,
            expected_change_gte: None,
            expected_change_lte: None,
            parameters: None,
        }
    }

    /// Set pubkey and return self for chaining
    pub fn with_pubkey(mut self, pubkey: String) -> Self {
        self.pubkey = Some(pubkey);
        self
    }

    /// Set expected change greater than or equal and return self for chaining
    pub fn with_expected_change_gte(mut self, value: f64) -> Self {
        self.expected_change_gte = Some(value);
        self
    }

    /// Set expected change less than or equal and return self for chaining
    pub fn with_expected_change_lte(mut self, value: f64) -> Self {
        self.expected_change_lte = Some(value);
        self
    }

    /// Add parameter and return self for chaining
    pub fn with_parameter(mut self, key: String, value: serde_json::Value) -> Self {
        self.parameters
            .get_or_insert_with(HashMap::new)
            .insert(key, value);
        self
    }
}

/// Success criterion for benchmark evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YmlSuccessCriterion {
    /// Type of success criterion
    pub criterion_type: String,
    /// Description of the criterion
    pub description: Option<String>,
    /// Whether this criterion is required
    pub required: Option<bool>,
    /// Weight of this criterion in overall score (0.0-1.0)
    pub weight: Option<f64>,
    /// Custom criterion parameters
    pub parameters: Option<HashMap<String, serde_json::Value>>,
}

impl YmlSuccessCriterion {
    /// Create a new success criterion
    pub fn new(criterion_type: String) -> Self {
        Self {
            criterion_type,
            description: None,
            required: Some(true),
            weight: Some(1.0),
            parameters: None,
        }
    }

    /// Set description and return self for chaining
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Set required and return self for chaining
    pub fn with_required(mut self, required: bool) -> Self {
        self.required = Some(required);
        self
    }

    /// Set weight and return self for chaining
    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = Some(weight);
        self
    }

    /// Add parameter and return self for chaining
    pub fn with_parameter(mut self, key: String, value: serde_json::Value) -> Self {
        self.parameters
            .get_or_insert_with(HashMap::new)
            .insert(key, value);
        self
    }
}

/// Expected data structure for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YmlDataStructure {
    /// JSON path to the data structure
    pub path: String,
    /// Expected data type
    pub type_: String,
    /// Required fields in the data structure
    pub required_fields: Option<Vec<String>>,
    /// Weight of this structure in overall score (0.0-1.0)
    pub weight: Option<f64>,
    /// Custom validation parameters
    pub parameters: Option<HashMap<String, serde_json::Value>>,
}

impl YmlDataStructure {
    /// Create a new data structure expectation
    pub fn new(path: String, type_: String) -> Self {
        Self {
            path,
            type_,
            required_fields: None,
            weight: Some(1.0),
            parameters: None,
        }
    }

    /// Set required fields and return self for chaining
    pub fn with_required_fields(mut self, fields: Vec<String>) -> Self {
        self.required_fields = Some(fields);
        self
    }

    /// Set weight and return self for chaining
    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = Some(weight);
        self
    }

    /// Add parameter and return self for chaining
    pub fn with_parameter(mut self, key: String, value: serde_json::Value) -> Self {
        self.parameters
            .get_or_insert_with(HashMap::new)
            .insert(key, value);
        self
    }
}

/// Expected flow complexity metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YmlFlowComplexity {
    /// Type of complexity metric
    pub complexity_type: String,
    /// Description of the complexity metric
    pub description: Option<String>,
    /// Whether this metric is required
    pub required: Option<bool>,
    /// Minimum number of steps/items
    pub min_steps: Option<u32>,
    /// Weight of this metric in overall score (0.0-1.0)
    pub weight: Option<f64>,
    /// Custom complexity parameters
    pub parameters: Option<HashMap<String, serde_json::Value>>,
}

impl YmlFlowComplexity {
    /// Create a new flow complexity expectation
    pub fn new(complexity_type: String) -> Self {
        Self {
            complexity_type,
            description: None,
            required: Some(true),
            min_steps: None,
            weight: Some(1.0),
            parameters: None,
        }
    }

    /// Set description and return self for chaining
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Set required and return self for chaining
    pub fn with_required(mut self, required: bool) -> Self {
        self.required = Some(required);
        self
    }

    /// Set minimum steps and return self for chaining
    pub fn with_min_steps(mut self, steps: u32) -> Self {
        self.min_steps = Some(steps);
        self
    }

    /// Set weight and return self for chaining
    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = Some(weight);
        self
    }

    /// Add parameter and return self for chaining
    pub fn with_parameter(mut self, key: String, value: serde_json::Value) -> Self {
        self.parameters
            .get_or_insert_with(HashMap::new)
            .insert(key, value);
        self
    }
}

/// Expected OpenTelemetry tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YmlOtelTracking {
    /// Type of tracking
    pub tracking_type: String,
    /// Description of what should be tracked
    pub description: Option<String>,
    /// Whether this tracking is required
    pub required: Option<bool>,
    /// Required tools to be tracked
    pub required_tools: Option<Vec<String>>,
    /// Required spans to be tracked
    pub required_spans: Option<Vec<String>>,
    /// Required metrics to be tracked
    pub required_metrics: Option<Vec<String>>,
    /// Weight of this tracking in overall score (0.0-1.0)
    pub weight: Option<f64>,
}

impl YmlOtelTracking {
    /// Create a new otel tracking expectation
    pub fn new(tracking_type: String) -> Self {
        Self {
            tracking_type,
            description: None,
            required: Some(true),
            required_tools: None,
            required_spans: None,
            required_metrics: None,
            weight: Some(1.0),
        }
    }

    /// Set description and return self for chaining
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Set required and return self for chaining
    pub fn with_required(mut self, required: bool) -> Self {
        self.required = Some(required);
        self
    }

    /// Set required tools and return self for chaining
    pub fn with_required_tools(mut self, tools: Vec<String>) -> Self {
        self.required_tools = Some(tools);
        self
    }

    /// Set required spans and return self for chaining
    pub fn with_required_spans(mut self, spans: Vec<String>) -> Self {
        self.required_spans = Some(spans);
        self
    }

    /// Set required metrics and return self for chaining
    pub fn with_required_metrics(mut self, metrics: Vec<String>) -> Self {
        self.required_metrics = Some(metrics);
        self
    }

    /// Set weight and return self for chaining
    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = Some(weight);
        self
    }
}

/// Expected recovery behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YmlRecoveryExpectation {
    /// Type of recovery scenario
    pub recovery_type: String,
    /// Description of the recovery scenario
    pub description: Option<String>,
    /// Whether this recovery behavior is required
    pub required: Option<bool>,
    /// Weight of this recovery expectation in overall score (0.0-1.0)
    pub weight: Option<f64>,
    /// Custom recovery parameters
    pub parameters: Option<HashMap<String, serde_json::Value>>,
}

impl YmlRecoveryExpectation {
    /// Create a new recovery expectation
    pub fn new(recovery_type: String) -> Self {
        Self {
            recovery_type,
            description: None,
            required: Some(true),
            weight: Some(1.0),
            parameters: None,
        }
    }

    /// Set description and return self for chaining
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Set required and return self for chaining
    pub fn with_required(mut self, required: bool) -> Self {
        self.required = Some(required);
        self
    }

    /// Set weight and return self for chaining
    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = Some(weight);
        self
    }

    /// Add parameter and return self for chaining
    pub fn with_parameter(mut self, key: String, value: serde_json::Value) -> Self {
        self.parameters
            .get_or_insert_with(HashMap::new)
            .insert(key, value);
        self
    }
}

/// Flow metadata for tracking and optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowMetadata {
    /// Flow category (swap, lend, etc.)
    pub category: String,
    /// Flow complexity score (1-5)
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

    /// Set version and return self for chaining
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }
}

/// YML context for dynamic content injection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YmlContext {
    /// Dynamic variables that can be referenced in prompts
    pub variables: HashMap<String, serde_json::Value>,
    /// Context generation timestamp
    pub generated_at: chrono::DateTime<chrono::Utc>,
    /// Wallet context snapshot
    pub wallet_context: Option<reev_types::flow::WalletContext>,
}

impl YmlContext {
    /// Create a new context
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            generated_at: chrono::Utc::now(),
            wallet_context: None,
        }
    }

    /// Add variable and return self for chaining
    pub fn with_variable(mut self, key: String, value: serde_json::Value) -> Self {
        self.variables.insert(key, value);
        self
    }

    /// Set wallet context and return self for chaining
    pub fn with_wallet_context(mut self, context: reev_types::flow::WalletContext) -> Self {
        self.wallet_context = Some(context);
        self
    }

    /// Get variable value
    pub fn get_variable(&self, key: &str) -> Option<&serde_json::Value> {
        self.variables.get(key)
    }
}

impl Default for YmlContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for creating common YML structures
pub mod builders {
    use super::*;
    use reev_types::benchmark::TokenBalance;
    use uuid::Uuid;

    /// Create a simple swap flow
    pub fn create_swap_flow(
        pubkey: String,
        lamports: u64,
        from_mint: String,
        to_mint: String,
        amount: f64,
    ) -> YmlFlow {
        let flow_id = Uuid::now_v7().to_string();

        let wallet_info = YmlWalletInfo::new(pubkey, lamports).with_token(TokenBalance::new(
            from_mint.clone(),
            (amount * 1_000_000_000.0) as u64,
        ));

        let step1 = YmlStep::new(
            "swap".to_string(),
            format!("swap {amount} {from_mint} to {to_mint}"),
            format!("Exchange {amount} {from_mint} for {to_mint}"),
        )
        .with_tool_call(YmlToolCall::new(ToolName::JupiterSwap, true))
        .with_expected_tools(vec![ToolName::JupiterSwap]);

        YmlFlow::new(
            flow_id,
            format!("swap {amount} {from_mint} to {to_mint}"),
            wallet_info,
        )
        .with_step(step1)
    }

    /// Create a simple lend flow
    pub fn create_lend_flow(pubkey: String, lamports: u64, mint: String, amount: f64) -> YmlFlow {
        let flow_id = Uuid::now_v7().to_string();

        let wallet_info = YmlWalletInfo::new(pubkey, lamports).with_token(TokenBalance::new(
            mint.clone(),
            (amount * 1_000_000_000.0) as u64,
        ));

        let step1 = YmlStep::new(
            "lend".to_string(),
            format!("lend {amount} {mint} to jupiter"),
            format!("Deposit {amount} {mint} in Jupiter earn for yield"),
        )
        .with_tool_call(YmlToolCall::new(ToolName::JupiterLendEarnDeposit, true))
        .with_expected_tools(vec![ToolName::JupiterLendEarnDeposit]);

        YmlFlow::new(
            flow_id,
            format!("lend {amount} {mint} to jupiter"),
            wallet_info,
        )
        .with_step(step1)
    }

    /// Create a swap then lend flow (common DeFi pattern)
    pub fn create_swap_then_lend_flow(
        pubkey: String,
        lamports: u64,
        from_mint: String,
        to_mint: String,
        amount: f64,
    ) -> YmlFlow {
        let flow_id = Uuid::now_v7().to_string();
        let swapped_amount_var = "SWAPPED_AMOUNT".to_string();

        // Clone pubkey to avoid moving it
        let pubkey_clone = pubkey.clone();

        let wallet_info = YmlWalletInfo::new(pubkey, lamports).with_token(TokenBalance::new(
            from_mint.clone(),
            (amount * 1_000_000_000.0) as u64,
        ));

        let step1 = YmlStep::new(
            "swap".to_string(),
            format!("swap {amount} {from_mint} to {to_mint}"),
            format!("Exchange {amount} {from_mint} for {to_mint}"),
        )
        .with_tool_call(YmlToolCall::new(ToolName::JupiterSwap, true))
        .with_expected_tools(vec![ToolName::JupiterSwap]);

        let step2 = YmlStep::new(
            "lend".to_string(),
            format!("lend {{{swapped_amount_var}}} {to_mint} to jupiter"),
            format!("Lend swapped {to_mint} for yield"),
        )
        .with_tool_call(YmlToolCall::new(ToolName::JupiterLendEarnDeposit, true))
        .with_expected_tools(vec![ToolName::JupiterLendEarnDeposit]);

        let ground_truth = YmlGroundTruth::new()
            .with_assertion(
                YmlAssertion::new("SolBalanceChange".to_string())
                    .with_pubkey(pubkey_clone)
                    .with_expected_change_gte(-(amount + 0.1) * 1_000_000_000.0),
            ) // Account for fees
            .with_tool_call(YmlToolCall::new(ToolName::JupiterSwap, true))
            .with_tool_call(YmlToolCall::new(ToolName::JupiterLendEarnDeposit, true));

        YmlFlow::new(
            flow_id,
            format!("swap {amount} {from_mint} to {to_mint} then lend"),
            wallet_info,
        )
        .with_step(step1)
        .with_step(step2)
        .with_ground_truth(ground_truth)
    }
}
