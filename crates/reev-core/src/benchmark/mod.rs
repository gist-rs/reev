//! Benchmark Module for Evaluating AI-Generated DeFi Flows
//!
//! This module provides components for evaluating execution results against
//! ground truth expectations, calculating scores, and generating benchmark reports.

pub mod conversion;
pub mod runner;
pub mod scorer;
pub mod types;

pub use runner::{BenchmarkRunner, DynamicBenchmarkRunner, StaticBenchmarkRunner};
pub use scorer::BenchmarkScorer;
pub use types::{
    BenchmarkCategory, BenchmarkReport, BenchmarkScore, ExecutionMetrics, ScoredResult,
    ValidationResult, ValidationResults,
};
