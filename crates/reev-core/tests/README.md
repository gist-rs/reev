# Integration Tests for reev-core

This directory contains end-to-end integration tests for the reev-core system, focusing on testing the complete flow from user prompt to blockchain transaction execution.

## Focused Logging for Clear Output

The tests now use filtered logging to show only the relevant steps in the swap flow, reducing noise and making it easier to follow the process. The logging is configured to show:
- YML prompt with wallet info sent to GLM-coding
- Tool calling from LLM
- Transaction generation and signing
- Transaction completion results

The tests follow a 6-step process that is clearly marked in the output.

## Tests

### end_to_end_swap.rs

This test file contains integration tests for the Jupiter swap functionality using the reev-core planner and executor. It tests two scenarios:

1. **test_swap_1_sol_for_usdc**: Tests swapping a fixed amount (1 SOL) for USDC
2. **test_sell_all_sol_for_usdc**: Tests selling all available SOL for USDC

Both tests:
- Load the default Solana keypair from `~/.config/solana/id.json`
- Connect to a surfpool instance running at `http://localhost:8899`
- Set up the wallet with SOL and USDC balances
- Use the planner to convert a natural language prompt to a structured YML flow
- Execute the flow using the executor which makes LLM calls via the ZAI API
- Sign and submit the transaction using the loaded keypair

## Prerequisites

1. **Surfpool running**: The tests require a surfpool instance running at `http://localhost:8899`:
   ```bash
   surfpool --fork-url https://api.mainnet-beta.solana.com --port 8899
   ```

2. **ZAI_API_KEY set**: The tests require a ZAI API key to be set in your environment or `.env` file:
   ```bash
   export ZAI_API_KEY=your_api_key_here
   ```

3. **Default Solana keypair**: The tests use the default Solana keypair located at `~/.config/solana/id.json`

## Running the Tests

The tests are marked with `#[ignore]` by default since they require external dependencies. To run them:

1. Ensure surfpool is running and ZAI_API_KEY is set
2. Run the tests with the `--ignored` flag:

   ```bash
   # Run with filtered logging (recommended)
   RUST_LOG=reev_core::planner=info,reev_core::executor=info,jup_sdk=info,warn cargo test -p reev-core --test end_to_end_swap test_swap_1_sol_for_usdc -- --nocapture --ignored
   
   # Or use the provided scripts (recommended)
   ./scripts/run_swap_test.sh
   ./scripts/run_sell_all_test.sh
   ```

The filtered logging approach ensures you see only the relevant steps:
1. Prompt input
2. YML prompt with wallet info sent to GLM-coding
3. Tool calling from LLM
4. Generated transaction
5. Transaction signing with default keypair
6. Transaction completion result

## Architecture

The integration tests validate the following components:

1. **Planner**: Converts natural language prompts to structured YML flows
2. **ContextResolver**: Resolves wallet information from the blockchain
3. **Executor**: Executes the YML flows using appropriate tools
4. **Tool Calling**: Validates that the LLM correctly calls the jupiter_swap_tool
5. **Transaction Signing**: Ensures transactions are properly signed and submitted

## What is Tested (6-Step Process)

1. **Prompt Processing**: The planner correctly parses "swap 1 SOL for USDC" or "sell all SOL for USDC"
2. **YML Generation**: YML prompt with wallet info from SURFPOOL is sent to GLM-coding via ZAI_API_KEY
3. **Tool Calling**: The LLM generates the appropriate tool calls for the Jupiter swap
4. **Transaction Generation**: The executor handles tool calls and generates the swap transaction
5. **Transaction Signing**: The transaction is correctly signed with the default keypair at `~/.config/solana/id.json`
6. **Transaction Completion**: The transaction is successfully submitted and completed via SURFPOOL

## Future Enhancements

Future versions of these tests could include:
- More comprehensive balance verification
- Testing of error recovery scenarios
- Additional DeFi operations (lending, liquidity provision, etc.)
- Performance benchmarks