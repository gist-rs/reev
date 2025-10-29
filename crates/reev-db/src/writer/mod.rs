//! Database writer module
//!
//! Provides modular database write operations organized by functionality:
//! - Core database operations
//! - Session management
//! - Benchmark synchronization
//! - Performance tracking
//! - Database monitoring

pub mod benchmarks;
pub mod core;
pub mod execution_states;
pub mod monitoring;
pub mod performance;
pub mod sessions;

// Re-export main DatabaseWriter for backward compatibility
pub use core::DatabaseWriter;

/// Database writer trait for common operations
#[allow(async_fn_in_trait)]
pub trait DatabaseWriterTrait: Send + Sync {
    /// Store execution state in database
    async fn store_execution_state(
        &self,
        state: &reev_types::ExecutionState,
    ) -> crate::error::Result<()>;

    /// Get execution state by ID
    async fn get_execution_state(
        &self,
        execution_id: &str,
    ) -> crate::error::Result<Option<reev_types::ExecutionState>>;
}

/// Implementation of DatabaseWriterTrait for DatabaseWriter
impl DatabaseWriterTrait for crate::writer::DatabaseWriter {
    /// Store execution state in database
    async fn store_execution_state(
        &self,
        state: &reev_types::ExecutionState,
    ) -> crate::error::Result<()> {
        use crate::writer::execution_states::ExecutionStatesWriter;
        let writer = ExecutionStatesWriter::new(&self.conn);
        writer.store_execution_state(state).await
    }

    /// Get execution state by ID
    async fn get_execution_state(
        &self,
        execution_id: &str,
    ) -> crate::error::Result<Option<reev_types::ExecutionState>> {
        use crate::writer::execution_states::ExecutionStatesWriter;
        let writer = ExecutionStatesWriter::new(&self.conn);
        writer.get_execution_state(execution_id).await
    }
}
