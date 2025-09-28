# Plan: Jupiter Swap Simulation with Surfpool

Our objective is to leverage `surfpool` to create a high-fidelity testing environment for Jupiter swaps, aiming to simulate mainnet conditions as accurately as possible. This will help us catch potential issues before they reach production.

We will establish two distinct testing methodologies:

1.  **Fast Pre-flight Check (Stops at Signature Verification):**
    *   **Goal:** Quickly validate the Jupiter API integration and transaction structure.
    *   **Process:**
        *   Fetch a quote from the Jupiter API.
        *   Request the serialized transaction (`swap_transaction`).
        *   Deserialize the transaction to a `VersionedTransaction`.
        *   Attempt to sign with a `NullSigner`.
        *   Send to the public mainnet RPC.
    *   **Expected Outcome:** The transaction will fail with a `Transaction signature verification failure`. This confirms the transaction is well-formed up to the point of signing.
    *   **Benefit:** This is a rapid check that doesn't require a running `surfpool` instance.

2.  **High-Fidelity Simulation (Full Surfpool Execution):**
    *   **Goal:** Execute a swap transaction in a forked mainnet environment to verify its on-chain behavior.
    *   **Process:**
        1.  Start a local `surfpool` validator, forking mainnet.
        2.  Create a temporary `Keypair` to act as the user wallet.
        3.  Use `surfnet_*` cheat codes to fund this wallet with SOL (for gas) and the input SPL token (e.g., USDC).
        4.  Fetch a quote and **swap instructions** (not the serialized transaction) from the Jupiter API.
        5.  Use the local `surfpool` RPC to fetch necessary on-chain data:
            *   A recent blockhash.
            *   Address Lookup Table (ALT) accounts.
        6.  Build the `VersionedTransaction` locally using the fetched instructions, blockhash, and ALTs.
        7.  **Crucial Step:** Identify any accounts required by the transaction that do not exist in the local `surfpool` cache. Proactively fetch these accounts from the public mainnet RPC and load them into `surfpool` using cheat codes. This prevents "Account not found" errors.
        8.  Sign the locally-built transaction with the temporary wallet `Keypair`.
        9.  Send the signed transaction to the `surfpool` RPC endpoint.
        10. Confirm the transaction and verify that the wallet's token balances have changed as expected (e.g., USDC decreased, SOL increased).
    *   **Benefit:** Provides the highest possible confidence that the swap will succeed on the real mainnet.

## Implementation Roadmap

1.  **Project Structure:**
    *   Rename the current `main.rs` to `src/fast_check.rs`.
    *   Rename `main2.rs` to `src/full_simulation.rs`.
    *   Create a new `main.rs` to serve as a test runner, allowing selection between `fast_check` and `full_simulation` via command-line arguments.
    *   Organize shared modules like `surfpool_client.rs` and `utils.rs` into a `src/common/` directory to improve modularity.

2.  **Code Refinement:**
    *   Adapt the code from the old `main.rs` and `main2.rs` into their new respective modules.
    *   Ensure the `surfpool_client.rs` is robust and correctly implements all necessary cheat codes (`set_account`, `set_token_account`, `reset_account`, `time_travel`, etc.).
    *   Implement the main test runner logic in the new `main.rs`.

By following this plan, we will create a comprehensive testing suite that allows for both rapid, lightweight checks and deep, high-fidelity simulations.