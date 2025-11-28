//! Reev Core
//!
//! Core architecture for verifiable AI-generated DeFi flows with YML schemas
//! and two-phase LLM approach.

pub mod context;
pub mod execution;
pub mod executor;
pub mod llm;
pub mod planner;
pub mod refiner;
pub mod validation;
pub mod yml_generator;
pub mod yml_schema;

// Re-export key types for convenience
pub use context::ContextResolver;
pub use executor::Executor;
pub use llm::glm_client::init_glm_client;
pub use planner::Planner;
pub use refiner::LanguageRefiner;
pub use validation::FlowValidator;
pub use yml_generator::YmlGenerator;
pub use yml_schema::{
    YmlAssertion, YmlContext, YmlFlow, YmlGroundTruth, YmlStep, YmlToolCall, YmlWalletInfo,
};

// Re-export context builder
pub use execution::context_builder::{
    MinimalAiContext, OperationMetadata, PreviousStepResult, TokenInfo, YmlContextBuilder,
    YmlOperationContext,
};
