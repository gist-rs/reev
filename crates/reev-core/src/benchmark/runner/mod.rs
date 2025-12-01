//! Benchmark Runner Module
//!
//! This module provides benchmark runners for executing flows using protocol interface.
//! It includes both static and dynamic benchmark runners.

pub mod dynamic_runner;
pub mod static_runner;
pub mod types;

// Re-export runners and types for convenience
pub use dynamic_runner::{DynamicBenchmarkError, DynamicBenchmarkRunner};
pub use static_runner::{BenchmarkError, StaticBenchmarkRunner};
pub use types::{Flow, FlowStep};
