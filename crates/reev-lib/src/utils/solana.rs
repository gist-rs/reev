//! Solana utilities for key handling and balance queries

use anyhow::{anyhow, Result};
use solana_client::rpc_client::RpcClient;
use solana_sdk::program_pack::Pack;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::signer::keypair::Keypair as SolanaKeypair;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use tracing::{info, warn};

/// Get a Solana keypair from ~/.config/solana/id.json
pub fn get_keypair() -> Result<SolanaKeypair> {
    // Only use default location for security
    let default_path = get_default_key_path()?;
    read_keypair_from_file(&default_path.to_string_lossy())
}

/// Get the default Solana key path
pub fn get_default_key_path() -> Result<PathBuf> {
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("Could not find home directory"))?;
    Ok(home_dir.join(".config/solana/id.json"))
}

/// Read keypair from file
pub fn read_keypair_from_file(path: &str) -> Result<Keypair> {
    let content = fs::read_to_string(path).map_err(|e| anyhow!("Failed to read key file: {e}"))?;

    if !path.ends_with(".json") {
        return Err(anyhow!("Key file must be in JSON format"));
    }

    // Parse as JSON array (standard Solana key file format)
    let json_value: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| anyhow!("Failed to parse JSON key file: {e}"))?;

    // Handle both direct array format and nested array format
    if let Some(key_array) = json_value.as_array() {
        // Check if this is a direct array of numbers (e.g., [234,171,...])
        if key_array.len() >= 64 && key_array.iter().all(|v| v.is_number()) {
            info!("Found direct array of {} numbers", key_array.len());
            let mut key_bytes = [0u8; 64];
            for (i, num) in key_array.iter().enumerate() {
                if let Some(n) = num.as_u64() {
                    key_bytes[i] = n as u8;
                } else {
                    return Err(anyhow!(
                        "Key array contains non-numeric value at position {i}"
                    ));
                }
            }

            // Use deprecated from_bytes method as try_from doesn't work with Vec<u8>
            #[allow(deprecated)]
            let solana_keypair = SolanaKeypair::from_bytes(&key_bytes)
                .map_err(|e| anyhow!("Failed to create keypair from bytes: {e}"))?;
            return Ok(solana_keypair);
        }

        // Check if this is a nested array format (e.g., [[234,171,...]])
        if let Some(key_data) = key_array.first() {
            if let Some(inner_array) = key_data.as_array() {
                if inner_array.len() >= 64 && inner_array.iter().all(|v| v.is_number()) {
                    info!("Found nested array of {} numbers", inner_array.len());
                    let mut key_bytes = [0u8; 64];
                    for (i, num) in inner_array.iter().enumerate() {
                        if let Some(n) = num.as_u64() {
                            key_bytes[i] = n as u8;
                        } else {
                            return Err(anyhow!(
                                "Key array contains non-numeric value at position {i}"
                            ));
                        }
                    }

                    // Use deprecated from_bytes method as try_from doesn't work with Vec<u8>
                    #[allow(deprecated)]
                    let solana_keypair = SolanaKeypair::from_bytes(&key_bytes)
                        .map_err(|e| anyhow!("Failed to create keypair from bytes: {e}"))?;
                    return Ok(solana_keypair);
                }
            }
        }
    }

    // If we get here, the format is not supported
    Err(anyhow!(
        "Key must be provided as an array of numbers (direct or nested), not as a string"
    ))
}

// read_keypair_from_string is no longer needed as we only load from file
// Kept for backward compatibility but returns an error
pub fn read_keypair_from_string(_key_str: &str) -> Result<SolanaKeypair> {
    Err(anyhow!("Reading keys from strings is not supported for security reasons. Use ~/.config/solana/id.json"))
}

/// Get public key as a string
pub fn get_pubkey(keypair: &SolanaKeypair) -> String {
    keypair.pubkey().to_string()
}

/// Get keypair for signing operations
pub fn get_signer() -> Result<impl solana_sdk::signature::Signer> {
    let keypair = get_keypair()?;
    Ok(keypair)
}

/// Query balance of a specific token for a wallet using SURFPOOL
/// Uses the same approach as balance_validation.rs with ATA (Associated Token Account)
pub fn query_token_balance(
    rpc_client: &RpcClient,
    wallet_pubkey: &str,
    token_mint: &str,
) -> Result<u64> {
    // Convert string pubkeys to Pubkey objects
    let wallet_pubkey =
        Pubkey::from_str(wallet_pubkey).map_err(|e| anyhow!("Invalid wallet pubkey: {e}"))?;

    let token_mint =
        Pubkey::from_str(token_mint).map_err(|e| anyhow!("Invalid token mint: {e}"))?;

    // Calculate Associated Token Account (ATA) for this token
    let ata =
        spl_associated_token_account::get_associated_token_address(&wallet_pubkey, &token_mint);

    // Query token account directly using same approach as balance_validation.rs
    match rpc_client.get_account(&ata) {
        Ok(account) => {
            // Check if the account is a valid token account
            if account.owner == spl_token::ID {
                // Parse the token account using spl-token's Account unpack
                match spl_token::state::Account::unpack(&account.data) {
                    Ok(token_account) => {
                        info!(
                            "Token account balance for mint {}: {} raw units",
                            token_mint, token_account.amount
                        );
                        Ok(token_account.amount)
                    }
                    Err(e) => {
                        warn!("Failed to parse token account: {}", e);
                        Err(anyhow!("Failed to parse token account"))
                    }
                }
            } else {
                // Account exists but is not a token account
                warn!(
                    "Account exists but is not a token account. Owner: {}",
                    account.owner
                );
                Ok(0)
            }
        }
        Err(e) => {
            // Account doesn't exist, balance is 0
            warn!("Failed to query token account: {}", e);
            Ok(0)
        }
    }
}
