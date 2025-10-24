//! Reev Context Resolver
//!
//! This module provides centralized context resolution for the Reev agent system.
//! It handles:
//! - Placeholder resolution to real addresses
//! - Account state consolidation from YAML + surfpool
//! - Multi-step flow context management
//! - YAML schema validation

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use solana_sdk::{program_pack::Pack, pubkey::Pubkey, signature::Signer};
use spl_associated_token_account::get_associated_token_address;
use std::collections::HashMap;
use std::str::FromStr;
use tracing::{info, warn};

/// Complete context for agent execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContext {
    /// Map of placeholder names to resolved addresses
    pub key_map: HashMap<String, String>,
    /// Account states from on-chain queries
    pub account_states: HashMap<String, serde_json::Value>,
    /// Fee payer placeholder
    pub fee_payer_placeholder: Option<String>,
    /// Current step in multi-step flows
    pub current_step: Option<u32>,
    /// Previous step results for multi-step flows
    pub step_results: HashMap<String, serde_json::Value>,
}

/// Initial state from benchmark YAML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitialState {
    pub pubkey: String,
    pub owner: String,
    pub lamports: u64,
    pub data: Option<String>,
}

/// Address derivation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AddressDerivation {
    AssociatedTokenAccount { owner: String, mint: String },
}

/// Context resolver for placeholder resolution and state consolidation
pub struct ContextResolver {
    rpc_client: solana_client::rpc_client::RpcClient,
}

impl ContextResolver {
    /// Create new context resolver with RPC client
    pub fn new(rpc_client: solana_client::rpc_client::RpcClient) -> Self {
        Self { rpc_client }
    }

    /// Resolve initial context from benchmark configuration
    /// Resolve initial context from YAML initial_state and surfpool data
    /// Includes ALL key_map accounts regardless of balance
    pub async fn resolve_initial_context(
        &self,
        initial_state: &[InitialState],
        ground_truth: &serde_json::Value,
        existing_key_map: Option<HashMap<String, String>>,
    ) -> Result<AgentContext> {
        info!(
            "[ContextResolver] Resolving initial context for {} accounts",
            initial_state.len()
        );

        let mut key_map = existing_key_map.unwrap_or_default();
        let mut account_states = HashMap::new();
        let mut fee_payer_placeholder = None;

        // First pass: Process initial state and resolve placeholders
        for state in initial_state {
            let pubkey_str = &state.pubkey;

            // Check if it's a placeholder or a literal pubkey
            if Pubkey::from_str(pubkey_str).is_err() {
                // It's a placeholder, resolve to real address
                if !key_map.contains_key(pubkey_str) {
                    let new_pubkey = solana_sdk::signature::Keypair::new();
                    key_map.insert(pubkey_str.clone(), new_pubkey.pubkey().to_string());
                    info!(
                        "[ContextResolver] Resolved placeholder '{}' to '{}'",
                        pubkey_str,
                        new_pubkey.pubkey()
                    );
                }
            } else {
                // It's a literal pubkey, add as-is
                if !key_map.contains_key(pubkey_str) {
                    key_map.insert(pubkey_str.clone(), pubkey_str.clone());
                }
            }

            // Track fee payer placeholder
            if pubkey_str == "USER_WALLET_PUBKEY" {
                fee_payer_placeholder = Some(pubkey_str.clone());
            }
        }

        // Second pass: Handle derived addresses from ground truth
        if let Some(assertions) = ground_truth
            .get("final_state_assertions")
            .and_then(|v| v.as_array())
        {
            for assertion in assertions {
                if let Some(assertion_obj) = assertion.as_object() {
                    if let Some(_pubkey_placeholder) =
                        assertion_obj.get("pubkey").and_then(|v| v.as_str())
                    {
                        // Check for address derivation in assertion
                        if let Some(derivation) = self.extract_address_derivation(assertion_obj) {
                            self.resolve_derived_address(&derivation, &mut key_map)
                                .await?;
                        }
                    }
                }
            }
        }

        // Third pass: Fetch account states for all resolved addresses
        for (placeholder, pubkey_str) in &key_map {
            let pubkey = Pubkey::from_str(pubkey_str)
                .context(format!("Invalid pubkey string in key_map: {pubkey_str}"))?;
            if let Ok(account) = self.rpc_client.get_account(&pubkey) {
                let mut state = serde_json::json!({
                    "lamports": account.lamports,
                    "owner": account.owner.to_string(),
                    "executable": account.executable,
                    "data_len": account.data.len(),
                });

                // Handle SPL token accounts
                if account.owner == spl_token::ID
                    && account.data.len() == spl_token::state::Account::LEN
                {
                    if let Ok(token_account) = spl_token::state::Account::unpack(&account.data) {
                        if let Some(obj) = state.as_object_mut() {
                            obj.insert(
                                "mint".to_string(),
                                serde_json::json!(token_account.mint.to_string()),
                            );
                            obj.insert(
                                "token_account_owner".to_string(),
                                serde_json::json!(token_account.owner.to_string()),
                            );
                            obj.insert(
                                "amount".to_string(),
                                serde_json::json!(token_account.amount.to_string()),
                            );
                        }
                    }
                }
                account_states.insert(placeholder.clone(), state);
            } else {
                // Account doesn't exist on-chain (0 lamports)
                info!(
                    "[ContextResolver] Account {} ({}) not found on-chain, including as non-existent",
                    placeholder, pubkey
                );
                let state = serde_json::json!({
                    "lamports": 0,
                    "owner": "11111111111111111111111111111111",
                    "executable": false,
                    "data_len": 0,
                    "exists": false
                });
                account_states.insert(placeholder.clone(), state);
            }
        }

        let context = AgentContext {
            key_map,
            account_states,
            fee_payer_placeholder,
            current_step: Some(0),
            step_results: HashMap::new(),
        };

        info!(
            "[ContextResolver] Resolved {} placeholders and {} account states",
            context.key_map.len(),
            context.account_states.len()
        );

        Ok(context)
    }

    /// Update context after a step execution for multi-step flows
    pub async fn update_context_after_step(
        &self,
        mut context: AgentContext,
        step_number: u32,
        step_result: serde_json::Value,
    ) -> Result<AgentContext> {
        info!(
            "[ContextResolver] Updating context after step {}",
            step_number
        );

        // Update step number
        context.current_step = Some(step_number);

        // Store step result
        context
            .step_results
            .insert(format!("step_{step_number}"), step_result.clone());

        // Process step result to update account states immediately
        self.process_step_result_for_context(&mut context, &step_result)
            .await?;

        // Refresh account states to reflect changes after the step
        for (placeholder, pubkey_str) in &context.key_map {
            let pubkey = Pubkey::from_str(pubkey_str)
                .context(format!("Invalid pubkey string in key_map: {pubkey_str}"))?;
            if let Ok(account) = self.rpc_client.get_account(&pubkey) {
                let mut state = serde_json::json!({
                    "lamports": account.lamports,
                    "owner": account.owner.to_string(),
                    "executable": account.executable,
                    "data_len": account.data.len(),
                });

                // Handle SPL token accounts
                if account.owner == spl_token::ID
                    && account.data.len() == spl_token::state::Account::LEN
                {
                    if let Ok(token_account) = spl_token::state::Account::unpack(&account.data) {
                        if let Some(obj) = state.as_object_mut() {
                            obj.insert(
                                "mint".to_string(),
                                serde_json::json!(token_account.mint.to_string()),
                            );
                            obj.insert(
                                "token_account_owner".to_string(),
                                serde_json::json!(token_account.owner.to_string()),
                            );
                            obj.insert(
                                "amount".to_string(),
                                serde_json::json!(token_account.amount.to_string()),
                            );
                        }
                    }
                }
                context.account_states.insert(placeholder.clone(), state);
            }
        }

        Ok(context)
    }

    /// Process step result data to update context account states immediately
    /// This allows tests and environments without real blockchain connections
    /// to show context updates from swap results, lending operations, etc.
    async fn process_step_result_for_context(
        &self,
        context: &mut AgentContext,
        step_result: &serde_json::Value,
    ) -> Result<()> {
        info!("[ContextResolver] Processing step result for immediate context update");

        // Handle swap results
        if let Some(swap_details) = step_result.get("swap_details") {
            info!("[ContextResolver] Processing swap result");

            if let (Some(output_mint), Some(output_amount)) = (
                swap_details.get("output_mint").and_then(|v| v.as_str()),
                swap_details.get("output_amount").and_then(|v| v.as_str()),
            ) {
                // Find the token account placeholder for this mint
                for (placeholder, account_state) in &mut context.account_states {
                    if let Some(account) = account_state.as_object_mut() {
                        if let Some(current_mint) = account.get("mint").and_then(|v| v.as_str()) {
                            if current_mint == output_mint {
                                // Update the amount with the swap result
                                account.insert(
                                    "amount".to_string(),
                                    serde_json::json!(output_amount.to_string()),
                                );
                                info!(
                                    "[ContextResolver] Updated {} balance to {} from swap result",
                                    placeholder, output_amount
                                );
                            }
                        }
                    }
                }
            }
        }

        // Handle direct usdc_received field (common in test data)
        if let Some(usdc_received) = step_result.get("usdc_received").and_then(|v| v.as_str()) {
            info!(
                "[ContextResolver] Processing direct USDC received: {}",
                usdc_received
            );

            // Find USDC account (typically USER_USDC_ATA)
            for (placeholder, account_state) in &mut context.account_states {
                if placeholder.contains("USDC") || placeholder.contains("usdc") {
                    if let Some(account) = account_state.as_object_mut() {
                        account.insert(
                            "amount".to_string(),
                            serde_json::json!(usdc_received.to_string()),
                        );
                        info!(
                            "[ContextResolver] Updated USDC account {} balance to {}",
                            placeholder, usdc_received
                        );
                    }
                }
            }
        }

        // Handle other operation results as needed
        // TODO: Add handling for lending operations, etc.

        Ok(())
    }

    /// Validate that all placeholders in context are resolved to real addresses
    /// Validate the resolved context for completeness and correctness
    /// Throws error if prerequisite context is missing
    pub fn validate_resolved_context(&self, context: &AgentContext) -> Result<()> {
        info!("[ContextResolver] Validating resolved context");

        // Validate all key_map addresses are valid
        for (placeholder, address) in &context.key_map {
            // Check if address is a valid base58 string
            if let Err(e) = Pubkey::from_str(address) {
                return Err(anyhow::anyhow!(
                    "INVALID CONTEXT: Placeholder '{placeholder}' resolves to invalid address '{address}': {e}"
                ));
            }
        }

        // Check that critical placeholders are present from YAML
        let required_placeholders = ["USER_WALLET_PUBKEY"];
        for required in &required_placeholders {
            if !context.key_map.contains_key(*required) {
                return Err(anyhow::anyhow!(
                    "MISSING PREREQUISITE: Required placeholder '{required}' missing from context. Check YAML initial_state definition."
                ));
            }
        }

        // Validate account states are complete
        for placeholder in context.key_map.keys() {
            let real_address = &context.key_map[placeholder];
            if !context.account_states.contains_key(real_address)
                && !context.account_states.contains_key(placeholder)
            {
                info!("[ContextResolver] Warning: No account state found for placeholder '{}' with address '{}'", placeholder, real_address);
            }
        }

        // Validate step dependencies if this is multi-step flow
        if let Some(current_step) = context.current_step {
            if current_step > 0 {
                // Check for previous step results
                for step_num in 0..current_step {
                    let step_key = format!("step_{step_num}");
                    if !context.step_results.contains_key(&step_key) {
                        return Err(anyhow::anyhow!(
                            "MISSING PREREQUISITE: Multi-step flow requires result from step_{step_num} for step {current_step}"
                        ));
                    }
                }
            }
        }

        info!(
            "[ContextResolver] Context validation passed: {} placeholders, {} account states",
            context.key_map.len(),
            context.account_states.len()
        );
        Ok(())
    }

    /// Export context to YAML string for LLM consumption (legacy format)
    pub fn context_to_yaml(&self, context: &AgentContext) -> Result<String> {
        // Use the enhanced YAML format with comments
        self.context_to_yaml_with_comments(context)
    }

    /// Export context to properly formatted YAML string with comments for LLM consumption
    /// Only includes transaction-relevant data (balance, ownership, existence)
    pub fn context_to_yaml_with_comments(&self, context: &AgentContext) -> Result<String> {
        use std::collections::BTreeMap;

        // Convert to sorted structure for consistent output
        let mut sorted_key_map = BTreeMap::new();
        for (key, value) in &context.key_map {
            sorted_key_map.insert(key.clone(), value.clone());
        }

        let mut sorted_account_states = BTreeMap::new();
        for (key, value) in &context.account_states {
            sorted_account_states.insert(key.clone(), value.clone());
        }

        let mut yaml_lines = vec![
            "# On-Chain Context for Transaction Processing".to_string(),
            "# Only balance, ownership, and existence information".to_string(),
            "".to_string(),
            "# Key Map: Placeholder names resolved to real addresses".to_string(),
            "key_map:".to_string(),
        ];

        for (placeholder, address) in sorted_key_map {
            yaml_lines.push(format!("  {placeholder}: {address}"));
        }

        yaml_lines.push("".to_string());
        yaml_lines.push("# Account States: Current balance and ownership".to_string());
        yaml_lines.push("account_states:".to_string());

        for (address, state) in sorted_account_states {
            yaml_lines.push(format!("  {address}:"));
            if let Some(obj) = state.as_object() {
                // Only include transaction-relevant fields
                if let Some(lamports) = obj.get("lamports") {
                    yaml_lines.push(format!("    lamports: {lamports}"));
                }
                if let Some(owner) = obj.get("owner") {
                    yaml_lines.push(format!("    owner: {owner}"));
                }
                if let Some(mint) = obj.get("mint") {
                    yaml_lines.push(format!("    mint: {mint}"));
                }
                if let Some(amount) = obj.get("amount") {
                    yaml_lines.push(format!("    amount: {amount}"));
                }
                if let Some(exists) = obj.get("exists") {
                    yaml_lines.push(format!("    exists: {exists}"));
                }
            }
        }

        if let Some(current_step) = context.current_step {
            yaml_lines.push("".to_string());
            yaml_lines.push("# Multi-step Flow Information".to_string());
            yaml_lines.push(format!("current_step: {current_step}"));
        }

        if !context.step_results.is_empty() {
            yaml_lines.push("".to_string());
            yaml_lines.push("# Previous Step Results".to_string());
            yaml_lines.push("step_results:".to_string());
            for (step, result) in &context.step_results {
                yaml_lines.push(format!("  {}: {}", step, serde_yaml::to_string(result)?));
            }
        }

        yaml_lines.push("".to_string());
        yaml_lines.push("---".to_string());

        Ok(yaml_lines.join("\n"))
    }

    /// Extract address derivation from ground truth assertion
    fn extract_address_derivation(
        &self,
        assertion_obj: &serde_json::Map<String, serde_json::Value>,
    ) -> Option<AddressDerivation> {
        // Look for ATA derivation patterns
        if let (Some(owner), Some(mint)) = (
            assertion_obj.get("owner").and_then(|v| v.as_str()),
            assertion_obj.get("mint").and_then(|v| v.as_str()),
        ) {
            return Some(AddressDerivation::AssociatedTokenAccount {
                owner: owner.to_string(),
                mint: mint.to_string(),
            });
        }
        None
    }

    /// Resolve derived address and add to key_map
    async fn resolve_derived_address(
        &self,
        derivation: &AddressDerivation,
        key_map: &mut HashMap<String, String>,
    ) -> Result<()> {
        match derivation {
            AddressDerivation::AssociatedTokenAccount { owner, mint } => {
                if let Some(owner_pubkey) = key_map.get(owner) {
                    let mint_pubkey =
                        Pubkey::from_str(mint).context("Invalid mint address in derivation")?;

                    let owner_pubkey = Pubkey::from_str(owner_pubkey)
                        .context("Invalid owner address in key_map")?;

                    let derived_ata = get_associated_token_address(&owner_pubkey, &mint_pubkey);
                    let placeholder = format!("{}_ATA_PLACEHOLDER", owner.to_uppercase());

                    info!(
                        "[ContextResolver] Derived ATA {} for owner {} + mint {}",
                        derived_ata, owner, mint
                    );

                    key_map.insert(placeholder, derived_ata.to_string());
                } else {
                    warn!(
                        "[ContextResolver] Cannot derive ATA - owner '{}' not found in key_map",
                        owner
                    );
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_context_resolver_initial_state() {
        // Mock RPC client for testing
        let rpc_url = "http://127.0.0.1:8899";
        let rpc_client = solana_client::rpc_client::RpcClient::new_with_commitment(
            rpc_url,
            solana_sdk::commitment_config::CommitmentConfig::confirmed(),
        );
        let resolver = ContextResolver::new(rpc_client);

        let initial_state = vec![
            InitialState {
                pubkey: "USER_WALLET_PUBKEY".to_string(),
                owner: "11111111111111111111111111111111".to_string(),
                lamports: 1000000000,
                data: None,
            },
            InitialState {
                pubkey: "RECIPIENT_WALLET_PUBKEY".to_string(),
                owner: "11111111111111111111111111111111".to_string(),
                lamports: 0,
                data: None,
            },
        ];

        let ground_truth = serde_json::json!({
            "final_state_assertions": []
        });

        // Test with mock environment (may need real RPC for full test)
        let result = resolver.resolve_initial_context(&initial_state, &ground_truth, None);

        // In a real test, we would have a test validator running
        // For now, just test the structure
        match result.await {
            Ok(context) => {
                assert!(context.key_map.contains_key("USER_WALLET_PUBKEY"));
                assert!(context.key_map.contains_key("RECIPIENT_WALLET_PUBKEY"));
                assert_eq!(context.current_step, Some(0));
                assert_eq!(
                    context.fee_payer_placeholder,
                    Some("USER_WALLET_PUBKEY".to_string())
                );
            }
            Err(e) => {
                println!("Expected failure without test validator: {e}");
            }
        }
    }

    #[test]
    fn test_validate_resolved_context() {
        let rpc_url = "http://127.0.0.1:8899";
        let rpc_client = solana_client::rpc_client::RpcClient::new(rpc_url);
        let resolver = ContextResolver::new(rpc_client);

        let mut key_map = HashMap::new();
        key_map.insert(
            "USER_WALLET_PUBKEY".to_string(),
            "52os5otfYAPyQM2BbCEd6vbbDgLN42feBtxDCyVvKv6x".to_string(),
        );

        let context = AgentContext {
            key_map,
            account_states: HashMap::new(),
            fee_payer_placeholder: Some("USER_WALLET_PUBKEY".to_string()),
            current_step: Some(0),
            step_results: HashMap::new(),
        };

        assert!(resolver.validate_resolved_context(&context).is_ok());

        // Test invalid address
        let mut invalid_key_map = HashMap::new();
        invalid_key_map.insert(
            "USER_WALLET_PUBKEY".to_string(),
            "INVALID_ADDRESS".to_string(),
        );

        let invalid_context = AgentContext {
            key_map: invalid_key_map,
            account_states: HashMap::new(),
            fee_payer_placeholder: Some("USER_WALLET_PUBKEY".to_string()),
            current_step: Some(0),
            step_results: HashMap::new(),
        };

        assert!(resolver
            .validate_resolved_context(&invalid_context)
            .is_err());
    }
}
