//! # jup-sdk
//!
//! A Rust SDK for interacting with the Jupiter Swap and Lend APIs.
//! This SDK provides a flexible, layered API to either:
//! 1. Build unsigned transactions for production use (to be signed by a wallet).
//! 2. Run end-to-end simulations against a `surfpool` mainnet fork for testing.

pub mod api;
pub mod api_client;
pub mod client;
pub mod config;
pub mod models;
pub mod surfpool;
pub mod transaction;

// Re-export key structs and the main client for easier access.
pub use client::Jupiter;
