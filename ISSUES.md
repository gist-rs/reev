# Issues

## Issue #51: Phase 1 - Database Schema & Methods for Consolidation

### Description
Implement database infrastructure for session consolidation in PingPongExecutor.

### Tasks
1. **Add reev-db dependency**
   - Add `reev-db = { path = "../reev-db" }` to `reev/crates/reev-orchestrator/Cargo.toml`

2. **Create consolidated_sessions table**
```sql
CREATE TABLE consolidated_sessions (
    id INTEGER PRIMARY KEY,
    execution_id TEXT NOT NULL,
    consolidated_session_id TEXT UNIQUE NOT NULL,
    consolidated_content TEXT NOT NULL,
    original_session_ids TEXT NOT NULL, -- JSON array of step session_ids
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    avg_score REAL,
    total_tools INTEGER,
    success_rate REAL,
    execution_duration_ms INTEGER,
    FOREIGN KEY (execution_id) REFERENCES execution_sessions(execution_id)
);
```

3. **Extend DatabaseWriterTrait**
```rust
// Add these methods to DatabaseWriterTrait
pub trait DatabaseWriterTrait: Send + Sync {
    // ... existing methods ...
    
    /// Store individual step session (for dynamic mode)
    async fn store_step_session(
        &self,
        execution_id: &str,
        step_index: usize,
        session_content: &str,
    ) -> crate::error::Result<()>;
    
    /// Get all sessions for consolidation (supports ping-pong)
    async fn get_sessions_for_consolidation(
        &self,
        execution_id: &str,
    ) -> crate::error::Result<Vec<SessionLog>>;
    
    /// Store consolidated session (ping-pong result)
    async fn store_consolidated_session(
        &self,
        consolidated_id: &str,
        execution_id: &str,
        content: &str,
        metadata: &ConsolidationMetadata,
    ) -> crate::error::Result<()>;
    
    /// Get consolidated session (for Mermaid generation)
    async fn get_consolidated_session(
        &self,
        consolidated_id: &str,
    ) -> crate::error::Result<Option<String>>;
    
    /// Begin transaction for step storage
    async fn begin_transaction(&self, execution_id: &str) -> crate::error::Result<()>;
    
    /// Commit transaction
    async fn commit_transaction(&self, execution_id: &str) -> crate::error::Result<()>;
    
    /// Rollback transaction on failure
    async fn rollback_transaction(&self, execution_id: &str) -> crate::error::Result<()>;
}
```

4. **Implement trait methods**
   - Add implementations in `reev/crates/reev-db/src/writer/mod.rs`
   - Create database migration script for new table
   - Add error handling for transaction operations

### Success Criteria
- All new DatabaseWriterTrait methods implemented and tested
- consolidated_sessions table created with proper constraints
- Transaction support working for step storage
- Can store and retrieve consolidated sessions

### Dependencies
- None (foundational phase)

### Notes
- This is foundational work for PingPongExecutor database integration
- Uses existing DatabaseWriter architecture in reev-db
- Follows existing error handling patterns in the codebase

## Issue #50
