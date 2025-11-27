//! Reev Core Library - Simplified Architecture
//!
//! This library implements the new reev-core architecture with:
//! - 18-step deterministic flow processing
//! - Snapshot-based testing for reliability
//! - Modular design with clear separation of concerns
//! - Mock-based testing for CI/CD reliability

pub mod core;
pub mod prompts;
pub mod test_snapshots;
pub mod types;
pub mod utils;

// Re-export main types for convenience
pub use core::*;
pub use test_snapshots::*;
pub use types::*;
pub use utils::*;

// Legacy modules that are kept for compatibility (to be removed later)
pub mod constants;
pub mod env;

// Modules needed for compatibility
pub mod actions;
pub mod agent;
pub mod benchmark;

// Legacy modules kept for compatibility (to be refactored later)
pub mod balance_validation;
pub mod db;
pub mod flow;
pub mod instruction_score;
pub mod llm_agent;
pub mod mock;
pub mod otel_extraction;
pub mod parsing;
pub mod results;
pub mod score;
pub mod server_utils;
pub mod session_logger;
pub mod solana_env;
pub mod test_scenarios;
pub mod trace;

// Remove obsolete modules - they cause errors and are not needed in new architecture
// (all modules now re-enabled for compatibility)

// Tests moved to tests/lib_tests.rs
