# 🪸 Reev Database Documentation

## 📚 Table of Contents
- [Overview](#overview)
- [Architecture](#architecture)
- [Database Schema](#database-schema)
- [Do's and Don'ts](#dos-and-donts)
- [Best Practices](#best-practices)
- [Common Issues & Solutions](#common-issues--solutions)
- [Testing & Validation](#testing--validation)
- [Migration Guide](#migration-guide)
- [Troubleshooting](#troubleshooting)

## Overview

The Reev project uses SQLite/Turso as its primary database for storing benchmarks, results, flow logs, and performance metrics. The database is designed for atomic operations and data integrity.

### Key Components
- **DatabaseWriter**: Main interface for database operations
- **DatabaseReader**: Query interface for reading data
- **Turso Integration**: SQLite-compatible database with remote sync capabilities

## Architecture

### Database Structure
```
reev_results.db
├── benchmarks (core benchmark data)
├── results (test execution results)
├── flow_logs (agent execution traces)
├── agent_performance (performance metrics)
└── yml_testresults (YAML-based test results)
```

### Connection Management
- **Single Connection**: One persistent connection per DatabaseWriter instance
- **Sequential Processing**: Eliminates race conditions
- **Connection Pooling**: NOT recommended for write operations

## Database Schema

### Table: benchmarks
```sql
CREATE TABLE benchmarks (
    id TEXT PRIMARY KEY,                    -- MD5 of benchmark_name:prompt
    benchmark_name TEXT NOT NULL,           -- e.g., "001-spl-transfer"
    prompt TEXT NOT NULL,                   -- The actual prompt text
    content TEXT NOT NULL,                  -- Full YAML content
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_benchmarks_name ON benchmarks(benchmark_name);
```

### Table: results
```sql
CREATE TABLE results (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    benchmark_id TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    prompt TEXT NOT NULL,
    generated_instruction TEXT NOT NULL,
    final_on_chain_state TEXT NOT NULL,
    final_status TEXT NOT NULL,
    score REAL NOT NULL,
    prompt_md5 TEXT,
    FOREIGN KEY (prompt_md5) REFERENCES benchmarks(id)
);

CREATE INDEX idx_results_prompt_md5 ON results(prompt_md5);
CREATE INDEX idx_results_timestamp ON results(timestamp);
```

### Table: flow_logs
```sql
CREATE TABLE flow_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL,
    benchmark_id TEXT NOT NULL,
    agent_type TEXT NOT NULL,
    start_time TEXT NOT NULL,
    end_time TEXT,
    final_result TEXT,
    flow_data TEXT NOT NULL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_flow_logs_benchmark_agent ON flow_logs(benchmark_id, agent_type);
```

### Table: agent_performance
```sql
CREATE TABLE agent_performance (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    benchmark_id TEXT NOT NULL,
    agent_type TEXT NOT NULL,
    score REAL NOT NULL,
    final_status TEXT NOT NULL,
    execution_time_ms INTEGER,
    timestamp TEXT NOT NULL,
    flow_log_id INTEGER,
    prompt_md5 TEXT,
    FOREIGN KEY (flow_log_id) REFERENCES flow_logs(id),
    FOREIGN KEY (prompt_md5) REFERENCES benchmarks(id)
);

CREATE INDEX idx_agent_performance_prompt_md5 ON agent_performance(prompt_md5);
CREATE INDEX idx_agent_performance_score ON agent_performance(score);
CREATE INDEX idx_agent_performance_timestamp ON agent_performance(timestamp);
```

## Do's and Don'ts

### ✅ Do's

#### Database Operations
- **DO** use sequential processing for write operations
- **DO** use single persistent connection per writer instance
- **DO** use ON CONFLICT DO UPDATE for upsert operations
- **DO** validate data before database operations
- **DO** use proper error handling with context
- **DO** log database operations for debugging

#### Connection Management
- **DO** create one DatabaseWriter instance per application
- **DO** share the same connection for related operations
- **DO** close connections properly when done
- **DO** use connection pooling for read operations only

#### Query Design
- **DO** use indexed columns in WHERE clauses
- **DO** parameterize queries to prevent SQL injection
- **DO** use transactions for multi-step operations
- **DO** limit result sets with appropriate filters

### ❌ Don'ts

#### Database Operations
- **DON'T** use multiple connections for the same write operation
- **DON'T** perform parallel writes to the same database
- **DON'T** ignore ON CONFLICT handling
- **DON'T use string interpolation for SQL queries
- **DON'T** store large blobs in frequently accessed tables

#### Connection Management
- **DON'T** create new connections for each operation
- **DON'T** share connections across threads without proper synchronization
- **DON'T** use connection pooling for write operations
- **DON'T** leave connections open indefinitely

#### Data Integrity
- **DON'T** ignore duplicate detection
- **DON'T** overwrite data without proper conflict resolution
- **DON'T** use mutable timestamps in primary keys
- **DON'T** rely on implicit data type conversions

## Best Practices

### 1. Atomic Operations
```rust
// ✅ GOOD: Use ON CONFLICT for atomic upserts
pub async fn upsert_benchmark(&self, name: &str, prompt: &str, content: &str) -> Result<String> {
    let prompt_md5 = format!("{:x}", md5::compute(format!("{}:{}", name, prompt).as_bytes()));
    
    self.conn.execute(
        "INSERT INTO benchmarks (id, benchmark_name, prompt, content, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?)
         ON CONFLICT(id) DO UPDATE SET
             benchmark_name = excluded.benchmark_name,
             prompt = excluded.prompt,
             content = excluded.content,
             updated_at = excluded.updated_at",
        [&prompt_md5, name, prompt, content, &timestamp, &timestamp]
    ).await?;
    
    Ok(prompt_md5)
}
```

### 2. Sequential Processing
```rust
// ✅ GOOD: Process files sequentially to avoid race conditions
pub async fn sync_benchmarks_to_db(&self, dir: &str) -> Result<usize> {
    let files = read_benchmark_files(dir).await?;
    let mut count = 0;
    
    for file in files {  // Sequential processing
        if let Ok(_) = self.sync_single_benchmark(&file).await {
            count += 1;
        }
    }
    
    Ok(count)
}
```

### 3. Single Connection Pattern
```rust
// ✅ GOOD: One connection per writer instance
pub struct DatabaseWriter {
    conn: Connection,  // Single persistent connection
}

impl DatabaseWriter {
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        let db = Builder::new_local(&config.path).build().await?;
        let conn = db.connect()?;  // Single connection
        
        Self::initialize_schema(&conn).await?;
        Ok(Self { conn })
    }
}
```

### 4. Error Handling with Context
```rust
// ✅ GOOD: Provide detailed error context
pub async fn get_benchmark(&self, id: &str) -> Result<Option<BenchmarkData>> {
    let mut rows = self.conn
        .query("SELECT * FROM benchmarks WHERE id = ?", [id])
        .await
        .context("Failed to query benchmark by ID")?;
    
    match rows.next().await? {
        Some(row) => {
            let benchmark = BenchmarkData {
                id: row.get(0).context("Failed to get benchmark ID")?,
                benchmark_name: row.get(1).context("Failed to get benchmark name")?,
                // ... other fields
            };
            Ok(Some(benchmark))
        }
        None => Ok(None)
    }
}
```

## Common Issues & Solutions

### Issue 1: Duplicate Record Creation
**Problem**: Multiple sync calls create duplicate records instead of updating existing ones.

**Root Causes**:
- Multiple database connections
- Parallel processing of same data
- Transaction boundary issues

**Solutions**:
```rust
// ✅ Solution 1: Single Connection Pattern
let db_writer = DatabaseWriter::new(config).await?;  // One instance

// ✅ Solution 2: Sequential Processing
for file in files {
    db_writer.sync_single_benchmark(&file).await?;  // Sequential
}

// ✅ Solution 3: Proper ON CONFLICT
"INSERT INTO benchmarks (...) VALUES (...) 
 ON CONFLICT(id) DO UPDATE SET ..."
```

### Issue 2: Race Conditions in Parallel Processing
**Problem**: Parallel tasks create inconsistent database state.

**Root Causes**:
- Shared connection used across threads
- Concurrent write operations
- Lack of proper synchronization

**Solutions**:
```rust
// ❌ BAD: Parallel processing with shared connection
let conn = db.connect()?;
for file in files {
    tokio::spawn(async move {
        conn.execute(...).await;  // Race condition!
    });
}

// ✅ GOOD: Sequential processing or separate connections
for file in files {
    conn.execute(...).await;  // Sequential
}

// OR use separate connections (not recommended for writes)
```

### Issue 3: MD5 Collision Concerns
**Problem**: Worried about MD5 collisions causing data loss.

**Reality**: MD5 is sufficient for this use case because:
- Input format is controlled (`benchmark_name:prompt`)
- Collision probability is negligible for dataset size
- Primary key constraint prevents accidental overwrites

**Solutions**:
```rust
// ✅ GOOD: Use MD5 for efficiency
let prompt_md5 = format!("{:x}", md5::compute(format!("{}:{}", name, prompt).as_bytes()));

// ✅ ALTERNATIVE: Use SHA-256 if extra security needed
let prompt_hash = format!("{:x}", sha256::digest(format!("{}:{}", name, prompt)));
```

### Issue 4: Timestamp Variations in Upserts
**Problem**: Same data generates different records due to timestamp differences.

**Root Cause**: Including timestamp in the content that generates the MD5.

**Solutions**:
```rust
// ❌ BAD: Include timestamp in content hash
let content_with_timestamp = format!("{}|timestamp:{}", content, timestamp);
let md5 = md5::compute(content_with_timestamp.as_bytes());

// ✅ GOOD: Hash only the core data
let prompt_md5 = format!("{:x}", md5::compute(format!("{}:{}", name, prompt).as_bytes()));
// Use timestamp separately for the database record
```

## Testing & Validation

### 1. Unit Testing Database Operations
```rust
#[tokio::test]
async fn test_duplicate_prevention() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test.db");
    let db_writer = DatabaseWriter::new(DatabaseConfig::new(db_path.to_string_lossy())).await?;
    
    // First insert
    let md5_1 = db_writer.upsert_benchmark("test", "prompt", "content").await?;
    
    // Second insert (should update, not create duplicate)
    let md5_2 = db_writer.upsert_benchmark("test", "prompt", "content").await?;
    
    assert_eq!(md5_1, md5_2);  // Same MD5
    
    let count = db_writer.get_all_benchmark_count().await?;
    assert_eq!(count, 1);  // Only one record
    
    Ok(())
}
```

### 2. Integration Testing Sync Operations
```rust
#[tokio::test]
async fn test_sync_no_duplicates() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let benchmarks_dir = create_test_benchmarks(&temp_dir)?;
    let db_writer = DatabaseWriter::new(DatabaseConfig::new(temp_dir.path().join("test.db").to_string_lossy())).await?;
    
    // First sync
    let count_1 = db_writer.sync_benchmarks_to_db(&benchmarks_dir.to_string_lossy()).await?;
    
    // Second sync
    let count_2 = db_writer.sync_benchmarks_to_db(&benchmarks_dir.to_string_lossy()).await?;
    
    assert_eq!(count_1, count_2);  // Same number of records
    
    let total_count = db_writer.get_all_benchmark_count().await?;
    assert_eq!(total_count, count_1 as i64);  // No duplicates
    
    Ok(())
}
```

### 3. Race Condition Testing
```rust
#[tokio::test]
async fn test_parallel_operations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_writer = Arc::new(DatabaseWriter::new(DatabaseConfig::new(temp_dir.path().join("test.db").to_string_lossy())).await?);
    
    let mut join_set = JoinSet::new();
    
    // Spawn parallel tasks (should be safe with proper implementation)
    for i in 0..10 {
        let writer = db_writer.clone();
        join_set.spawn(async move {
            writer.upsert_benchmark(
                &format!("test-{}", i % 3),  // Some duplicates
                "test prompt",
                "test content"
            ).await
        });
    }
    
    let mut results = Vec::new();
    while let Some(result) = join_set.join_next().await {
        results.push(result??);
    }
    
    let total_count = db_writer.get_all_benchmark_count().await?;
    assert_eq!(total_count, 3);  // Should have 3 unique records
    
    Ok(())
}
```

## Migration Guide

### Adding New Tables
```sql
-- 1. Create new table
CREATE TABLE new_table (
    id TEXT PRIMARY KEY,
    data TEXT NOT NULL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- 2. Create indexes
CREATE INDEX idx_new_table_data ON new_table(data);

-- 3. Add foreign key constraints if needed
ALTER TABLE existing_table 
ADD COLUMN new_table_id TEXT 
REFERENCES new_table(id);
```

### Schema Updates
```rust
// In DatabaseWriter::initialize_schema()
async fn initialize_schema(conn: &Connection) -> Result<()> {
    // Create tables with IF NOT EXISTS
    conn.execute("CREATE TABLE IF NOT EXISTS new_table (...)", ()).await?;
    
    // Create indexes with IF NOT EXISTS (SQLite 3.25+)
    conn.execute("CREATE INDEX IF NOT EXISTS idx_new_table ON new_table(column)", ()).await?;
    
    Ok(())
}
```

### Data Migration
```rust
pub async fn migrate_data_v1_to_v2(conn: &Connection) -> Result<()> {
    // Start transaction
    conn.execute("BEGIN TRANSACTION", ()).await?;
    
    // Migrate data
    conn.execute(
        "INSERT INTO new_table (id, data) 
         SELECT old_id, old_data FROM old_table",
        ()
    ).await?;
    
    // Commit transaction
    conn.execute("COMMIT", ()).await?;
    
    Ok(())
}
```

## Troubleshooting

### Common Error Messages

#### "Database is locked"
**Cause**: Multiple connections trying to write simultaneously
**Solution**:
```rust
// Ensure single connection usage
let db_writer = DatabaseWriter::new(config).await?;
// Process sequentially, not in parallel
```

#### "UNIQUE constraint failed"
**Cause**: Attempting to insert duplicate primary key
**Solution**:
```rust
// Use ON CONFLICT for proper upsert behavior
"INSERT INTO table (...) VALUES (...) 
 ON CONFLICT(id) DO UPDATE SET ..."
```

#### "No such table"
**Cause**: Database schema not initialized
**Solution**:
```rust
// Ensure schema initialization
Self::initialize_schema(&conn).await?;
```

### Debugging Tools

#### 1. Database Inspection Script
```rust
// Use this to inspect database state
async fn inspect_database(db_path: &str) -> Result<()> {
    let db = Builder::new_local(db_path).build().await?;
    let conn = db.connect()?;
    
    // Check table counts
    let mut rows = conn.query("SELECT name FROM sqlite_master WHERE type='table'", ()).await?;
    while let Some(row) = rows.next().await? {
        let table_name: String = row.get(0)?;
        let mut count_rows = conn.query(&format!("SELECT COUNT(*) FROM {}", table_name), ()).await?;
        if let Some(count_row) = count_rows.next().await? {
            let count: i64 = count_row.get(0)?;
            println!("Table {}: {} records", table_name, count);
        }
    }
    
    Ok(())
}
```

#### 2. Duplicate Detection
```rust
// Check for duplicates in benchmarks table
async fn check_duplicates(conn: &Connection) -> Result<Vec<(String, i64)>> {
    let mut rows = conn.query(
        "SELECT id, COUNT(*) as count FROM benchmarks 
         GROUP BY id HAVING COUNT(*) > 1",
        ()
    ).await?;
    
    let mut duplicates = Vec::new();
    while let Some(row) = rows.next().await? {
        let id: String = row.get(0)?;
        let count: i64 = row.get(1)?;
        duplicates.push((id, count));
    }
    
    Ok(duplicates)
}
```

#### 3. Performance Monitoring
```rust
// Monitor database performance
async fn database_stats(conn: &Connection) -> Result<()> {
    // Get database size
    let mut rows = conn.query("SELECT page_count * page_size as size FROM pragma_page_count(), pragma_page_size()", ()).await?;
    if let Some(row) = rows.next().await? {
        let size: i64 = row.get(0)?;
        println!("Database size: {} bytes", size);
    }
    
    // Get record counts
    let tables = vec!["benchmarks", "results", "flow_logs"];
    for table in tables {
        let mut rows = conn.query(&format!("SELECT COUNT(*) FROM {}", table), ()).await?;
        if let Some(row) = rows.next().await? {
            let count: i64 = row.get(0)?;
            println!("{}: {} records", table, count);
        }
    }
    
    Ok(())
}
```

---

## 📝 Maintenance Notes

### Regular Tasks
- Monitor database size and performance
- Check for duplicate records periodically
- Backup database before major changes
- Validate data integrity after updates

### Performance Optimization
- Keep indexes minimal and targeted
- Archive old results data regularly
- Use EXPLAIN QUERY PLAN for slow queries
- Monitor connection usage patterns

### Security Considerations
- Validate all inputs before database operations
- Use parameterized queries exclusively
- Monitor for unusual access patterns
- Regular database integrity checks

---

*Last Updated: 2025-10-15*  
*Version: 1.0*  
*Maintainer: Reev Development Team*