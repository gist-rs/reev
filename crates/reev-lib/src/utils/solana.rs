//! Solana utilities for key handling and balance queries

use anyhow::{anyhow, Result};
use solana_client::rpc_client::RpcClient;
use solana_sdk::program_pack::Pack;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::signer::keypair::Keypair as SolanaKeypair;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tracing::{info, warn};

/// Source of Solana private key
#[derive(Debug, Clone)]
pub enum KeySource {
    /// From environment variable (direct key string or file path)
    Environment(String),
    /// Default location: ~/.config/solana/id.json
    DefaultPath,
}

/// Get a Solana keypair from various sources
pub fn get_keypair() -> Result<SolanaKeypair> {
    // First try to get from environment variable
    if let Ok(key_str) = std::env::var("SOLANA_PRIVATE_KEY") {
        // Check if it looks like a file path (doesn't contain spaces and exists)
        if !key_str.contains(' ') && Path::new(&key_str).exists() {
            // Treat as file path
            read_keypair_from_file(&key_str)
        } else {
            // Treat as direct key string
            read_keypair_from_string(&key_str)
        }
    } else {
        // Use default location
        let default_path = get_default_key_path()?;
        read_keypair_from_file(&default_path.to_string_lossy())
    }
}

/// Get the default Solana key path
pub fn get_default_key_path() -> Result<PathBuf> {
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("Could not find home directory"))?;
    Ok(home_dir.join(".config/solana/id.json"))
}

/// Read keypair from file
pub fn read_keypair_from_file(path: &str) -> Result<Keypair> {
    let content = fs::read_to_string(path).map_err(|e| anyhow!("Failed to read key file: {e}"))?;

    if path.ends_with(".json") {
        // Try to parse as JSON array (standard Solana key file format)
        let json_value: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| anyhow!("Failed to parse JSON key file: {e}"))?;

        if let Some(array) = json_value.as_array() {
            if let Some(key_data) = array.first() {
                if let Some(key_str) = key_data.as_str() {
                    // Try to parse as base58
                    let key_bytes = bs58::decode(key_str)
                        .into_vec()
                        .map_err(|e| anyhow!("Failed to decode base58 key: {e}"))?;

                    // Try to create keypair from bytes using deprecated from_bytes method
                    // We need to use this for now due to compatibility with existing code
                    #[allow(deprecated)]
                    let solana_keypair = SolanaKeypair::from_bytes(&key_bytes)
                        .map_err(|e| anyhow!("Failed to create keypair from bytes: {e}"))?;
                    Ok(solana_keypair)
                } else {
                    Err(anyhow!("Invalid key format in JSON file"))
                }
            } else {
                Err(anyhow!("Empty key array in JSON file"))
            }
        } else {
            Err(anyhow!("Invalid JSON key file format, expected array"))
        }
    } else {
        // Read the file content and decode as base58
        let content = fs::read_to_string(path)?;
        let key_bytes = bs58::decode(content.trim())
            .into_vec()
            .map_err(|e| anyhow!("Failed to decode base58 key: {e}"))?;

        // Create keypair directly from decoded bytes
        // Convert to array of expected size (64 bytes for Ed25519 keypair)
        let mut key_array = [0u8; 64];
        if key_bytes.len() >= 64 {
            key_array.copy_from_slice(&key_bytes[..64]);
        } else {
            // If we have less than 64 bytes, pad with zeros
            key_array[..key_bytes.len()].copy_from_slice(&key_bytes);
        }

        // Use the deprecated from_bytes method as try_from doesn't work with Vec<u8>
        #[allow(deprecated)]
        let solana_keypair = SolanaKeypair::from_bytes(&key_array)
            .map_err(|e| anyhow!("Failed to create Solana keypair: {e}"))?;
        Ok(solana_keypair)
    }
}

/// Read keypair from string
pub fn read_keypair_from_string(key_str: &str) -> Result<SolanaKeypair> {
    // Try to parse as JSON first
    if key_str.trim().starts_with('{') {
        Err(anyhow!("Direct JSON key strings are not supported"))
    } else {
        // Try to parse as base58 encoded string
        let key_bytes = bs58::decode(key_str)
            .into_vec()
            .map_err(|e| anyhow!("Failed to decode base58 key: {e}"))?;

        // Create keypair directly from decoded bytes
        // Convert to array of expected size (64 bytes for Ed25519 keypair)
        let mut key_array = [0u8; 64];
        if key_bytes.len() >= 64 {
            key_array.copy_from_slice(&key_bytes[..64]);
        } else {
            // If we have less than 64 bytes, pad with zeros
            key_array[..key_bytes.len()].copy_from_slice(&key_bytes);
        }

        // Use the deprecated from_bytes method as try_from doesn't work with Vec<u8>
        #[allow(deprecated)]
        let solana_keypair = SolanaKeypair::from_bytes(&key_array)
            .map_err(|e| anyhow!("Failed to create Solana keypair: {e}"))?;
        Ok(solana_keypair)
    }
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
