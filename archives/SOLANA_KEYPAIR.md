# Solana Keypair Handling

This document explains how the Reev system handles Solana keypairs for authentication and signing operations.

## Overview

The system supports three methods for providing Solana keypairs:

1. Direct private key string via `SOLANA_PRIVATE_KEY` environment variable
2. File path to a keyfile via `SOLANA_PRIVATE_KEY` environment variable
3. Default location: `~/.config/solana/id.json` (used when environment variable is not set)

## Implementation Details

### Environment Variable Configuration

The `SOLANA_PRIVATE_KEY` environment variable can accept:

1. **Direct Private Key**: A base58-encoded private key string
2. **File Path**: A path to a file containing the private key

If `SOLANA_PRIVATE_KEY` is not set, the system will automatically check the default Solana key location at `~/.config/solana/id.json`.

### Key File Formats

The system supports two key file formats:

1. **JSON Format** (standard Solana format):
   ```json
   ["base58_encoded_private_key"]
   ```

2. **Raw Base58 Format**:
   ```
   base58_encoded_private_key
   ```

### Implementation

The keypair handling is implemented in `reev-core/src/utils/solana.rs` with the following functions:

- `get_keypair()`: Main function that determines which method to use
- `read_keypair_from_file()`: Reads a keypair from a file (JSON or raw format)
- `read_keypair_from_string()`: Reads a keypair from a base58 string
- `get_default_key_path()`: Returns the default Solana key file path

### Usage Examples

#### Using a Direct Private Key

```bash
# Set the environment variable with a direct private key
export SOLANA_PRIVATE_KEY="SomeBase64ofYourPrivateKeyGoHere"
```

#### Using a File Path

```bash
# Set the environment variable with a file path, default to `~/.config/solana/id.json`
export SOLANA_PRIVATE_KEY="/path/to/my/keyfile.json"
```

#### Using Default Location (No Environment Variable)

```bash
# Don't set SOLANA_PRIVATE_KEY, system will use ~/.config/solana/id.json
unset SOLANA_PRIVATE_KEY
```

## Security Considerations

1. **Never commit private keys to version control**
2. **Use file permissions to restrict access to key files**: `chmod 600 keyfile.json`
3. **Consider using a secure key management system for production deployments**
4. **Be aware that environment variables can be accessed by other processes**

## Testing

The keypair handling is tested in `tests/solana/solana_key_test.rs` with tests covering:

- Reading keypairs from direct strings
- Reading keypairs from files (both JSON and raw formats)
- Fallback to default location when environment variable is not set
- Error handling for invalid inputs

## Integration

The keypair handling is integrated with the rest of the system through the `get_signer()` function, which provides a `Signer` trait implementation for signing operations.
