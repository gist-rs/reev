# 🪸 Reev TOFIX Tasks - Current Issues

## 🐛 **SYNC ENDPOINT DUPLICATE CREATION ISSUE**

### Issue Description
The POST /api/v1/sync endpoint creates duplicate records instead of updating existing ones when called multiple times.

### Root Cause Identified ✅
- **Implementation Issue**: The problem is NOT with Turso's ON CONFLICT DO UPDATE functionality
- **Pure SQLite ON CONFLICT**: ✅ Works correctly (proven with test)
- **Our Implementation**: ❌ Creates duplicates despite identical IDs

### Proof Created ✅
**Location**: `/Users/katopz/git/gist/reev/turso-test/minimal_test.rs`
**Test Results**:
```bash
🧪 Minimal Turso ON CONFLICT Test
📝 Test 1: Pure SQLite ON CONFLICT
SQLite result: 1
same-id|second
✅ SUCCESS: Pure SQLite ON CONFLICT works - 1 record with updated name
```

### Partial Fix Applied ✅
1. **Fixed MD5 Collision**: Resolved issue with 002-spl-transfer being overwritten
2. **Improved Sync Logic**: Sequential processing without concurrency
3. **Enhanced Error Handling**: Better logging and failure recovery

### Current Status ⚠️
- **First sync**: ✅ Creates 13 unique benchmark records 
- **Second sync**: ❌ Creates 13 additional duplicates (total 26)
- **Root cause**: Issue in our database connection/transaction handling

### Technical Analysis
- **File**: `crates/reev-lib/src/db/writer.rs`
- **Functions**: `sync_benchmarks_to_db()`, `upsert_benchmark()`
- **Issue**: Database connection management or transaction boundaries
- **Evidence**: Pure SQLite ON CONFLICT works, our Turso usage doesn't

### Remaining Tasks
1. **Investigate Database Connection**: Check if multiple connections are causing issues
2. **Fix Transaction Management**: Ensure proper transaction boundaries
3. **Test Connection Isolation**: Verify database connection behavior
4. **Implement Workaround**: Use manual upsert if needed

### Expected Behavior
- First sync: Creates 13 unique benchmark records ✅
- Second sync: Updates existing records, creates no duplicates ❌
- All prompt_md5 lookups should work correctly
- No database integrity issues

### Priority: HIGH
- Core functionality works but creates data bloat
- System is functional but inefficient
- Needs proper investigation and fix

### Notes for Next Task
- Use the proof in `/turso-test` directory as reference
- Focus on database connection and transaction management
- Do NOT blame Turso library - the issue is in our implementation
- Pure SQLite ON CONFLICT works perfectly as proven