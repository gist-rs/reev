use crate::solana_env::MockAccountState;
use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;

pub mod sol_transfer;
pub mod spl_token_transfer;

/// A type alias for the mocked in-memory state of the Solana ledger.
pub type MockedState = HashMap<String, MockAccountState>;

/// Defines the common interface for all mocked on-chain actions.
///
/// Each specific transaction type (e.g., SOL transfer, SPL-Token transfer)
/// will have its own struct that implements this trait. This allows the
/// environment's `step` function to be a simple dispatcher that routes
/// an `AgentAction` to the correct action handler.
pub trait Action {
    /// Executes the specific action, validating parameters and mutating the
    /// in-memory state map.
    ///
    /// # Arguments
    ///
    /// * `state` - A mutable reference to the entire mocked blockchain state.
    /// * `params` - The parameters for this specific action, extracted from the
    ///   `AgentAction` and represented as a `serde_json::Value`.
    ///
    /// # Returns
    ///
    /// A `Result` that is `Ok(())` on success or an `Err` with a descriptive
    /// error message on failure. This error message is what the agent will see,
    /// so it should be informative (e.g., "Insufficient funds," "Source account
    /// not found").
    fn execute(&self, state: &mut MockedState, params: &Value) -> Result<()>;
}
