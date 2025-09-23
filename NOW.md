# NOW: Mocking the Solana Environment and Modularizing Actions

**Main Goal:** Refactor the `SolanaEnv` to be a fully mocked, in-memory simulation of the Solana blockchain. This will unblock development of the end-to-end evaluation framework by removing the dependency on the actual `solana-sdk` for now.

**Immediate Tasks:**
1.  Remove all `solana-test-validator` process management and `solana-*` dependencies.
2.  Implement an in-memory key/value store in `SolanaEnv` to represent account states (pubkey -> lamports, owner, data).
3.  Create a new `reev-lib/src/actions` module.
4.  Implement a mocked version of a `sol_transfer` action.
5.  Update `TASKS.md` to reflect the new, modular, mock-first approach for all action types.