#![cfg(test)]

use super::surfpool_client::SurfpoolClient;
use anyhow::{Context, Result, anyhow};
use jupiter_swap_api_client::{
    JupiterSwapApiClient, quote::QuoteRequest, swap::SwapRequest,
    transaction_config::TransactionConfig,
};
use reev_lib::{
    actions::spl_transfer, agent::AgentObservation, benchmark::TestCase, env::GymEnv,
    solana_env::environment::SolanaEnv,
};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    address_lookup_table::{AddressLookupTableAccount, state::AddressLookupTable},
    instruction::Instruction,
    message::v0,
    pubkey::Pubkey,
};
use solana_system_interface::instruction as system_instruction;
use std::{collections::HashMap, fs, path::Path, str::FromStr, thread, time::Duration};
use tracing::{info, warn};

/// A standard helper to set up the `SolanaEnv` for a given benchmark file.
///
/// This function is now the single entry point for setting up any benchmark. It
/// reads the test case from the file and calls `env.reset()`. The `SolanaEnv`'s
/// `reset` implementation now contains all the centralized logic (including complex
/// SPL setup), ensuring a consistent environment for all tests.
pub async fn setup_env_for_benchmark(
    benchmark_path: &Path,
) -> Result<(SolanaEnv, TestCase, AgentObservation)> {
    // HACK: Add a small delay and health check to mitigate race conditions
    // where the surfpool validator is not yet ready when the test starts.
    thread::sleep(Duration::from_millis(500));
    let rpc_client = RpcClient::new("http://127.0.0.1:8899".to_string());
    for i in 0..20 {
        if rpc_client.get_health().is_ok() {
            break;
        }
        if i == 19 {
            return Err(anyhow!("Timed out waiting for validator to be healthy."));
        }
        thread::sleep(Duration::from_millis(500));
    }

    let f = fs::File::open(benchmark_path)?;
    let test_case: TestCase = serde_yaml::from_reader(f)?;

    let mut env = SolanaEnv::new()?;
    // `env.reset` now contains the centralized setup logic from `reev-lib`.
    let initial_observation = env
        .reset(None, Some(serde_json::to_value(&test_case)?))
        .await?;

    Ok((env, test_case, initial_observation))
}

/// Creates a "mock perfect" instruction based on the benchmark ID.
/// This represents the ideal action an agent should take.
pub fn mock_perfect_instruction(
    test_case: &TestCase,
    key_map: &HashMap<String, String>,
) -> Result<Instruction> {
    match test_case.id.as_str() {
        "001-SOL-TRANSFER" => create_sol_transfer_instruction(key_map, 100_000_000), // 0.1 SOL
        "002-SPL-TRANSFER" | "003-SPL-TRANSFER-FAIL" => {
            create_spl_transfer_instruction(key_map, 15_000_000) // 15 USDC
        }
        "100-JUP-SWAP-SOL-USDC" => {
            // For complex benchmarks, the "perfect" instruction is a template derived from the
            // ground truth. The `SolanaEnv::step` function will detect this instruction shape
            // (by its program ID) and trigger the full, stateful environment preparation
            // (account pre-loading, etc.) before execution.
            let ix_template = test_case
                .ground_truth
                .expected_instructions
                .first()
                .ok_or_else(|| {
                    anyhow!(
                        "No expected instruction found in ground truth for {}",
                        test_case.id
                    )
                })?;

            let program_id = Pubkey::from_str(&ix_template.program_id)
                .context("Failed to parse program_id from ground truth")?;

            let accounts: Vec<solana_sdk::instruction::AccountMeta> = ix_template
                .accounts
                .iter()
                .map(|acc| {
                    let pubkey_str = key_map.get(&acc.pubkey).ok_or_else(|| {
                        anyhow!("Placeholder '{}' not found in key_map", acc.pubkey)
                    })?;
                    let pubkey = Pubkey::from_str(pubkey_str)?;
                    Ok(solana_sdk::instruction::AccountMeta {
                        pubkey,
                        is_signer: acc.is_signer,
                        is_writable: acc.is_writable,
                    })
                })
                .collect::<Result<Vec<_>>>()?;

            // The data is a placeholder "..." in the YAML. We pass an empty vector because
            // the `step` function will replace this entire instruction with a fresh one
            // from the Jupiter API. This instruction just acts as a trigger.
            Ok(Instruction::new_with_bytes(program_id, &[], accounts))
        }
        "110-JUP-LEND-SOL" => create_sol_transfer_instruction(key_map, 1_000_000_000), // 1 SOL
        "111-JUP-LEND-USDC" => create_spl_transfer_instruction(key_map, 100_000_000),  // 100 USDC
        _ => Err(anyhow!(
            "No mock instruction builder found for benchmark ID: {}",
            test_case.id
        )),
    }
}

/// Helper to create a SOL transfer instruction.
fn create_sol_transfer_instruction(
    key_map: &HashMap<String, String>,
    lamports: u64,
) -> Result<Instruction> {
    let from_pubkey_str = key_map
        .get("USER_WALLET_PUBKEY")
        .ok_or_else(|| anyhow!("Pubkey placeholder 'USER_WALLET_PUBKEY' not found in key_map"))?;
    let to_pubkey_str = key_map
        .get("RECIPIENT_WALLET_PUBKEY")
        .cloned()
        .unwrap_or_else(|| Pubkey::new_unique().to_string());

    let from_pubkey = Pubkey::from_str(from_pubkey_str)?;
    let to_pubkey = Pubkey::from_str(&to_pubkey_str)?;

    info!("[mock] Creating SOL Transfer IX:");
    info!("[mock]   From: {}", from_pubkey);
    info!("[mock]   To: {}", to_pubkey);
    info!("[mock]   Amount (lamports): {}", lamports);

    Ok(system_instruction::transfer(
        &from_pubkey,
        &to_pubkey,
        lamports,
    ))
}

/// Helper to create an SPL transfer instruction.
fn create_spl_transfer_instruction(
    key_map: &HashMap<String, String>,
    amount: u64,
) -> Result<Instruction> {
    let source_pubkey_str = key_map
        .get("USER_USDC_ATA")
        .ok_or_else(|| anyhow!("Pubkey placeholder 'USER_USDC_ATA' not found in key_map"))?;
    let authority_pubkey_str = key_map
        .get("USER_WALLET_PUBKEY")
        .ok_or_else(|| anyhow!("Pubkey placeholder 'USER_WALLET_PUBKEY' not found in key_map"))?;
    let destination_pubkey_str = key_map.get("RECIPIENT_USDC_ATA").ok_or_else(|| {
        anyhow!("Pubkey placeholder 'RECIPIENT_USDC_ATA' not found in key_map after setup")
    })?;

    let source_pubkey = Pubkey::from_str(source_pubkey_str)?;
    let destination_pubkey = Pubkey::from_str(destination_pubkey_str)?;
    let authority_pubkey = Pubkey::from_str(authority_pubkey_str)?;

    info!("[mock] Creating SPL Transfer IX:");
    info!("[mock]   Source ATA: {}", source_pubkey);
    info!("[mock]   Destination ATA: {}", destination_pubkey);
    info!("[mock]   Authority: {}", authority_pubkey);
    info!("[mock]   Amount: {}", amount);

    spl_transfer::create_instruction(
        &source_pubkey,
        &destination_pubkey,
        &authority_pubkey,
        amount,
    )
}

/// Prepares the on-chain environment for a Jupiter swap and returns the real instructions.
///
/// This function is the core of the "smart test" approach. It performs all the
/// necessary off-chain work (API calls, account pre-loading) that a real agent
/// would do, ensuring the environment is ready for a successful transaction.
pub async fn prepare_jupiter_swap(
    env: &SolanaEnv,
    _test_case: &TestCase,
    key_map: &HashMap<String, String>,
) -> Result<Vec<Instruction>> {
    const PUBLIC_RPC_URL: &str = "https://api.mainnet-beta.solana.com";

    let user_pubkey_str = key_map
        .get("USER_WALLET_PUBKEY")
        .context("USER_WALLET_PUBKEY not found")?;
    let user_pubkey = Pubkey::from_str(user_pubkey_str)?;

    // These parameters are specific to the `100-JUP-SWAP-SOL-USDC` benchmark.
    let input_mint = spl_token::native_mint::ID;
    let output_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")?;
    let amount = 100_000_000; // 0.1 SOL
    let slippage_bps = 500; // 5%

    // 1. Get quote and full instructions from the Jupiter API.
    let jupiter_client = JupiterSwapApiClient::new("https://quote-api.jup.ag/v6".to_string());
    let quote_request = QuoteRequest {
        amount,
        input_mint,
        output_mint,
        slippage_bps,
        ..Default::default()
    };
    info!("[TestSetup] Getting Jupiter quote...");
    let quote_response = jupiter_client.quote(&quote_request).await?;
    let instructions_response = jupiter_client
        .swap_instructions(&SwapRequest {
            user_public_key: user_pubkey,
            quote_response,
            config: TransactionConfig::default(),
        })
        .await?;
    info!("[TestSetup] Received instructions from Jupiter API.");

    // 2. Pre-load all required accounts into the local surfpool fork.
    let surfpool_client = SurfpoolClient::new();
    let mut all_instructions: Vec<Instruction> = instructions_response.setup_instructions;
    all_instructions.push(instructions_response.swap_instruction);
    if let Some(cleanup) = instructions_response.cleanup_instruction {
        all_instructions.push(cleanup);
    }

    let lookup_table_keys: Vec<Pubkey> = instructions_response
        .address_lookup_table_addresses
        .to_vec();

    // Load the ALTs themselves first.
    if !lookup_table_keys.is_empty() {
        let existing = env.rpc_client.get_multiple_accounts(&lookup_table_keys)?;
        let missing: Vec<Pubkey> = existing
            .iter()
            .enumerate()
            .filter_map(|(i, acc)| {
                if acc.is_none() {
                    Some(lookup_table_keys[i])
                } else {
                    None
                }
            })
            .collect();

        if !missing.is_empty() {
            info!("[TestSetup] Pre-loading {} ALT accounts...", missing.len());
            let public_rpc = RpcClient::new(PUBLIC_RPC_URL.to_string());
            let accounts_to_load = public_rpc.get_multiple_accounts(&missing)?;
            for (pubkey, acc_opt) in missing.iter().zip(accounts_to_load.iter()) {
                if let Some(acc) = acc_opt {
                    surfpool_client
                        .set_account_from_account(pubkey, acc.clone())
                        .await?;
                }
            }
        }
    }

    // Now deserialize the ALTs.
    let alt_accounts = if !lookup_table_keys.is_empty() {
        env.rpc_client
            .get_multiple_accounts(&lookup_table_keys)?
            .into_iter()
            .zip(lookup_table_keys)
            .filter_map(|(acc_opt, key)| acc_opt.map(|acc| (key, acc)))
            .map(|(key, acc)| {
                let table = AddressLookupTable::deserialize(&acc.data)?;
                Ok(AddressLookupTableAccount {
                    key,
                    addresses: table.addresses.to_vec(),
                })
            })
            .collect::<Result<Vec<_>>>()?
    } else {
        Vec::new()
    };

    // 3. Compile the message to discover all account keys.
    surfpool_client.time_travel_to_now().await?;
    let latest_blockhash = env.rpc_client.get_latest_blockhash()?;
    let message = v0::Message::try_compile(
        &user_pubkey,
        &all_instructions,
        &alt_accounts,
        latest_blockhash,
    )?;

    let static_keys = &message.account_keys;
    let alt_keys: Vec<Pubkey> = alt_accounts
        .iter()
        .flat_map(|t| t.addresses.clone())
        .collect();
    let mut all_unique_keys: Vec<Pubkey> = static_keys.iter().cloned().chain(alt_keys).collect();
    all_unique_keys.sort();
    all_unique_keys.dedup();
    info!(
        "[TestSetup] Transaction requires {} unique accounts.",
        all_unique_keys.len()
    );

    // 4. Find and load any missing accounts.
    let mut missing_keys = Vec::new();
    for chunk in all_unique_keys.chunks(100) {
        let accounts = env.rpc_client.get_multiple_accounts(chunk)?;
        for (key, acc_opt) in chunk.iter().zip(accounts.iter()) {
            if acc_opt.is_none() {
                missing_keys.push(*key);
            }
        }
    }
    missing_keys.retain(|&pk| pk != user_pubkey);

    if !missing_keys.is_empty() {
        info!(
            "[TestSetup] Pre-loading {} missing accounts...",
            missing_keys.len()
        );
        let public_rpc = RpcClient::new(PUBLIC_RPC_URL.to_string());
        let accounts_to_load = public_rpc.get_multiple_accounts(&missing_keys)?;
        for (pubkey, acc_opt) in missing_keys.iter().zip(accounts_to_load.iter()) {
            if let Some(acc) = acc_opt {
                surfpool_client
                    .set_account_from_account(pubkey, acc.clone())
                    .await?;
            } else {
                warn!(
                    "[TestSetup] Could not fetch account {} from mainnet.",
                    pubkey
                );
            }
        }
    } else {
        info!("[TestSetup] All required accounts already exist locally.");
    }

    Ok(all_instructions)
}
