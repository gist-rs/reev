//! Solana utilities for key handling

use anyhow::{anyhow, Result};
use solana_sdk::signature::{Keypair, Signer};
use std::fs;
use std::path::{Path, PathBuf};

/// Source of the Solana private key
#[derive(Debug, Clone)]
pub enum KeySource {
    /// From environment variable (direct key string)
    Environment(String),
    /// From a file path
    FilePath(PathBuf),
    /// Default location: ~/.config/solana/id.json
    DefaultPath,
}

/// Get the Solana keypair from various sources
pub fn get_keypair() -> Result<Keypair> {
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
        let default_path = get_default_key_path();
        read_keypair_from_file(&default_path.to_string_lossy())
    }
}

/// Get the default Solana key path
pub fn get_default_key_path() -> PathBuf {
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("Could not find home directory"))?;
    Ok(home_dir.join(".config/solana/id.json"))
}

/// Read keypair from file
pub fn read_keypair_from_file(path: &str) -> Result<Keypair> {
    let content =
        fs::read_to_string(path).map_err(|e| anyhow!("Failed to read key file: {}", e))?;

    if path.ends_with(".json") {
        // Try to parse as JSON
        serde_json::from_str(&content).map_err(|e| anyhow!("Failed to parse JSON key file: {}", e))
    } else {
        // Try to parse as base58 encoded string
        let key_bytes =
            bs58::decode(content).map_err(|e| anyhow!("Failed to decode base58 key: {}", e))?;

        if key_bytes.len() < 64 {
            return Err(anyhow!("Invalid key length"));
        }

        let mut key_array = [0u8; 64];
        key_array.copy_from_slice(&key_bytes[..64]);

        Keypair::from_bytes(&key_array).map_err(|e| anyhow!("Failed to create keypair: {}", e))
    }
}

/// Read keypair from string
pub fn read_keypair_from_string(key_str: &str) -> Result<Keypair> {
    // Try to parse as JSON first
    if key_str.trim().starts_with('{') {
        serde_json::from_str(key_str).map_err(|e| anyhow!("Failed to parse JSON key string: {}", e))
    } else {
        // Try to parse as base58 encoded string
        let key_bytes =
            bs58::decode(key_str).map_err(|e| anyhow!("Failed to decode base58 key: {}", e))?;

        if key_bytes.len() < 64 {
            return Err(anyhow!("Invalid key length"));
        }

        let mut key_array = [0u8; 64];
        key_array.copy_from_slice(&key_bytes[..64]);

        Keypair::from_bytes(&key_array).map_err(|e| anyhow!("Failed to create keypair: {}", e))
    }
}

/// Get the public key as a string
pub fn get_pubkey(keypair: &Keypair) -> String {
    keypair.pubkey().to_string()
}

/// Get the public key from environment or default
pub fn get_pubkey_from_env() -> Result<String> {
    let keypair = get_keypair()?;
    Ok(get_pubkey(&keypair))
}
