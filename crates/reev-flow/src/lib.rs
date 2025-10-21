//! # Reev Flow
//!
//! Shared flow types and utilities for the reev ecosystem.
//!
//! This crate provides core types for tracking and analyzing agent execution flows,
//! designed to be:
//! 1. Database-friendly when database feature is enabled
//! 2. Generic enough for different use cases
//! 3. Easily convertible to/from domain-specific types
//! 4. Serializable and deserializable for storage and API communication

pub mod error;
pub mod logger;
pub mod otel;
pub mod renderer;
pub mod types;
pub mod utils;
pub mod website_exporter;

// Re-export specific items to avoid ambiguity
pub use error::{FlowError, FlowResult};
pub use logger::{init_flow_tracing, AgentPerformanceData, DatabaseWriter, FlowLogger};
pub use otel::FlowTracer;
pub use renderer::{render_flow_file_as_ascii_tree, FlowLogRenderer};
pub use types::*;
pub use utils::{
    calculate_execution_statistics, get_default_flow_log_path, FlowSummary, FlowUtils,
};
pub use website_exporter::WebsiteExporter;

#[cfg(feature = "database")]
pub mod database;

#[cfg(feature = "database")]
pub use database::*;
