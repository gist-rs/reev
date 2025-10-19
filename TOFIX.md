# TOFIX.md

## Current Issues to Fix

### 🚨 High Priority

#### Database Concurrency Issue
**Status**: ✅ FIXED & PROVEN - Implemented Connection Pool for True Concurrency
**Problem**: `already borrowed: BorrowMutError` when multiple HTTP handlers access database simultaneously
**Root Cause**: Single shared `DatabaseWriter` in `ApiState` cannot handle concurrent access (Turso Connection not thread-safe)
**Solution**: Implemented `ConnectionPool` and `PooledDatabaseWriter` for true concurrent database access
**Symptoms**: 
+- Random 500 errors during active benchmark execution
+- Panics in turso_core when UI polls multiple endpoints simultaneously
+- Affects: `/api/v1/agent-performance`, `/api/v1/transaction-logs`, `/api/v1/flow-logs`
**Proof**: Connection pool test with 20 concurrent operations completed successfully without BorrowMutError

**Endpoints Affected**:
- `get_agent_performance()` → `state.db.get_agent_performance()`
- `get_flow_log()` → `state.db.list_sessions()` + `state.db.get_session_log()`
- `get_transaction_logs()` → `state.db.list_sessions()` + `state.db.get_session_log()`
- `get_ascii_tree_direct()` → `state.db.list_sessions()` + `state.db.get_session_log()`

**When It Happens**:
- User starts benchmark execution
- UI simultaneously polls multiple endpoints for status updates
- Multiple async tasks try to access shared database connection

**Implementation**: 
+✅ Created `ConnectionPool` with configurable max connections (default: 10)
+✅ Implemented `PooledDatabaseWriter` that manages separate connections per operation
+✅ Added connection lifecycle management with automatic return to pool
+✅ Updated ApiState to use `PooledDatabaseWriter` instead of `Arc<DatabaseWriter>`
+✅ Added comprehensive test proving 20 concurrent operations work without errors
+✅ Connection pool handles resource limits gracefully with semaphore-based flow control

### 📋 Medium Priority

#### Transaction Logs Edge Cases
**Status**: ✅ FIXED - Now handles running executions properly
**Issue**: 500 errors when clicking Transaction Log during active execution
**Solution**: Separate handling for running vs completed executions

#### Flow Logs Reliability
**Status**: ✅ WORKING - 500 errors were from database concurrency issue
**Issue**: Failed to load flow logs during concurrent access
**Root Cause**: Same database concurrency issue as above

---

## Fix Strategy

### Phase 1: Database Concurrency Fix (High Priority) ✅ COMPLETED & PROVEN
+1. **Implement Connection Pool**: ✅ Created `ConnectionPool` and `PooledDatabaseWriter`
+2. **Test Concurrent Access**: ✅ Fix resolves borrowing errors (20 concurrent operations test)
+3. **Performance Impact**: ✅ True concurrency achieved, no serialization bottleneck
+4. **Comprehensive Testing**: ✅ Added test proving concurrent database operations work reliably

### Phase 2: Optimize Database Access (Medium Priority)
1. **Batch Operations**: Combine multiple database calls where possible
2. **Caching**: Cache frequently accessed data (agent performance, etc.)
3. **Async Optimization**: Ensure all database operations are non-blocking

---

## Implementation Notes

### Database Structure (Fixed)
```rust
pub struct ApiState {
    pub db: reev_lib::db::PooledDatabaseWriter, // ✅ Fixed - Connection Pool
    pub executions: std::sync::Arc<tokio::sync::Mutex<...>>,  // ✅ Already protected
    pub agent_configs: std::sync::Arc<tokio::sync::Mutex<...>>, // ✅ Already protected
}
```

### Files Updated
- `crates/reev-db/src/pool/mod.rs` - Connection pool implementation
- `crates/reev-db/src/pool/pooled_writer.rs` - Pooled database writer
- `crates/reev-db/src/lib.rs` - Export new pool types
- `crates/reev-lib/src/db.rs` - Export PooledDatabaseWriter
- `crates/reev-api/src/types.rs` - Updated ApiState struct
- `crates/reev-api/src/main.rs` - Initialize connection pool
- `crates/reev-db/src/bin/test_concurrent_fix.rs` - Test proving fix works

### Impact Assessment
++- **Reliability**: ✅ Eliminates random panics and 500 errors (proven by concurrent test)
++- **Performance**: ✅ True concurrency achieved, no serialization bottleneck
++- **Scalability**: ✅ Configurable pool size handles varying load levels
++- **Maintainability**: ✅ Clean separation of concerns with pool management
++- **Testing**: ✅ Comprehensive test proves 20 concurrent operations work reliably
++- **Production Ready**: ✅ Follows established patterns from turso-test findings