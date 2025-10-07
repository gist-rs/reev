use anyhow::Result;
use jup_sdk::{models::DepositParams, Jupiter};
use reev_lib::agent::RawInstruction;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use tracing::{debug, info};

/// Executes a Jupiter redeem operation, which is essentially the same as a withdraw
/// but specifically for redeeming jTokens back to underlying tokens. This follows the same pattern as lend_withdraw.
pub async fn execute_jupiter_lend_redeem(
    asset: &Pubkey,
    shares: u64,
    key_map: &std::collections::HashMap<String, String>,
) -> Result<Vec<RawInstruction>> {
    info!(
        "Executing Jupiter lend redeem: {} (shares: {})",
        asset, shares
    );

    // Get user pubkey from key_map
    let user_pubkey = if let Some(pubkey_str) = key_map.get("USER_WALLET_PUBKEY") {
        Pubkey::from_str(pubkey_str)
            .map_err(|e| anyhow::anyhow!("Invalid USER_WALLET_PUBKEY: {e}"))?
    } else {
        return Err(anyhow::anyhow!("USER_WALLET_PUBKEY not found in key_map"));
    };

    // Create redeem params - redeeming jTokens is essentially withdrawing underlying tokens
    let redeem_params = DepositParams {
        asset_mint: *asset,
        amount: shares, // Use shares as the amount (1:1 for jTokens)
    };

    // The jup-sdk's client is designed to work with a local validator.
    let jupiter_client = Jupiter::surfpool().with_user_pubkey(user_pubkey);

    info!("[lend_redeem] Calling Jupiter SDK for redeem instructions");
    debug!("[lend_redeem] Asset: {}, Amount: {}", asset, shares);

    // The sdk's redeem builder will handle instruction generation
    // against the local surfpool instance (same as lend_withdraw)
    let (jupiter_sdk_instructions, _alt_accounts) = jupiter_client
        .redeem(redeem_params)
        .prepare_transaction_components()
        .await?;

    debug!(
        "[lend_redeem] Generated {} instructions from Jupiter",
        jupiter_sdk_instructions.len()
    );

    // Convert Jupiter SDK instructions to RawInstruction format with Base58 encoding
    let raw_instructions: Vec<RawInstruction> = jupiter_sdk_instructions
        .into_iter()
        .map(|inst| {
            let accounts = inst
                .accounts
                .into_iter()
                .map(|acc| reev_lib::agent::RawAccountMeta {
                    pubkey: acc.pubkey.to_string(),
                    is_signer: acc.is_signer,
                    is_writable: acc.is_writable,
                })
                .collect();

            RawInstruction {
                program_id: inst.program_id.to_string(),
                accounts,
                data: bs58::encode(inst.data).into_string(), // ✅ Convert bytes to Base58
            }
        })
        .collect();

    info!(
        "[lend_redeem] Successfully converted {} instructions to RawInstruction format with Base58 encoding",
        raw_instructions.len()
    );

    Ok(raw_instructions)
}
