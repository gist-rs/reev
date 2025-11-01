//! Deterministic Agent Response Parser
//!
//! Handles parsing of deterministic agent responses which have a specific format:
//! {result: {text: Vec<RawInstruction>}, transactions: null}
//!
//! This is separate from general parsing to maintain clean architecture
//! and avoid mixing different response format handling logic.

use anyhow::Result;
use serde_json::Value;
use tracing::{debug, info, warn};

use crate::agent::LlmResponse;

/// Parser specifically for deterministic agent responses
pub struct DeterministicParser;

impl DeterministicParser {
    /// Parse deterministic agent response format
    ///
    /// Deterministic agent returns:
    /// {
    ///   "result": {"text": Vec<RawInstruction>},
    ///   "transactions": null,
    ///   "summary": null,
    ///   ...
    /// }
    pub fn parse_response(response_text: &str) -> Result<Option<LlmResponse>> {
        info!("[DeterministicParser] Parsing deterministic agent response");
        debug!("[DeterministicParser] Response text: {}", response_text);

        // Try to parse as LlmResponse first
        debug!("[DeterministicParser] Attempting to parse response as LlmResponse");
        if let Ok(response) = serde_json::from_str::<LlmResponse>(response_text) {
            debug!("[DeterministicParser] Successfully parsed LlmResponse: transactions={:?}, result={:?}",
                response.transactions.is_some(), response.result.is_some());
            // Check if this is a deterministic agent response
            if response.transactions.is_none() {
                if let Some(result) = &response.result {
                    // result.text should contain Vec<RawInstruction> after deserialization
                    debug!(
                        "[DeterministicParser] Result text length: {}",
                        result.text.len()
                    );
                    if !result.text.is_empty() {
                        info!(
                            "[DeterministicParser] Successfully extracted {} transactions from result.text",
                            result.text.len()
                        );
                        return Ok(Some(LlmResponse {
                            transactions: Some(result.text.clone()),
                            result: response.result,
                            summary: response.summary,
                            signatures: response.signatures,
                            flows: response.flows,
                        }));
                    } else {
                        warn!("[DeterministicParser] Result text is empty, no transactions to extract");
                    }
                } else {
                    warn!("[DeterministicParser] Response has no result field");
                }
            } else {
                debug!("[DeterministicParser] Response already has transactions, not deterministic format");
            }

            // Return original response if no deterministic pattern found
            info!(
                "[DeterministicParser] No deterministic pattern found, returning original response"
            );
            return Ok(Some(response));
        }

        // Parse failed completely
        warn!(
            "[DeterministicParser] Failed to parse response as valid JSON: {}",
            response_text
        );
        Ok(None)
    }

    /// Check if response appears to be from deterministic agent
    ///
    /// Quick heuristic to identify deterministic responses before full parsing
    pub fn is_deterministic_response(response_text: &str) -> bool {
        if let Ok(value) = serde_json::from_str::<Value>(response_text) {
            // Deterministic responses typically have:
            // - transactions: null (or empty)
            // - result.text field with instructions
            let has_null_transactions = value
                .get("transactions")
                .and_then(|t| t.as_null())
                .is_some();

            let has_result_text = value.get("result").and_then(|r| r.get("text")).is_some();

            has_null_transactions && has_result_text
        } else {
            false
        }
    }
}
