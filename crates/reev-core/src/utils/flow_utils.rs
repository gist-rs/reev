//! Flow utilities for execution flow steps and standardization
//!
//! This module provides utilities for creating and executing standardized
//! flow steps across tests, API, and runner components. It follows the
//! 6-step process defined in PLAN_CORE_V3.md and provides composable
//! functions for different scenarios.

use crate::context::{ContextResolver, SolanaEnvironment};
use crate::planner::Planner;
use crate::yml_schema::YmlFlow;
use crate::Executor;
use anyhow::{anyhow, Result};
use reev_types::flow::{FlowResult, WalletContext};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use tracing::info;

/// Executes the 6-step flow process for a given prompt
///
/// This function implements the standardized 6-step process from PLAN_CORE_V3.md:
/// 1. Create YML prompt with wallet context
/// 2. Send prompt to LLM
/// 3. Generate flow from prompt
/// 4. Execute flow with tools
/// 5. Extract transaction signature
/// 6. Return result with metadata
///
/// # Arguments
/// * `prompt` - The user prompt to process
/// * `pubkey` - The public key of the wallet
/// * `initial_sol_balance` - Initial SOL balance in lamports
/// * `initial_token_balances` - Optional map of token mints to their amounts
/// * `total_value_usd` - Total USD value of the wallet
///
/// # Returns
/// A Result containing the execution result and transaction signature
pub async fn execute_six_step_flow(
    prompt: &str,
    pubkey: &Pubkey,
    initial_sol_balance: u64,
    initial_token_balances: Option<HashMap<String, f64>>,
    total_value_usd: f64,
) -> Result<(FlowResult, String)> {
    info!(
        "\n🚀 Starting 6-step flow execution with prompt: {}",
        prompt
    );

    // Step 1: Create YML prompt with wallet context
    info!("📋 Step 1: Creating YML prompt with wallet context");
    let yml_prompt = create_yml_prompt_for_flow(
        prompt,
        pubkey,
        initial_sol_balance,
        initial_token_balances,
        total_value_usd,
    )?;
    info!("📝 Created YML prompt:\n{}", yml_prompt);

    // Step 2: Send prompt to LLM (handled by planner.refine_and_plan)
    info!("\n🤖 Step 2: Sending prompt to GLM-4.6 model via ZAI_API_KEY...");

    // Step 3: Generate flow from prompt
    info!("\n⚙️ Step 3: Generating flow from prompt");
    let (flow, wallet_context) = create_flow_from_prompt(prompt, pubkey).await?;
    info!("✅ Flow generated successfully");

    // Step 4: Execute flow with tools
    info!("\n🔧 Step 4: Executing flow with tools");
    let executor = Executor::new_with_rig().await?;
    let result = executor.execute_flow(&flow, &wallet_context).await?;
    info!("✅ Flow execution completed");

    // Step 5: Extract transaction signature
    info!("\n🔍 Step 5: Extracting transaction signature");
    let signature = crate::utils::result_utils::extract_transaction_signature(&result)?;
    info!("✅ Extracted transaction signature: {}", signature);

    // Step 6: Return result with metadata
    info!("\n📊 Step 6: Processing execution result");
    let metadata = crate::utils::result_utils::extract_metadata(&result);
    info!("✅ Execution completed with metadata: {:?}", metadata);

    Ok((result, signature))
}

/// Creates a YML prompt for a flow
///
/// This function creates a YML prompt for a flow using the standardized
/// format from PLAN_CORE_V3.md.
///
/// # Arguments
/// * `prompt` - The user prompt
/// * `pubkey` - The public key of the wallet
/// * `initial_sol_balance` - Initial SOL balance in lamports
/// * `initial_token_balances` - Optional map of token mints to their amounts
/// * `total_value_usd` - Total USD value of the wallet
///
/// # Returns
/// A Result containing the YML prompt string
pub fn create_yml_prompt_for_flow(
    prompt: &str,
    pubkey: &Pubkey,
    initial_sol_balance: u64,
    initial_token_balances: Option<HashMap<String, f64>>,
    total_value_usd: f64,
) -> Result<String> {
    // Determine the operation type from the prompt
    let operation_type = determine_operation_type(prompt);

    // Create the appropriate step based on the operation type
    let step = match operation_type.as_str() {
        "transfer" => {
            let recipient = extract_recipient_from_prompt(prompt)
                .ok_or_else(|| anyhow!("Could not extract recipient from transfer prompt"))?;
            crate::utils::yml_utils::create_transfer_step(prompt, &recipient, true)
        }
        "swap" => crate::utils::yml_utils::create_swap_step(prompt, true),
        "lend" => crate::utils::yml_utils::create_lend_step(prompt, true),
        _ => {
            return Err(anyhow!("Unsupported operation type: {operation_type}"));
        }
    };

    // Generate a unique flow ID
    let flow_id = format!(
        "flow_{}_{:x}",
        chrono::Utc::now().timestamp(),
        rand::random::<u32>()
    );

    // Create YML prompt
    let yml_params = crate::utils::yml_utils::YmlPromptParams {
        flow_id: flow_id.to_string(),
        user_prompt: prompt.to_string(),
        refined_prompt: None,
        pubkey: *pubkey,
        lamports: initial_sol_balance,
        tokens: initial_token_balances,
        total_value_usd,
        steps: vec![step],
    };
    crate::utils::yml_utils::create_yml_prompt(yml_params)
}

/// Creates a flow from a prompt
///
/// This function creates a flow from a prompt using the planner.
///
/// # Arguments
/// * `prompt` - The user prompt
/// * `pubkey` - The public key of the wallet
///
/// # Returns
/// A Result containing the flow and wallet context
pub async fn create_flow_from_prompt(
    prompt: &str,
    pubkey: &Pubkey,
) -> Result<(YmlFlow, WalletContext)> {
    // Set up the context resolver with SURFPOOL RPC URL to match transaction execution
    let context_resolver = ContextResolver::new(SolanaEnvironment {
        rpc_url: Some("http://localhost:8899".to_string()),
    });

    // Create a planner with GLM client
    let mut planner = Planner::new_with_glm(context_resolver.clone())?;

    // Generate the flow using the planner
    let flow = planner.refine_and_plan(prompt, &pubkey.to_string()).await?;

    // Get the wallet context from the resolver
    let wallet_context = context_resolver
        .resolve_wallet_context(&pubkey.to_string())
        .await?;

    Ok((flow, wallet_context))
}

/// Determines the operation type from a prompt
///
/// This function analyzes the prompt to determine the type of operation.
///
/// # Arguments
/// * `prompt` - The user prompt
///
/// # Returns
/// A string representing the operation type ("transfer", "swap", or "lend")
pub fn determine_operation_type(prompt: &str) -> String {
    let lower_prompt = prompt.to_lowercase();

    if lower_prompt.contains("send") || lower_prompt.contains("transfer") {
        "transfer".to_string()
    } else if lower_prompt.contains("swap")
        || lower_prompt.contains("sell")
        || lower_prompt.contains("buy")
    {
        "swap".to_string()
    } else if lower_prompt.contains("lend") || lower_prompt.contains("deposit") {
        "lend".to_string()
    } else {
        "unknown".to_string()
    }
}

/// Extracts recipient from a transfer prompt
///
/// This function extracts the recipient address from a transfer prompt.
///
/// # Arguments
/// * `prompt` - The user prompt
///
/// # Returns
/// An Option containing the recipient address string if found
pub fn extract_recipient_from_prompt(prompt: &str) -> Option<String> {
    // Simple extraction logic - looks for a Solana address pattern in the prompt
    // This is a basic implementation and may need to be enhanced for more complex prompts
    let words: Vec<&str> = prompt.split_whitespace().collect();

    for word in words {
        if word.len() >= 32 && word.len() <= 44 && word.chars().all(|c| c.is_alphanumeric()) {
            return Some(word.to_string());
        }
    }

    None
}

/// Validates execution result against expected outcome
///
/// This function validates an execution result against an expected outcome.
///
/// # Arguments
/// * `result` - The execution result to validate
/// * `expected_signature_pattern` - Expected pattern in the transaction signature
///
/// # Returns
/// A Result indicating success or failure with an error message
pub fn validate_execution_result(
    result: &FlowResult,
    expected_signature_pattern: Option<&str>,
) -> Result<()> {
    // Check if execution is successful
    if !crate::utils::result_utils::is_execution_successful(result) {
        let errors = crate::utils::result_utils::extract_error_messages(result);
        return Err(anyhow!("Execution failed with errors: {errors:?}"));
    }

    // Extract transaction signature
    let signature = crate::utils::result_utils::extract_transaction_signature(result)?;

    // Check signature pattern if provided
    if let Some(pattern) = expected_signature_pattern {
        if !signature.contains(pattern) {
            return Err(anyhow!(
                "Transaction signature does not match expected pattern. Expected: {pattern}, Got: {signature}"
            ));
        }
    }

    Ok(())
}

/// Creates a standardized test result for comparison
///
/// This function creates a standardized test result for comparison across tests.
///
/// # Arguments
/// * `result` - The execution result
/// * `signature` - The transaction signature
/// * `test_name` - The name of the test
///
/// # Returns
/// A HashMap containing the standardized test result
pub fn create_standardized_test_result(
    result: &FlowResult,
    signature: &str,
    test_name: &str,
) -> HashMap<String, String> {
    let mut test_result = HashMap::new();

    test_result.insert("test_name".to_string(), test_name.to_string());
    test_result.insert("transaction_signature".to_string(), signature.to_string());
    test_result.insert(
        "step_count".to_string(),
        result.step_results.len().to_string(),
    );
    test_result.insert(
        "is_successful".to_string(),
        crate::utils::result_utils::is_execution_successful(result).to_string(),
    );

    // Add error messages if any
    let errors = crate::utils::result_utils::extract_error_messages(result);
    if !errors.is_empty() {
        test_result.insert("errors".to_string(), errors.join("; "));
    }

    // Add tool names used
    let metadata = crate::utils::result_utils::extract_metadata(result);
    if let Some(tool_names) = metadata.get("tool_names") {
        test_result.insert("tool_names".to_string(), tool_names.clone());
    }

    test_result
}
