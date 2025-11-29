//! YML utilities for creating standardized prompts and structures
//!
//! This module provides utilities for creating YML prompts and structures
//! in a standardized way across tests, API, and runner components.
//! It follows the YML structure defined in PLAN_CORE_V3.md.

use anyhow::Result;
use serde_json::Value;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;

/// Creates a subject_wallet_info YML section for a given pubkey
///
/// This function generates the subject_wallet_info section of the YML prompt
/// following the structure defined in PLAN_CORE_V3.md.
///
/// # Arguments
/// * `pubkey` - The public key of the wallet
/// * `lamports` - The lamports balance of the wallet
/// * `tokens` - Optional map of token mints to their amounts
/// * `total_value_usd` - Total USD value of the wallet
///
/// # Returns
/// A formatted string with the subject_wallet_info section
pub fn create_subject_wallet_info_yml(
    pubkey: &Pubkey,
    lamports: u64,
    tokens: Option<HashMap<String, f64>>,
    total_value_usd: f64,
) -> String {
    let sol_balance = lamports as f64 / 1_000_000_000.0;

    let mut result = format!(
        "subject_wallet_info:\n  - pubkey: \"{pubkey}\"\n    lamports: {lamports} # {sol_balance} SOL\n    total_value_usd: {total_value_usd}"
    );

    if let Some(tokens) = tokens {
        result.push_str("\n    tokens:");
        for (mint, amount) in tokens {
            result.push_str(&format!(
                "\n      - mint: \"{mint}\"\n        amount: {amount}"
            ));
        }
    }

    result
}

/// Creates a full YML prompt with wallet info and steps
///
/// This function generates a complete YML prompt following the structure
/// defined in PLAN_CORE_V3.md.
///
/// # Arguments
/// * `flow_id` - Unique identifier for the flow
/// * `user_prompt` - The original user prompt
/// * `refined_prompt` - The refined prompt (optional, will use user_prompt if None)
/// * `pubkey` - The public key of the wallet
/// * `lamports` - The lamports balance of the wallet
/// * `tokens` - Optional map of token mints to their amounts
/// * `total_value_usd` - Total USD value of the wallet
/// * `steps` - List of steps to execute
///
/// # Returns
/// A formatted string with the complete YML prompt
/// Parameters for creating a YML prompt
pub struct YmlPromptParams {
    pub flow_id: String,
    pub user_prompt: String,
    pub refined_prompt: Option<String>,
    pub pubkey: Pubkey,
    pub lamports: u64,
    pub tokens: Option<HashMap<String, f64>>,
    pub total_value_usd: f64,
    pub steps: Vec<YmlStepData>,
}

pub fn create_yml_prompt(params: YmlPromptParams) -> Result<String> {
    let YmlPromptParams {
        flow_id,
        user_prompt,
        refined_prompt,
        pubkey,
        lamports,
        tokens,
        total_value_usd,
        steps,
    } = params;
    let refined_prompt_str = refined_prompt.unwrap_or_else(|| user_prompt.clone());
    let mut result = format!(
        "flow_id: {}\nuser_prompt: \"{}\"\nrefined_prompt: \"{}\"\ncreated_at: {}\n",
        flow_id,
        user_prompt,
        refined_prompt_str,
        chrono::Utc::now().to_rfc3339()
    );

    // Add subject_wallet_info section
    result.push_str(&create_subject_wallet_info_yml(
        &pubkey,
        lamports,
        tokens,
        total_value_usd,
    ));

    // Add steps section
    result.push_str("\n\nsteps:");
    for step in steps {
        result.push_str(&format!(
            "\n  - step_id: {}\n    refined_prompt: \"{}\"\n    context: \"{}\"\n    critical: {}\n    expected_tools: [{}]",
            step.step_id,
            step.refined_prompt,
            step.context,
            step.critical,
            step.expected_tools.join(", ")
        ));

        if !step.additional_fields.is_empty() {
            for (key, value) in step.additional_fields {
                result.push_str(&format!("\n    {key}: \"{value}\""));
            }
        }
    }

    Ok(result)
}

/// Creates a simple step for a transfer operation
pub fn create_transfer_step(prompt: &str, recipient: &str, critical: bool) -> YmlStepData {
    YmlStepData {
        step_id: "transfer_step".to_string(),
        refined_prompt: prompt.to_string(),
        context: "Executing a SOL transfer using Solana system instructions".to_string(),
        critical,
        expected_tools: vec!["sol_transfer".to_string()],
        additional_fields: HashMap::from([
            ("recipient".to_string(), recipient.to_string()),
            ("intent".to_string(), "send".to_string()),
        ]),
    }
}

/// Creates a simple step for a swap operation
pub fn create_swap_step(prompt: &str, critical: bool) -> YmlStepData {
    YmlStepData {
        step_id: "swap_step".to_string(),
        refined_prompt: prompt.to_string(),
        context: "Executing a swap using Jupiter".to_string(),
        critical,
        expected_tools: vec!["jupiter_swap".to_string()],
        additional_fields: HashMap::from([("intent".to_string(), "swap".to_string())]),
    }
}

/// Creates a simple step for a lend operation
pub fn create_lend_step(prompt: &str, critical: bool) -> YmlStepData {
    YmlStepData {
        step_id: "lend_step".to_string(),
        refined_prompt: prompt.to_string(),
        context: "Executing a lend operation using Jupiter".to_string(),
        critical,
        expected_tools: vec!["jupiter_lend_earn_deposit".to_string()],
        additional_fields: HashMap::from([("intent".to_string(), "lend".to_string())]),
    }
}

/// Data structure representing a step in the YML flow
#[derive(Debug, Clone)]
pub struct YmlStepData {
    pub step_id: String,
    pub refined_prompt: String,
    pub context: String,
    pub critical: bool,
    pub expected_tools: Vec<String>,
    pub additional_fields: HashMap<String, String>,
}

/// Creates a YML ground truth section for validation
///
/// This function generates the ground_truth section of the YML structure
/// following the format defined in PLAN_CORE_V3.md.
///
/// # Arguments
/// * `final_state_assertions` - Assertions about the final state
/// * `expected_tool_calls` - Expected tool calls during execution
///
/// # Returns
/// A formatted string with the ground_truth section
pub fn create_ground_truth_yml(
    final_state_assertions: Vec<YmlAssertionData>,
    expected_tool_calls: Vec<YmlToolCallData>,
) -> String {
    let mut result = String::from("\n\nground_truth:");

    // Add final_state_assertions
    result.push_str("\n  final_state_assertions:");
    for assertion in final_state_assertions {
        result.push_str(&format!(
            "\n    - type: {}\n      pubkey: \"{}\"\n      expected_change_lte: {}\n      error_tolerance: {}",
            assertion.assertion_type,
            assertion.pubkey,
            assertion.expected_change_lte,
            assertion.error_tolerance
        ));
    }

    // Add expected_tool_calls
    result.push_str("\n  expected_tool_calls:");
    for tool_call in expected_tool_calls {
        result.push_str(&format!(
            "\n    - tool_name: \"{}\"\n      critical: {}",
            tool_call.tool_name, tool_call.critical
        ));
    }

    result
}

/// Data structure representing a final state assertion
#[derive(Debug, Clone)]
pub struct YmlAssertionData {
    pub assertion_type: String,
    pub pubkey: String,
    pub expected_change_lte: f64,
    pub error_tolerance: f64,
}

/// Data structure representing an expected tool call
#[derive(Debug, Clone)]
pub struct YmlToolCallData {
    pub tool_name: String,
    pub critical: bool,
}

/// Creates a complete YML structure with ground truth for validation
///
/// This function generates a complete YML structure including the ground_truth
/// section for validation purposes.
///
/// # Arguments
/// * `flow_id` - Unique identifier for the flow
/// * `user_prompt` - The original user prompt
/// * `refined_prompt` - The refined prompt (optional, will use user_prompt if None)
/// * `pubkey` - The public key of the wallet
/// * `lamports` - The lamports balance of the wallet
/// * `tokens` - Optional map of token mints to their amounts
/// * `total_value_usd` - Total USD value of the wallet
/// * `steps` - List of steps to execute
/// * `final_state_assertions` - Assertions about the final state
/// * `expected_tool_calls` - Expected tool calls during execution
///
/// # Returns
/// A formatted string with the complete YML structure including ground truth
/// Parameters for creating a complete YML with ground truth
pub struct CompleteYmlParams {
    pub flow_id: String,
    pub user_prompt: String,
    pub refined_prompt: Option<String>,
    pub pubkey: Pubkey,
    pub lamports: u64,
    pub tokens: Option<HashMap<String, f64>>,
    pub total_value_usd: f64,
    pub steps: Vec<YmlStepData>,
    pub final_state_assertions: Vec<YmlAssertionData>,
    pub expected_tool_calls: Vec<YmlToolCallData>,
}

pub fn create_complete_yml_with_ground_truth(params: CompleteYmlParams) -> Result<String> {
    let CompleteYmlParams {
        flow_id,
        user_prompt,
        refined_prompt,
        pubkey,
        lamports,
        tokens,
        total_value_usd,
        steps,
        final_state_assertions: _,
        expected_tool_calls: _,
    } = params;

    let mut result = create_yml_prompt(YmlPromptParams {
        flow_id,
        user_prompt,
        refined_prompt,
        pubkey,
        lamports,
        tokens,
        total_value_usd,
        steps,
    })?;

    result.push_str(&create_ground_truth_yml(
        params.final_state_assertions,
        params.expected_tool_calls,
    ));

    Ok(result)
}

/// Parses a YML string into a structured format
///
/// This function parses a YML string into a serde_json::Value for easier
/// manipulation and validation.
///
/// # Arguments
/// * `yml_str` - The YML string to parse
///
/// # Returns
/// A Result containing the parsed YML as a serde_json::Value
pub fn parse_yml(yml_str: &str) -> Result<Value> {
    let parsed: Value = serde_yaml::from_str(yml_str)?;
    Ok(parsed)
}

/// Validates a YML structure against the expected format
///
/// This function validates a YML structure against the expected format
/// defined in PLAN_CORE_V3.md.
///
/// # Arguments
/// * `yml_str` - The YML string to validate
///
/// # Returns
/// A Result indicating success or failure with an error message
pub fn validate_yml_structure(yml_str: &str) -> Result<()> {
    let parsed = parse_yml(yml_str)?;

    // Check for required top-level fields
    if parsed.get("flow_id").is_none() {
        return Err(anyhow::anyhow!("Missing required field: flow_id"));
    }

    if parsed.get("user_prompt").is_none() {
        return Err(anyhow::anyhow!("Missing required field: user_prompt"));
    }

    if parsed.get("subject_wallet_info").is_none() {
        return Err(anyhow::anyhow!(
            "Missing required field: subject_wallet_info"
        ));
    }

    if parsed.get("steps").is_none() {
        return Err(anyhow::anyhow!("Missing required field: steps"));
    }

    Ok(())
}
