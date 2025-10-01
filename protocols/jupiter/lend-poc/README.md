# Solana Mainnet-Fork Simulation for DeFi Protocols

This project serves as a proof-of-concept and a technical guide for performing high-fidelity simulations of Solana transactions against a forked mainnet environment using `surfpool`. While the example implements Jupiter lend deposit and withdraw, the techniques and findings documented here are applicable to a wide range of DeFi interactions, such as simulating deposits, withdraws, or other complex contract calls.

## Testing Methodologies

We have established two primary methodologies for testing, each serving a different purpose:

1.  **Deposit Simulation (`cargo run -- deposit`)**:
    *   **Purpose**: To execute a Jupiter lend deposit in a local, forked mainnet environment, providing high-fidelity simulation.
    *   **Process**: It fetches the deposit instruction from the Jupiter lend API, builds the transaction locally, and sends it to the forked environment.
    *   **Expected Outcome**: The deposit transaction is successfully confirmed on the local validator, and we can verify the resulting on-chain state changes (e.g., token balances).

2.  **Withdraw Simulation (`cargo run -- withdraw`)**:
    *   **Purpose**: To execute a Jupiter lend withdraw in a local, forked mainnet environment, providing the highest possible confidence that it will succeed on the real mainnet.
    *   **Process**: It uses a running `surfpool` instance to locally build, sign, and execute the withdraw transaction.
    *   **Expected Outcome**: The withdraw transaction is successfully confirmed on the local validator, and we can verify the resulting on-chain state changes (e.g., token balances).

---

## Key Findings & Techniques for `surfpool` Simulation

Simulating lend deposit and withdraw transactions on a mainnet fork presents unique challenges. The following techniques were found to be essential for success.

### 1. Initial State Setup with RPC Cheat Codes

The foundation of any simulation is setting up a valid initial state. `surfpool` provides RPC "cheat codes" for this purpose.

*   **`surfnet_setAccount`**: Used to fund a newly created temporary wallet (`Keypair`) with SOL to cover transaction fees.
*   **`surfnet_setTokenAccount`**: Used to set the balance of any SPL token for the wallet, creating the Associated Token Account (ATA) if it doesn't exist. This is how we give our test wallet its initial USDC balance.

**Precaution**: RPC requests must be perfectly formed. A simple typo in the request body (e.g., `"jsonrpc": "2.d"` instead of `"2.0"`) will cause the request to fail silently, leading to hard-to-debug errors like `Attempt to debit an account but found no record of a prior credit`.

### 2. Local Transaction Construction

For a valid simulation, the transaction must be built locally using fresh data from the forked environment.

*   **Use Instruction-Based APIs**: Instead of fetching a pre-built, serialized transaction (like from Jupiter's `/lend/v1/earn/deposit` and `/withdraw` endpoints), use endpoints that provide raw instructions (like the Jupiter lend API).
*   **Fetch Fresh Blockhash**: Always get a `latest_blockhash` from the local `surfpool` RPC endpoint. A stale blockhash from a public RPC will cause the transaction to fail.
*   **Fetch ALTs**: If the transaction uses Address Lookup Tables (ALTs), fetch their account data directly from the `surfpool` RPC.

### 3. The "Missing Account" Problem and Proactive Pre-loading

This is the most critical challenge. A `surfpool` fork starts empty and only fetches mainnet accounts on-demand when they are first accessed. However, a transaction often needs to access dozens of accounts simultaneously. If any of these are not in `surfpool`'s cache, the simulation fails.

**The Solution**: We must proactively find and load all required accounts into `surfpool` *before* sending the transaction.

The process is as follows:
1.  **Compile the Message**: After fetching instructions, compile the transaction message locally. This gives access to `message.account_keys` and the keys within the fetched ALTs.
2.  **Identify All Keys**: Aggregate all unique public keys from the static account keys and the loaded ALTs.
3.  **Check Local Availability**: Use `rpc_client.get_multiple_accounts` against the local `surfpool` RPC to identify which accounts are missing from its cache.
4.  **Fetch from Mainnet**: For the list of missing accounts, use a public mainnet RPC (e.g., `api.mainnet-beta.solana.com`) to fetch their full account data.
5.  **Load into Surfpool**: Use the `surfnet_setAccount` cheat code to write the fetched mainnet account data directly into `surfpool`'s state.

### 4. Handling Ephemeral and Locally-Generated Accounts

A crucial refinement to the pre-loading technique is handling accounts that are *not supposed* to exist on mainnet.

*   **Locally-Generated Wallet**: Our own temporary wallet will be identified as "missing." We must explicitly filter it out of the list of accounts to be fetched from mainnet.
*   **Ephemeral Accounts**: Transactions often create new accounts (like ATAs) as part of their execution. These will also appear "missing."

**Precaution**: When an account cannot be fetched from the mainnet RPC, **do not** fall back to creating an empty placeholder account (e.g., via `surfnet_resetAccount`). Doing so injects a zero-lamport account into the fork, which will cause the transaction to fail if it attempts to debit rent. The correct approach is to simply ignore these unfetchable accounts and assume the transaction itself is responsible for their creation and funding.

By following these techniques, we can create reliable, deterministic simulations of complex DeFi transactions, enabling robust testing and validation before deployment.