# 🪸 Reev Benchmark Management System - TOFIX Tasks

## 🎯 **PHASE 23: BENCHMARK MANAGEMENT SYSTEM** 

**Objective**: Create centralized benchmark management with database-backed storage

---

## 📋 **DATABASE SCHEMA UPDATES**

### Task 1: Create Benchmark Table
- [ ] Create `benchmarks` table with schema:
```sql
CREATE TABLE benchmarks (
    id TEXT PRIMARY KEY,  -- MD5 of prompt
    prompt TEXT NOT NULL,
    content TEXT NOT NULL, -- Full YML content
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
);
```

### Task 2: Update Existing Tables
- [ ] Add `prompt_md5` field to `agent_performance` table
- [ ] Add `prompt_md5` field to `results` table  
- [ ] Create index on `prompt_md5` for performance
```sql
CREATE INDEX idx_agent_performance_prompt_md5 ON agent_performance(prompt_md5);
CREATE INDEX idx_results_prompt_md5 ON results(prompt_md5);
```

---

## 📋 **CORE IMPLEMENTATION TASKS**

### Task 3: Benchmark Upsert Functions
- [ ] Create `upsert_benchmark_to_db()` function in `reev-lib/src/db/`
- [ ] Implement MD5 hash calculation for prompt identification
- [ ] Add YML content validation before database insertion
- [ ] Handle duplicate detection and updates gracefully

### Task 4: Startup Sync Process
- [ ] Create `sync_benchmarks_to_db()` function for startup
- [ ] Scan `benchmarks/` directory for all `.yml` files
- [ ] Parse each YAML file and calculate prompt MD5
- [ ] Bulk upsert all benchmarks to database
- [ ] Add logging for sync process and error handling

### Task 5: Update Test Result Storage
- [ ] Modify `insert_agent_performance()` to accept `prompt_md5`
- [ ] Update `insert_result()` to include `prompt_md5` field
- [ ] Modify flow logging to capture and store prompt hash
- [ ] Update web API services to calculate and pass prompt MD5

### Task 6: API Endpoint Implementation
- [ ] Create `/upsert_yml` POST endpoint in `reev-api/src/handlers.rs`
- [ ] Implement YML content validation and parsing
- [ ] Add MD5 calculation and database upsert logic
- [ ] Add proper error responses and logging
- [ ] Include benchmark ID in response for confirmation

---

## 📋 **RUNTIME INTEGRATION TASKS**

### Task 7: Database-First Benchmark Reading
- [ ] Create `get_benchmark_by_id()` function reading from DB
- [ ] Update all benchmark reading logic to use database instead of filesystem
- [ ] Add caching layer for frequently accessed benchmarks
- [ ] Implement fallback to filesystem if database lookup fails

### Task 8: API Response Enhancement
- [ ] Update `get_agent_performance()` to include prompt content when available
- [ ] Join with `benchmarks` table to get full prompt when needed
- [ ] Optimize query to avoid performance impact
- [ ] Add optional prompt content to API responses

### Task 9: Web UI Integration Preparation
- [ ] Add benchmark content to API responses for future UI
- [ ] Include benchmark metadata (description, tags) in responses
- [ ] Structure responses for easy UI consumption
- [ ] Add benchmark update capabilities for future editing

---

## 📋 **MIGRATION & TESTING TASKS**

### Task 10: Database Migration Script
- [ ] Create migration script for existing data
- [ ] Handle current `agent_performance` records without prompt hashes
- [ ] Backfill prompt MD5 for existing records using benchmark file matching
- [ ] Add data validation after migration

### Task 11: Comprehensive Testing
- [ ] Test benchmark upsert with various YML formats
- [ ] Test MD5 hash consistency and collision handling
- [ ] Test API endpoint with valid/invalid YML content
- [ ] Test startup sync with large number of benchmark files
- [ ] Test database-first reading performance

### Task 12: Error Handling & Edge Cases
- [ ] Handle duplicate prompt MD5 detection
- [ ] Handle invalid YML content gracefully
- [ ] Handle database connection failures during sync
- [ ] Handle filesystem access issues during startup
- [ ] Add proper logging for all error scenarios

---

## 📋 **PERFORMANCE OPTIMIZATION**

### Task 13: Caching Strategy
- [ ] Implement in-memory caching for frequently accessed benchmarks
- [ ] Add cache invalidation on benchmark updates
- [ ] Optimize database queries with proper indexing
- [ ] Monitor performance impact of new joins

### Task 14: Storage Optimization
- [ ] Compress YML content in database if needed
- [ ] Implement cleanup for old benchmark versions
- [ ] Monitor disk space usage with new storage approach
- [ ] Add database maintenance routines

---

## 🎯 **SUCCESS CRITERIA**

- [ ] All benchmarks stored in database with MD5-based identification
- [ ] Startup sync process runs automatically and reliably
- [ ] `/upsert_yml` API endpoint functional and tested
- [ ] All benchmark reads use database instead of filesystem
- [ ] API responses include prompt content when requested
- [ ] Performance meets or exceeds current filesystem-based approach
- [ ] Comprehensive test coverage for all new functionality
- [ ] Error handling robust across all scenarios

---

## 🚀 **FUTURE FOUNDATION**

This system will enable:
- Runtime benchmark management without server restarts
- Future UI-based benchmark editing capabilities
- Better traceability between test results and benchmark content
- Efficient storage using MD5 hashes instead of full prompts
- Foundation for benchmark versioning and history tracking

**Priority**: HIGH - Core infrastructure for future development
**Estimated Effort**: 2-3 days for full implementation
**Dependencies**: Database consolidation (Phase 22) - COMPLETED ✅