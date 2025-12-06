//! Result utilities for transaction signature extraction and processing
//!
//! This module provides utilities for extracting transaction signatures from
//! execution results in a standardized way across tests, API, and runner components.
//! It handles various response formats from different tools and provides a unified
//! interface for signature extraction.

use anyhow::{anyhow, Result};
use reev_types::flow::FlowResult;
use serde_json::Value;
use std::collections::HashMap;

/// Extracts transaction signature from execution result
///
/// This function extracts the transaction signature from an execution result,
/// handling various response formats from different tools (transfer, swap, lend).
///
/// # Arguments
/// * `result` - The execution result from which to extract the signature
///
/// # Returns
/// A Result containing the transaction signature string or an error if not found
pub fn extract_transaction_signature(result: &FlowResult) -> Result<String> {
    // Iterate through step results to find the signature
    for step_result in &result.step_results {
        // Check for signature in tool_results array (RigAgent format)
        if let Some(tool_results) = &step_result.tool_results {
            for result_item in tool_results {
                // Check for transaction_signature in the result field (ToolResultWrapper format)
                if let Some(result_field) = result_item.get("result") {
                    // Check for GenericOperation variant
                    if let Some(result_obj) = result_field.as_object() {
                        // Check for transaction_signature in metadata
                        if let Some(metadata) = result_obj.get("metadata") {
                            if let Some(sig) = metadata.get("transaction_signature") {
                                if let Some(sig_str) = sig.as_str() {
                                    return Ok(sig_str.to_string());
                                }
                            }
                        }

                        // Check for transaction_signature directly in result object
                        if let Some(sig) = result_obj.get("transaction_signature") {
                            if let Some(sig_str) = sig.as_str() {
                                return Ok(sig_str.to_string());
                            }
                        }
                    }
                }

                // Check for transaction_signature directly in the tool result (backward compatibility)
                if let Some(sig) = result_item.get("transaction_signature") {
                    if let Some(sig_str) = sig.as_str() {
                        return Ok(sig_str.to_string());
                    }
                }

                // Check for signatures in various tool-specific formats
                if let Some(signature) = extract_signature_from_tool_result(result_item) {
                    return Ok(signature);
                }
            }
        }

        // Check for signature in output object (older format)
        if let Some(tool_results) = step_result.output.get("tool_results") {
            if let Some(results_array) = tool_results.as_array() {
                for result_item in results_array {
                    // Check for transaction_signature directly in the tool result
                    if let Some(sig) = result_item.get("transaction_signature") {
                        if let Some(sig_str) = sig.as_str() {
                            return Ok(sig_str.to_string());
                        }
                    }

                    // Check for signatures in various tool-specific formats
                    if let Some(signature) = extract_signature_from_tool_result(result_item) {
                        return Ok(signature);
                    }
                }
            }
        }

        // Check for signatures in output object directly
        if let Some(signature) = extract_signature_from_output(&step_result.output) {
            return Ok(signature);
        }

        // Check tool calls array
        for call in &step_result.tool_calls {
            if call.contains("transaction_signature") {
                // Extract signature from JSON string
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(call) {
                    if let Some(sig) = json.get("transaction_signature") {
                        if let Some(sig_str) = sig.as_str() {
                            return Ok(sig_str.to_string());
                        }
                    }
                }
            }
        }
    }

    Err(anyhow!("No transaction signature found in result"))
}

/// Extracts signature from a tool-specific result
///
/// This function handles various tool-specific formats for transaction signatures.
///
/// # Arguments
/// * `tool_result` - The tool result from which to extract the signature
///
/// # Returns
/// An Option containing the transaction signature string if found
fn extract_signature_from_tool_result(tool_result: &Value) -> Option<String> {
    // Handle tagged enum serialization - check for tool_name field
    if tool_result.get("tool_name").is_some() {
        // Check for transaction_signature directly in the enum variant
        if let Some(sig) = tool_result.get("transaction_signature") {
            if let Some(sig_str) = sig.as_str() {
                return Some(sig_str.to_string());
            }
        }
    }

    // Handle nested object format for backward compatibility
    if let Some(sol_transfer) = tool_result.get("sol_transfer") {
        if let Some(sig) = sol_transfer.get("transaction_signature") {
            if let Some(sig_str) = sig.as_str() {
                return Some(sig_str.to_string());
            }
        }
    }

    // Check for Jupiter swap tool signatures
    if let Some(jupiter_swap) = tool_result.get("jupiter_swap") {
        if let Some(sig) = jupiter_swap.get("transaction_signature") {
            if let Some(sig_str) = sig.as_str() {
                return Some(sig_str.to_string());
            }
        }
    }

    // Check for Jupiter lend tool signatures
    if let Some(jupiter_lend) = tool_result.get("jupiter_lend_earn_deposit") {
        if let Some(sig) = jupiter_lend.get("transaction_signature") {
            if let Some(sig_str) = sig.as_str() {
                return Some(sig_str.to_string());
            }
        }
    }

    None
}

/// Extracts signature from the output object
///
/// This function checks for signatures in various locations in the output object.
///
/// # Arguments
/// * `output` - The output object from which to extract the signature
///
/// # Returns
/// An Option containing the transaction signature string if found
fn extract_signature_from_output(output: &Value) -> Option<String> {
    // Check for transaction_signature directly in output
    if let Some(sig) = output.get("transaction_signature") {
        if let Some(sig_str) = sig.as_str() {
            return Some(sig_str.to_string());
        }
    }

    // Handle tagged enum serialization - check for tool_name field
    if output.get("tool_name").is_some() {
        // Check for transaction_signature directly in the enum variant
        if let Some(sig) = output.get("transaction_signature") {
            if let Some(sig_str) = sig.as_str() {
                return Some(sig_str.to_string());
            }
        }
    }

    // Check for transfer tool signatures
    if let Some(sol_transfer) = output.get("sol_transfer") {
        if let Some(sig) = sol_transfer.get("transaction_signature") {
            if let Some(sig_str) = sig.as_str() {
                return Some(sig_str.to_string());
            }
        }
    }

    // Check for Jupiter swap tool signatures
    if let Some(jupiter_swap) = output.get("jupiter_swap") {
        if let Some(sig) = jupiter_swap.get("transaction_signature") {
            if let Some(sig_str) = sig.as_str() {
                return Some(sig_str.to_string());
            }
        }
    }

    // Check for Jupiter lend tool signatures
    if let Some(jupiter_lend) = output.get("jupiter_lend_earn_deposit") {
        if let Some(sig) = jupiter_lend.get("transaction_signature") {
            if let Some(sig_str) = sig.as_str() {
                return Some(sig_str.to_string());
            }
        }
    }

    None
}

/// Extracts tool results from execution result
///
/// This function extracts all tool results from an execution result,
/// providing a unified interface for processing tool outputs.
///
/// # Arguments
/// * `result` - The execution result from which to extract tool results
///
/// # Returns
/// A vector of tool results as serde_json::Value
pub fn extract_tool_results(result: &FlowResult) -> Vec<Value> {
    let mut tool_results = Vec::new();

    for step_result in &result.step_results {
        // Check for tool_results array (RigAgent format)
        if let Some(results) = step_result.output.get("tool_results") {
            if let Some(results_array) = results.as_array() {
                for result_item in results_array {
                    tool_results.push(result_item.clone());
                }
            }
        }

        // Also include any tool-specific outputs
        if step_result.output.get("sol_transfer").is_some()
            || step_result.output.get("jupiter_swap").is_some()
            || step_result
                .output
                .get("jupiter_lend_earn_deposit")
                .is_some()
        {
            tool_results.push(step_result.output.clone());
        }
    }

    tool_results
}

/// Extracts error messages from execution result
///
/// This function extracts error messages from an execution result,
/// providing a unified interface for error handling.
///
/// # Arguments
/// * `result` - The execution result from which to extract error messages
///
/// # Returns
/// A vector of error message strings
pub fn extract_error_messages(result: &FlowResult) -> Vec<String> {
    let mut error_messages = Vec::new();

    for step_result in &result.step_results {
        // Check for errors in tool_results array (RigAgent format)
        if let Some(tool_results) = step_result.output.get("tool_results") {
            if let Some(results_array) = tool_results.as_array() {
                for result_item in results_array {
                    // Check for errors in various tool formats
                    if let Some(error) = extract_error_from_tool_result(result_item) {
                        error_messages.push(error);
                    }
                }
            }
        }

        // Check for errors in output object directly
        if let Some(error) = extract_error_from_output(&step_result.output) {
            error_messages.push(error);
        }

        // Check for errors in step_result directly
        if let Some(error) = step_result.output.get("error") {
            if let Some(error_str) = error.as_str() {
                error_messages.push(error_str.to_string());
            }
        }
    }

    error_messages
}

/// Extracts error message from a tool-specific result
///
/// This function handles various tool-specific formats for error messages.
///
/// # Arguments
/// * `tool_result` - The tool result from which to extract the error
///
/// # Returns
/// An Option containing the error message string if found
fn extract_error_from_tool_result(tool_result: &Value) -> Option<String> {
    // Check for transfer tool errors
    if let Some(sol_transfer) = tool_result.get("sol_transfer") {
        if let Some(error) = sol_transfer.get("error") {
            if let Some(error_str) = error.as_str() {
                return Some(format!("Transfer error: {error_str}"));
            }
        }
    }

    // Check for Jupiter swap tool errors
    if let Some(jupiter_swap) = tool_result.get("jupiter_swap") {
        if let Some(error) = jupiter_swap.get("error") {
            if let Some(error_str) = error.as_str() {
                return Some(format!("Jupiter swap error: {error_str}"));
            }
        }
    }

    // Check for Jupiter lend tool errors
    if let Some(jupiter_lend) = tool_result.get("jupiter_lend_earn_deposit") {
        if let Some(error) = jupiter_lend.get("error") {
            if let Some(error_str) = error.as_str() {
                return Some(format!("Jupiter lend error: {error_str}"));
            }
        }
    }

    None
}

/// Extracts error message from the output object
///
/// This function checks for errors in various locations in the output object.
///
/// # Arguments
/// * `output` - The output object from which to extract the error
///
/// # Returns
/// An Option containing the error message string if found
fn extract_error_from_output(output: &Value) -> Option<String> {
    // Check for error directly in output
    if let Some(error) = output.get("error") {
        if let Some(error_str) = error.as_str() {
            return Some(error_str.to_string());
        }
    }

    // Check for transfer tool errors
    if let Some(sol_transfer) = output.get("sol_transfer") {
        if let Some(error) = sol_transfer.get("error") {
            if let Some(error_str) = error.as_str() {
                return Some(format!("Transfer error: {error_str}"));
            }
        }
    }

    // Check for Jupiter swap tool errors
    if let Some(jupiter_swap) = output.get("jupiter_swap") {
        if let Some(error) = jupiter_swap.get("error") {
            if let Some(error_str) = error.as_str() {
                return Some(format!("Jupiter swap error: {error_str}"));
            }
        }
    }

    // Check for Jupiter lend tool errors
    if let Some(jupiter_lend) = output.get("jupiter_lend_earn_deposit") {
        if let Some(error) = jupiter_lend.get("error") {
            if let Some(error_str) = error.as_str() {
                return Some(format!("Jupiter lend error: {error_str}"));
            }
        }
    }

    None
}

/// Extracts metadata from execution result
///
/// This function extracts metadata from an execution result,
/// providing additional context about the execution.
///
/// # Arguments
/// * `result` - The execution result from which to extract metadata
///
/// # Returns
/// A HashMap containing metadata key-value pairs
pub fn extract_metadata(result: &FlowResult) -> HashMap<String, String> {
    let mut metadata = HashMap::new();

    // Add step count
    metadata.insert(
        "step_count".to_string(),
        result.step_results.len().to_string(),
    );

    // Add tool names used
    let mut tool_names = Vec::new();
    for step_result in &result.step_results {
        for call in &step_result.tool_calls {
            if call.contains("sol_transfer") {
                tool_names.push("sol_transfer".to_string());
            } else if call.contains("jupiter_swap") {
                tool_names.push("jupiter_swap".to_string());
            } else if call.contains("jupiter_lend_earn_deposit") {
                tool_names.push("jupiter_lend_earn_deposit".to_string());
            }
        }
    }
    metadata.insert("tool_names".to_string(), tool_names.join(","));

    metadata
}

/// Checks if execution result indicates success
///
/// This function determines if an execution result indicates success
/// based on the presence of transaction signatures and absence of errors.
///
/// # Arguments
/// * `result` - The execution result to check
///
/// # Returns
/// A boolean indicating if the execution was successful
pub fn is_execution_successful(result: &FlowResult) -> bool {
    // Check if there are any error messages
    let error_messages = extract_error_messages(result);
    if !error_messages.is_empty() {
        return false;
    }

    // Check if there's at least one transaction signature
    extract_transaction_signature(result).is_ok()
}
