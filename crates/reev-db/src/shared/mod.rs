//! Shared types module for reev-db
//!
//! This module contains common types that can be used across different projects
//! in the reev ecosystem. These types are designed to be:
//! - Database-friendly (use String timestamps, JSON serializable)
//! - Generic enough for different use cases
//! - Easily convertible to/from domain-specific types
//! - Ready for extraction to separate crate later
//!
//! ## Structure
//!
//! - `flow/`: Flow logging and execution tracking types
//! - `benchmark/`: Benchmark-related types
//! - `performance/`: Performance monitoring types
//!
//! ## Usage Patterns
//!
//! 1. **Direct Usage**: Use these types directly for database operations
//! 2. **Conversion**: Implement `FlowLogConverter` for domain-specific types
//! 3. **Extension**: Add new types in appropriate modules
//!
//! ## Migration Path
//!
//! These types can be easily extracted to a separate `reev-types` crate:
//! 1. Copy the `shared/` directory to new crate
//! 2. Update imports across projects
//! 3. Add `reev-types` as dependency

pub mod benchmark;
pub mod flow;
pub mod performance;

pub use benchmark::*;
pub use flow::*;
pub use performance::*;

/// Re-export all shared types for convenience
pub mod prelude {
    pub use super::{benchmark::*, flow::prelude::*, performance::*};
}
