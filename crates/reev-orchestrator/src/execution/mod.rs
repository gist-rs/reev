//! Execution Module for Orchestrator-Agent Coordination
//!
//! This module provides step-by-step execution capabilities that were
//! missing from the original orchestrator design. It implements the
//! critical ping-pong coordination mechanism between orchestrator and agents.

pub mod context;
pub mod ping_pong_executor;

pub use context::{ExecutionContext, StepResultExt};
pub use ping_pong_executor::PingPongExecutor;
