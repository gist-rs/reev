//! Dynamic Flow Handlers
//!
//! This module provides API endpoints for executing dynamic flows through REST API.
//! It integrates with reev-orchestrator to provide same functionality available via CLI.

pub mod execute_dynamic_flow;
pub mod execute_flow_plan_with_ping_pong;
pub mod execute_recovery_flow;
pub mod extract_transaction_details;
pub mod get_recovery_metrics;

// Re-export public functions
pub use execute_dynamic_flow::execute_dynamic_flow;
pub use execute_recovery_flow::execute_recovery_flow;
pub use get_recovery_metrics::get_recovery_metrics;

// Private functions are not re-exported
// execute_flow_plan_with_ping_pong and extract_transaction_details are internal utilities
