# Integration Tests for reev-core

This directory contains end-to-end integration tests for the reev-core system, focusing on testing the complete flow from user prompt to blockchain transaction execution.

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
   # Run a specific test
   cargo test -p reev-core test_swap_1_sol_for_usdc -- --ignored
   
   # Run all swap tests
   cargo test -p reev-core test_ -- --ignored
   ```

3. Alternatively, use the provided script:

   ```bash
   ./scripts/run_integration_tests.sh
   ```

## Architecture

The integration tests validate the following components:

1. **Planner**: Converts natural language prompts to structured YML flows
2. **ContextResolver**: Resolves wallet information from the blockchain
3. **Executor**: Executes the YML flows using appropriate tools
4. **Tool Calling**: Validates that the LLM correctly calls the jupiter_swap_tool
5. **Transaction Signing**: Ensures transactions are properly signed and submitted

## What is Tested

1. The planner correctly parses the user intent from natural language
2. The LLM generates the appropriate tool calls via the ZAI API
3. The executor properly handles the tool calls and generates transactions
4. The transactions are correctly signed with the user's keypair
5. The transactions are successfully submitted to surfpool
6. The wallet balances are updated correctly after the transaction

## Future Enhancements

Future versions of these tests could include:
- More comprehensive balance verification
- Testing of error recovery scenarios
- Additional DeFi operations (lending, liquidity provision, etc.)
- Performance benchmarks