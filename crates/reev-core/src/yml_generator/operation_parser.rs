//! DEPRECATED: Operation Parser for Dynamic Operation Sequences
//!
//! This module is deprecated in V3 architecture. Tool selection should be handled
//! by RigAgent, not rule-based parsing. Keeping this file only for
//! backward compatibility with existing tests that will be removed.

use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;

/// Operation types that can be parsed from prompts
#[derive(Debug, Clone)]
pub enum Operation {
    Swap {
        from: String,
        to: String,
        amount: f64,
    },
    Lend {
        mint: String,
        amount: f64,
    },
    Transfer {
        mint: String,
        to: String,
        amount: f64,
    },
}

/// Template types for common operation patterns
#[derive(Debug, Clone)]
pub enum FlowTemplate {
    SingleOperation,
    MultiOperation,
    SwapThenLend,
    Custom(String),
}

/// Parser for extracting operations from refined prompts
pub struct OperationParser {
    /// Token mappings for symbol to mint address
    token_mappings: HashMap<String, String>,
}

impl Default for OperationParser {
    fn default() -> Self {
        Self::new()
    }
}

impl OperationParser {
    /// Create a new operation parser with default token mappings
    pub fn new() -> Self {
        let mut token_mappings = HashMap::new();

        // Add common token mappings
        token_mappings.insert(
            "SOL".to_string(),
            "So11111111111111111111111111111111111111112".to_string(),
        );
        token_mappings.insert(
            "USDC".to_string(),
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        );
        token_mappings.insert(
            "USDT".to_string(),
            "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".to_string(),
        );

        Self { token_mappings }
    }

    /// Create an operation parser with custom token mappings
    pub fn with_token_mappings(token_mappings: HashMap<String, String>) -> Self {
        Self { token_mappings }
    }

    /// Parse a refined prompt into a sequence of operations
    pub fn parse_operations(&self, refined_prompt: &str) -> Result<Vec<Operation>> {
        let prompt_lower = refined_prompt.to_lowercase();

        // First determine if this is a multi-operation prompt
        if self.is_multi_operation(&prompt_lower) {
            return self.parse_multi_operation(refined_prompt);
        }

        // Parse single operation
        self.parse_single_operation(refined_prompt)
    }

    /// Determine if a prompt describes multiple operations
    fn is_multi_operation(&self, prompt_lower: &str) -> bool {
        // Check for sequencing keywords
        prompt_lower.contains(" then ")
            || prompt_lower.contains(" and ")
            || prompt_lower.contains(" followed by ")
            || prompt_lower.contains(" next ")
            || prompt_lower.contains(" after ")
            || prompt_lower.contains(" swap then lend ")
            || prompt_lower.contains(" lend then swap ")
    }

    /// Parse a single operation from a prompt
    fn parse_single_operation(&self, refined_prompt: &str) -> Result<Vec<Operation>> {
        let prompt_lower = refined_prompt.to_lowercase();

        // Check for swap operations first - highest priority to avoid misinterpretation
        if prompt_lower.contains("swap")
            || prompt_lower.contains("exchange")
            || prompt_lower.contains("sell")
        {
            let swap_op = self.parse_swap_operation(refined_prompt)?;
            return Ok(vec![swap_op]);
        }

        // Check for lend operations
        if prompt_lower.contains("lend") || prompt_lower.contains("deposit") {
            let lend_op = self.parse_lend_operation(refined_prompt)?;
            return Ok(vec![lend_op]);
        }

        // Check for transfer operations - lowest priority to avoid overriding swap/lend
        if prompt_lower.contains("transfer") || prompt_lower.contains("send") {
            let transfer_op = self.parse_transfer_operation(refined_prompt)?;
            return Ok(vec![transfer_op]);
        }

        Err(anyhow::anyhow!(
            "Unable to parse operation from prompt: {refined_prompt}"
        ))
    }

    /// Parse a multi-operation prompt into a sequence of operations
    fn parse_multi_operation(&self, refined_prompt: &str) -> Result<Vec<Operation>> {
        let prompt_lower = refined_prompt.to_lowercase();

        // Check for common patterns first
        if prompt_lower.contains("swap then lend") || prompt_lower.contains("swap and lend") {
            return self.parse_swap_then_lend(refined_prompt);
        }

        // Generic multi-operation parsing
        // Split by sequencing keywords and parse each part
        let parts = self.split_multi_operation(refined_prompt);
        let mut operations = Vec::new();

        for part in parts {
            let part_operations = self.parse_single_operation(part.trim())?;
            operations.extend(part_operations);
        }

        Ok(operations)
    }

    /// Extract amount from prompt with special handling for "all"
    fn extract_amount_from_prompt(&self, prompt: &str) -> f64 {
        // Check if prompt contains "all"
        if prompt.to_lowercase().contains("all") {
            return 0.0; // Special value for "all"
        }

        // Try to extract specific amount
        if let Some(percentage) = self.extract_percentage(prompt) {
            return percentage;
        }

        // Look for specific amount
        let amount_regex = Regex::new(r"(\d+\.?\d*)\s*(sol|usdc|usdt|eth|btc)?").unwrap();
        if let Some(captures) = amount_regex.captures(prompt) {
            if let Ok(val) = captures[1].parse::<f64>() {
                return val;
            }
        }
        // Default amount
        1.0
    }

    /// Parse a swap operation from a prompt
    fn parse_swap_operation(&self, refined_prompt: &str) -> Result<Operation> {
        // Extract tokens
        let from = self.extract_from_token(refined_prompt)?;
        let to = self.extract_to_token(refined_prompt)?;
        let amount = self.extract_amount(refined_prompt, &from)?;

        Ok(Operation::Swap { from, to, amount })
    }

    /// Parse a transfer operation from a prompt
    fn parse_transfer_operation(&self, refined_prompt: &str) -> Result<Operation> {
        // Check if this might actually be a swap operation that was misinterpreted
        let prompt_lower = refined_prompt.to_lowercase();

        // If it mentions two different tokens with "for" or "to", it's likely a swap
        // rather than a transfer (which would have a recipient address)
        let has_token_pair = (prompt_lower.contains("for ") || prompt_lower.contains(" to "))
            && self.contains_multiple_tokens(refined_prompt);

        if has_token_pair {
            // This looks more like a swap than a transfer
            return self.parse_swap_operation(refined_prompt);
        }

        // Extract recipient - transfer operations must have a recipient
        let recipient = self.extract_recipient(refined_prompt)?;

        // Extract token (default to SOL)
        let mint = self
            .extract_from_token(refined_prompt)
            .unwrap_or_else(|_| "SOL".to_string());

        // Extract amount
        let amount = self.extract_amount_from_prompt(refined_prompt);
        Ok(Operation::Transfer {
            mint,
            to: recipient,
            amount,
        })
    }

    /// Parse a lend operation from a prompt
    fn parse_lend_operation(&self, refined_prompt: &str) -> Result<Operation> {
        // Extract token (default to USDC)
        let mint = self
            .extract_from_token(refined_prompt)
            .unwrap_or_else(|_| "USDC".to_string());

        // Extract amount
        let amount = self.extract_amount_from_prompt(refined_prompt);

        Ok(Operation::Lend { mint, amount })
    }

    /// Parse a swap then lend operation from a prompt
    fn parse_swap_then_lend(&self, refined_prompt: &str) -> Result<Vec<Operation>> {
        // Parse the swap part
        let swap_op = self.parse_swap_operation(refined_prompt)?;

        // Extract the "to" token from the swap to use for lending
        let to_token = match &swap_op {
            Operation::Swap { to, .. } => to.clone(),
            _ => "USDC".to_string(), // Default fallback
        };

        // Create lend operation with the swapped token
        let lend_op = Operation::Lend {
            mint: to_token,
            amount: match &swap_op {
                Operation::Swap { amount, .. } => *amount,
                _ => 100.0, // Default fallback
            },
        };

        Ok(vec![swap_op, lend_op])
    }

    /// Split a multi-operation prompt into individual operation parts
    fn split_multi_operation<'a>(&self, refined_prompt: &'a str) -> Vec<&'a str> {
        // Try to split by common sequencing keywords
        let mut parts = Vec::new();

        // Check for "then" pattern
        if let Some(index) = refined_prompt.to_lowercase().find(" then ") {
            parts.push(&refined_prompt[..index]);
            parts.push(&refined_prompt[index + 6..]);
            return parts;
        }

        // Check for "and" pattern
        if let Some(index) = refined_prompt.to_lowercase().find(" and ") {
            parts.push(&refined_prompt[..index]);
            parts.push(&refined_prompt[index + 5..]);
            return parts;
        }

        // Default to the whole prompt
        vec![refined_prompt]
    }

    /// Extract "from" token from a prompt
    fn extract_from_token(&self, refined_prompt: &str) -> Result<String> {
        let prompt_lower = refined_prompt.to_lowercase();

        // Check for "sell" pattern first
        if prompt_lower.contains("sell") {
            // Look for pattern "sell X TOKEN" or "sell all TOKEN"
            let sell_regex = Regex::new(r"sell(?:\s+all)?\s+(?:\d+\.?\d*\s+)?(\w+)").unwrap();
            if let Some(captures) = sell_regex.captures(refined_prompt) {
                if let Some(token_match) = captures.get(1) {
                    let token = token_match.as_str().to_uppercase();
                    if self.token_mappings.contains_key(&token) {
                        return Ok(token);
                    }
                }
            }
        }

        // Look for pattern "TOKEN to" or "TOKEN for" to identify the source token
        for token in self.token_mappings.keys() {
            let token_lower = token.to_lowercase();
            if prompt_lower.contains(&format!("{} to", token_lower))
                || prompt_lower.contains(&format!("{} for", token_lower))
                || prompt_lower.contains(&format!("swap {}", token_lower))
            {
                return Ok(token.clone());
            }
        }

        // Default to SOL if no token found
        Ok("SOL".to_string())
    }

    /// Extract "to" token from a prompt
    fn extract_to_token(&self, refined_prompt: &str) -> Result<String> {
        let prompt_lower = refined_prompt.to_lowercase();

        // Check for "sell" pattern first
        if prompt_lower.contains("sell") {
            // Look for pattern "sell X for TOKEN" to identify destination token
            let sell_regex =
                Regex::new(r"sell\s+(?:\d+\.?\d*\s+)?(?:\w+)\s+(?:all\s+)?(?:for|to)\s+(\w+)")
                    .unwrap();
            if let Some(captures) = sell_regex.captures(refined_prompt) {
                if let Some(token_match) = captures.get(1) {
                    let token = token_match.as_str().to_uppercase();
                    if self.token_mappings.contains_key(&token) {
                        return Ok(token);
                    }
                }
            }
        }

        // Check for "swap then lend" pattern
        if prompt_lower.contains("swap then lend") || prompt_lower.contains("swap and lend") {
            // Find the "to" token in the swap part
            let swap_to_regex = Regex::new(r"swap\s+\d+\.?\d*\s+\w+\s+(?:to|for)\s+(\w+)").unwrap();
            if let Some(captures) = swap_to_regex.captures(refined_prompt) {
                if let Some(token_match) = captures.get(1) {
                    let token = token_match.as_str().to_uppercase();
                    if self.token_mappings.contains_key(&token) {
                        return Ok(token);
                    }
                }
            }
        }

        // Try to extract "to" token
        for token in self.token_mappings.keys() {
            let token_lower = token.to_lowercase();
            if prompt_lower.contains(&format!(" to {}", token_lower))
                || prompt_lower.contains(&format!(" for {}", token_lower))
            {
                return Ok(token.clone());
            }
        }

        // Default to USDC if no token found
        Ok("USDC".to_string())
    }

    /// Extract the recipient from a transfer prompt
    fn extract_recipient(&self, refined_prompt: &str) -> Result<String> {
        // Try to extract Solana public key
        let pubkey_regex = Regex::new(r"([A-HJ-NP-Za-km-z1-9]{32,44})").unwrap();
        if let Some(captures) = pubkey_regex.captures(refined_prompt) {
            return Ok(captures[1].to_string());
        }

        Err(anyhow::anyhow!(
            "No recipient found in prompt: {refined_prompt}"
        ))
    }

    /// Extract amount from a prompt, with context of the token being used
    fn extract_amount(&self, refined_prompt: &str, _token: &str) -> Result<f64> {
        let prompt_lower = refined_prompt.to_lowercase();

        // Check for "all" for SOL operations
        if prompt_lower.contains("all") && prompt_lower.contains("sol") {
            return Ok(0.0); // Special value for "all" to be handled by step builders
        }

        // Try to extract percentage
        if let Some(percentage) = self.extract_percentage(refined_prompt) {
            return Ok(percentage);
        }

        // Look for specific amount
        let amount_regex = Regex::new(r"(\d+\.?\d*)\s*(sol|usdc|usdt|eth|btc)?").unwrap();
        if let Some(captures) = amount_regex.captures(refined_prompt) {
            if let Ok(val) = captures[1].parse::<f64>() {
                return Ok(val);
            }
        }

        // Default amount
        Ok(1.0)
    }

    /// Extract percentage from prompt
    fn extract_percentage(&self, refined_prompt: &str) -> Option<f64> {
        let regex = Regex::new(r"(\d+\.?\d*)%").unwrap();
        regex
            .captures(refined_prompt)
            .and_then(|captures| captures[1].parse::<f64>().ok())
    }

    /// Check if a prompt contains multiple different token symbols
    fn contains_multiple_tokens(&self, refined_prompt: &str) -> bool {
        let prompt_lower = refined_prompt.to_lowercase();
        let mut token_count = 0;

        // Check for common token symbols in the prompt
        for token in self.token_mappings.keys() {
            if prompt_lower.contains(&token.to_lowercase()) {
                token_count += 1;
                if token_count >= 2 {
                    return true;
                }
            }
        }

        false
    }

    /// Determine the flow template to use for the operations
    pub fn determine_flow_template(&self, operations: &[Operation]) -> FlowTemplate {
        match operations.len() {
            1 => FlowTemplate::SingleOperation,
            2 if matches!(&operations[0], Operation::Swap { .. })
                && matches!(&operations[1], Operation::Lend { .. }) =>
            {
                FlowTemplate::SwapThenLend
            }
            2..=5 => FlowTemplate::MultiOperation,
            _ => FlowTemplate::Custom("complex_multi_operation".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests removed - OperationParser is deprecated in V3 architecture
    // Tool selection should be handled by RigAgent, not rule-based parsing
}
