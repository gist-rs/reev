//! Reev Core
//!
//! Core architecture for verifiable AI-generated DeFi flows with YML schemas
//! and two-phase LLM approach.

pub mod context;
pub mod executor;
pub mod planner;
pub mod validation;
pub mod yml_schema;

// Re-export key types for convenience
pub use context::ContextResolver;
pub use executor::Executor;
pub use planner::Planner;
pub use validation::FlowValidator;
pub use yml_schema::{
    YmlAssertion, YmlContext, YmlFlow, YmlGroundTruth, YmlStep, YmlToolCall, YmlWalletInfo,
};
