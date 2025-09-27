# TODO

- [x] Create a new crate: `jupiter-lend`
- [ ] Create `jupiter-lend/src/lend.rs` to define lend/borrow request and response structs.
- [ ] Create `jupiter-lend/src/client.rs` which will act as the API client for Jupiter's Lend API.
- [ ] Create `jupiter-lend/src/transaction_config.rs`, adapting from the swap client.
- [ ] Create the `jupiter-lend/src/serde_helpers` module for serialization helpers.
- [ ] Update `jupiter-lend/Cargo.toml` with necessary dependencies (`reqwest`, `serde`, `solana-sdk`, `tokio`, `axum`).
- [ ] Implement the web server in `jupiter-lend/src/main.rs` to expose the transaction builder via an API endpoint.
- [ ] Add a `README.md` to the `jupiter-lend` project with usage instructions.