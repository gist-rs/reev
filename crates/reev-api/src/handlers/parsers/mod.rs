//! Execution trace parsers module
//!
//! This module provides parsers for converting execution data into human-readable
//! ASCII trace formats. The parsers handle different data sources including:
//! - Session data from completed executions
//! - Session logs from database
//! - Various execution result formats

pub mod execution_trace;
pub mod transaction_logs;

pub use execution_trace::ExecutionTraceParser;
pub use transaction_logs::TransactionLogParser;
