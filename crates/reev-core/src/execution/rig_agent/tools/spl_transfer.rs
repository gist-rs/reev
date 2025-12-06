//! SPL Token Transfer Tool Implementation
//!
//! This module contains the implementation of the SPL token transfer tool.

use anyhow::{anyhow, Result};
use reev_protocols::native::handle_spl_transfer;
use reev_types::flow::WalletContext;
use solana_sdk::pubkey::Pubkey;
use spl_associated_token_account::get_associated_token_address;
use std::collections::HashMap;
use std::str::FromStr;
use tracing::info;

use super::tool_params::SplTransferParams;
use super::tool_results::{SplTransferResult, ToolResult};

/// Execute SPL token transfer
pub async fn execute_spl_transfer(
    params: &HashMap<String, String>,
    wallet_context: &WalletContext,
) -> Result<ToolResult> {
    // Parse parameters into typed struct
    let transfer_params = SplTransferParams::from_hashmap(params)?;

    // Convert mint address string to Pubkey
    let token_mint = Pubkey::from_str(&transfer_params.mint_address)
        .map_err(|e| anyhow!("Invalid mint address: {e}"))?;

    // Parse amount (convert to token units)
    let amount = if transfer_params.amount.to_lowercase() == "all" {
        // For "all" keyword, we'll transfer the entire balance
        u64::MAX
    } else {
        // Parse the amount and convert to token units based on decimals
        let amount_value: f64 = transfer_params
            .amount
            .parse()
            .map_err(|_| anyhow!("Invalid amount: {}", transfer_params.amount))?;

        // Get decimals for token (default to 6 for common SPL tokens)
        let decimals = get_token_decimals_from_mint(&transfer_params.mint_address);

        (amount_value * 10_f64.powi(decimals)) as u64
    };

    // Parse recipient and sender pubkeys
    let recipient_pubkey = Pubkey::from_str(&transfer_params.recipient)
        .map_err(|e| anyhow!("Invalid recipient address: {e}"))?;
    let sender_pubkey = Pubkey::from_str(&wallet_context.owner)
        .map_err(|e| anyhow!("Invalid sender address: {e}"))?;

    // Get or create associated token accounts
    let (source_ata, destination_ata) =
        get_or_create_token_accounts(sender_pubkey, recipient_pubkey, token_mint).await?;

    // Use the protocol handler to create the transfer instruction
    let instructions = handle_spl_transfer(
        source_ata,
        destination_ata,
        sender_pubkey, // Authority is the wallet owner
        amount,
        &HashMap::new(), // Empty key_map for now
    )
    .await?;

    // Get the default keypair for signing
    let keypair = reev_lib::get_keypair().map_err(|e| anyhow!("Failed to get keypair: {e}"))?;

    // Execute the transaction
    let transaction_signature =
        reev_lib::execute_transaction(instructions, sender_pubkey, &keypair)
            .await
            .map_err(|e| anyhow!("Failed to execute transaction: {e}"))?;

    // Log the successful transaction
    info!(
        "SPL transfer executed with signature: {}",
        transaction_signature
    );

    Ok(ToolResult::SplTransfer(SplTransferResult {
        tool_name: "spl_transfer".to_string(),
        recipient: transfer_params.recipient.clone(),
        amount: transfer_params.amount.clone(),
        mint_address: transfer_params.mint_address.clone(),
        token_mint: token_mint.to_string(),
        wallet: wallet_context.owner.clone(),
        transaction_signature: Some(transaction_signature),
        success: true,
        error: None,
    }))
}

/// Get decimals for token
// get_token_decimals is now unused since we handle decimals from mint addresses
fn get_token_decimals_from_mint(mint_address: &str) -> i32 {
    match mint_address {
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" => 6, // USDC
        "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB" => 6, // USDT
        _ => 6,                                              // Default to 6 decimals
    }
}

/// Get or create associated token accounts for sender and recipient
async fn get_or_create_token_accounts(
    sender: Pubkey,
    recipient: Pubkey,
    token_mint: Pubkey,
) -> Result<(Pubkey, Pubkey)> {
    use solana_client::nonblocking::rpc_client::RpcClient;
    use solana_sdk::signature::Signer;
    use spl_associated_token_account::instruction::create_associated_token_account;

    // Get sender's ATA (Associated Token Account)
    let sender_ata = get_associated_token_address(&sender, &token_mint);

    // Get recipient's ATA
    let recipient_ata = get_associated_token_address(&recipient, &token_mint);

    // Create RPC client to check if recipient's ATA exists
    let rpc_client = RpcClient::new("http://localhost:8899".to_string());

    // Check if recipient's ATA exists
    let account_exists = rpc_client.get_account(&recipient_ata).await.is_ok();

    if !account_exists {
        // Create recipient's ATA
        info!("Creating recipient ATA: {}", recipient_ata);

        // Get keypair for signing
        let keypair = reev_lib::get_keypair().map_err(|e| anyhow!("Failed to get keypair: {e}"))?;

        // Create instruction to create ATA
        let create_ata_ix = create_associated_token_account(
            &keypair.pubkey(),
            &recipient,
            &token_mint,
            &spl_token::id(),
        );

        // Execute creation instruction
        reev_lib::execute_transaction(vec![create_ata_ix.into()], keypair.pubkey(), &keypair)
            .await
            .map_err(|e| anyhow!("Failed to create recipient ATA: {e}"))?;

        info!("✅ Successfully created recipient ATA: {}", recipient_ata);
    }

    Ok((sender_ata, recipient_ata))
}

/// Execute SPL token transfer using typed parameters
pub async fn execute_spl_transfer_with_params(
    params: &SplTransferParams,
    wallet_context: &WalletContext,
) -> Result<ToolResult> {
    // Convert the typed params back to HashMap to reuse the main function
    let mut params_map = HashMap::new();
    params_map.insert("recipient".to_string(), params.recipient.clone());
    params_map.insert("amount".to_string(), params.amount.clone());
    params_map.insert("mint_address".to_string(), params.mint_address.clone());

    execute_spl_transfer(&params_map, wallet_context).await
}
