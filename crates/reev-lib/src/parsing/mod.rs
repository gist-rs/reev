//! Response parsing module for handling different LLM response formats
//!
//! This module provides a unified interface for parsing various response formats:
//! - GLM-style responses (both local agent and GLM Coding formats)
//! - Jupiter-style responses with complex transaction structures
//! - Standard reev API responses
//!
//! Each format has its own parser function, and a fallback mechanism ensures
//! compatibility across different agent types and response structures.

use anyhow::{Context, Result};
use serde_json::Value;
use tracing::{debug, info, warn};

use crate::agent::{LlmResponse, RawInstruction};

pub mod deterministic_parser;
pub use deterministic_parser::DeterministicParser;

/// Main response parser with fallback mechanism
///
/// Attempts to parse responses in order: GLM -> Jupiter -> Standard reev format
/// Returns the first successfully parsed response or creates a fallback response
pub struct ResponseParser {
    pub is_glm: bool,
}

impl ResponseParser {
    /// Create a new response parser
    pub fn new(is_glm: bool) -> Self {
        Self { is_glm }
    }

    /// Parse LLM response with fallback mechanism: GLM -> Jupiter -> Deterministic -> Standard
    pub fn parse_with_fallback(&self, response_text: &str) -> LlmResponse {
        if self.is_glm {
            info!("[ResponseParser] Attempting GLM-style response parsing");
            if let Some(response) = self.parse_glm_response(response_text) {
                return response;
            }
        }

        info!("[ResponseParser] Attempting Jupiter-style response parsing");
        if let Some(response) = self.parse_jupiter_response(response_text) {
            return response;
        }

        info!("[ResponseParser] Attempting deterministic agent response parsing");
        if let Some(response) = DeterministicParser::parse_response(response_text).unwrap_or(None) {
            return response;
        }

        info!("[ResponseParser] Attempting standard reev API response parsing");
        self.parse_standard_reev_response(response_text)
            .unwrap_or_else(|e| {
                warn!(
                    "[ResponseParser] All parsing methods failed, creating fallback response: {}",
                    e
                );
                LlmResponse {
                    transactions: None,
                    result: None,
                    summary: Some(format!("Response parsing failed: {e}")),
                    signatures: None,
                    flows: None,
                }
            })
    }

    /// Parse GLM-style responses (both local agent format and GLM Coding format)
    fn parse_glm_response(&self, response_text: &str) -> Option<LlmResponse> {
        if let Ok(glm_response) = serde_json::from_str::<Value>(response_text) {
            // First check if transactions are at the root level (local agent format)
            if let Some(root_transactions) =
                glm_response.get("transactions").and_then(|t| t.as_array())
            {
                info!(
                    "[ResponseParser] Processing transactions at root level (local agent format)"
                );
                return Some(self.parse_transaction_array(&glm_response, root_transactions));
            } else if let Some(result) = glm_response.get("result") {
                info!("[ResponseParser] Processing GLM Coding response with result field");
                return self.parse_glm_coding_response(&glm_response, result);
            }
        }
        None
    }

    /// Parse GLM Coding response with nested structure
    fn parse_glm_coding_response(
        &self,
        glm_response: &Value,
        result: &Value,
    ) -> Option<LlmResponse> {
        // Check if result has a "text" field (nested response)
        if let Some(text_field) = result.get("text").and_then(|t| t.as_str()) {
            info!("[ResponseParser] Found nested GLM Coding response in text field");

            // Parse the text field as JSON to get the inner result
            if let Ok(inner_response) = serde_json::from_str::<Value>(text_field) {
                if let Some(inner_result) = inner_response.get("result") {
                    info!("[ResponseParser] Processing inner GLM Coding result");

                    // Extract transactions from inner result.transactions
                    let transactions = inner_result
                        .get("transactions")
                        .and_then(|t| t.as_array())
                        .map(|txs| {
                            txs.iter()
                                .filter_map(|tx| {
                                    // Handle nested arrays from ZAI agent: [{"accounts":...}] vs direct objects
                                    let instruction_to_parse = if tx.is_array() {
                                        // ZAI agent returns nested arrays, extract the first element
                                        tx.as_array()
                                            .and_then(|arr| arr.first())
                                            .unwrap_or(tx)
                                    } else {
                                        tx
                                    };

                                    match serde_json::from_value::<RawInstruction>(
                                        instruction_to_parse.clone(),
                                    ) {
                                        Ok(instruction) => {
                                            info!(
                                                "[ResponseParser] GLM Coding - Successfully parsed RawInstruction: {:?}",
                                                instruction
                                            );
                                            Some(instruction)
                                        }
                                        Err(e) => {
                                            warn!(
                                                "[ResponseParser] GLM Coding - Failed to parse instruction: {}. Data: {:?}",
                                                e, instruction_to_parse
                                            );
                                            None
                                        }
                                    }
                                })
                                .collect::<Vec<RawInstruction>>()
                        });

                    let summary = inner_response
                        .get("summary")
                        .and_then(|s| s.as_str())
                        .map(|s| s.to_string());

                    let signatures = inner_response
                        .get("signatures")
                        .and_then(|s| s.as_array())
                        .map(|sigs| {
                            sigs.iter()
                                .filter_map(|sig| sig.as_str().map(|s| s.to_string()))
                                .collect()
                        });

                    info!(
                        "[ResponseParser] GLM Coding - Extracted {} transactions from nested response, summary: {}",
                        transactions.as_ref().map_or(0, |t| t.len()),
                        summary.as_deref().unwrap_or("none")
                    );

                    Some(LlmResponse {
                        transactions,
                        result: None,
                        summary,
                        signatures,
                        flows: None,
                    })
                } else {
                    warn!("[ResponseParser] GLM Coding inner response has no result field");
                    Some(LlmResponse {
                        transactions: None,
                        result: None,
                        summary: Some(text_field.to_string()),
                        signatures: None,
                        flows: None,
                    })
                }
            } else {
                warn!("[ResponseParser] GLM Coding text field is not valid JSON, using as summary");
                Some(LlmResponse {
                    transactions: None,
                    result: None,
                    summary: Some(text_field.to_string()),
                    signatures: None,
                    flows: None,
                })
            }
        } else {
            // Regular GLM response with result field
            match serde_json::from_value::<LlmResponse>(glm_response.clone()) {
                Ok(response) => {
                    info!("[ResponseParser] Successfully parsed GLM Coding response");
                    Some(response)
                }
                Err(e) => {
                    warn!(
                        "[ResponseParser] Failed to parse GLM Coding response: {}",
                        e
                    );
                    warn!("[ResponseParser] GLM response: {}", glm_response);
                    Some(LlmResponse {
                        transactions: None,
                        result: None,
                        summary: Some(format!("GLM response parsing failed: {e}")),
                        signatures: None,
                        flows: None,
                    })
                }
            }
        }
    }

    /// Parse Jupiter-style responses with complex transaction structure
    fn parse_jupiter_response(&self, response_text: &str) -> Option<LlmResponse> {
        if let Ok(reev_response) = serde_json::from_str::<Value>(response_text) {
            // Check if this is a Jupiter-style response with complex transaction structure
            if let Some(transactions) = reev_response.get("transactions").and_then(|t| t.as_array())
            {
                info!(
                    "[ResponseParser] Processing Jupiter-style response with {} transactions",
                    transactions.len()
                );

                let parsed_transactions = Some(
                    transactions
                        .iter()
                        .flat_map(|tx| {
                            // First try to extract instructions array (Jupiter format)
                            if let Some(instructions) = tx.get("instructions").and_then(|i| i.as_array()) {
                                instructions
                                    .iter()
                                    .filter_map(|instruction| {
                                        debug!("[ResponseParser] Attempting to parse Jupiter instruction: {:?}", instruction);
                                        match serde_json::from_value::<RawInstruction>(
                                            instruction.clone(),
                                        ) {
                                            Ok(raw_instruction) => {
                                                debug!(
                                                    "[ResponseParser] Successfully parsed Jupiter RawInstruction with program_id: {}",
                                                    raw_instruction.program_id
                                                );
                                                Some(raw_instruction)
                                            }
                                            Err(e) => {
                                                warn!(
                                                    "[ResponseParser] Failed to parse Jupiter RawInstruction: {}. Instruction: {}",
                                                    e, instruction
                                                );
                                                None
                                            }
                                        }
                                    })
                                    .collect::<Vec<RawInstruction>>()
                            } else if let Some(tx_array) = tx.as_array() {
                                // Handle GLM double-nested format: transactions[[{...}]]
                                debug!("[ResponseParser] Transaction is array, trying GLM double-nested format");
                                tx_array
                                    .iter()
                                    .filter_map(|inner_tx| {
                                        match serde_json::from_value::<RawInstruction>(inner_tx.clone()) {
                                            Ok(raw_instruction) => {
                                                debug!(
                                                    "[ResponseParser] Successfully parsed GLM nested RawInstruction with program_id: {}",
                                                    raw_instruction.program_id
                                                );
                                                Some(raw_instruction)
                                            }
                                            Err(e) => {
                                                warn!(
                                                    "[ResponseParser] Failed to parse GLM nested RawInstruction: {}. Transaction: {}",
                                                    e, inner_tx
                                                );
                                                None
                                            }
                                        }
                                    })
                                    .collect::<Vec<RawInstruction>>()
                            } else {
                                // Try to parse transaction as direct instruction (simple format)
                                debug!("[ResponseParser] Transaction has no instructions array, trying direct instruction format");
                                match serde_json::from_value::<RawInstruction>(tx.clone()) {
                                    Ok(raw_instruction) => {
                                        debug!(
                                            "[ResponseParser] Successfully parsed direct RawInstruction with program_id: {}",
                                            raw_instruction.program_id
                                        );
                                        vec![raw_instruction]
                                    }
                                    Err(e) => {
                                        warn!(
                                            "[ResponseParser] Failed to parse direct RawInstruction: {}. Transaction: {}",
                                            e, tx
                                        );
                                        vec![]
                                    }
                                }
                            }
                        })
                        .collect::<Vec<RawInstruction>>(),
                );

                let summary = reev_response
                    .get("summary")
                    .and_then(|s| s.as_str())
                    .map(|s| s.to_string());

                let signatures = reev_response
                    .get("signatures")
                    .and_then(|s| s.as_array())
                    .map(|sigs| {
                        sigs.iter()
                            .filter_map(|sig| sig.as_str().map(|s| s.to_string()))
                            .collect()
                    });

                info!(
                    "[ResponseParser] Jupiter-style parsed {} transactions, summary: {}",
                    parsed_transactions.as_ref().map_or(0, |t| t.len()),
                    summary.as_deref().unwrap_or("none")
                );

                return Some(LlmResponse {
                    transactions: parsed_transactions,
                    result: None,
                    summary,
                    signatures,
                    flows: None,
                });
            }
        }
        None
    }

    /// Parse standard reev API responses (non-deterministic, non-Jupiter, non-GLM)
    fn parse_standard_reev_response(&self, response_text: &str) -> Result<LlmResponse> {
        serde_json::from_str(response_text).context("Failed to deserialize LLM API response")
    }

    /// Parse transaction array from response (shared by GLM and Jupiter parsers)
    fn parse_transaction_array(&self, response: &Value, transactions: &[Value]) -> LlmResponse {
        debug!(
            "[ResponseParser] Raw transactions array: {:?}",
            transactions
        );

        let parsed_transactions = Some(
            transactions
                .iter()
                .flat_map(|tx| {
                    debug!("[ResponseParser] Processing transaction: {:?}", tx);

                    // First try to extract instructions array (Jupiter format)
                    if let Some(instructions) = tx.get("instructions").and_then(|i| i.as_array()) {
                        instructions
                            .iter()
                            .filter_map(|instruction| {
                                debug!("[ResponseParser] Attempting to parse nested instruction: {:?}", instruction);
                                match serde_json::from_value::<RawInstruction>(
                                    instruction.clone(),
                                ) {
                                    Ok(raw_instruction) => {
                                        debug!(
                                            "[ResponseParser] Successfully parsed nested RawInstruction with program_id: {}",
                                            raw_instruction.program_id
                                        );
                                        Some(raw_instruction)
                                    }
                                    Err(e) => {
                                        warn!(
                                            "[ResponseParser] Failed to parse nested RawInstruction: {}. Instruction: {}",
                                            e, instruction
                                        );
                                        None
                                    }
                                }
                            })
                            .collect::<Vec<RawInstruction>>()
                    } else if let Some(tx_array) = tx.as_array() {
                        // Handle GLM double-nested format: transactions[[{...}]]
                        debug!("[ResponseParser] Transaction is array, trying GLM double-nested format");
                        tx_array
                            .iter()
                            .filter_map(|inner_tx| {
                                match serde_json::from_value::<RawInstruction>(inner_tx.clone()) {
                                    Ok(raw_instruction) => {
                                        debug!(
                                            "[ResponseParser] Successfully parsed GLM nested RawInstruction with program_id: {}",
                                            raw_instruction.program_id
                                        );
                                        Some(raw_instruction)
                                    }
                                    Err(e) => {
                                        warn!(
                                            "[ResponseParser] Failed to parse GLM nested RawInstruction: {}. Transaction: {}",
                                            e, inner_tx
                                        );
                                        None
                                    }
                                }
                            })
                            .collect::<Vec<RawInstruction>>()
                    } else {
                        // Try to parse transaction as direct instruction (simple format)
                        debug!("[ResponseParser] Transaction has no instructions array, trying direct instruction format");
                        match serde_json::from_value::<RawInstruction>(tx.clone()) {
                            Ok(raw_instruction) => {
                                debug!(
                                    "[ResponseParser] Successfully parsed direct RawInstruction with program_id: {}",
                                    raw_instruction.program_id
                                );
                                vec![raw_instruction]
                            }
                            Err(e) => {
                                warn!(
                                    "[ResponseParser] Failed to parse direct RawInstruction: {}. Transaction: {}",
                                    e, tx
                                );
                                vec![]
                            }
                        }
                    }
                })
                .collect::<Vec<RawInstruction>>(),
        );

        info!(
            "[ResponseParser] Debug - Parsed {} RawInstructions",
            parsed_transactions.as_ref().map_or(0, |t| t.len())
        );

        let summary = response
            .get("summary")
            .and_then(|s| s.as_str())
            .map(|s| s.to_string());

        let signatures = response
            .get("signatures")
            .and_then(|s| s.as_array())
            .map(|sigs| {
                sigs.iter()
                    .filter_map(|sig| sig.as_str().map(|s| s.to_string()))
                    .collect()
            });

        let tx_count = parsed_transactions.as_ref().map_or(0, |t| t.len());
        if tx_count > 0 {
            info!(
                "[ResponseParser] Parsed {} transactions successfully: {}",
                tx_count,
                summary.as_deref().unwrap_or("no summary")
            );
        } else {
            debug!(
                "[ResponseParser] No transactions parsed, summary: {}",
                summary.as_deref().unwrap_or("none")
            );
        }

        LlmResponse {
            transactions: parsed_transactions,
            result: None,
            summary,
            signatures,
            flows: None,
        }
    }
}
