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

pub mod enhanced_otel;
pub mod error;
pub mod jsonl_converter;
pub mod logger;
pub mod otel;
pub mod renderer;
pub mod types;
pub mod utils;
pub mod website_exporter;

// Re-export specific items to avoid ambiguity
pub use enhanced_otel::{
    get_enhanced_otel_logger, init_enhanced_otel_logging, init_enhanced_otel_logging_with_session,
    EnhancedOtelError, EnhancedOtelLogger, EnhancedToolCall, EventType, PromptInfo, TimingInfo,
    ToolExecutionStatus, ToolInputInfo, ToolOutputInfo,
};

// Re-export macros at crate level (they're exported from enhanced_otel module)
pub use error::{FlowError, FlowResult};

// Export logging macros for use by other crates
// Macros are now defined directly in enhanced_otel.rs and exported via #[macro_export]
// Macros are exported at crate level via #[macro_export] - no need for pub use
pub use jsonl_converter::{JsonlToYmlConverter, SessionData, SessionSummary};
pub use logger::{init_flow_tracing, AgentPerformanceData, DatabaseWriter, FlowLogger};
pub use otel::{init_flow_tracing_with_session, FlowTracer};
pub use renderer::{render_flow_file_as_ascii_tree, FlowLogRenderer};
pub use types::*;
pub use utils::{calculate_execution_statistics, FlowSummary, FlowUtils};
pub use website_exporter::WebsiteExporter;

#[cfg(feature = "database")]
pub mod database;

#[cfg(feature = "database")]
pub use database::*;
