//! Validation logic for structured LLM responses

use crate::prompt_processor::types::{
    PromptAction, PromptParameters, StructuredRefinedPrompt, ValidationResult,
};

/// Validate a structured response against the original prompt
pub fn validate_structured_response(
    response: &StructuredRefinedPrompt,
    original_prompt: &str,
) -> ValidationResult {
    let mut issues = Vec::new();

    // Check if target_pubkey appears in original prompt
    if let Some(target) = &response.target_pubkey {
        if !original_prompt.contains(target) {
            issues.push(format!(
                "Extracted target_pubkey {target} not found in original prompt"
            ));
        }
    }

    // Check if action matches prompt intent
    match response.action {
        PromptAction::Transfer => {
            if !original_prompt.to_lowercase().contains("transfer")
                && !original_prompt.to_lowercase().contains("send")
            {
                issues.push("Action 'transfer' doesn't match prompt intent".to_string());
            }
        }
        PromptAction::Swap => {
            if !original_prompt.to_lowercase().contains("swap") {
                issues.push("Action 'swap' doesn't match prompt intent".to_string());
            }
        }
        PromptAction::Lend => {
            if !original_prompt.to_lowercase().contains("lend") {
                issues.push("Action 'lend' doesn't match prompt intent".to_string());
            }
        }
        PromptAction::Earn => {
            if !original_prompt.to_lowercase().contains("earn") {
                issues.push("Action 'earn' doesn't match prompt intent".to_string());
            }
        }
        PromptAction::Borrow => {
            if !original_prompt.to_lowercase().contains("borrow") {
                issues.push("Action 'borrow' doesn't match prompt intent".to_string());
            }
        }
        _ => {} // Unknown action - no validation
    }

    // Check if refined prompt contains action keyword
    match response.action {
        PromptAction::Transfer => {
            if !response.refined_prompt.to_lowercase().contains("transfer")
                && !response.refined_prompt.to_lowercase().contains("send")
            {
                issues.push("Refined prompt doesn't contain transfer action".to_string());
            }
        }
        PromptAction::Swap => {
            if !response.refined_prompt.to_lowercase().contains("swap") {
                issues.push("Refined prompt doesn't contain swap action".to_string());
            }
        }
        PromptAction::Lend => {
            if !response.refined_prompt.to_lowercase().contains("lend") {
                issues.push("Refined prompt doesn't contain lend action".to_string());
            }
        }
        PromptAction::Earn => {
            if !response.refined_prompt.to_lowercase().contains("earn") {
                issues.push("Refined prompt doesn't contain earn action".to_string());
            }
        }
        PromptAction::Borrow => {
            if !response.refined_prompt.to_lowercase().contains("borrow") {
                issues.push("Refined prompt doesn't contain borrow action".to_string());
            }
        }
        _ => {} // Unknown action - no validation
    }

    // Validate parameters
    validate_parameters(&response.parameters, &mut issues);

    // Check confidence level
    if response.confidence < 0.5 {
        issues.push("Low confidence level (< 0.5)".to_string());
    }

    if issues.is_empty() {
        ValidationResult::Valid
    } else {
        ValidationResult::Invalid(issues)
    }
}

/// Validate parameters extracted from the prompt
fn validate_parameters(parameters: &PromptParameters, issues: &mut Vec<String>) {
    // Validate amount format
    if let Some(amount) = &parameters.amount {
        // Check if amount is a valid number or "all"
        if amount.to_lowercase() != "all" {
            match amount.parse::<f64>() {
                Ok(value) => {
                    if value <= 0.0 {
                        issues.push("Amount must be positive".to_string());
                    }
                }
                Err(_) => {
                    issues.push("Invalid amount format".to_string());
                }
            }
        }
    }

    // Validate mint addresses (basic check)
    if let Some(mint) = &parameters.input_mint {
        if !is_valid_pubkey(mint) {
            issues.push("Invalid input_mint address".to_string());
        }
    }

    if let Some(mint) = &parameters.output_mint {
        if !is_valid_pubkey(mint) {
            issues.push("Invalid output_mint address".to_string());
        }
    }
}

/// Check if a string is a valid Solana pubkey format
fn is_valid_pubkey(pubkey: &str) -> bool {
    // Check for known mint addresses first (most common case)
    match pubkey {
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" => return true, // USDC
        "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB" => return true, // USDT
        "So11111111111111111111111111111111111111112" => return true,  // SOL
        _ => {}                                                        // Continue with other checks
    }

    // Basic check for Solana pubkey format (base58, typically 43-44 characters)
    if pubkey.len() < 32 || pubkey.len() > 44 {
        return false;
    }

    // Check if it contains only base58 characters
    for c in pubkey.chars() {
        if !(('1'..'9').contains(&c)
            || ('A'..'H').contains(&c)
            || ('J'..'N').contains(&c)
            || ('P'..'Z').contains(&c)
            || ('a'..'k').contains(&c)
            || ('m'..'z').contains(&c))
        {
            return false;
        }
    }

    true
}

/// Calculate confidence score for a structured response
pub fn calculate_confidence_score(
    response: &StructuredRefinedPrompt,
    original_prompt: &str,
) -> f32 {
    let mut confidence = 0.5f32; // Base confidence

    // Increase confidence based on action match
    match response.action {
        PromptAction::Transfer => {
            if original_prompt.to_lowercase().contains("transfer")
                || original_prompt.to_lowercase().contains("send")
            {
                confidence += 0.2;
            }
        }
        PromptAction::Swap => {
            if original_prompt.to_lowercase().contains("swap") {
                confidence += 0.2;
            }
        }
        PromptAction::Lend => {
            if original_prompt.to_lowercase().contains("lend") {
                confidence += 0.2;
            }
        }
        PromptAction::Earn => {
            if original_prompt.to_lowercase().contains("earn") {
                confidence += 0.2;
            }
        }
        PromptAction::Borrow => {
            if original_prompt.to_lowercase().contains("borrow") {
                confidence += 0.2;
            }
        }
        _ => {} // Unknown action - no adjustment
    }

    // Increase confidence if target_pubkey is extracted and valid
    if let Some(target) = &response.target_pubkey {
        if original_prompt.contains(target) && is_valid_pubkey(target) {
            confidence += 0.1;
        }
    }

    // Increase confidence if parameters are complete
    if response.parameters.amount.is_some() {
        confidence += 0.05;
    }
    if response.parameters.input_mint.is_some() {
        confidence += 0.05;
    }
    if response.parameters.output_mint.is_some() {
        confidence += 0.05;
    }

    // Decrease confidence if parameters are inconsistent with the prompt
    match response.action {
        PromptAction::Transfer => {
            if response.parameters.input_mint.is_some() && response.parameters.output_mint.is_some()
            {
                confidence -= 0.1; // Transfers typically don't have both input and output mints
            }
        }
        PromptAction::Swap => {
            if response.parameters.output_mint.is_none() {
                confidence -= 0.1; // Swaps should have an output mint
            }
        }
        _ => {}
    }

    // Cap at 1.0
    confidence.min(1.0)
}
