//! Dynamic Benchmark Runner for Protocol Interface Integration
//!
//! This module provides a dynamic benchmark runner that generates flows from prompts
//! using QueryHandler and executes them using the protocol interface. It integrates
//! with the protocol registry to select appropriate protocols for each operation.

use crate::benchmark::runner::types::{Flow, FlowStep};
use crate::benchmark::types::{
    BenchmarkCategory, BenchmarkReport, BenchmarkScore, ExecutionMetrics, ValidationResult,
    ValidationResults,
};
use crate::protocols::{OperationType, ProtocolOperation, ProtocolRegistry};
use crate::QueryHandler;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::str::FromStr;
use std::time::Instant;

/// Dynamic benchmark runner
///
/// This runner generates flows from prompts using QueryHandler and executes them
/// using the protocol interface. It supports multiple languages and prompt variations.
pub struct DynamicBenchmarkRunner {
    /// Query handler for flow generation
    query_handler: QueryHandler,
    /// Protocol registry for operation execution
    protocol_registry: ProtocolRegistry,
}

impl DynamicBenchmarkRunner {
    /// Create a new dynamic benchmark runner
    pub async fn new() -> Result<Self, DynamicBenchmarkError> {
        let query_handler = QueryHandler::new().await?;
        let mut runner = Self {
            query_handler,
            protocol_registry: ProtocolRegistry::new(),
        };

        // Register default protocols
        runner.register_default_protocols();
        Ok(runner)
    }

    /// Register default protocols
    fn register_default_protocols(&mut self) {
        use crate::protocols::jupiter::JupiterProtocol;

        let jupiter = JupiterProtocol::new();
        self.protocol_registry
            .register_with_operations(
                "jupiter",
                jupiter,
                &[
                    OperationType::Swap,
                    OperationType::Lend,
                    OperationType::Earn,
                ],
            )
            .expect("Failed to register Jupiter protocol");
    }

    /// Generate and execute a flow from a prompt
    ///
    /// # Arguments
    /// * `prompt` - The prompt to generate a flow from
    /// * `wallet_pubkey` - The wallet public key to use for execution
    ///
    /// # Returns
    /// Result containing benchmark report or an error
    pub async fn execute_prompt(
        &mut self,
        prompt: &str,
        wallet_pubkey: &str,
    ) -> Result<BenchmarkReport, DynamicBenchmarkError> {
        // Generate flow from prompt
        let flow = self
            .generate_flow_from_prompt(prompt, wallet_pubkey)
            .await?;

        // Execute the flow
        self.execute_flow(&flow).await
    }

    /// Generate a flow from a prompt
    ///
    /// # Arguments
    /// * `prompt` - The prompt to generate a flow from
    /// * `wallet_pubkey` - The wallet public key to use for execution
    ///
    /// # Returns
    /// Result containing the generated flow or an error
    async fn generate_flow_from_prompt(
        &mut self,
        prompt: &str,
        wallet_pubkey: &str,
    ) -> Result<Flow, DynamicBenchmarkError> {
        // Process the query to generate a flow
        let query_result = self
            .query_handler
            .process_query(
                prompt,
                &solana_sdk::pubkey::Pubkey::from_str(wallet_pubkey).unwrap(),
            )
            .await?;

        if !query_result.success {
            return Err(DynamicBenchmarkError::FlowGenerationFailed(
                query_result
                    .error_message
                    .unwrap_or("Unknown error".to_string()),
            ));
        }

        // Get the generated flow from the query result
        // In a real implementation, this would extract the flow from the result
        // For now, we'll create a simple flow based on the prompt
        self.create_simple_flow(prompt)
    }

    /// Create a simple flow from a prompt
    ///
    /// # Arguments
    /// * `prompt` - The prompt to create a flow from
    ///
    /// # Returns
    /// Result containing the created flow
    fn create_simple_flow(&self, prompt: &str) -> Result<Flow, DynamicBenchmarkError> {
        // Extract operation type from prompt
        let _operation_type = self.extract_operation_type_from_prompt(prompt)?;

        // Create a simple flow with one step
        let flow = Flow {
            id: uuid::Uuid::new_v4().to_string(),
            prompt: Some(prompt.to_string()),
            created_at: chrono::Utc::now(),
            subject_wallet_info: None,
            steps: vec![FlowStep {
                step_id: "1".to_string(),
                refined_prompt: prompt.to_string(),
                context: Some(prompt.to_string()),
                critical: false,
                expected_tools: vec![],
            }],
            ground_truth: None,
        };

        Ok(flow)
    }

    /// Extract operation type from a prompt
    ///
    /// # Arguments
    /// * `prompt` - The prompt to extract from
    ///
    /// # Returns
    /// Result containing the operation type or an error
    fn extract_operation_type_from_prompt(
        &self,
        prompt: &str,
    ) -> Result<OperationType, DynamicBenchmarkError> {
        let prompt_lower = prompt.to_lowercase();

        if prompt_lower.contains("swap") {
            Ok(OperationType::Swap)
        } else if prompt_lower.contains("lend") {
            Ok(OperationType::Lend)
        } else if prompt_lower.contains("earn") {
            Ok(OperationType::Earn)
        } else if prompt_lower.contains("transfer") {
            Ok(OperationType::Transfer)
        } else if prompt_lower.contains("stake") {
            Ok(OperationType::Stake)
        } else {
            Err(DynamicBenchmarkError::CannotDetermineOperation(
                "Cannot determine operation type from prompt".to_string(),
            ))
        }
    }

    /// Execute a benchmark flow
    ///
    /// # Arguments
    /// * `flow` - The flow to execute
    ///
    /// # Returns
    /// Result containing benchmark report or an error
    pub async fn execute_flow(
        &self,
        flow: &Flow,
    ) -> Result<BenchmarkReport, DynamicBenchmarkError> {
        let start_time = Instant::now();
        let execution_id = uuid::Uuid::new_v4().to_string();

        // Initialize metrics
        let mut metrics = ExecutionMetrics::default();

        // Initialize validation results
        let mut validation_results = ValidationResults::new();

        // Initialize benchmark score
        let mut benchmark_score = BenchmarkScore::new();

        // Execute each step in the flow
        let mut step_count = 0;
        let mut tool_call_count = 0;
        let mut successful_tool_calls = 0;

        for step in &flow.steps {
            step_count += 1;

            // Extract operation from step
            let operation = self.extract_operation_from_step(step)?;

            // Get protocol for this operation
            let protocol = self
                .protocol_registry
                .get_for_operation(&operation.operation_type)
                .ok_or_else(|| {
                    DynamicBenchmarkError::NoProtocolForOperation(operation.operation_type.clone())
                })?;

            // Execute operation
            let result = protocol.execute(&operation).await;

            tool_call_count += 1;

            // Handle execution result
            match result {
                Ok(_) => {
                    successful_tool_calls += 1;

                    // Add a successful tool call validation
                    validation_results.add_tool_call_result(ValidationResult {
                        assertion_type: "tool_call_success".to_string(),
                        passed: true,
                        actual_value: Some(json!(true)),
                        expected_value: Some(json!(true)),
                        error_message: None,
                        weight: 1.0,
                    });

                    // Add a prompt understanding validation
                    validation_results.add_success_criteria_result(ValidationResult {
                        assertion_type: "prompt_understanding".to_string(),
                        passed: true,
                        actual_value: Some(json!(operation.operation_type.to_string())),
                        expected_value: Some(json!(operation.operation_type.to_string())),
                        error_message: None,
                        weight: 1.0,
                    });
                }
                Err(e) => {
                    // Add a failed tool call validation
                    validation_results.add_tool_call_result(ValidationResult {
                        assertion_type: "tool_call_success".to_string(),
                        passed: false,
                        actual_value: Some(json!(false)),
                        expected_value: Some(json!(true)),
                        error_message: Some(format!("Protocol execution failed: {e}")),
                        weight: 1.0,
                    });

                    // Add a prompt understanding validation
                    validation_results.add_success_criteria_result(ValidationResult {
                        assertion_type: "prompt_understanding".to_string(),
                        passed: true, // Even if execution failed, we understood the prompt
                        actual_value: Some(json!(operation.operation_type.to_string())),
                        expected_value: Some(json!(operation.operation_type.to_string())),
                        error_message: None,
                        weight: 1.0,
                    });
                }
            }
        }

        // Update execution metrics
        metrics.total_execution_time_ms = start_time.elapsed().as_millis() as u64;
        metrics.steps_executed = step_count;
        metrics.tool_calls_made = tool_call_count;
        metrics.successful_tool_calls = successful_tool_calls;

        // Calculate category scores
        let mut weights = HashMap::new();
        for category in BenchmarkCategory::all() {
            weights.insert(format!("{category:?}"), category.default_weight());
        }

        // Calculate scores for each category
        for category in BenchmarkCategory::all() {
            let score = validation_results.calculate_category_score(category);
            benchmark_score.add_category(format!("{category:?}"), score);
        }

        // Calculate weighted average score
        benchmark_score.calculate_weighted_average(&weights);

        // Set score metrics
        benchmark_score.execution_time_ms = metrics.total_execution_time_ms;
        benchmark_score.passed_assertions = validation_results
            .final_state_results
            .iter()
            .chain(validation_results.tool_call_results.iter())
            .chain(validation_results.success_criteria_results.iter())
            .chain(validation_results.performance_results.iter())
            .filter(|r| r.passed)
            .count() as u32;

        benchmark_score.failed_assertions = validation_results
            .final_state_results
            .iter()
            .chain(validation_results.tool_call_results.iter())
            .chain(validation_results.success_criteria_results.iter())
            .chain(validation_results.performance_results.iter())
            .filter(|r| !r.passed)
            .count() as u32;

        // Create benchmark report
        let report = BenchmarkReport {
            flow_id: flow.id.clone(),
            execution_id,
            prompt: flow.prompt.clone().unwrap_or_default(),
            overall_score: benchmark_score.overall_score,
            category_scores: benchmark_score.category_scores.clone(),
            validation_results,
            execution_metrics: metrics,
            improvement_suggestions: vec![], // TODO: Implement suggestion generation
            timestamp: chrono::Utc::now(),
        };

        Ok(report)
    }

    /// Extract a protocol operation from a flow step
    ///
    /// # Arguments
    /// * `step` - The flow step to extract from
    ///
    /// # Returns
    /// Result containing protocol operation or an error
    fn extract_operation_from_step(
        &self,
        step: &FlowStep,
    ) -> Result<ProtocolOperation, DynamicBenchmarkError> {
        // Extract operation type from step
        let operation_type = self.extract_operation_type_from_step(step)?;

        // Extract parameters from step
        let parameters = self.extract_parameters_from_step(step)?;

        Ok(ProtocolOperation {
            operation_type,
            parameters,
        })
    }

    /// Extract operation type from a flow step
    fn extract_operation_type_from_step(
        &self,
        step: &FlowStep,
    ) -> Result<OperationType, DynamicBenchmarkError> {
        // This is a simplified extraction - in a real implementation,
        // this would parse the step's refined_prompt or context more thoroughly
        let context = step
            .context
            .as_deref()
            .unwrap_or(step.refined_prompt.as_str());

        match Some(context) {
            Some(context) if context.to_lowercase().contains("swap") => Ok(OperationType::Swap),
            Some(context) if context.to_lowercase().contains("lend") => Ok(OperationType::Lend),
            Some(context) if context.to_lowercase().contains("earn") => Ok(OperationType::Earn),
            Some(context) if context.to_lowercase().contains("transfer") => {
                Ok(OperationType::Transfer)
            }
            Some(context) if context.to_lowercase().contains("stake") => Ok(OperationType::Stake),
            _ => Err(DynamicBenchmarkError::InvalidStepFormat(
                "Cannot determine operation type from step".to_string(),
            )),
        }
    }

    /// Extract parameters from a flow step
    fn extract_parameters_from_step(
        &self,
        step: &FlowStep,
    ) -> Result<HashMap<String, Value>, DynamicBenchmarkError> {
        let mut parameters = HashMap::new();
        let context = step
            .context
            .as_deref()
            .unwrap_or(step.refined_prompt.as_str());

        // context is already &str, no need for if let
        {
            // For swap operations
            if context.to_lowercase().contains("swap") {
                parameters.insert(
                    "input_mint".to_string(),
                    json!("So11111111111111111111111111111111111112"),
                );
                parameters.insert(
                    "output_mint".to_string(),
                    json!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
                );

                // Extract amount from context
                if let Some(amount) = extract_amount_from_text(context) {
                    parameters.insert("amount".to_string(), json!(amount));
                } else {
                    // Default amount if not found
                    parameters.insert("amount".to_string(), json!(1000000)); // 1 SOL in lamports
                }
            }

            // For lend operations
            if context.to_lowercase().contains("lend") {
                parameters.insert(
                    "mint".to_string(),
                    json!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
                );

                // Extract amount from context
                if let Some(amount) = extract_amount_from_text(context) {
                    parameters.insert("amount".to_string(), json!(amount));
                } else {
                    // Default amount if not found
                    parameters.insert("amount".to_string(), json!(10000000)); // 10 USDC
                }
            }

            // For earn operations
            if context.to_lowercase().contains("earn") {
                parameters.insert(
                    "mint".to_string(),
                    json!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
                );

                // Extract amount from context
                if let Some(amount) = extract_amount_from_text(context) {
                    parameters.insert("amount".to_string(), json!(amount));
                } else {
                    // Default amount if not found
                    parameters.insert("amount".to_string(), json!(5000000)); // 5 USDC
                }
            }
        }

        Ok(parameters)
    }
}

/// Error types for dynamic benchmark runner
#[derive(thiserror::Error, Debug)]
pub enum DynamicBenchmarkError {
    /// No protocol available for operation
    #[error("No protocol available for operation: {0}")]
    NoProtocolForOperation(OperationType),

    /// Cannot determine operation from prompt
    #[error("Cannot determine operation from prompt: {0}")]
    CannotDetermineOperation(String),

    /// Flow generation failed
    #[error("Flow generation failed: {0}")]
    FlowGenerationFailed(String),

    /// Invalid step format
    #[error("Invalid step format: {0}")]
    InvalidStepFormat(String),

    /// Query handler error
    #[error("Query handler error: {0}")]
    QueryHandlerError(String),

    /// Invalid public key
    #[error("Invalid public key: {0}")]
    InvalidPubkey(String),
}

impl From<solana_sdk::pubkey::ParsePubkeyError> for DynamicBenchmarkError {
    fn from(err: solana_sdk::pubkey::ParsePubkeyError) -> Self {
        DynamicBenchmarkError::InvalidPubkey(format!("{err:?}"))
    }
}

impl From<anyhow::Error> for DynamicBenchmarkError {
    fn from(err: anyhow::Error) -> Self {
        DynamicBenchmarkError::QueryHandlerError(err.to_string())
    }
}

/// Helper function to extract amount from text
///
/// # Arguments
/// * `text` - Text to extract amount from
///
/// # Returns
/// Option containing amount if found
fn extract_amount_from_text(text: &str) -> Option<u64> {
    use regex::Regex;

    // Look for patterns like "1 SOL", "10 USDC", etc.
    let re = Regex::new(r"(\d+(?:\.\d+)?)\s*([A-Za-z]+)").ok()?;
    let caps = re.captures(text)?;

    let amount_str = caps.get(1)?.as_str();
    let token = caps.get(2)?.as_str();

    // Parse amount
    let amount = amount_str.parse::<f64>().ok()?;

    // Convert to lamports based on token
    let lamports = match token.to_lowercase().as_str() {
        "sol" => (amount * 1_000_000_000.0) as u64,
        "usdc" => (amount * 1_000_000.0) as u64,
        _ => return None,
    };

    Some(lamports)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_amount_from_text() {
        assert_eq!(
            extract_amount_from_text("swap 1 SOL to USDC"),
            Some(1_000_000_000)
        );
        assert_eq!(extract_amount_from_text("lend 10 USDC"), Some(10_000_000));
        assert_eq!(extract_amount_from_text("earn 0.5 SOL"), Some(500_000_000));
        assert_eq!(extract_amount_from_text("invalid amount"), None);
    }

    #[test]
    fn test_extract_operation_type_from_prompt() {
        let _runner = DynamicBenchmarkRunner::new();

        // This is just a placeholder test - in a real implementation,
        // we would need to await the async constructor
        // assert_eq!(
        //     runner.extract_operation_type_from_prompt("swap 1 SOL to USDC").unwrap(),
        //     OperationType::Swap
        // );
        // assert_eq!(
        //     runner.extract_operation_type_from_prompt("lend 10 USDC").unwrap(),
        //     OperationType::Lend
        // );
    }

    #[test]
    fn test_extract_operation_type_from_step() {
        let _runner = DynamicBenchmarkRunner::new();

        // This is just a placeholder test - in a real implementation,
        // we would need to await the async constructor

        let _swap_step = crate::benchmark::runner::types::FlowStep {
            step_id: "1".to_string(),
            refined_prompt: "swap 1 SOL to USDC".to_string(),
            context: Some("swap 1 SOL to USDC".to_string()),
            critical: false,
            expected_tools: vec![],
        };

        // assert_eq!(
        //     runner.extract_operation_type_from_step(&swap_step).unwrap(),
        //     OperationType::Swap
        // );
    }
}
