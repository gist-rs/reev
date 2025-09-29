use crate::{
    agent::AgentObservation,
    solana_env::{environment::SolanaEnv, observation},
    test_scenarios,
};
use anyhow::{Context, Result};
use serde_json::Value;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use std::{str::FromStr, thread, time::Duration};
use tracing::{info, instrument};

#[instrument(skip_all, name = "env.reset")]
pub(crate) async fn handle_reset(
    env: &mut SolanaEnv,
    options: Option<Value>,
) -> Result<AgentObservation> {
    info!("Resetting Solana environment...");

    // 1. Health check for the surfpool validator.
    for i in 0..10 {
        if env.rpc_client.get_health().is_ok() {
            break;
        }
        if i == 9 {
            anyhow::bail!(
                "Could not connect to `surfpool` validator at {}",
                "http://127.0.0.1:8899"
            );
        }
        thread::sleep(Duration::from_secs(1));
    }
    info!("Validator is healthy.");

    // 2. Clear any state from previous runs.
    env.keypair_map.clear();
    env.pubkey_map.clear();
    env.fee_payer = None;

    // 3. Parse the full TestCase from the benchmark file options.
    let options = options.context("Benchmark options are required")?;
    let test_case: crate::benchmark::TestCase =
        serde_json::from_value(options).context("Failed to deserialize options into TestCase")?;

    // 4. First pass: Discover all placeholders and literal pubkeys from the initial state.
    for account_config in &test_case.initial_state {
        // Handle the main pubkey field. If it's not a valid pubkey string, it's a placeholder.
        let pubkey_placeholder = &account_config.pubkey;
        if Pubkey::from_str(pubkey_placeholder).is_err() {
            // It's a placeholder, so create a keypair for it if one doesn't exist.
            if !env.keypair_map.contains_key(pubkey_placeholder) {
                let keypair = Keypair::new();
                env.pubkey_map
                    .insert(pubkey_placeholder.clone(), keypair.pubkey());
                env.keypair_map.insert(pubkey_placeholder.clone(), keypair);
            }
        }

        // Handle the owner field. If it IS a valid pubkey, it's a literal program ID.
        let owner_pubkey_str = &account_config.owner;
        if let Ok(pubkey) = Pubkey::from_str(owner_pubkey_str) {
            // It's a literal pubkey (like a program ID). Add it to the map
            // so it can be resolved by name later. The key and value are the same string.
            env.pubkey_map.insert(owner_pubkey_str.clone(), pubkey);
        }
    }

    // 5. Set the fee payer, which is a requirement for all benchmarks.
    if env.keypair_map.contains_key("USER_WALLET_PUBKEY") {
        env.fee_payer = Some("USER_WALLET_PUBKEY".to_string());
    } else {
        anyhow::bail!(
            "A placeholder 'USER_WALLET_PUBKEY' must be present in initial_state to act as fee payer."
        );
    }

    // 6. Fund the fee payer using a validator airdrop.
    let fee_payer_placeholder = env.fee_payer.as_ref().context("Fee payer not set")?;
    let fee_payer_keypair = env.get_fee_payer_keypair()?;
    let initial_lamports = test_case
        .initial_state
        .iter()
        .find(|acc| acc.pubkey == *fee_payer_placeholder)
        .map(|acc| acc.lamports)
        .context("Fee payer 'lamports' not found or invalid in initial state")?;

    if initial_lamports > 0 {
        info!(
            "Funding fee payer ({}) with {} lamports...",
            fee_payer_keypair.pubkey(),
            initial_lamports
        );
        let sig = env
            .rpc_client
            .request_airdrop(&fee_payer_keypair.pubkey(), initial_lamports)?;
        env.rpc_client
            .confirm_transaction(&sig)
            .context("Failed to confirm fee payer airdrop")?;
        info!("Fee payer funded.");
    }

    // 7. Get the initial observation before any complex scenario setup.
    let mut initial_observation =
        observation::get_observation(env, "Initial state before SPL setup", None, vec![])?;

    // 8. Delegate complex SPL setup to the centralized scenario handler.
    // This function handles ATA derivation and uses RPC cheat codes to fund accounts,
    // mutating the environment and observation maps with the real derived addresses.
    test_scenarios::setup_spl_scenario(env, &test_case, &mut initial_observation)
        .await
        .context("Failed to set up SPL scenario")?;

    info!("Environment reset complete.");
    Ok(initial_observation)
}
