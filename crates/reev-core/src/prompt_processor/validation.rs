//! Validation logic for structured LLM responses

use crate::prompt_processor::max_amount_calculator::{MaxAmountCalculator, MaxAmounts};
use crate::prompt_processor::types::{
    PromptAction, PromptParameters, StructuredRefinedPrompt, ValidationResult,
};

/// Validate a structured response against the original prompt and max amounts
pub fn validate_structured_response_with_max_amounts(
    response: &StructuredRefinedPrompt,
    original_prompt: &str,
    max_amounts_yml: &str,
) -> ValidationResult {
    let mut issues = Vec::new();

    // Parse max amounts from YML
    let max_amounts = match MaxAmountCalculator::parse_from_yml(max_amounts_yml) {
        Ok(max_amounts) => max_amounts,
        Err(e) => {
            issues.push(format!("Failed to parse max amounts: {e}"));
            return ValidationResult::Invalid(issues);
        }
    };

    // Check if target_pubkey appears in original prompt
    if let Some(target) = &response.target_pubkey {
        if !original_prompt.contains(target) {
            issues.push(format!(
                "Extracted target_pubkey {target} not found in original prompt"
            ));
        }
    }

    // Check if action matches prompt intent (more relaxed validation)
    // Check if action matches prompt intent (using refined_prompt since it should have corrected typos)
    match response.action {
        PromptAction::Transfer => {
            if !response.refined_prompt.to_lowercase().contains("transfer")
                && !response.refined_prompt.to_lowercase().contains("send")
            {
                issues.push("Action 'transfer' doesn't match prompt intent".to_string());
            }
        }
        PromptAction::Swap => {
            if !response.refined_prompt.to_lowercase().contains("swap")
                && !response.refined_prompt.to_lowercase().contains("exchange")
            {
                issues.push("Action 'swap' doesn't match prompt intent".to_string());
            }
        }
        PromptAction::Lend => {
            if !response.refined_prompt.to_lowercase().contains("lend")
                && !response.refined_prompt.to_lowercase().contains("deposit")
            {
                issues.push("Action 'lend' doesn't match prompt intent".to_string());
            }
        }
        PromptAction::Earn => {
            if !response.refined_prompt.to_lowercase().contains("earn")
                && !response.refined_prompt.to_lowercase().contains("stake")
            {
                issues.push("Action 'earn' doesn't match prompt intent".to_string());
            }
        }
        PromptAction::Borrow => {
            if !response.refined_prompt.to_lowercase().contains("borrow")
                && !response.refined_prompt.to_lowercase().contains("loan")
            {
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

    // Validate parameters including max amounts
    validate_parameters_with_max_amounts(
        &response.parameters,
        &response.action,
        &mut issues,
        &max_amounts,
    );

    // Special validation for "all" keyword
    if original_prompt.to_lowercase().contains("all") {
        validate_all_keyword_handling(response, &max_amounts, &mut issues);
    }

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

/// Validate a structured response against the original prompt (legacy function for backward compatibility)
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

/// Validate parameters extracted from the prompt (legacy function)
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

/// Validate parameters extracted from the prompt with max amounts
fn validate_parameters_with_max_amounts(
    parameters: &PromptParameters,
    action: &PromptAction,
    issues: &mut Vec<String>,
    max_amounts: &MaxAmounts,
) {
    // Validate amount format
    if let Some(amount) = &parameters.amount {
        // Check if amount is a valid number
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

        // If amount is numeric, validate against max amount
        if let Some(amount_str) = &parameters.amount {
            if let Ok(amount) = amount_str.parse::<f64>() {
                let token_symbol = match mint.as_str() {
                    "So11111111111111111111111111111111111111112" => "SOL",
                    "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" => "USDC",
                    "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB" => "USDT",
                    _ => {
                        // Try to determine token symbol from additional parameters
                        if let Some(token_name) = parameters.additional.get("token_name") {
                            if let Some(token_str) = token_name.as_str() {
                                &token_str.to_uppercase()
                            } else {
                                "UNKNOWN"
                            }
                        } else {
                            "UNKNOWN"
                        }
                    }
                };

                let action_str = match action {
                    PromptAction::Transfer => "transfer",
                    PromptAction::Swap => "swap",
                    PromptAction::Lend => "lend",
                    PromptAction::Earn => "earn",
                    PromptAction::Borrow => "borrow",
                    PromptAction::Unknown => "unknown",
                };

                if let Err(e) = MaxAmountCalculator::validate_amount_for_action(
                    max_amounts,
                    action_str,
                    token_symbol,
                    amount,
                ) {
                    issues.push(format!("Amount validation failed: {e}"));
                }
            }
        }
    }

    if let Some(mint) = &parameters.output_mint {
        if !is_valid_pubkey(mint) {
            issues.push("Invalid output_mint address".to_string());
        }
    }
}

/// Validate "all" keyword handling
fn validate_all_keyword_handling(
    response: &StructuredRefinedPrompt,
    max_amounts: &MaxAmounts,
    issues: &mut Vec<String>,
) {
    // Check if usable_amount was calculated
    if response.usable_amount.is_none() {
        issues.push("Usable amount should be calculated when 'all' keyword is present".to_string());
        return;
    }

    let usable_amount = response.usable_amount.unwrap();

    // Check if amount in parameters matches usable_amount
    if let Some(amount_str) = &response.parameters.amount {
        if let Ok(amount) = amount_str.parse::<f64>() {
            if (amount - usable_amount).abs() > 0.000001 {
                issues.push(format!(
                    "Amount in parameters ({amount}) doesn't match usable_amount ({usable_amount})"
                ));
            }
        }
    }

    // Validate that the usable_amount doesn't exceed the max for the action type
    let action_str = match response.action {
        PromptAction::Transfer => "transfer",
        PromptAction::Swap => "swap",
        PromptAction::Lend => "lend",
        PromptAction::Earn => "earn",
        PromptAction::Borrow => "borrow",
        PromptAction::Unknown => "unknown",
    };

    // Determine the token symbol from the mint address or other parameters
    let token_symbol = if let Some(mint) = &response.parameters.input_mint {
        match mint.as_str() {
            "So11111111111111111111111111111111111111112" => "SOL",
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" => "USDC",
            "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB" => "USDT",
            _ => "UNKNOWN",
        }
    } else if let Some(token_name) = response.parameters.additional.get("token_name") {
        token_name.as_str().unwrap_or("UNKNOWN")
    } else {
        "UNKNOWN"
    };

    if let Some(max_amount) =
        MaxAmountCalculator::get_max_amount_for_action_token(max_amounts, action_str, token_symbol)
    {
        if usable_amount > max_amount {
            issues.push(format!(
                "Usable amount ({usable_amount}) exceeds max amount ({max_amount}) for {action_str} {token_symbol}"
            ));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_amount_for_action() {
        let mut max_amounts_map = std::collections::HashMap::new();
        let mut sol_amounts = std::collections::HashMap::new();
        sol_amounts.insert("SOL".to_string(), 10.5);

        max_amounts_map.insert(
            "transfer".to_string(),
            crate::prompt_processor::max_amount_calculator::TokenAmounts {
                amounts: sol_amounts,
            },
        );

        let max_amounts = MaxAmounts {
            max_amounts: max_amounts_map,
        };

        // Valid amount should pass
        let response = StructuredRefinedPrompt::builder()
            .action(PromptAction::Transfer)
            .refined_prompt("transfer 5.0 SOL to address".to_string())
            .parameters(PromptParameters {
                amount: Some("5.0".to_string()),
                input_mint: Some("So11111111111111111111111111111111111111112".to_string()),
                output_mint: None,
                additional: std::collections::HashMap::new(),
            })
            .build();

        let result = validate_structured_response_with_max_amounts(
            &response,
            "transfer 5.0 SOL to address",
            &serde_yaml::to_string(&max_amounts).unwrap(),
        );

        assert!(matches!(result, ValidationResult::Valid));
    }

    #[test]
    fn test_validate_amount_exceeds_max() {
        let mut max_amounts_map = std::collections::HashMap::new();
        let mut sol_amounts = std::collections::HashMap::new();
        sol_amounts.insert("SOL".to_string(), 10.5);

        max_amounts_map.insert(
            "transfer".to_string(),
            crate::prompt_processor::max_amount_calculator::TokenAmounts {
                amounts: sol_amounts,
            },
        );

        let max_amounts = MaxAmounts {
            max_amounts: max_amounts_map,
        };

        // Amount exceeding max should fail
        let response = StructuredRefinedPrompt::builder()
            .action(PromptAction::Transfer)
            .refined_prompt("transfer 15.0 SOL to address".to_string())
            .parameters(PromptParameters {
                amount: Some("15.0".to_string()),
                input_mint: Some("So11111111111111111111111111111111111111112".to_string()),
                output_mint: None,
                additional: std::collections::HashMap::new(),
            })
            .build();

        let result = validate_structured_response_with_max_amounts(
            &response,
            "transfer 15.0 SOL to address",
            &serde_yaml::to_string(&max_amounts).unwrap(),
        );

        if let ValidationResult::Invalid(issues) = result {
            assert!(issues.iter().any(|issue| issue.contains("exceeds max")));
        } else {
            panic!("Expected invalid validation result");
        }
    }

    #[test]
    fn test_validate_all_keyword() {
        let mut max_amounts_map = std::collections::HashMap::new();
        let mut sol_amounts = std::collections::HashMap::new();
        sol_amounts.insert("SOL".to_string(), 10.5);

        max_amounts_map.insert(
            "transfer".to_string(),
            crate::prompt_processor::max_amount_calculator::TokenAmounts {
                amounts: sol_amounts,
            },
        );

        let max_amounts = MaxAmounts {
            max_amounts: max_amounts_map,
        };

        // Valid "all" keyword handling
        let response = StructuredRefinedPrompt::builder()
            .action(PromptAction::Transfer)
            .refined_prompt("transfer 10.5 SOL to address".to_string())
            .parameters(PromptParameters {
                amount: Some("10.5".to_string()),
                input_mint: Some("So11111111111111111111111111111111111111112".to_string()),
                output_mint: None,
                additional: std::collections::HashMap::new(),
            })
            .usable_amount(Some(10.5))
            .build();

        let result = validate_structured_response_with_max_amounts(
            &response,
            "transfer all SOL to address",
            &serde_yaml::to_string(&max_amounts).unwrap(),
        );

        assert!(matches!(result, ValidationResult::Valid));
    }
}
