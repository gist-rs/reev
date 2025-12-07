//! Benchmark Runner Module
//!
//! This module provides benchmark runners for executing flows using protocol interface.
//! It includes both static and dynamic benchmark runners, as well as a unified benchmark runner
//! that supports both execution modes.

pub mod benchmark_runner;
pub mod dynamic_runner;
pub mod static_runner;
pub mod types;

// Re-export runners and types for convenience
pub use benchmark_runner::{BenchmarkConfig, BenchmarkExecutionResult, BenchmarkRunner};
pub use dynamic_runner::{DynamicBenchmarkError, DynamicBenchmarkRunner};
pub use static_runner::{BenchmarkError, StaticBenchmarkRunner};
pub use types::{Flow, FlowStep};
