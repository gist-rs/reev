//! Dynamic Mode Execution
//!
//! This module handles dynamic user request execution by generating
//! temporary YML files and planning. It provides clean separation
//! from benchmark mode while sharing same execution pipeline.

use crate::gateway::OrchestratorGateway;
use crate::Result;
use crate::{benchmark_mode::ExecutionMetadata, benchmark_mode::ExecutionMode};
use reev_types::execution::ExecutionResponse;
use reev_types::flow::WalletContext;
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
    executor(plan.yml_path, plan.agent).await
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

    // Try to extract decimal amounts
    let decimal_regex = Regex::new(r"\b\d+\.?\d*\s+(SOL|USDC|%)\b").ok();
    if let Some(re) = decimal_regex {
        if let Some(caps) = re.captures(prompt) {
            return caps
                .get(0)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
        }
    }

    // Try to extract simple amounts
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
        let context = WalletContext::new("test_wallet".to_string());

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
