//! # Flow Benchmark Module
//!
//! This module defines the structure for multi-step flow benchmarks.
//! It extends the single-step benchmark format to support complex
//! workflows that require multiple tool invocations in sequence.
//!
//! ## Flow Benchmark Format
//!
//! ```yaml
//! id: 200-jup-swap-then-lend-deposit
//! description: Multi-step flow: User swaps SOL to USDC then deposits USDC into Jupiter lending
//! tags: ["jupiter", "swap", "lend", "multi-step", "flow"]
//!
//! initial_state:
//!   - pubkey: "USER_WALLET_PUBKEY"
//!     owner: "11111111111111111111111111111111"
//!     lamports: 2000000000
//!
//! flow:
//!   - step: 1
//!     description: "Swap 0.5 SOL to USDC using Jupiter"
//!     prompt: "Swap 0.5 SOL from my wallet (USER_WALLET_PUBKEY) to USDC using Jupiter"
//!
//!   - step: 2
//!     description: "Deposit received USDC into Jupiter lending"
//!     prompt: "Deposit all the USDC I just received into Jupiter lending to earn yield"
//!     depends_on: ["step_1_result"]
//!
//! ground_truth:
//!   final_state_assertions:
//!     - type: SolBalance
//!       pubkey: "USER_WALLET_PUBKEY"
//!       expected_approx: 1500000000
//!       weight: 0.3
//!   expected_instructions:
//!     - step: 1
//!       program_id: "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4"
//!       instruction_count_range: [4, 8]
//!       weight: 0.5
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a single step in a multi-step flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowStep {
    /// Step number (1-based)
    pub step: usize,
    /// Description of what this step accomplishes
    pub description: String,
    /// The prompt to send to the LLM for this step
    pub prompt: String,
    /// List of step IDs this step depends on
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub depends_on: Vec<String>,
    /// Optional timeout for this step (in seconds)
    #[serde(default = "default_step_timeout")]
    pub timeout: u64,
    /// Whether this step is critical for flow success
    #[serde(default = "default_critical")]
    pub critical: bool,
    /// Optional retry configuration
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retry: Option<RetryConfig>,
}

/// Retry configuration for flow steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    #[serde(default = "default_max_retries")]
    pub max_retries: usize,
    /// Delay between retries (in seconds)
    #[serde(default = "default_retry_delay")]
    pub retry_delay: u64,
    /// Whether to retry on specific error types
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub retry_on_errors: Vec<String>,
}

/// Represents expected instructions for multi-step flows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedInstruction {
    /// Step number this instruction belongs to
    #[serde(default = "default_step")]
    pub step: usize,
    /// Program ID that should execute the instruction
    pub program_id: String,
    /// Expected number of instructions for this step
    #[serde(default = "default_instruction_count_opt")]
    pub instruction_count: Option<usize>,
    /// Range of acceptable instruction counts
    #[serde(default = "default_instruction_count_range_opt")]
    pub instruction_count_range: Option<(usize, usize)>,
    /// Weight of this instruction in scoring
    #[serde(default = "default_weight")]
    pub weight: f64,
    /// Whether this instruction is critical for success
    #[serde(default)]
    pub critical: bool,
}

/// Represents a multi-step flow benchmark
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowBenchmark {
    /// Unique identifier for the flow (prefixed with 200-)
    pub id: String,
    /// Human-readable description of the flow
    pub description: String,
    /// Tags for categorizing and filtering flows
    pub tags: Vec<String>,
    /// Initial on-chain state before flow execution
    pub initial_state: Vec<AccountState>,
    /// The sequence of steps to execute
    pub flow: Vec<FlowStep>,
    /// Ground truth for evaluation
    pub ground_truth: FlowGroundTruth,
    /// Optional metadata
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Ground truth for evaluating multi-step flows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowGroundTruth {
    /// Final state assertions after flow completion
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub final_state_assertions: Vec<StateAssertion>,
    /// Expected instructions for each step
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub expected_instructions: Vec<ExpectedInstruction>,
    /// Minimum acceptable score for the entire flow
    #[serde(default = "default_min_score")]
    pub min_score: f64,
    /// Success criteria for the flow
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub success_criteria: Vec<SuccessCriteria>,
}

/// Account state for initial setup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountState {
    /// Public key of the account
    pub pubkey: String,
    /// Account owner (program ID)
    pub owner: String,
    /// Account balance in lamports
    pub lamports: u64,
    /// Optional account data
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<AccountData>,
}

/// Account data for token accounts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountData {
    /// Token mint
    pub mint: String,
    /// Account owner
    pub owner: String,
    /// Token amount
    pub amount: String,
}

/// State assertion for final validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateAssertion {
    /// Type of assertion (SolBalance, TokenBalance, etc.)
    #[serde(rename = "type")]
    pub assertion_type: String,
    /// Account public key
    pub pubkey: String,
    /// Expected exact value
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected: Option<u64>,
    /// Expected approximate value (for ranges)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_approx: Option<u64>,
    /// Token mint for token balance assertions
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mint: Option<String>,
    /// Weight of this assertion in scoring
    #[serde(default = "default_weight")]
    pub weight: f64,
}

/// Success criteria for flow evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessCriteria {
    /// Type of success criteria
    #[serde(rename = "type")]
    pub criteria_type: String,
    /// Description of the criteria
    pub description: String,
    /// Required value or condition
    pub required: serde_json::Value,
    /// Weight of this criteria in success evaluation
    #[serde(default = "default_weight")]
    pub weight: f64,
}

impl FlowBenchmark {
    /// Load a flow benchmark from a YAML file
    pub fn from_file(file_path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read file: {file_path}"))?;

        let benchmark: FlowBenchmark = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse YAML from: {file_path}"))?;

        // Validate the benchmark
        benchmark
            .validate()
            .with_context(|| format!("Invalid benchmark format in: {file_path}"))?;

        Ok(benchmark)
    }

    /// Validate the flow benchmark structure
    pub fn validate(&self) -> Result<()> {
        // Validate ID prefix
        if !self.id.starts_with("200-") {
            return Err(anyhow::anyhow!(
                "Flow benchmark ID must start with '200-' prefix. Got: {}",
                self.id
            ));
        }

        // Validate steps
        if self.flow.is_empty() {
            return Err(anyhow::anyhow!("Flow must have at least one step"));
        }

        // Check for sequential step numbers
        let mut expected_step = 1;
        for step in &self.flow {
            if step.step != expected_step {
                return Err(anyhow::anyhow!(
                    "Step numbers must be sequential starting from 1. Expected step {}, got {}",
                    expected_step,
                    step.step
                ));
            }
            expected_step += 1;
        }

        // Validate dependencies
        for step in &self.flow {
            for dep in &step.depends_on {
                if !dep.starts_with("step_") {
                    return Err(anyhow::anyhow!(
                        "Dependencies must reference steps with 'step_N' format. Got: {dep}"
                    ));
                }

                let dep_step: usize = dep[5..]
                    .parse()
                    .map_err(|_| anyhow::anyhow!("Invalid step number in dependency: {dep}"))?;

                if dep_step >= step.step {
                    return Err(anyhow::anyhow!(
                        "Dependencies cannot reference current or future steps. Step {} depends on {}",
                        step.step, dep_step
                    ));
                }
            }
        }

        // Validate expected instructions
        for instruction in &self.ground_truth.expected_instructions {
            if instruction.step > self.flow.len() {
                return Err(anyhow::anyhow!(
                    "Expected instruction references step {} but flow only has {} steps",
                    instruction.step,
                    self.flow.len()
                ));
            }
        }

        Ok(())
    }

    /// Get the total number of steps in the flow
    pub fn total_steps(&self) -> usize {
        self.flow.len()
    }

    /// Get a specific step by number
    pub fn get_step(&self, step_num: usize) -> Option<&FlowStep> {
        self.flow.iter().find(|step| step.step == step_num)
    }

    /// Get all critical steps
    pub fn get_critical_steps(&self) -> Vec<&FlowStep> {
        self.flow.iter().filter(|step| step.critical).collect()
    }

    /// Get the maximum timeout across all steps
    pub fn max_timeout(&self) -> u64 {
        self.flow
            .iter()
            .map(|step| step.timeout)
            .max()
            .unwrap_or(60) // Default 60 seconds
    }

    /// Check if this flow has any retry configurations
    pub fn has_retries(&self) -> bool {
        self.flow.iter().any(|step| step.retry.is_some())
    }

    /// Get step dependencies as a directed graph
    pub fn get_dependency_graph(&self) -> HashMap<usize, Vec<usize>> {
        let mut graph = HashMap::new();

        for step in &self.flow {
            let deps: Vec<usize> = step
                .depends_on
                .iter()
                .filter_map(|dep| dep.strip_prefix("step_").and_then(|s| s.parse().ok()))
                .collect();

            graph.insert(step.step, deps);
        }

        graph
    }

    /// Check if the flow has any circular dependencies
    pub fn has_circular_dependencies(&self) -> bool {
        let graph = self.get_dependency_graph();
        let mut visited = HashMap::new();
        let mut rec_stack = Vec::new();

        for step in 1..=self.flow.len() {
            if !visited.contains_key(&step)
                && has_cycle_util(step, &graph, &mut visited, &mut rec_stack)
            {
                return true;
            }
        }

        false
    }

    /// Get a summary of the flow
    pub fn get_summary(&self) -> String {
        format!(
            "Flow: {}\n\
            Description: {}\n\
            Steps: {}\n\
            Critical Steps: {}\n\
            Tags: {}\n\
            Has Retries: {}",
            self.id,
            self.description,
            self.total_steps(),
            self.get_critical_steps().len(),
            self.tags.join(", "),
            self.has_retries()
        )
    }
}

// Helper function for cycle detection
fn has_cycle_util(
    node: usize,
    graph: &HashMap<usize, Vec<usize>>,
    visited: &mut HashMap<usize, bool>,
    rec_stack: &mut Vec<usize>,
) -> bool {
    if rec_stack.contains(&node) {
        return true;
    }

    if visited.contains_key(&node) {
        return false;
    }

    visited.insert(node, true);
    rec_stack.push(node);

    if let Some(neighbors) = graph.get(&node) {
        for neighbor in neighbors {
            if has_cycle_util(*neighbor, graph, visited, rec_stack) {
                return true;
            }
        }
    }

    rec_stack.pop();
    false
}

// Default value functions
fn default_step_timeout() -> u64 {
    60
}

fn default_critical() -> bool {
    false
}

fn default_max_retries() -> usize {
    3
}

fn default_retry_delay() -> u64 {
    5
}

#[allow(dead_code)]
fn default_instruction_count() -> usize {
    1
}

fn default_instruction_count_opt() -> Option<usize> {
    Some(1)
}

#[allow(dead_code)]
fn default_instruction_count_range() -> (usize, usize) {
    (1, 1)
}

fn default_instruction_count_range_opt() -> Option<(usize, usize)> {
    Some((1, 1))
}

fn default_weight() -> f64 {
    1.0
}

fn default_min_score() -> f64 {
    0.7
}

fn default_step() -> usize {
    1
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: default_max_retries(),
            retry_delay: default_retry_delay(),
            retry_on_errors: Vec::new(),
        }
    }
}
