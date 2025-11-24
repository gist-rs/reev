//! Dynamic Mode Execution
//!
//! This module handles dynamic user request execution by generating
//! temporary YML files and planning. It provides clean separation
//! from benchmark mode while sharing same execution pipeline.

use crate::gateway::OrchestratorGateway;
use crate::Result;
use crate::{benchmark_mode::ExecutionMetadata, benchmark_mode::ExecutionMode};
use chrono::Utc;
use reev_types::execution::{ExecutionResponse, ExecutionStatus};
use reev_types::flow::{DynamicFlowPlan, WalletContext};
use reev_types::ToolCallSummary;
use std::path::PathBuf;
use tempfile::NamedTempFile;
use tracing::{debug, info, instrument};

/// Execute a user request dynamically
///
/// This function:
/// 1. Analyzes user prompt
/// 2. Generates a temporary YML file
/// 3. Returns execution plan for caller to execute
///
/// # Arguments
/// * `prompt` - The natural language user request
/// * `context` - Wallet context containing balances and metadata
/// * `agent` - Optional agent type to use
///
/// # Returns
/// * `Result<DynamicExecutionPlan>` - The execution plan
///
/// # Errors
/// * If prompt analysis fails
/// * If YML generation fails
#[instrument(skip_all)]
pub async fn prepare_user_request(
    prompt: &str,
    context: &WalletContext,
    agent: Option<&str>,
) -> Result<DynamicExecutionPlan> {
    info!("Preparing dynamic user request: {}", prompt);

    // Create orchestrator gateway for YML generation
    let gateway = OrchestratorGateway::new().await?;

    // Generate temporary YML file from user prompt
    let (yml_path, temp_file) = generate_dynamic_yml(&gateway, prompt, context).await?;

    debug!("Generated temporary YML: {}", yml_path.display());

    let plan = DynamicExecutionPlan {
        yml_path,
        temp_file: Some(temp_file),
        prompt: prompt.to_string(),
        wallet: context.owner.clone(),
        agent: agent.map(|a| a.to_string()),
        execution_mode: ExecutionMode::Dynamic,
        metadata: ExecutionMetadata {
            source: "dynamic_generation".to_string(),
            created_at: chrono::Utc::now(),
            benchmark_type: "user_request".to_string(),
            description: format!("Dynamic execution: {prompt}"),
            tags: vec!["dynamic".to_string(), "user_request".to_string()],
        },
    };

    info!("Dynamic request preparation completed: {}", prompt);
    Ok(plan)
}

/// Execute a user request dynamically
///
/// This is a convenience function that calls prepare_user_request
/// and delegates actual execution to caller
///
/// # Arguments
/// * `prompt` - User request
/// * `context` - Wallet context
/// * `agent` - Optional agent type
/// * `executor` - Function to execute YML file
///
/// # Returns
/// * `Result<ExecutionResult>` - The execution result
pub async fn execute_user_request<F, Fut>(
    prompt: &str,
    context: &WalletContext,
    agent: Option<&str>,
    executor: F,
) -> Result<ExecutionResponse>
where
    F: FnOnce(PathBuf, Option<String>) -> Fut,
    Fut: std::future::Future<Output = Result<ExecutionResponse>>,
{
    // Validate user request before execution
    validate_user_request(prompt, context)?;

    let plan = prepare_user_request(prompt, context, agent).await?;

    // Check if should use database flow (PingPongExecutor)
    let gateway = OrchestratorGateway::new().await?;
    if gateway.should_use_database_flow(&plan.yml_path).await? {
        info!("[DynamicMode] Using database flow execution");

        // Parse the YML to create DynamicFlowPlan
        let yml_content = tokio::fs::read_to_string(&plan.yml_path).await?;
        let flow_plan: DynamicFlowPlan = serde_yaml::from_str(&yml_content)?;

        // Execute with PingPongExecutor (database + consolidation)
        let execution_result = gateway
            .execute_dynamic_flow_with_consolidation(
                &flow_plan,
                &plan.agent.unwrap_or_else(|| "glm-4.6-coding".to_string()),
            )
            .await?;

        // Convert ExecutionResult to ExecutionResponse
        let status = if execution_result.success {
            ExecutionStatus::Completed
        } else {
            ExecutionStatus::Failed
        };

        // Convert step results to tool call summaries
        let tool_calls: Vec<ToolCallSummary> = execution_result
            .step_results
            .iter()
            .filter_map(|step| {
                if !step.tool_calls.is_empty() {
                    Some(ToolCallSummary {
                        tool_name: step.tool_calls.first()?.clone(),
                        timestamp: Utc::now(),
                        duration_ms: step.execution_time_ms,
                        success: step.success,
                        error: step.error_message.clone(),
                        params: None,
                        result_data: Some(step.output.clone()),
                        tool_args: None,
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(ExecutionResponse {
            execution_id: execution_result.execution_id,
            status,
            duration_ms: execution_result.execution_time_ms,
            result: Some(serde_json::json!({
                "completed_steps": execution_result.completed_steps,
                "total_steps": execution_result.total_steps,
                "consolidated_session_id": execution_result.consolidated_session_id,
            })),
            error: execution_result.error_message,
            logs: vec![], // Could be populated if needed
            tool_calls,
        })
    } else {
        // Use traditional file-based execution
        info!("[DynamicMode] Using file-based execution");
        executor(plan.yml_path, plan.agent).await
    }
}

/// Generate a temporary YML file from user prompt
///
/// This function:
/// 1. Creates an enhanced flow plan from the prompt
/// 2. Generates YML content from the flow plan
/// 3. Writes to a temporary file
///
/// # Arguments
/// * `gateway` - Orchestrator gateway for flow planning
/// * `prompt` - User prompt to analyze
/// * `context` - Wallet context
///
/// # Returns
/// * `Result<(PathBuf, NamedTempFile)>` - Path to temp file and file handle
///
/// # Errors
/// * If flow plan generation fails
/// * If YML writing fails
async fn generate_dynamic_yml(
    gateway: &OrchestratorGateway,
    prompt: &str,
    context: &WalletContext,
) -> Result<(PathBuf, NamedTempFile)> {
    // Generate enhanced flow plan from user prompt
    let flow_plan = gateway
        .generate_enhanced_flow_plan(prompt, context, None)
        .await?;

    debug!("Generated flow plan with {} steps", flow_plan.steps.len());

    // Generate YML from user prompt using YML generator
    let intent = gateway.analyze_user_intent(prompt, context).await?;
    let yml_content = gateway
        .generate_dynamic_yml(&intent, prompt, context)
        .await?;

    // Write to temporary file
    let temp_file = NamedTempFile::new()?;
    let yml_path = temp_file.path().to_path_buf();

    std::fs::write(&yml_path, yml_content)?;

    debug!("Wrote temporary YML to: {}", yml_path.display());

    Ok((yml_path, temp_file))
}

/// Simple intent analysis moved to gateway module
/// Simple intent analysis for user requests
///
/// This provides basic categorization without over-engineering.
/// For production use, this could be enhanced with LLM-based analysis.
///
/// # Arguments
/// * `prompt` - User prompt to analyze
///
/// # Returns
/// * `UserIntent` - Simple intent analysis
pub fn analyze_simple_intent(prompt: &str) -> UserIntent {
    let prompt_lower = prompt.to_lowercase();

    let (primary_tool, parameters) = if prompt_lower.contains("swap") {
        (
            reev_types::tools::ToolName::JupiterSwap,
            extract_amount(prompt),
        )
    } else if prompt_lower.contains("lend") || prompt_lower.contains("deposit") {
        (
            reev_types::tools::ToolName::JupiterLendEarnDeposit,
            extract_amount(prompt),
        )
    } else if prompt_lower.contains("withdraw") {
        (
            reev_types::tools::ToolName::JupiterLendEarnWithdraw,
            extract_amount(prompt),
        )
    } else if prompt_lower.contains("position") || prompt_lower.contains("balance") {
        (
            reev_types::tools::ToolName::GetJupiterLendEarnPosition,
            extract_amount(prompt),
        )
    } else {
        // Default to swap for unknown intents
        (
            reev_types::tools::ToolName::JupiterSwap,
            extract_amount(prompt),
        )
    };

    UserIntent {
        primary_tool,
        parameters,
        complexity: if prompt_lower.contains("%") || prompt_lower.contains("multiply") {
            "complex"
        } else {
            "simple"
        },
    }
}

/// Simple user intent analysis result
#[derive(Debug, Clone)]
pub struct UserIntent {
    /// Primary tool identified
    pub primary_tool: reev_types::tools::ToolName,
    /// Extracted parameters (amounts, percentages, etc.)
    pub parameters: String,
    /// Intent complexity level
    pub complexity: &'static str,
}

/// Extract amount information from prompt
///
/// Simple regex-based extraction for common patterns:
/// - "1 SOL"
/// - "50%"
/// - "0.5 USDC"
///
/// # Arguments
/// * `prompt` - Prompt to extract from
///
/// # Returns
/// * `String` - Extracted amount or empty string
fn extract_amount(prompt: &str) -> String {
    use regex::Regex;

    // First try to match percentages (e.g., "50%")
    let percent_regex = Regex::new(r"\d+\.?\d*%").ok();
    if let Some(re) = percent_regex {
        if let Some(caps) = re.captures(prompt) {
            return caps
                .get(0)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
        }
    }

    // Then try to match amounts with units (e.g., "1.5 SOL", "100 USDC")
    let amount_regex = Regex::new(r"\b\d+\.?\d*\s+(SOL|USDC)\b").ok();
    if let Some(re) = amount_regex {
        if let Some(caps) = re.captures(prompt) {
            return caps
                .get(0)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
        }
    }

    // Finally try to match simple numbers
    let simple_regex = Regex::new(r"\b\d+\b").ok();
    if let Some(re) = simple_regex {
        if let Some(caps) = re.captures(prompt) {
            return caps
                .get(0)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
        }
    }

    String::new()
}

/// Dynamic execution plan
#[derive(Debug)]
pub struct DynamicExecutionPlan {
    /// Path to temporary YML file
    pub yml_path: PathBuf,
    /// Temporary file handle (to keep file alive)
    pub temp_file: Option<NamedTempFile>,
    /// Original user prompt
    pub prompt: String,
    /// Wallet address
    pub wallet: String,
    /// Agent type to use
    pub agent: Option<String>,
    /// Execution mode
    pub execution_mode: ExecutionMode,
    /// Execution metadata
    pub metadata: ExecutionMetadata,
}

/// Use gateway's UserIntent instead
/// Validate user request before execution
///
/// # Arguments
/// * `prompt` - User request to validate
/// * `context` - Wallet context
///
/// # Returns
/// * `Result<()>` - Ok if valid, Err with reason if invalid
pub fn validate_user_request(prompt: &str, context: &WalletContext) -> Result<()> {
    if prompt.trim().is_empty() {
        return Err(anyhow::anyhow!("Empty prompt provided"));
    }

    if prompt.len() > 1000 {
        return Err(anyhow::anyhow!("Prompt too long (max 1000 characters)"));
    }

    // Check if user has any funds to work with
    if context.sol_balance == 0 && context.token_balances.is_empty() {
        return Err(anyhow::anyhow!(
            "No funds available in wallet for operations"
        ));
    }

    // Check for potential harmful requests
    let prompt_lower = prompt.to_lowercase();
    let blocked_keywords = vec!["drain", "steal", "hack", "exploit"];
    for keyword in blocked_keywords {
        if prompt_lower.contains(keyword) {
            return Err(anyhow::anyhow!(
                "Request contains blocked keyword: {keyword}"
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use reev_types::flow::WalletContext;
    use serial_test::serial;
    use std::io::Write;
    use tempfile::NamedTempFile;
    use tokio::fs;

    #[tokio::test]
    #[serial]
    async fn test_should_use_database_flow() {
        // Set test mode to avoid requiring ZAI_API_KEY
        std::env::set_var("REEV_TEST_MODE", "true");
        // Use test method that doesn't require ZAI_API_KEY
        let gateway = OrchestratorGateway::new_for_test(None).await.unwrap();

        // Test 1: Dynamic flow should use database
        let dynamic_yml = r#"
flow_id: "test_dynamic"
flow_type: "dynamic"
user_prompt: "Test dynamic flow"
description: "Test dynamic flow"
steps:
  - step_id: "step1"
    agent: "test_agent"
    prompt_template: "Test prompt"
    description: "Test step description"
    required_tools: []
    estimated_time_seconds: 30
    critical: true
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(dynamic_yml.as_bytes()).unwrap();
        let dynamic_path = temp_file.path().to_path_buf();

        let should_use_db = gateway
            .should_use_database_flow(&dynamic_path)
            .await
            .unwrap();
        assert!(should_use_db, "Dynamic flow should use database");

        // Test 2: Static flow should use file-based
        let static_yml = r#"
flow_id: "test_static"
user_prompt: "Test static flow"
description: "Test static flow"
steps:
  - step_id: "step1"
    agent: "test_agent"
    prompt_template: "Test prompt"
    description: "Test step description"
    required_tools: []
    estimated_time_seconds: 30
    critical: true
"#;

        let mut temp_file2 = NamedTempFile::new().unwrap();
        temp_file2.write_all(static_yml.as_bytes()).unwrap();
        let static_path = temp_file2.path().to_path_buf();

        let should_use_db = gateway
            .should_use_database_flow(&static_path)
            .await
            .unwrap();
        assert!(!should_use_db, "Static flow should use file-based");
    }

    #[tokio::test]
    #[serial]
    async fn test_dynamic_flow_with_database_routing() {
        // This test verifies that the database routing logic works
        // Full end-to-end test would require actual agent execution
        // Set test mode to avoid requiring ZAI_API_KEY
        std::env::set_var("REEV_TEST_MODE", "true");
        // Use test method that doesn't require ZAI_API_KEY
        let gateway = OrchestratorGateway::new_for_test(None).await.unwrap();

        let dynamic_yml = r#"
flow_id: "test_dynamic_flow"
flow_type: "dynamic"
user_prompt: "Test dynamic flow for database routing"
description: "Test dynamic flow for database routing"
atomic_mode: "Strict"
steps:
  - step_id: "balance_check"
    agent: "glm-4.6-coding"
    prompt_template: "Get account balance"
    description: "Check account balance"
    required_tools: ["GetAccountBalance"]
    estimated_time_seconds: 10
    critical: true
initial_state:
  - name: "wallet_address"
    value: "test_wallet_address"
context:
  owner: "test_wallet"
  sol_balance: 1000000000
  token_balances: {}
  token_prices: {}
  total_value_usd: 100.0
metadata:
  created_at: "2024-01-01T00:00:00Z"
  category: "test"
  complexity_score: 1
  tags: ["test"]
  version: "1.0"
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(dynamic_yml.as_bytes()).unwrap();
        let yml_path = temp_file.path().to_path_buf();

        // Verify the routing decision
        let should_use_db = gateway.should_use_database_flow(&yml_path).await.unwrap();
        assert!(should_use_db, "Should route to database");

        // Verify the YML can be parsed as DynamicFlowPlan
        let yml_content = fs::read_to_string(&yml_path).await.unwrap();
        let flow_plan: DynamicFlowPlan = serde_yaml::from_str(&yml_content).unwrap();

        assert_eq!(flow_plan.flow_id, "test_dynamic_flow");
        assert_eq!(flow_plan.steps.len(), 1);
        assert_eq!(flow_plan.steps[0].step_id, "balance_check");
    }

    #[test]
    fn test_simple_intent_analysis() {
        let intent = analyze_simple_intent("swap 1 SOL to USDC");
        assert_eq!(
            intent.primary_tool,
            reev_types::tools::ToolName::JupiterSwap
        );
        assert_eq!(intent.parameters, "1 SOL");
        assert_eq!(intent.complexity, "simple");

        let intent = analyze_simple_intent("use 50% of SOL for lending");
        assert_eq!(
            intent.primary_tool,
            reev_types::tools::ToolName::JupiterLendEarnDeposit
        );
        assert_eq!(intent.complexity, "complex");
    }

    #[test]
    fn test_amount_extraction() {
        assert_eq!(extract_amount("swap 1.5 SOL to USDC"), "1.5 SOL");
        assert_eq!(extract_amount("use 50% of SOL"), "50%");
        assert_eq!(extract_amount("lend 100 USDC"), "100 USDC");
        assert_eq!(extract_amount("check balance"), "");
    }

    #[test]
    fn test_request_validation() {
        let mut context = WalletContext::new("test_wallet".to_string());
        context.sol_balance = 1_000_000_000; // 1 SOL in lamports

        // Valid request
        assert!(validate_user_request("swap 1 SOL", &context).is_ok());

        // Empty request
        assert!(validate_user_request("", &context).is_err());

        // Too long request
        let long_prompt = "a".repeat(1001);
        assert!(validate_user_request(&long_prompt, &context).is_err());

        // Blocked keyword
        assert!(validate_user_request("drain wallet", &context).is_err());
    }

    #[test]
    fn test_empty_wallet_validation() {
        let empty_context = WalletContext::new("empty_wallet".to_string());

        // Empty wallet should fail
        assert!(validate_user_request("swap 1 SOL", &empty_context).is_err());
    }
}
