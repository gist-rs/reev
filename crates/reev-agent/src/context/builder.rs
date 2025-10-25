//! Context builder implementation for extracting account information from benchmark data

use crate::context::{AccountContext, ContextBuilder};
use reev_lib::benchmark::InitialStateItem;
use reev_lib::constants::addresses::programs::SYSTEM_PROGRAM;
use std::collections::HashMap;
use tracing::{debug, info, warn};

impl ContextBuilder {
    /// Build context from benchmark data with enhanced logging
    pub fn build_from_benchmark(
        &self,
        initial_state: &[InitialStateItem],
        key_map: &HashMap<String, String>,
        benchmark_id: &str,
    ) -> Result<AccountContext, crate::context::ContextError> {
        info!(
            "[ContextBuilder] Building account context for benchmark: {}",
            benchmark_id
        );
        debug!(
            "[ContextBuilder] Processing {} initial state items",
            initial_state.len()
        );
        debug!(
            "[ContextBuilder] Key map contains {} mappings",
            key_map.len()
        );

        let context = self.build_context(initial_state, key_map)?;

        // Log summary of extracted information
        let mut summary = String::new();

        if context.sol_balance.is_some() {
            summary.push_str(&format!(
                "SOL: {:.4} SOL, ",
                context.sol_balance.unwrap() as f64 / 1_000_000_000.0
            ));
        }

        summary.push_str(&format!("Tokens: {}, ", context.token_balances.len()));
        summary.push_str(&format!("Positions: {}", context.lending_positions.len()));

        info!("[ContextBuilder] Context built successfully: {}", summary);
        debug!(
            "[ContextBuilder] Formatted context length: {} characters",
            context.formatted_context.len()
        );

        Ok(context)
    }

    /// Build minimal context for benchmarks with limited account data
    pub fn build_minimal_context(&self, key_map: &HashMap<String, String>) -> AccountContext {
        info!("[ContextBuilder] Building minimal context (account keys only)");

        let mut context = String::from("AVAILABLE ACCOUNTS:\n\n");

        // List available account keys
        for (key_name, key_value) in key_map {
            if key_name.contains("WALLET") {
                context.push_str(&format!("💰 {key_name}: {key_value}\n"));
            } else if key_name.contains("ATA") {
                context.push_str(&format!("💎 {key_name}: {key_value}\n"));
            }
        }

        context.push_str("\n💡 Limited account information provided. Use jupiter_earn tools to check positions and balances first.");
        context.push_str("\n\n🚨 IMPORTANT: When making transfers, use the exact placeholder names above (e.g., 'RECIPIENT_USDC_ATA') rather than generating new addresses.");

        AccountContext {
            sol_balance: None,
            token_balances: HashMap::new(),
            lending_positions: HashMap::new(),
            formatted_context: context,
        }
    }

    /// Determine if context should be provided based on benchmark characteristics
    pub fn should_provide_context(
        &self,
        benchmark_id: &str,
        initial_state: &[InitialStateItem],
    ) -> bool {
        // Always provide context for Jupiter lending benchmarks
        if benchmark_id.contains("jup")
            && (benchmark_id.contains("lend") || benchmark_id.contains("earn"))
        {
            debug!(
                "[ContextBuilder] Providing context for Jupiter lending benchmark: {}",
                benchmark_id
            );
            return true;
        }

        // Provide context for benchmarks with token accounts
        let has_token_accounts = initial_state
            .iter()
            .any(|item| item.data.as_ref().is_some_and(|data| !data.mint.is_empty()));

        // Special case for 002-spl-transfer: use initial_state to preserve pre-generated ATAs
        if benchmark_id.contains("002-spl-transfer") {
            debug!(
                "[ContextBuilder] Using initial_state for 002-spl-transfer to preserve ATA addresses"
            );
            return true; // Use initial_state directly, not observation
        }

        if has_token_accounts {
            debug!(
                "[ContextBuilder] Providing context for benchmark with token accounts: {}",
                benchmark_id
            );
            return true;
        }

        // Provide context for SOL transfer benchmarks with lamports in initial_state
        let has_sol_accounts = initial_state
            .iter()
            .any(|item| item.data.is_none() && item.owner == "11111111111111111111111111111111");

        if has_sol_accounts {
            info!(
                "[ContextBuilder] Providing context for SOL benchmark: {}",
                benchmark_id
            );
            return true;
        }

        info!(
            "[ContextBuilder] Not providing context for simple benchmark: {}",
            benchmark_id
        );
        false
    }

    /// Extract token symbol from mint address with fallback
    pub fn get_token_symbol(&self, mint: &str) -> String {
        self.token_symbols.get(mint).cloned().unwrap_or_else(|| {
            warn!("[ContextBuilder] Unknown token mint: {}", mint);
            format!("TOKEN_{}", &mint[..8.min(mint.len())])
        })
    }

    /// Get token decimals with fallback
    pub fn get_token_decimals(&self, mint: &str) -> u8 {
        self.token_decimals.get(mint).copied().unwrap_or_else(|| {
            warn!(
                "[ContextBuilder] Unknown decimals for mint: {}, using 0",
                mint
            );
            0
        })
    }

    /// Validate context completeness
    pub fn validate_context(&self, context: &AccountContext) -> Result<(), String> {
        let mut issues = Vec::new();

        // Check if we have meaningful account information
        // For Jupiter lending operations, SOL balance, token balances, or lending positions should be sufficient
        if context.sol_balance.is_none()
            && context.token_balances.is_empty()
            && context.lending_positions.is_empty()
        {
            issues.push("No SOL balance, token balances, or lending positions found".to_string());
        }

        // Check for meaningful token balances (non-zero amounts)
        let meaningful_token_balances = context
            .token_balances
            .values()
            .filter(|balance| balance.amount > 0)
            .count();

        // For Jupiter lending operations, having meaningful token balances should be sufficient
        // even without SOL balance (as long as we have some tokens to lend)
        if meaningful_token_balances > 0 {
            // Remove issues about lack of SOL balance if we have meaningful token balances
            issues.retain(|issue| {
                !issue.contains("No SOL balance, token balances, or lending positions found")
            });
        }

        // Check for zero balances that might indicate setup issues
        let zero_balances = context
            .token_balances
            .values()
            .filter(|balance| balance.amount == 0)
            .count();

        if zero_balances > 0 && context.token_balances.len() == zero_balances {
            // Only flag as issue if we also don't have SOL balance and no lending positions
            // Jupiter lending can work with SOL balance alone, token balances alone, or with existing lending positions
            if context.sol_balance.is_none() && context.lending_positions.is_empty() {
                issues.push(
                    "All token balances are zero and no SOL balance or lending positions found"
                        .to_string(),
                );
            }
        }

        // For Jupiter lending benchmarks with lending positions, consider context valid
        // even if token balances are minimal, as long as we have lending positions
        if !context.lending_positions.is_empty() {
            // Remove any issues about lack of token balances if we have lending positions
            issues.retain(|issue| {
                !issue.contains("No SOL balance, token balances, or lending positions found")
            });
            issues.retain(|issue| !issue.contains("All token balances are zero"));
        }

        if !issues.is_empty() {
            return Err(format!("Context validation failed: {}", issues.join(", ")));
        }

        Ok(())
    }

    /// Create context for discovery scenarios (extended depth)
    pub fn build_discovery_context(
        &self,
        key_map: &HashMap<String, String>,
        benchmark_id: &str,
    ) -> AccountContext {
        info!(
            "[ContextBuilder] Building discovery context for: {}",
            benchmark_id
        );

        let mut context = String::from("DISCOVERY MODE - Limited Context:\n\n");
        context.push_str(&format!("Benchmark: {benchmark_id}\n"));
        context.push_str(
            "💡 Account information not provided. Use tools to discover balances and positions.\n",
        );
        context
            .push_str("🔍 You have extended conversation depth (5-7 turns) for exploration.\n\n");

        // List key accounts that might be relevant
        let mut relevant_accounts = Vec::new();
        for key_name in key_map.keys() {
            if key_name.contains("USER_WALLET") {
                relevant_accounts.push(format!("💰 User Wallet: {key_name}"));
            } else if key_name.contains("USDC") {
                relevant_accounts.push(format!("💎 USDC Account: {key_name}"));
            } else if key_name.contains("SOL") || key_name.contains("L-") || key_name.contains("j")
            {
                relevant_accounts.push(format!("🏦 Lending Account: {key_name}"));
            }
        }

        if !relevant_accounts.is_empty() {
            context.push_str("Available Accounts:\n");
            for account in relevant_accounts {
                context.push_str(&format!("  • {account}\n"));
            }
        }

        context.push_str(
            "\n💡 Start by calling jupiter_earn or token tools to discover current state.",
        );

        AccountContext {
            sol_balance: None,
            token_balances: HashMap::new(),
            lending_positions: HashMap::new(),
            formatted_context: context,
        }
    }

    /// Build context from actual surfpool observation state (REAL balances)
    pub fn build_context_from_observation(
        &self,
        account_states: &HashMap<String, serde_json::Value>,
        key_map: &HashMap<String, String>,
        benchmark_id: &str,
    ) -> Result<AccountContext, crate::context::ContextError> {
        info!(
            "[ContextBuilder] Building account context from observation for benchmark: {}",
            benchmark_id
        );
        debug!(
            "[ContextBuilder] Processing {} account states from observation",
            account_states.len()
        );

        let context = self.build_context_from_observation_internal(account_states, key_map)?;

        // Log summary of extracted information
        let mut summary = String::new();

        if context.sol_balance.is_some() {
            summary.push_str(&format!(
                "SOL: {:.4} SOL, ",
                context.sol_balance.unwrap() as f64 / 1_000_000_000.0
            ));
        }

        summary.push_str(&format!("Tokens: {}, ", context.token_balances.len()));
        summary.push_str(&format!("Positions: {}", context.lending_positions.len()));

        info!(
            "[ContextBuilder] Context built from observation: {}",
            summary
        );
        debug!(
            "[ContextBuilder] Formatted context length: {} characters",
            context.formatted_context.len()
        );

        Ok(context)
    }

    /// Internal method to build context from observation state
    fn build_context_from_observation_internal(
        &self,
        account_states: &HashMap<String, serde_json::Value>,
        key_map: &HashMap<String, String>,
    ) -> Result<AccountContext, crate::context::ContextError> {
        let mut sol_balance = None;
        let mut token_balances = HashMap::new();
        let mut lending_positions = HashMap::new();

        for (account_name, state) in account_states {
            let lamports = state.get("lamports").and_then(|v| v.as_u64()).unwrap_or(0);
            let owner = state.get("owner").and_then(|v| v.as_str()).unwrap_or("");

            // Check if this is a SOL account (System Program owned)
            if owner == SYSTEM_PROGRAM {
                sol_balance = Some(lamports);
            }

            // Check if this is a token account
            if let (Some(mint), Some(amount_str)) = (
                state.get("mint").and_then(|v| v.as_str()),
                state.get("amount").and_then(|v| v.as_str()),
            ) {
                let amount = amount_str
                    .parse::<u64>()
                    .map_err(|e| crate::context::ContextError::InvalidAmount(e.to_string()))?;

                let token_symbol = self
                    .token_symbols
                    .get(mint)
                    .cloned()
                    .unwrap_or_else(|| format!("TOKEN_{}", &mint[..8]));

                let decimals = self.token_decimals.get(mint).copied().unwrap_or(0);
                let formatted_amount = self.format_token_amount(amount, decimals, &token_symbol);

                // Get the token account owner
                let token_owner = state
                    .get("token_account_owner")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                // Check if this is a lending position token
                if self.is_lending_token(mint) {
                    let position_type = if mint.contains("jupiter") || mint.contains("Jupiter") {
                        format!("j{token_symbol}")
                    } else {
                        format!("L-{token_symbol}")
                    };

                    let position_type_clone = position_type.clone();
                    lending_positions.insert(
                        account_name.clone(),
                        crate::context::LendingPosition {
                            mint: mint.to_string(),
                            shares: amount,
                            owner: token_owner,
                            position_type,
                            formatted_shares: self.format_token_amount(
                                amount,
                                decimals,
                                &position_type_clone,
                            ),
                        },
                    );
                } else {
                    token_balances.insert(
                        account_name.clone(),
                        crate::context::TokenBalance {
                            mint: mint.to_string(),
                            amount,
                            owner: token_owner,
                            formatted_amount,
                        },
                    );
                }
            }
        }

        // Add resolved addresses from key_map that might not exist on-chain yet
        for (account_name, resolved_address) in key_map {
            if !account_states.contains_key(account_name) {
                // This account might be a placeholder that was resolved but doesn't exist on-chain yet
                if account_name.contains("WALLET")
                    && (account_name.contains("RECIPIENT") || account_name.contains("USER"))
                {
                    info!(
                        "[ContextBuilder] Adding non-existent wallet account to context: {} -> {}...{}",
                        account_name, &resolved_address[..8], &resolved_address[resolved_address.len() - 8..]
                    );
                    // For wallet accounts, show as 0 SOL if not on-chain
                    if account_name.contains("RECIPIENT") && sol_balance.is_none() {
                        sol_balance = Some(0);
                    }
                }
            }
        }

        // Build formatted context string
        let formatted_context = self.build_formatted_context(
            &sol_balance,
            &token_balances,
            &lending_positions,
            key_map,
        );

        // Add resolved addresses from key_map that might not exist on-chain yet
        for (account_name, resolved_address) in key_map {
            if !account_states.contains_key(account_name) {
                // This account might be a placeholder that was resolved but doesn't exist on-chain yet
                if account_name.contains("WALLET")
                    && (account_name.contains("RECIPIENT") || account_name.contains("USER"))
                {
                    info!(
                        "[ContextBuilder] Adding non-existent wallet account to context: {} -> {}",
                        account_name, resolved_address
                    );
                    // For wallet accounts, show as 0 SOL if not on-chain
                    if account_name.contains("RECIPIENT") {
                        sol_balance = sol_balance.or(Some(0)); // Don't override existing balance
                    }
                }
            }
        }

        Ok(crate::context::AccountContext {
            sol_balance,
            token_balances,
            lending_positions,
            formatted_context,
        })
    }
}
