use anyhow::{Context, Result};
use jup_sdk::{models::SwapParams, surfpool::SurfpoolClient as JupSurfpoolClient, Jupiter};
use reev_lib::agent::RawInstruction;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use tracing::info;

const LOCAL_RPC_URL: &str = "http://127.0.0.1:8899";

/// Handles the logic for a Jupiter swap by delegating to the `jup-sdk`.
///
/// This function now acts as a client to the `jup-sdk`, which encapsulates all
/// the complex logic of interacting with the Jupiter API and preparing the
/// `surfpool` environment.
///
/// The process is as follows:
/// 1. Initialize the `jup-sdk`'s `Jupiter` client, configured for `surfpool`.
/// 2. Use the SDK to fetch all the necessary transaction components (instructions and ALTs).
/// 3. Use the SDK's `preload_accounts` utility to load all required accounts from mainnet
///    into the local fork.
/// 4. Convert the final `solana_sdk::Instruction` objects into `RawInstruction` for the runner.
pub async fn handle_jupiter_swap(
    user_pubkey: Pubkey,
    input_mint: Pubkey,
    output_mint: Pubkey,
    amount: u64,
    slippage_bps: u16,
    _key_map: &HashMap<String, String>,
) -> Result<Vec<RawInstruction>> {
    info!(
        "[reev-agent] Handling Jupiter swap with jup-sdk: IN={}, OUT={}, amount={}",
        input_mint, output_mint, amount
    );

    // 1. Initialize the Jupiter client for surfpool simulation.
    let jupiter_client = Jupiter::surfpool_with_rpc(RpcClient::new(LOCAL_RPC_URL.to_string()))
        .with_user_pubkey(user_pubkey);

    let swap_params = SwapParams {
        input_mint,
        output_mint,
        amount,
        slippage_bps,
    };

    // 2. Use the SDK to fetch all transaction components. The SDK handles the API calls.
    info!("[reev-agent] Getting Jupiter transaction components via SDK...");
    let (instructions, alt_accounts) = jupiter_client
        .swap(swap_params)
        .prepare_transaction_components()
        .await
        .context("Failed to get Jupiter swap components from jup-sdk")?;

    // 3. Use the SDK to preload all necessary accounts into surfpool.
    info!("[reev-agent] Starting account pre-loading process via SDK...");
    let surfpool_client = JupSurfpoolClient::new(LOCAL_RPC_URL);
    let local_rpc_client = RpcClient::new(LOCAL_RPC_URL.to_string());
    jup_sdk::surfpool::preload_accounts(
        &local_rpc_client,
        &surfpool_client,
        &user_pubkey,
        &instructions,
        &alt_accounts,
    )
    .await
    .context("jup-sdk failed to preload accounts")?;

    // 4. Convert to RawInstruction for the runner.
    let raw_instructions: Vec<RawInstruction> =
        instructions.into_iter().map(|ix| ix.into()).collect();
    info!(
        "[reev-agent] Successfully generated and prepared {} Jupiter swap instructions.",
        raw_instructions.len()
    );

    Ok(raw_instructions)
}
