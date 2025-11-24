//! Solana utilities for key handling

use anyhow::{anyhow, Result};
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::signer::keypair::Keypair as SolanaKeypair;
use std::fs;
use std::path::{Path, PathBuf};

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

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use solana_sdk::signer::keypair::Keypair as SolanaKeypair;
    use std::env;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    #[serial]
    fn test_get_default_key_path() {
        let default_path = get_default_key_path().unwrap();
        let expected = dirs::home_dir().unwrap().join(".config/solana/id.json");
        assert_eq!(default_path, expected);
    }

    #[test]
    #[serial]
    fn test_read_keypair_from_string() {
        // Generate a new keypair for testing
        let test_keypair = SolanaKeypair::new();
        let test_key = test_keypair.to_base58_string();
        let expected_pubkey = test_keypair.pubkey();

        let keypair = read_keypair_from_string(&test_key);
        if let Err(ref e) = keypair {
            println!("Error reading keypair: {e}");
        }
        assert!(keypair.is_ok(), "Should successfully parse base58 key");

        // Verify the public key matches
        let pubkey = keypair.unwrap().pubkey();
        assert_eq!(pubkey, expected_pubkey);
    }

    #[test]
    #[serial]
    fn test_read_keypair_from_json_file() {
        // Create a temporary directory for our test file
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_keypair.json");

        // Create a test keypair and write it to file in JSON format
        let test_keypair = SolanaKeypair::new();
        let pubkey = test_keypair.pubkey();

        // Standard Solana key file format is JSON array with the secret key as base58
        let key_base58 = test_keypair.to_base58_string();
        let json_content = format!(r#"[{}]"#, serde_json::to_string(&key_base58).unwrap());

        fs::write(&file_path, json_content).unwrap();

        // Read the keypair from the file
        let loaded_keypair = read_keypair_from_file(file_path.to_str().unwrap()).unwrap();

        // Verify the public key matches
        assert_eq!(loaded_keypair.pubkey(), pubkey);
    }

    #[test]
    #[serial]
    fn test_read_keypair_from_raw_file() {
        // Create a temporary directory for our test file
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_keypair.txt");

        // Create a test keypair and write it to file as raw base58
        let test_keypair = SolanaKeypair::new();
        let pubkey = test_keypair.pubkey();

        // Write the secret key as raw base58
        let key_base58 = test_keypair.to_base58_string();
        fs::write(&file_path, key_base58).unwrap();

        // Read the keypair from the file
        let loaded_keypair = read_keypair_from_file(file_path.to_str().unwrap()).unwrap();

        // Verify the public key matches
        assert_eq!(loaded_keypair.pubkey(), pubkey);
    }

    #[test]
    #[serial]
    fn test_get_keypair_from_env_string() {
        // Save the original environment variable
        let original_key = env::var("SOLANA_PRIVATE_KEY");

        // Create a test keypair and use its secret
        let test_keypair = SolanaKeypair::new();
        let test_key = test_keypair.to_base58_string();
        let expected_pubkey = test_keypair.pubkey();

        // Set the test key as the environment variable
        env::set_var("SOLANA_PRIVATE_KEY", &test_key);

        // Get the keypair
        let keypair = get_keypair();

        // Restore the original environment variable
        match original_key {
            Ok(key) => env::set_var("SOLANA_PRIVATE_KEY", key),
            Err(_) => env::remove_var("SOLANA_PRIVATE_KEY"),
        }

        // Verify the keypair was loaded correctly
        assert!(keypair.is_ok(), "Should successfully load keypair from env");
        let pubkey = keypair.unwrap().pubkey();
        assert_eq!(pubkey, expected_pubkey);
    }

    #[test]
    #[serial]
    fn test_get_keypair_from_env_file_path() {
        // Create a temporary directory for our test file
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_keypair.json");

        // Create a test keypair and write it to file
        let test_keypair = SolanaKeypair::new();
        let pubkey = test_keypair.pubkey();

        // Write the keypair to file in standard Solana JSON format
        let key_base58 = test_keypair.to_base58_string();
        let json_content = format!(r#"[{}]"#, serde_json::to_string(&key_base58).unwrap());
        fs::write(&file_path, json_content).unwrap();

        // Save the original environment variable
        let original_key = env::var("SOLANA_PRIVATE_KEY");

        // Set the file path as the environment variable
        env::set_var("SOLANA_PRIVATE_KEY", file_path.to_str().unwrap());

        // Get the keypair
        let keypair = get_keypair();

        // Restore the original environment variable
        match original_key {
            Ok(key) => env::set_var("SOLANA_PRIVATE_KEY", key),
            Err(_) => env::remove_var("SOLANA_PRIVATE_KEY"),
        }

        // Verify the keypair was loaded correctly
        assert!(
            keypair.is_ok(),
            "Should successfully load keypair from file path"
        );
        let loaded_pubkey = keypair.unwrap().pubkey();
        assert_eq!(loaded_pubkey, pubkey);
    }

    #[test]
    #[serial]
    fn test_get_keypair_fallback_to_default() {
        // Save the original environment variable
        let original_key = env::var("SOLANA_PRIVATE_KEY");

        // Remove the environment variable
        env::remove_var("SOLANA_PRIVATE_KEY");

        // Create a temporary directory for our default key file
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join(".config/solana/id.json");

        // Create the directory structure
        fs::create_dir_all(file_path.parent().unwrap()).unwrap();

        // Create a test keypair and write it to the default location
        let test_keypair = SolanaKeypair::new();
        let pubkey = test_keypair.pubkey();

        // Write the keypair to file in standard Solana JSON format
        let key_base58 = test_keypair.to_base58_string();
        let json_content = format!(r#"[{}]"#, serde_json::to_string(&key_base58).unwrap());
        fs::write(&file_path, json_content).unwrap();

        // Temporarily modify the home directory to point to our temp directory
        let original_home = env::var("HOME");
        env::set_var("HOME", temp_dir.path());

        // Get the keypair (should fall back to default location)
        let keypair = get_keypair();

        // Restore the original environment variables
        match original_home {
            Ok(home) => env::set_var("HOME", home),
            Err(_) => env::remove_var("HOME"),
        }

        match original_key {
            Ok(key) => env::set_var("SOLANA_PRIVATE_KEY", key),
            Err(_) => env::remove_var("SOLANA_PRIVATE_KEY"),
        }

        // Verify the keypair was loaded correctly
        assert!(
            keypair.is_ok(),
            "Should successfully load keypair from default path"
        );
        let loaded_pubkey = keypair.unwrap().pubkey();
        assert_eq!(loaded_pubkey, pubkey);
    }

    #[test]
    #[serial]
    fn test_get_keypair_no_key_available() {
        // Save the original environment variables
        let original_key = env::var("SOLANA_PRIVATE_KEY");
        let original_home = env::var("HOME");

        // Remove the environment variables
        env::remove_var("SOLANA_PRIVATE_KEY");

        // Create a temporary directory with no Solana key file
        let temp_dir = TempDir::new().unwrap();
        env::set_var("HOME", temp_dir.path());

        // Try to get the keypair (should fail)
        let result = get_keypair();

        // Restore the original environment variables
        match original_home {
            Ok(home) => env::set_var("HOME", home),
            Err(_) => env::remove_var("HOME"),
        }

        match original_key {
            Ok(key) => env::set_var("SOLANA_PRIVATE_KEY", key),
            Err(_) => env::remove_var("SOLANA_PRIVATE_KEY"),
        }

        // Verify the operation failed
        assert!(result.is_err(), "Should fail when no key is available");
    }
}
