use anyhow::{Context, Result};
use jup_sdk::{
    models::{DepositParams, WithdrawParams},
    surfpool::SurfpoolClient as JupSurfpoolClient,
    Jupiter,
};
use reev_lib::agent::RawInstruction;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use tracing::info;

const LOCAL_RPC_URL: &str = "http://127.0.0.1:8899";

/// Handles the logic for a Jupiter lend deposit by delegating to the `jup-sdk`.
///
/// This function mirrors the swap handler, using the `jup-sdk` to manage the
/// complexities of the Jupiter Lend API and `surfpool` environment preparation.
pub async fn handle_jupiter_deposit(
    user_pubkey: Pubkey,
    asset_mint: Pubkey,
    amount: u64,
    _key_map: &HashMap<String, String>,
) -> Result<Vec<RawInstruction>> {
    info!(
        "[reev-agent] Handling Jupiter lend deposit with jup-sdk: ASSET={}, amount={}",
        asset_mint, amount
    );

    // 1. Initialize the Jupiter client for surfpool simulation.
    let jupiter_client = Jupiter::surfpool_with_rpc(RpcClient::new(LOCAL_RPC_URL.to_string()))
        .with_user_pubkey(user_pubkey);

    let deposit_params = DepositParams { asset_mint, amount };

    // 2. Use the SDK to fetch all transaction components.
    info!("[reev-agent] Getting Jupiter transaction components via SDK...");
    let (instructions, alt_accounts) = jupiter_client
        .deposit(deposit_params)
        .prepare_transaction_components()
        .await
        .context("Failed to get Jupiter lend deposit components from jup-sdk")?;

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
        "[reev-agent] Successfully generated and prepared {} Jupiter lend deposit instructions.",
        raw_instructions.len()
    );

    Ok(raw_instructions)
}

/// Handles the logic for a Jupiter lend withdrawal by delegating to the `jup-sdk`.
pub async fn handle_jupiter_withdraw(
    user_pubkey: Pubkey,
    asset_mint: Pubkey,
    amount: u64,
    _key_map: &HashMap<String, String>,
) -> Result<Vec<RawInstruction>> {
    info!(
        "[reev-agent] Handling Jupiter lend withdrawal with jup-sdk: ASSET={}, amount={}",
        asset_mint, amount
    );

    let jupiter_client = Jupiter::surfpool_with_rpc(RpcClient::new(LOCAL_RPC_URL.to_string()))
        .with_user_pubkey(user_pubkey);

    let withdraw_params = WithdrawParams { asset_mint, amount };

    info!("[reev-agent] Getting Jupiter transaction components via SDK...");
    let (instructions, alt_accounts) = jupiter_client
        .withdraw(withdraw_params)
        .prepare_transaction_components()
        .await
        .context("Failed to get Jupiter lend withdraw components from jup-sdk")?;

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

    let raw_instructions: Vec<RawInstruction> =
        instructions.into_iter().map(|ix| ix.into()).collect();
    info!(
        "[reev-agent] Successfully generated and prepared {} Jupiter lend withdraw instructions.",
        raw_instructions.len()
    );

    Ok(raw_instructions)
}
