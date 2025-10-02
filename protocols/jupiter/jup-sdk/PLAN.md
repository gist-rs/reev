# jup-sdk Refactoring Plan

## 1. Overview & Goals

The current `jup-sdk` implementation successfully demonstrates swapping and lending operations but suffers from two main issues:
1.  **Repetitive Code:** Significant code duplication exists between `swap.rs`, `lend.rs`, and even within functions (e.g., wallet setup, transaction building, pre-loading accounts).
2.  **Tight Coupling:** The logic for generating transactions is tightly coupled with the `surfpool` simulation environment, making it difficult to use the SDK for generating real transactions that can be signed by a separate wallet.

The primary goals of this refactoring are:
-   **DRY (Don't Repeat Yourself):** Abstract common logic into shared, reusable components.
-   **Separation of Concerns:** Decouple the core Jupiter API interaction and transaction building from the `surfpool` simulation and execution logic.
-   **Flexible API:** Introduce a builder pattern to provide a clear, composable, and ergonomic API for both production and simulation use cases.
-   **Extensibility:** Create a structure that is easy to extend with new features (e.g., different lending protocols, more swap options) for both real and simulated environments.

## 2. Core Design: A Layered Builder Pattern

We will introduce a central `Jupiter` client struct that acts as a builder. This client will be configured to operate in one of two modes, creating a clear separation between layers.

### Layer 1: Core Engine (For Production)

-   **Responsibility:** Fetching quotes/instructions from the Jupiter API and constructing an unsigned, serializable `VersionedTransaction`.
-   **Environment:** Agnostic. It only needs a standard Solana RPC endpoint for fetching blockhashes and account info.
-   **Output:** A structure containing the unsigned transaction, ready to be passed to a wallet (e.g., a frontend client) for signing. It will not perform any signing or execution itself.

### Layer 2: Simulation Engine (For Testing & Evaluation)

-   **Responsibility:** Orchestrating an end-to-end simulation using `surfpool`.
-   **Environment:** `surfpool` RPC endpoint.
-   **Functionality:** It will wrap the Core Engine and add:
    -   `surfpool` cheat codes for setting up the environment (funding wallets, setting token balances).
    -   Signing the transaction with a temporary `Keypair`.
    -   Pre-loading required mainnet accounts into the `surfpool` instance.
    -   Submitting the transaction and verifying the outcome.
    -   Returning a rich result object with signatures and detailed debug information.

## 3. Proposed Module Structure

The `src` directory will be reorganized to reflect this new architecture.

```
src/
├── client.rs          # The main `Jupiter` builder struct and its API.
├── surfpool.rs        # All surfpool-specific logic and the simulation runner.
├── transaction.rs     # Shared logic for building/processing transactions.
├── api/
│   ├── mod.rs
│   ├── swap.rs        # Logic to fetch swap quotes and instructions from Jupiter API.
│   └── lend.rs        # Logic to fetch lend/deposit/withdraw instructions.
└── models.rs          # Shared data structures (API responses, result types, etc.).
└── lib.rs             # Main library entry, re-exporting key components.
```

-   `client.rs`: Will contain `Jupiter`, the main entry point for all operations.
-   `api/swap.rs` & `api/lend.rs`: These modules will be responsible *only* for fetching the necessary instructions from the Jupiter HTTP API. They will not build or execute transactions.
-   `transaction.rs`: Will contain the DRY logic for converting API instructions into `solana_sdk::Instruction`, fetching ALTs, and compiling the final `VersionedTransaction`.
-   `surfpool.rs`: Will contain the `SurfpoolClient` (for cheat codes) and the simulation logic that takes a `VersionedTransaction`, prepares the `surfpool` state, signs, and executes.
-   `models.rs`: Will define clear input parameter structs (e.g., `SwapParams`) and output structs (e.g., `UnsignedTransaction`, `SimulationResult`).

## 4. Proposed API Design & Usage

### 4.1. The `Jupiter` Client

The builder will be initialized differently based on the desired layer.

```rust
// For Layer 1: Production transaction building
let jupiter_client = Jupiter::new(RpcClient::new("https://api.mainnet-beta.solana.com"));

// For Layer 2: Surfpool simulation
let jupiter_client = Jupiter::surfpool(RpcClient::new("http://127.0.0.1:8899"));
```

### 4.2. Use Case 1: Generating a Real Swap Transaction (Layer 1)

This flow is for when you need to create a transaction that will be signed and sent by a third-party wallet.

```rust
// /dev/null/example.rs
use jup_sdk::{Jupiter, models::SwapParams};

async fn generate_swap_tx() {
    let client = Jupiter::new(RpcClient::new("...")); // Mainnet RPC

    let params = SwapParams {
        input_mint: usdc_mint,
        output_mint: sol_mint,
        amount: 50_000_000,
        slippage_bps: 500,
        // etc.
    };

    // This method only builds the transaction, it does not sign or send it.
    let unsigned_tx = client
        .swap(params)
        .build_unsigned_transaction()
        .await
        .unwrap();

    // The result can be serialized (e.g., to base64) and sent to a wallet.
    let serialized_tx = bincode::serialize(&unsigned_tx.transaction).unwrap();
    let tx_base64 = base64::encode(serialized_tx);
    
    // --> send tx_base64 to frontend/wallet for signing
}
```

The output would be a struct like:
```rust
// /dev/null/models.rs
pub struct UnsignedTransaction {
    /// The unsigned, versioned transaction.
    pub transaction: VersionedTransaction,
    /// The blockhash used to create the transaction.
    pub last_valid_block_height: u64,
}
```

### 4.3. Use Case 2: Simulating a Swap with Surfpool (Layer 2)

This flow is for testing, evaluation, and automated agents running against a local fork.

```rust
// /dev/null/example.rs
use jup_sdk::{Jupiter, models::SwapParams};

async fn simulate_swap() {
    let signer = Keypair::new();
    let client = Jupiter::surfpool(RpcClient::new("http://127.0.0.1:8899"))
        .with_signer(&signer);

    let params = SwapParams {
        input_mint: usdc_mint,
        output_mint: sol_mint,
        amount: 50_000_000,
        // ...
    };

    // The `.commit()` method runs the full simulation:
    // 1. Sets up wallet with SOL and tokens.
    // 2. Fetches instructions.
    // 3. Builds the transaction.
    // 4. Pre-loads accounts into surfpool.
    // 5. Signs and sends the transaction.
    // 6. Confirms and returns a rich result.
    let result = client.swap(params).commit().await.unwrap();

    println!("Swap successful! Signature: {}", result.signature);
    println!("Final USDC balance: {}", result.debug_info.final_usdc_balance);
}
```

The output would be a struct like:
```rust
// /dev/null/models.rs
pub struct SimulationResult {
    pub signature: String,
    pub debug_info: DebugInfo,
}

pub struct DebugInfo {
    /// Human-readable list of accounts in the transaction.
    pub readable_tx: Vec<String>,
    /// Errors that occurred during the transaction, if any.
    pub tx_errors: Option<String>,
    /// The final result of the transaction.
    pub tx_result: TransactionResult,
    // Other useful metrics like pre/post balances.
    pub initial_usdc_balance: u64,
    pub final_usdc_balance: u64,
}

pub enum TransactionResult {
    Success,
    Failure,
}
```

## 5. Refactoring Steps

1.  **Phase 1: Setup Core Structure & Models**
    -   Create the new directory structure (`api/`, etc.).
    -   Create `client.rs` with a skeleton `Jupiter` struct.
    -   Define the initial data models in `models.rs` (`SwapParams`, `SimulationResult`, etc.).

2.  **Phase 2: Abstract API Logic**
    -   Move the `reqwest` calls for fetching swap and lend instructions from `swap.rs` and `lend.rs` into `api/swap.rs` and `api/lend.rs` respectively. These functions should take parameters and return the raw instruction data (`ApiResponse`).

3.  **Phase 3: Implement Core Transaction Builder (Layer 1)**
    -   In `transaction.rs`, create a function that takes an `ApiResponse` and returns an `UnsignedTransaction`. This will contain the logic for parsing instructions, fetching ALTs, and compiling the `VersionedMessage`.
    -   Implement the `.build_unsigned_transaction()` method on the `Jupiter` client.

4.  **Phase 4: Implement Surfpool Simulation Engine (Layer 2)**
    -   In `surfpool.rs`, create the simulation runner function. It will take a `Jupiter` client config and an `UnsignedTransaction`.
    -   This function will encapsulate all the `surfpool`-specific logic: `setup_wallet`, `execute_transaction` (including pre-loading accounts), and balance verification.
    -   Implement the `.commit()` method on the `Jupiter` client, which calls the simulation runner.

5.  **Phase 5: Update Examples & Cleanup**
    -   Rewrite the examples in `examples/` to use the new, clean builder API.
    -   Remove the old `swap.rs` and `lend.rs` files at the root of `src`, along with the now-redundant `common/` module.

This structured plan will result in a more professional, maintainable, and powerful SDK that serves both production and testing needs effectively.