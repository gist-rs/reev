//! # Environment Interface for Agent Evaluation
//!
//! This module defines the core environment interface that enables agents to interact
//! with Solana blockchain state in a controlled, reproducible manner. The design follows
//! the Gymnasium API pattern, providing a familiar interface for reinforcement learning
//! and agent evaluation scenarios.
//!
//! ## Environment Philosophy
//!
//! The environment serves as a bridge between LLM agents and the Solana blockchain,
//! providing:
//! - **Deterministic State Management**: Precise control over initial conditions
//! - **Hermetic Execution**: Isolated test runs with no external dependencies
//! - **Real Program Interaction**: Agents interact with actual deployed Solana programs
//! - **Comprehensive Observation**: Rich state information for agent decision-making
//!
//! ## Key Components
//!
//! ### Step<Obs>
//! Represents the outcome of a single agent action, containing:
//! - New environment state (observation)
//! - Reward signal for reinforcement learning
//! - Episode termination flags
//! - Diagnostic information for debugging

use crate::benchmark::GroundTruth;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Represents the output of a single step in the environment.
///
/// This struct encapsulates the complete result of an agent's action, providing
/// both the new state information and metadata needed for evaluation and learning.
///
/// ## Type Parameters
///
/// * `Obs` - The observation type representing environment state
///
/// ## Fields
///
/// * `observation` - The new environment state after executing the action
/// * `reward` - Reward signal for reinforcement learning algorithms (typically 0.0 for evaluation)
/// * `terminated` - True if the episode ended due to goal completion or unrecoverable failure
/// * `truncated` - True if the episode was cut short by external conditions (timeouts, etc.)
/// * `info` - Additional diagnostic data for debugging and analysis
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Step<Obs> {
    /// The observation of the environment's state after action execution.
    ///
    /// This contains the complete blockchain state including account balances,
    /// transaction results, and any other relevant information the agent needs
    /// to make subsequent decisions.
    pub observation: Obs,

    /// The reward signal from the environment.
    ///
    /// For evaluation scenarios, this is typically 0.0. For reinforcement learning
    /// training, this would provide feedback based on action quality.
    pub reward: f32,

    /// Indicates if the episode has ended due to internal conditions.
    ///
    /// This flag is set to true when:
    /// - The agent successfully completed the task
    /// - An unrecoverable error occurred
    /// - The environment reached a terminal state
    pub terminated: bool,

    /// Indicates if the episode was cut short by external conditions.
    ///
    /// This flag is set to true when:
    /// - Time limits were exceeded
    /// - External services became unavailable
    /// - Other environmental constraints were triggered
    pub truncated: bool,

    /// Diagnostic information for debugging and analysis.
    ///
    /// This field contains structured data useful for:
    /// - Performance profiling
    /// - Error diagnosis
    /// - Execution trace analysis
    /// - Agent behavior analysis
    pub info: Value,
}

/// The core environment trait for agent evaluation.
///
/// This trait defines the standard interface that enables agents to interact with
/// Solana blockchain state in a controlled, reproducible manner. It follows the
/// Gymnasium API pattern, providing a familiar interface for reinforcement learning
/// and agent evaluation scenarios.
///
/// ## Design Principles
///
/// - **Deterministic Behavior**: Given the same seed and initial conditions, the environment
///   produces identical results across runs
/// - **Hermetic Execution**: Each test run is isolated from external influences
/// - **Real Program Interaction**: Agents interact with actual deployed Solana programs
/// - **Rich Observations**: Comprehensive state information for informed decision-making
///
/// ## Type Parameters
///
/// * `Action` - The type of actions agents can perform (typically Solana instructions)
/// * `Observation` - The type of state information agents receive
///
/// ## Lifecycle
///
/// 1. **Reset**: Initialize environment to a known state
/// 2. **Step**: Execute agent actions and observe results
/// 3. **Render**: (Optional) Visualize current state
/// 4. **Close**: Clean up resources and terminate
pub trait GymEnv {
    /// The type of action the agent can take in the environment.
    ///
    /// For Solana environments, this is typically `AgentAction` containing
    /// Solana instructions that the agent wants to execute.
    type Action;

    /// The type of observation the agent receives from the environment.
    ///
    /// This contains all the information the agent needs to make decisions,
    /// including account states, transaction results, and contextual data.
    type Observation;

    /// Resets the environment to a well-defined initial state.
    ///
    /// This method prepares the environment for a new evaluation episode by:
    /// - Setting up the specified initial blockchain state
    /// - Clearing any previous transaction history
    /// - Applying deterministic seeding for reproducible results
    ///
    /// # Arguments
    ///
    /// * `seed` - Optional seed for random number generation to ensure reproducibility.
    ///   When provided, the environment will produce identical initial states across runs.
    /// * `options` - Optional configuration dictionary for customizing the reset process.
    ///   Implementation-specific options may include custom account setups or special conditions.
    ///
    /// # Returns
    ///
    /// The initial observation of the environment state, containing:
    /// - Account balances and states as specified in the benchmark
    /// - Placeholder mappings for dynamic address resolution
    /// - Clean transaction history and error state
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The environment fails to connect to required services
    /// - Initial state setup encounters problems
    /// - Invalid configuration options are provided

    fn reset(
        &mut self,
        seed: Option<u64>,
        options: Option<Value>,
    ) -> impl std::future::Future<Output = anyhow::Result<Self::Observation>> + Send;

    /// Executes agent actions and returns the resulting environment state.
    ///
    /// This is the core method that enables agents to interact with the Solana blockchain.
    /// It processes the agent's actions, executes them on the blockchain (or simulation),
    /// and returns the resulting state along with evaluation metadata.
    ///
    /// # Arguments
    ///
    /// * `actions` - A vector of actions the agent wants to execute. For Solana environments,
    ///   these are typically Solana instructions representing transactions.
    /// * `ground_truth` - The benchmark's ground truth criteria used for evaluation
    ///   and scoring during the execution process.
    ///
    /// # Returns
    ///
    /// A `Step` struct containing:
    /// - `observation`: New environment state after execution
    /// - `reward`: Reward signal (typically 0.0 for evaluation scenarios)
    /// - `terminated`: Whether the episode has ended
    /// - `truncated`: Whether the episode was cut short externally
    /// - `info`: Diagnostic information including execution details
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Transaction execution fails due to invalid instructions
    /// - Network connectivity issues occur
    /// - State corruption is detected
    /// - Resource constraints prevent execution

    fn step(
        &mut self,
        actions: Vec<Self::Action>,
        ground_truth: &GroundTruth,
    ) -> anyhow::Result<Step<Self::Observation>>;

    /// Renders a representation of the environment's current state.
    ///
    /// This method provides a way to visualize or display the current environment
    /// state for debugging, monitoring, or analysis purposes. The specific output
    /// format depends on the implementation.
    ///
    /// ## Common Implementations
    ///
    /// - **Console Output**: Print account balances and transaction history
    /// - **JSON Export**: Structured data for programmatic analysis
    /// - **GUI Display**: Visual representation for interactive debugging
    /// - **File Export**: Persistent state snapshots
    fn render(&self);

    /// Performs cleanup and resource release when the environment is no longer needed.
    ///
    /// This method should be called to properly terminate the environment and release
    /// any held resources. Failing to call this method may result in resource leaks
    /// or dangling processes.
    ///
    /// ## Cleanup Responsibilities
    ///
    /// - Terminate any child processes (e.g., surfpool instances)
    /// - Close network connections and database handles
    /// - Flush any pending logs or buffered data
    /// - Release temporary files or directories
    /// - Signal to external services that the session has ended
    ///
    /// # Errors
    ///
    /// Returns an error if cleanup encounters issues, but the environment should
    /// make a best effort to clean up as much as possible even if some operations fail.
    fn close(&mut self) -> anyhow::Result<()>;
}
