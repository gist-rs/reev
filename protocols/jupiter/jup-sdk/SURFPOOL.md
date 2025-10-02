# Surfpool: High-Performance Solana Testnet

## Introduction

`surfpool` is a high-performance, in-memory Solana testnet, analogous to what `anvil` is for the Ethereum ecosystem. It provides a lightweight and blazing-fast local simulation of the Solana Mainnet, designed for development, testing, and debugging.

One of its most powerful features is its ability to **point-fork Solana mainnet instantly**.

## Key Feature: Mainnet Forking

`surfpool` operates as a *real* mainnet fork. When a transaction requires access to an account that does not exist in the local, in-memory state, `surfpool` will **dynamically fetch the account data from Solana Mainnet on-demand**.

This capability is revolutionary for testing because it allows developers to:
-   Interact with real, deployed mainnet programs without redeploying them.
-   Test against actual on-chain state (e.g., liquidity pools, user accounts) without manual setup or mocking.
-   Ensure that tests accurately reflect how transactions will behave in a live environment.

## RPC Cheat Codes for State Manipulation

To facilitate advanced testing scenarios, `surfpool` exposes a special set of JSON-RPC methods under the `surfnet_*` namespace. These "cheat codes" allow for direct and programmatic manipulation of the blockchain's state, bypassing the normal constraints of transactions.

### `surfnet_setAccount`

Directly sets or updates the properties of any account. This is a low-level tool for modifying lamports, owner, data, and other account fields.

**Method:** `surfnet_setAccount`

**Parameters:**
1.  `pubkey` (string): The base-58 encoded public key of the account to modify.
2.  `update` (object): An object containing the fields to update.
    -   `lamports` (u64, optional)
    -   `owner` (string, optional): Base-58 encoded pubkey of the new owner.
    -   `executable` (boolean, optional)
    -   `rent_epoch` (u64, optional)
    -   `data` (string, optional): Hex-encoded account data.

**Example:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "surfnet_setAccount",
  "params": [
    "9WzDXwBbmkg8ZTbNMq1a1ePz8b25k52u1d4a8c6f7g3h",
    {
      "lamports": 1000000000,
      "owner": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
    }
  ]
}
```

### `surfnet_setTokenAccount`

A specialized and highly useful cheat code for creating or updating an SPL token account. It automatically handles the creation of the Associated Token Account (ATA) if it doesn't exist and sets its balance. This is the primary method for funding test wallets with SPL tokens like USDC.

**Method:** `surfnet_setTokenAccount`

**Parameters:**
1.  `owner` (string): The base-58 encoded public key of the wallet that owns the token account.
2.  `mint` (string): The base-58 encoded public key of the token's mint address (e.g., USDC mint).
3.  `update` (object): An object containing the token account fields to update.
    -   `amount` (u64): The raw token amount (e.g., for USDC with 6 decimals, `100_000_000` is 100 USDC).
4.  `token_program` (string, optional): The token program ID. Defaults to the standard SPL Token program.

**Example (Setting a user's USDC balance to 100):**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "surfnet_setTokenAccount",
  "params": [
    "5HUz9qfHhFGAL8Y3QHqTELfTjyhKSNMGSNGY782p26bw", // User's wallet address
    "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC Mint address
    {
      "amount": 100000000
    },
    "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
  ]
}
```

### Time Manipulation

`surfpool` provides cheat codes to control the passage of time in the simulation, which is essential for testing time-sensitive on-chain logic.

-   **`surfnet_timeTravel`**: Jumps the blockchain to a specific slot, epoch, or Unix timestamp.
-   **`surfnet_pauseClock`**: Halts the progression of slots.
-   **`surfnet_resumeClock`**: Resumes the progression of slots after a pause.

## Practical Usage in `reev`

The `reev` testing framework leverages these cheat codes to set up the precise initial conditions required by each benchmark. For example, before running a benchmark that involves swapping USDC, a test setup script will:

1.  Start the `surfpool` instance.
2.  Make an RPC call to `surfnet_setTokenAccount`.
3.  Fund the test user's wallet with a specific amount of USDC.

This ensures that the environment is in a known, controlled state before the agent attempts to solve the benchmark, making the tests reliable and deterministic.