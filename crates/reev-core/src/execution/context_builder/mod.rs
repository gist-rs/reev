//! Context builder for structured YML context generation

use reev_types::flow::{StepResult, WalletContext};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use tracing::info;

/// Minimal AI context containing only relevant information for AI operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinimalAiContext {
    /// Wallet pubkey
    pub pubkey: String,
    /// SOL balance in lamports
    pub sol_balance: u64,
    /// Selected token balances (only relevant tokens)
    pub tokens: HashMap<String, TokenInfo>,
    /// Results from previous steps (if any)
    pub previous_results: Vec<PreviousStepResult>,
}

/// Information about a token balance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    /// Token balance in smallest denomination
    pub balance: u64,
    /// Token symbol if available
    pub symbol: Option<String>,
    /// Token price in USD
    pub price_usd: Option<f64>,
}

/// Balance change information for tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceChange {
    /// Token mint address
    pub mint: String,
    /// Balance before operation
    pub balance_before: u64,
    /// Balance after operation
    pub balance_after: u64,
    /// Amount of change (positive or negative)
    pub change_amount: i64,
    /// Symbol for readability
    pub symbol: Option<String>,
}

/// Simplified result from a previous step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviousStepResult {
    /// Step identifier
    pub step_id: String,
    /// Whether step succeeded
    pub success: bool,
    /// Extracted key information for next step
    pub key_info: HashMap<String, serde_json::Value>,
    /// Balance changes from this step
    pub balance_changes: Vec<BalanceChange>,
    /// Any constraints for next step
    pub next_step_constraints: Vec<String>,
    /// Available tokens for next step
    pub available_tokens: HashMap<String, u64>,
}

/// Structured YML context for AI operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YmlOperationContext {
    /// Minimal AI context
    pub ai_context: MinimalAiContext,
    /// Additional metadata for debugging
    pub metadata: OperationMetadata,
    /// YML generation timestamp
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

/// Metadata about the operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationMetadata {
    /// Total number of steps in the flow
    pub total_steps: Option<usize>,
    /// Current step number
    pub current_step: Option<usize>,
    /// Operation type (swap, lend, etc.)
    pub operation_type: Option<String>,
    /// Any constraints from previous steps
    pub constraints: Vec<String>,
}

impl YmlOperationContext {
    /// Create a new operation context
    pub fn new(wallet_context: &WalletContext) -> Self {
        let ai_context = MinimalAiContext::from_wallet(wallet_context);

        Self {
            ai_context,
            metadata: OperationMetadata::default(),
            generated_at: chrono::Utc::now(),
        }
    }

    /// Serialize to YML string
    pub fn to_yml(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(self)
    }

    /// Deserialize from YML string
    pub fn from_yml(yml: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yml)
    }

    /// Convert to JSON for LLM consumption
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Add step metadata
    pub fn with_step_info(mut self, current: usize, total: usize) -> Self {
        self.metadata.current_step = Some(current);
        self.metadata.total_steps = Some(total);
        self
    }

    /// Add operation type
    pub fn with_operation_type(mut self, op_type: &str) -> Self {
        self.metadata.operation_type = Some(op_type.to_string());
        self
    }

    /// Add constraint
    pub fn with_constraint(mut self, constraint: &str) -> Self {
        self.metadata.constraints.push(constraint.to_string());
        self
    }
}

impl MinimalAiContext {
    /// Create minimal context from wallet
    pub fn from_wallet(wallet_context: &WalletContext) -> Self {
        let mut tokens = HashMap::new();

        // Add all tokens from wallet
        for (mint, balance_info) in &wallet_context.token_balances {
            let token_info = TokenInfo {
                balance: balance_info.balance,
                symbol: balance_info.symbol.clone(),
                price_usd: wallet_context.token_prices.get(mint).copied(),
            };
            tokens.insert(mint.clone(), token_info);
        }

        Self {
            pubkey: wallet_context.owner.clone(),
            sol_balance: wallet_context.sol_balance,
            tokens,
            previous_results: Vec::new(),
        }
    }

    /// Add previous step result
    pub fn with_previous_result(mut self, result: &PreviousStepResult) -> Self {
        self.previous_results.push(result.clone());
        self
    }

    /// Filter to only relevant tokens based on operation
    pub fn filter_relevant_tokens(mut self, operation_type: &str) -> Self {
        // For swap operations, we need both input and output tokens
        // For lend operations, we only need tokens that can be lent
        // This is a simplified example - in a real implementation, we'd need more sophisticated logic
        match operation_type {
            "swap" => {
                // Keep all tokens for now since we don't know which ones will be swapped
            }
            "lend" => {
                // Only keep tokens that can be lent (simplified example)
                let common_lend_tokens = [
                    "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC
                    "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB", // USDT
                ];

                self.tokens
                    .retain(|mint, _| common_lend_tokens.contains(&mint.as_str()));
            }
            _ => {
                // Default: keep all tokens
            }
        }

        self
    }

    /// Convert to prompt-friendly format
    pub fn to_prompt_format(&self) -> String {
        let mut prompt = String::new();

        // Add wallet information
        prompt.push_str(&format!(
            "Wallet: {} with {} SOL lamports\n",
            self.pubkey, self.sol_balance
        ));

        if !self.tokens.is_empty() {
            prompt.push_str("Token balances:\n");
            for (mint, token) in &self.tokens {
                let symbol = token.symbol.as_deref().unwrap_or("Unknown");
                prompt.push_str(&format!(
                    "  {}: {} units ({})\n",
                    mint, token.balance, symbol
                ));
            }
        }

        // Add previous step information with balance tracking
        if !self.previous_results.is_empty() {
            prompt.push_str("\nPrevious steps:\n");
            for (i, result) in self.previous_results.iter().enumerate() {
                prompt.push_str(&format!(
                    "Step {}: {} - {}\n",
                    i + 1,
                    result.step_id,
                    if result.success { "Success" } else { "Failed" }
                ));

                // Add balance changes
                if !result.balance_changes.is_empty() {
                    prompt.push_str("  Balance changes:\n");
                    for change in &result.balance_changes {
                        let change_type = if change.change_amount > 0 {
                            "Received"
                        } else {
                            "Spent"
                        };

                        let symbol = change.symbol.as_deref().unwrap_or("tokens");
                        prompt.push_str(&format!(
                            "    {} {} {} ({})\n",
                            change_type,
                            change.change_amount.abs(),
                            symbol,
                            change.mint
                        ));
                    }
                }

                // Add constraints for next step
                if !result.next_step_constraints.is_empty() {
                    prompt.push_str("  Constraints for next step:\n");
                    for constraint in &result.next_step_constraints {
                        prompt.push_str(&format!("    - {constraint}\n"));
                    }
                }

                // Add available tokens
                if !result.available_tokens.is_empty() {
                    prompt.push_str("  Available tokens:\n");
                    for (mint, amount) in &result.available_tokens {
                        let symbol = self
                            .tokens
                            .get(mint)
                            .and_then(|t| t.symbol.as_deref())
                            .unwrap_or("Unknown");
                        prompt
                            .push_str(&format!("    {amount} units of {symbol} ({mint})\n"));
                    }
                }

                // Add extracted key info
                for (key, value) in &result.key_info {
                    match key.as_str() {
                        "swap" => {
                            if let Some(output_mint) =
                                value.get("output_mint").and_then(|v| v.as_str())
                            {
                                if let Some(output_amount) =
                                    value.get("output_amount").and_then(|v| v.as_u64())
                                {
                                    prompt.push_str(&format!(
                                        "  Key info: Swapped for {output_amount} units of {output_mint}\n"
                                    ));
                                }
                            }
                        }
                        "lend" => {
                            if let Some(asset_mint) =
                                value.get("asset_mint").and_then(|v| v.as_str())
                            {
                                if let Some(amount) = value.get("amount").and_then(|v| v.as_u64()) {
                                    prompt.push_str(&format!(
                                        "  Key info: Lent {amount} units of {asset_mint}\n"
                                    ));
                                }
                            }
                        }
                        "operation" => {
                            if let Some(op_type) = value.get("type").and_then(|v| v.as_str()) {
                                prompt.push_str(&format!(
                                    "  Key info: Completed operation: {op_type}\n"
                                ));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        prompt
    }
}

impl OperationMetadata {
    /// Create new operation metadata
    pub fn new() -> Self {
        Self {
            total_steps: None,
            current_step: None,
            operation_type: None,
            constraints: Vec::new(),
        }
    }
}

impl Default for OperationMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating YML operation contexts
pub struct YmlContextBuilder {
    wallet_context: WalletContext,
    previous_results: Vec<PreviousStepResult>,
    metadata: OperationMetadata,
}

impl YmlContextBuilder {
    /// Create a new builder with wallet context
    pub fn new(wallet_context: WalletContext) -> Self {
        Self {
            wallet_context,
            previous_results: Vec::new(),
            metadata: OperationMetadata::default(),
        }
    }

    /// Convert StepResult to PreviousStepResult with balance change tracking
    fn convert_step_result_to_previous(&self, result: &StepResult) -> PreviousStepResult {
        let mut key_info = HashMap::new();
        let mut balance_changes = Vec::new();
        let mut next_step_constraints = Vec::new();
        let mut available_tokens = HashMap::new();

        // Extract key information based on tool calls
        if result.success {
            if let Some(tool_results) = result.output.get("tool_results") {
                if let Some(results_array) = tool_results.as_array() {
                    for tool_result in results_array {
                        // Extract swap information
                        if let Some(jupiter_swap) = tool_result.get("jupiter_swap") {
                            if let (
                                Some(input_mint),
                                Some(output_mint),
                                Some(input_amount),
                                Some(output_amount),
                            ) = (
                                jupiter_swap.get("input_mint").and_then(|v| v.as_str()),
                                jupiter_swap.get("output_mint").and_then(|v| v.as_str()),
                                jupiter_swap.get("input_amount").and_then(|v| v.as_u64()),
                                jupiter_swap.get("output_amount").and_then(|v| v.as_u64()),
                            ) {
                                key_info.insert(
                                    "swap".to_string(),
                                    json!({
                                        "input_mint": input_mint,
                                        "output_mint": output_mint,
                                        "input_amount": input_amount,
                                        "output_amount": output_amount,
                                        "output_amount_for_lend": output_amount,
                                    }),
                                );

                                // Track balance changes for swap
                                let input_symbol = self
                                    .wallet_context
                                    .token_balances
                                    .get(input_mint)
                                    .and_then(|t| t.symbol.clone());

                                balance_changes.push(BalanceChange {
                                    mint: input_mint.to_string(),
                                    balance_before: input_amount,
                                    balance_after: 0,
                                    change_amount: -(input_amount as i64),
                                    symbol: input_symbol,
                                });

                                let output_symbol = self
                                    .wallet_context
                                    .token_balances
                                    .get(output_mint)
                                    .and_then(|t| t.symbol.clone());

                                balance_changes.push(BalanceChange {
                                    mint: output_mint.to_string(),
                                    balance_before: 0,
                                    balance_after: output_amount,
                                    change_amount: output_amount as i64,
                                    symbol: output_symbol,
                                });

                                // Add constraint for next step
                                next_step_constraints.push(format!(
                                    "Use exactly {output_amount} units of {output_mint} from previous swap"
                                ));

                                // Update available tokens
                                available_tokens.insert(output_mint.to_string(), output_amount);
                            }
                        }
                        // Extract lend information
                        else if let Some(jupiter_lend) = tool_result.get("jupiter_lend") {
                            if let (Some(asset_mint), Some(amount)) = (
                                jupiter_lend.get("asset_mint").and_then(|v| v.as_str()),
                                jupiter_lend.get("amount").and_then(|v| v.as_u64()),
                            ) {
                                key_info.insert(
                                    "lend".to_string(),
                                    json!({
                                        "asset_mint": asset_mint,
                                        "amount": amount,
                                    }),
                                );

                                // Track balance changes for lend
                                let symbol = self
                                    .wallet_context
                                    .token_balances
                                    .get(asset_mint)
                                    .and_then(|t| t.symbol.clone());

                                balance_changes.push(BalanceChange {
                                    mint: asset_mint.to_string(),
                                    balance_before: amount,
                                    balance_after: 0,
                                    change_amount: -(amount as i64),
                                    symbol,
                                });

                                // Add constraint for next step
                                next_step_constraints.push(format!(
                                    "{amount} units of {asset_mint} are now lent and unavailable"
                                ));
                            }
                        }
                        // Extract generic operation info
                        else if let Some(operation_type) =
                            tool_result.get("operation_type").and_then(|v| v.as_str())
                        {
                            key_info.insert(
                                "operation".to_string(),
                                json!({
                                    "type": operation_type,
                                    "details": tool_result,
                                }),
                            );
                        }
                    }
                }
            }
        } else {
            // Handle failed step
            if let Some(error_message) = &result.error_message {
                next_step_constraints.push(format!("Previous step failed: {error_message}"));

                if error_message.contains("insufficient") {
                    next_step_constraints
                        .push("Consider reducing amount for next operation".to_string());
                }
            }
        }

        // Populate available tokens with current token balances
        for (mint, token) in &self.wallet_context.token_balances {
            available_tokens.insert(mint.clone(), token.balance);
        }

        PreviousStepResult {
            step_id: result.step_id.clone(),
            success: result.success,
            key_info,
            balance_changes,
            next_step_constraints,
            available_tokens,
        }
    }

    /// Update wallet context based on previous step results
    pub fn with_updated_wallet(mut self) -> Self {
        // Apply balance changes to wallet context
        for result in &self.previous_results {
            if result.success {
                // Apply balance changes to wallet context
                for change in &result.balance_changes {
                    if let Some(token) = self.wallet_context.token_balances.get_mut(&change.mint) {
                        token.balance = change.balance_after;
                    }
                }

                // Update SOL balance if there were any SOL changes
                for change in &result.balance_changes {
                    if change.mint == "So11111111111111111111111111111111111111112" {
                        self.wallet_context.sol_balance = change.balance_after;
                        break;
                    }
                }
            }
        }

        self
    }

    /// Add error recovery information for failed steps
    pub fn with_error_recovery(mut self) -> Self {
        // Add recovery constraints to existing PreviousStepResults
        for result in &mut self.previous_results {
            if !result.success {
                // Add recovery constraints based on error message
                // Note: We need to extract error info from key_info for failed steps
                if let Some(error_info) = result.key_info.get("error") {
                    if let Some(error_message) = error_info.get("message").and_then(|v| v.as_str())
                    {
                        result
                            .next_step_constraints
                            .push(format!("Previous step failed: {error_message}"));

                        if error_message.contains("insufficient") {
                            result
                                .next_step_constraints
                                .push("Consider reducing amount for next operation".to_string());
                        } else if error_message.contains("slippage") {
                            result
                                .next_step_constraints
                                .push("Increase slippage tolerance for next attempt".to_string());
                        } else {
                            result
                                .next_step_constraints
                                .push("Alternative operation path may be required".to_string());
                        }
                    }
                }
            }
        }

        self
    }

    /// Add previous step result
    pub fn with_previous_result(mut self, result: &StepResult) -> Self {
        // Create a PreviousStepResult from StepResult
        let prev_result = self.convert_step_result_to_previous(result);
        self.previous_results.push(prev_result);
        self
    }

    /// Add previous step results
    pub fn with_previous_results(mut self, results: &[StepResult]) -> Self {
        // Process each StepResult to create PreviousStepResult
        for result in results {
            let prev_result = self.convert_step_result_to_previous(result);
            self.previous_results.push(prev_result);
        }
        self
    }

    /// Add step information
    pub fn with_step_info(mut self, current: usize, total: usize) -> Self {
        self.metadata.current_step = Some(current);
        self.metadata.total_steps = Some(total);
        self
    }

    /// Add operation type
    pub fn with_operation_type(mut self, op_type: &str) -> Self {
        self.metadata.operation_type = Some(op_type.to_string());
        self
    }

    /// Add constraint
    pub fn with_constraint(mut self, constraint: &str) -> Self {
        self.metadata.constraints.push(constraint.to_string());
        self
    }

    /// Build the context with error recovery and wallet updates
    pub fn build(self) -> YmlOperationContext {
        // First apply error recovery to process step results
        let builder = self.with_error_recovery();

        // Then apply wallet updates if needed
        let builder = builder.with_updated_wallet();

        // Create the base AI context from wallet
        let mut ai_context = MinimalAiContext::from_wallet(&builder.wallet_context);

        // Add previous results (already processed)
        for result in &builder.previous_results {
            // Add to AI context directly without re-processing
            ai_context.previous_results.push(result.clone());
        }

        // Filter tokens based on operation type if specified
        if let Some(op_type) = &builder.metadata.operation_type {
            ai_context = ai_context.filter_relevant_tokens(op_type);
        }

        // Debug log the context
        info!(
            "Built YML context for wallet {} with {} tokens and {} previous results",
            ai_context.pubkey,
            ai_context.tokens.len(),
            ai_context.previous_results.len()
        );

        // Add additional context for multi-step flows
        if !ai_context.previous_results.is_empty() {
            // Add metadata about context state
            let successful_steps = ai_context
                .previous_results
                .iter()
                .filter(|r| r.success)
                .count();

            let failed_steps = ai_context
                .previous_results
                .iter()
                .filter(|r| !r.success)
                .count();

            info!(
                "Context includes {} successful steps and {} failed steps",
                successful_steps, failed_steps
            );
        }

        YmlOperationContext {
            ai_context,
            metadata: builder.metadata,
            generated_at: chrono::Utc::now(),
        }
    }
}
