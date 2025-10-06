# Reflection on Debugging Session (100-jup-swap-sol-usdc)

This document outlines the root causes and solutions for a series of cascading failures encountered while debugging the `100-jup-swap-sol-usdc.yml` benchmark with the `local-model` agent.

## Summary of Failures and Fixes

The debugging process revealed a critical misunderstanding of the testing architecture, specifically how our local `surfpool` validator interacts with external APIs. The issues evolved as we peeled back layers of the problem.

### 1. Initial Failure: `Missing 'instruction' field`

-   **Symptom:** The `reev-runner` crashed with an error: `LLM API request failed... Missing 'instruction' field in tool call response`.
-   **Root Cause:** The `reev-agent` expected the tool's JSON output to be wrapped in an `{"instruction": ...}` object, but the `JupiterSwapTool` was returning a raw JSON array of instructions.
-   **Solution:** The code in `reev-agent/src/run.rs` was modified to be more flexible, accepting either a raw array or the wrapped object. This fixed the immediate crash but was a symptom of a deeper problem.

### 2. Second Failure: `invalid account data for instruction`

-   **Symptom:** After fixing the agent's parsing, the transaction simulation on the local `surfpool` validator began failing with `Error processing Instruction 0: invalid account data for instruction`.
-   **Root Cause:** This was the central issue. The `JupiterSwapTool` was using `reqwest` to make a direct API call to the **public Jupiter API endpoint**.
    -   Our test runner creates temporary, ephemeral wallets on a local `surfpool` (mainnet fork) instance.
    -   The public Jupiter API has no knowledge of our local validator. It runs its own simulations against the *real* Solana mainnet.
    -   The simulation failed on Jupiter's end because our ephemeral test wallet does not exist and has no funds on mainnet.
    -   Because the public API call returned an error (`simulationError`), our tool's code fell back to using placeholder/mock logic, which was incorrect and caused the failure on the local validator.
-   **Solution:** The entire implementation of the `JupiterSwapTool::handle_jupiter_swap` function was refactored. The `reqwest` call to the public API was removed and replaced with the `jup-sdk`. The SDK is designed to work with a local `surfpool` instance, so it correctly builds and simulates transactions against the state of our local test environment where the wallets are properly funded.

### 3. Third Failure: `jup-sdk` Integration Errors

-   **Symptom:** While implementing the `jup-sdk`, several compilation and runtime errors occurred, such as `no method named 'instructions'` and `incorrect program id for instruction`.
-   **Root Cause:** A misunderstanding of the `jup-sdk`'s API.
    -   The `.swap()` method returns a `SwapBuilder`, not the final transaction. The correct method to get the transaction details is `.prepare_transaction_components().await`.
    -   The `solana_sdk::instruction::Instruction` struct returned by the SDK was not being correctly converted to our internal `RawInstruction` format.
-   **Solution:** The code was corrected to use the `SwapBuilder` properly and to correctly map all fields (program_id, accounts, data) from the SDK's instruction format to our `RawInstruction` struct.

---

# Reflection on Debugging Session (110-jup-lend-deposit-sol)

This document outlines the fixes for a regression found in the `110-jup-lend-deposit-sol.yml` benchmark after a major refactoring. The issue stemmed from a placeholder implementation and a misunderstanding of on-chain program requirements.

## Summary of Failures and Fixes

### 1. Initial Failure: `Invalid base58 data`

-   **Symptom:** The `reev-runner` crashed with `Error: Invalid base58 data: deposit_100000000`.
-   **Root Cause:** After the refactor, the `handle_jupiter_deposit` function in `reev-agent/src/protocols/jupiter/lend_deposit.rs` contained a placeholder implementation. It was generating a fake instruction with `data: format!("deposit_{amount:?}")`, which is not a valid base58 string and was correctly rejected by the deserializer.
-   **Solution:** The placeholder logic was completely replaced with a real implementation using the `jup-sdk`. This involved initializing the `Jupiter` client, creating `DepositParams`, calling `.deposit(params).prepare_transaction_components().await`, and converting the resulting SDK instructions into the `RawInstruction` format used by the agent.

### 2. Second Failure: `AccountNotInitialized`

-   **Symptom:** After fixing the `base58` error, the benchmark ran but the transaction simulation failed with the error: `AnchorError caused by account: depositor_token_account. Error Code: AccountNotInitialized`.
-   **Root Cause:** Depositing native SOL into Jupiter's lend program requires the user to first wrap the SOL into WSOL. This involves creating an Associated Token Account (ATA) for WSOL, transferring the SOL amount to it, and syncing the native mint. This logic was present in the *integration test setup* (`prepare_jupiter_lend_deposit` in `helpers.rs`) but was missing from the actual deterministic agent's logic (`d_110_jup_lend_deposit_sol.rs`). The agent was only generating the final Jupiter `deposit` instruction, without the prerequisite setup instructions.
-   **Solution:** The responsibility for creating a complete and valid transaction was moved from the test helpers to the agent itself.
    1.  The SOL wrapping logic (creating the WSOL ATA, transferring SOL, and syncing the account) was moved from `reev-runner/tests/common/helpers.rs` into `reev-agent/src/agents/coding/d_110_jup_lend_deposit_sol.rs`.
    2.  The agent now constructs a transaction containing all three setup instructions followed by the Jupiter deposit instruction.
    3.  The test helper (`prepare_jupiter_lend_deposit`) was simplified, as it no longer needs to create these setup instructions.

## Key Takeaways and Future Best Practices

1.  **Isolate the Test Environment:** Tools designed for on-chain interactions within our test framework **must** communicate with the local `surfpool` RPC endpoint (`http://127.0.0.1:8899`). They should **never** call public mainnet APIs (`https://quote-api.jup.ag`, etc.), as the state will be inconsistent.

2.  **Leverage the Correct SDK:** When an SDK like `jup-sdk` is available and designed to work with `surfpool`, it should always be preferred over direct API calls. It correctly handles the context of the local forked environment.

3.  **Comprehensive Logging is Non-Negotiable:** The breakthrough in diagnosing the core issue came from logging the full JSON response from the external API call in the `reev-agent.log`. This revealed the `simulationError` and proved the problem was with the API interaction, not the agent's logic.

4.  **A "Passing" Score Isn't Always a Success:** A benchmark can run to completion and receive a high score (e.g., 75%) even if the on-chain transaction fails. The score often reflects that the LLM generated a *structurally* correct tool call, but the `OBSERVATION: Failure` is the true indicator of the outcome and must be the focus of debugging.

5.  **Agent Logic Should Be Self-Contained:** Deterministic agents should be responsible for generating the *entire* sequence of instructions required to complete a task. Prerequisite steps (like wrapping SOL) should not be handled by the test harness, as this hides the true complexity of the task from the agent and can lead to discrepancies between testing and real-world execution.