use crate::benchmark::GroundTruth;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Represents the output of a single step in the environment.
/// It is generic over the observation type `Obs`.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Step<Obs> {
    /// The observation of the environment's state.
    pub observation: Obs,
    /// The reward signal from the environment.
    pub reward: f32,
    /// A boolean flag indicating if the episode has ended (e.g., goal reached, unrecoverable failure).
    pub terminated: bool,
    /// A boolean flag indicating if the episode was cut short by an external condition (e.g., time limit).
    pub truncated: bool,
    /// A dictionary for diagnostic information. This data is for debugging and analysis, not for the agent.
    pub info: Value,
}

/// The core environment trait, analogous to the Gymnasium `Env` class.
///
/// This trait defines the standard interface for an agent to interact with an environment.
/// It is generic over the `Action` the agent can take and the `Observation` it receives.
pub trait GymEnv {
    /// The type of action the agent can take in the environment.
    type Action;
    /// The type of observation the agent receives from the environment.
    type Observation;

    /// Resets the environment to a well-defined initial state.
    ///
    /// # Arguments
    /// * `seed`: An optional seed for the environment's random number generator to ensure reproducibility.
    /// * `options`: Optional dictionary to configure the reset process.
    ///
    /// # Returns
    /// The initial observation of the environment.
    fn reset(
        &mut self,
        seed: Option<u64>,
        options: Option<Value>,
    ) -> impl std::future::Future<Output = anyhow::Result<Self::Observation>> + Send;

    /// Executes a single step in the environment based on the agent's action.
    ///
    /// # Arguments
    /// * `action`: The action to be performed by the agent.
    ///
    /// # Returns
    /// A `Step` struct containing the outcome of the action.
    fn step(
        &mut self,
        action: Self::Action,
        ground_truth: &GroundTruth,
    ) -> anyhow::Result<Step<Self::Observation>>;

    /// Renders a representation of the environment's current state.
    ///
    /// The specific output (e.g., to console, a GUI window) is implementation-dependent.
    fn render(&self);

    /// Performs any necessary cleanup for the environment.
    ///
    /// This should be called when the environment is no longer needed to release resources
    /// (e.g., terminate child processes, close network connections).
    fn close(&mut self) -> anyhow::Result<()>;
}
