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

#[cfg(test)]
mod tests {
    use super::prelude::*;

    #[test]
    fn test_shared_module_structure() {
        // Test that we can import and use shared types
        let flow_log = FlowLogUtils::create(
            "test-session".to_string(),
            "test-benchmark".to_string(),
            "test-agent".to_string(),
        );

        assert_eq!(flow_log.session_id(), "test-session");
        assert_eq!(flow_log.benchmark_id(), "test-benchmark");
        assert_eq!(flow_log.agent_type(), "test-agent");
    }
}
