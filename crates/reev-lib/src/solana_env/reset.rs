use crate::{
    agent::AgentObservation,
    solana_env::{observation, SolanaEnv},
    test_scenarios, // Import the new centralized module
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

    // 3. Parse options and the full TestCase from the benchmark file.
    let options = options.context("Benchmark options are required")?;
    let test_case: crate::benchmark::TestCase = serde_json::from_value(options.clone())
        .context("Failed to deserialize options into TestCase")?;
    let initial_state_val = options
        .get("initial_state")
        .cloned()
        .context("Benchmark options must include 'initial_state'")?;
    let accounts: Vec<Value> = serde_json::from_value(initial_state_val)?;

    // 4. First pass: Discover all placeholders, create keypairs for them,
    //    and set up the fee payer.
    for account_config in &accounts {
        let placeholder = account_config["pubkey"]
            .as_str()
            .context("Missing 'pubkey' placeholder in account config")?;

        if let Ok(pubkey) = Pubkey::from_str(placeholder) {
            env.pubkey_map.insert(placeholder.to_string(), pubkey);
        } else {
            let keypair = Keypair::new();
            env.pubkey_map
                .insert(placeholder.to_string(), keypair.pubkey());
            env.keypair_map.insert(placeholder.to_string(), keypair);
        }

        if placeholder == "USER_WALLET_PUBKEY" {
            env.fee_payer = Some(placeholder.to_string());
        }
    }

    // 5. Fund the fee payer from the validator airdrop.
    let fee_payer_placeholder = env.fee_payer.as_ref().context("Fee payer not set")?;
    let fee_payer_keypair = env.get_fee_payer_keypair()?;
    let initial_lamports = accounts
        .iter()
        .find(|acc| acc["pubkey"].as_str() == Some(fee_payer_placeholder))
        .and_then(|acc| acc["lamports"].as_u64())
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

    // --- THIS IS THE CRITICAL CHANGE ---
    // 6. Delegate complex SPL setup to the centralized scenario handler.
    // This function will handle ATA derivation and RPC calls to fund accounts.
    let mut initial_observation =
        observation::get_observation(env, "Initial state before SPL setup", None, vec![])?;

    // The setup function will mutate the env's pubkey_map and the observation's key_map
    // with the real, derived ATA addresses.
    test_scenarios::setup_spl_scenario(env, &test_case, &mut initial_observation)
        .await
        .expect("Reset failed");

    // The key_map in the observation might have been updated by the setup function,
    // so we return the final, corrected observation.
    info!("Environment reset complete.");
    Ok(initial_observation)
}
