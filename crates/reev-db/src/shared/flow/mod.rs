//! Shared flow types and utilities
//!
//! This module provides common flow logging types that can be used across
//! the reev ecosystem and other projects. These types are designed to be:
//! - Database-friendly (String timestamps, JSON serializable)
//! - Generic enough for different use cases
//! - Easily convertible to/from domain-specific types
//!
//! ## Usage
//!
//! See the individual module documentation for detailed usage examples.

pub mod converter;
pub mod types;

// Import converter items separately to avoid ambiguity
pub use converter::{FlowConverter, FlowLogConverter};
// Import types separately to avoid ambiguity
pub use types::{
    ConversionError, EventContent, ExecutionResult, ExecutionStatistics, FlowEvent, FlowEventType,
    FlowLog, FlowLogUtils, ScoringBreakdown,
};

/// Re-export commonly used types for convenience
pub mod prelude {
    pub use super::{
        ConversionError, EventContent, ExecutionResult, ExecutionStatistics, FlowEvent,
        FlowEventType, FlowLog, FlowLogConverter, FlowLogUtils, ScoringBreakdown,
    };
}
