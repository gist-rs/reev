use anyhow::{anyhow, Result};

/// Converts a 32-byte hex string (like a Pyth FeedId) into a base58 encoded string.
///
/// The input hex string can be 64 characters long or 66 characters with a "0x" prefix.
///
/// # Arguments
///
/// * `hex_str` - A string slice that holds the hex string.
///
/// # Returns
///
/// A `Result` containing the base58 encoded string on success, or an error if the
/// input is invalid.
///
/// # Example
///
/// ```
/// use crate::utils::hex_to_base58;
/// let hex_feed_id = "0xe62df6c8b4a85fe1a67db44dc12de5db330f7ac66b72dc658afedf0f4a415b43";
/// let base58_address = hex_to_base58(hex_feed_id).unwrap();
/// assert_eq!(base58_address, "H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712K4AQJEG");
/// ```
pub fn hex_to_base58(hex_str: &str) -> Result<String> {
    let hex_to_decode = hex_str.strip_prefix("0x").unwrap_or(hex_str);

    if hex_to_decode.len() != 64 {
        return Err(anyhow!(
            "Hex string must be 64 characters long (excluding '0x' prefix)"
        ));
    }

    let bytes = hex::decode(hex_to_decode)?;
    if bytes.len() != 32 {
        return Err(anyhow!("Decoded hex string must be 32 bytes long"));
    }

    let base58_string = bs58::encode(bytes).into_string();
    Ok(base58_string)
}
