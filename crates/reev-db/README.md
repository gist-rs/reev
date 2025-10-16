# 🪸 reev-db

A robust SQLite/Turso database library for the Reev project, providing atomic operations with duplicate prevention and comprehensive monitoring capabilities.

## 🚀 Features

- **Atomic Upsert Operations**: Prevent duplicate records with MD5-based deduplication
- **Dynamic Query Filtering**: Advanced filtering with parameterized queries
- **Comprehensive Error Handling**: Detailed error context with thiserror integration
- **Connection Management**: Single connection pattern for optimal performance
- **Monitoring & Diagnostics**: Built-in duplicate detection and database statistics
- **Async/First Design**: Full async support with tokio integration

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
reev-db = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

## 🏗️ Architecture

### Core Components

- **`DatabaseWriter`**: Primary interface for write operations and upserts
- **`DatabaseReader`**: Query interface with advanced filtering capabilities
- **`DatabaseConfig`**: Configuration builder for local and remote databases
- **`QueryFilter`**: Type-safe dynamic query building

### Database Schema

```sql
-- Core benchmarks table
CREATE TABLE benchmarks (
    id TEXT PRIMARY KEY,                    -- MD5 of benchmark_name:prompt
    benchmark_name TEXT NOT NULL,           -- e.g., "001-spl-transfer"
    prompt TEXT NOT NULL,                   -- The actual prompt text
    content TEXT NOT NULL,                  -- Full YAML content
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- Test execution results
CREATE TABLE results (
    id TEXT PRIMARY KEY,
    benchmark_id TEXT,
    timestamp TEXT,
    prompt TEXT,
    generated_instruction TEXT,
    final_on_chain_state TEXT,
    final_status TEXT,
    score REAL,
    prompt_md5 TEXT,
    FOREIGN KEY (benchmark_id) REFERENCES benchmarks(id)
);

-- Agent execution traces
CREATE TABLE flow_logs (
    id TEXT PRIMARY KEY,
    benchmark_id TEXT,
    timestamp TEXT,
    agent_type TEXT,
    action TEXT,
    input_data TEXT,
    output_data TEXT,
    execution_time_ms INTEGER,
    FOREIGN KEY (benchmark_id) REFERENCES benchmarks(id)
);

-- Performance metrics
CREATE TABLE agent_performance (
    id TEXT PRIMARY KEY,
    benchmark_id TEXT,
    agent_type TEXT,
    score REAL,
    execution_time_ms INTEGER,
    timestamp TEXT,
    FOREIGN KEY (benchmark_id) REFERENCES benchmarks(id)
);
```

## 🔧 Quick Start

### Basic Usage

```rust
use reev_db::{DatabaseConfig, DatabaseWriter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create database configuration
    let config = DatabaseConfig::new("path/to/database.db");

    // Create database writer
    let db = DatabaseWriter::new(config).await?;

    // Upsert benchmark (creates or updates)
    let md5 = db.upsert_benchmark(
        "001-spl-transfer",
        "Transfer 1 SOL to recipient",
        "full_yaml_content_here"
    ).await?;

    println!("Benchmark upserted with MD5: {}", md5);

    // Check for duplicates
    let duplicates = db.check_for_duplicates().await?;
    if duplicates.is_empty() {
        println!("No duplicates found");
    }

    Ok(())
}
```

### Advanced Querying

```rust
use reev_db::{DatabaseReader, QueryFilter, DatabaseConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = DatabaseConfig::new("path/to/database.db");
    let reader = DatabaseReader::from_config(config).await?;

    // Build dynamic query filter
    let filter = QueryFilter::new()
        .benchmark_name("spl-transfer")
        .agent_type("llm")
        .score_range(0.5, 1.0)
        .paginate(10, 0)
        .sort_by("score", "desc");

    // Execute query
    let results = reader.get_test_results(Some(filter)).await?;

    for result in results {
        println!("Benchmark: {}, Score: {}",
            result.benchmark_name,
            result.score.unwrap_or(0.0)
        );
    }

    Ok(())
}
```

## ⚡ Performance & Concurrency

### Critical: Sequential Processing Only

**Turso has significant concurrency limitations**. Based on extensive testing:

- ✅ **Sequential Processing**: 100% reliable, recommended for production
- ⚠️ **Low Concurrency** (5-10 items): May work but not guaranteed
- ❌ **High Concurrency** (10-20+ items): Expected to fail with connection/locking issues

### Recommended Pattern

```rust
// ✅ GOOD: Sequential processing
let benchmarks = load_benchmarks_from_directory().await?;
for benchmark in benchmarks {
    db.upsert_benchmark(&benchmark.name, &benchmark.prompt, &benchmark.content)
        .await?;
}

// ❌ BAD: Parallel processing (will cause issues)
use tokio::task::JoinSet;
let mut join_set = JoinSet::new();
for benchmark in benchmarks {
    let db_clone = db.clone();
    join_set.spawn(async move {
        db_clone.upsert_benchmark(&benchmark.name, &benchmark.prompt, &benchmark.content).await
    });
}
// This will likely fail with 10+ items
```

### Connection Management

- **Single Connection**: One connection per `DatabaseWriter` instance
- **No Connection Pooling**: Not recommended for write operations
- **Sequential Access**: Prevents race conditions and locking issues

## 🔍 Error Handling

The library provides comprehensive error types with context:

```rust
use reev_db::DatabaseError;

match db.upsert_benchmark("test", "prompt", "content").await {
    Ok(md5) => println!("Success: {}", md5),
    Err(DatabaseError::DuplicateDetected { id, count }) => {
        println!("Duplicate detected: {} ({} occurrences)", id, count);
    }
    Err(DatabaseError::Connection(msg)) => {
        println!("Connection error: {}", msg);
    }
    Err(e) => println!("Other error: {}", e),
}
```

## 🛠️ Configuration

### Local Database

```rust
let config = DatabaseConfig::new("path/to/database.db");
```

### In-Memory Database

```rust
let config = DatabaseConfig::new(":memory:");
```

### Remote Turso Database

```rust
let config = DatabaseConfig::turso(
    "libsql://my-db.turso.io",
    "your-auth-token".to_string()
);
```

### Builder Pattern

```rust
let config = DatabaseConfigBuilder::new("database.db")
    .timeout(60)
    .max_retries(5)
    .enable_pooling(true)
    .max_pool_size(10)
    .build();
```

## 📊 Monitoring & Diagnostics

### Database Statistics

```rust
let stats = db.get_database_stats().await?;
println!("Total benchmarks: {}", stats.total_benchmarks);
println!("Duplicate count: {}", stats.duplicate_count);
```

### Duplicate Detection

```rust
let duplicates = db.check_for_duplicates().await?;
for duplicate in duplicates {
    println!("Duplicate: {} ({} occurrences)",
        duplicate.benchmark_name,
        duplicate.count
    );
}
```

### Database Inspection

The library includes inspection tools:

```bash
# Run database inspector
cargo run --bin db-inspector -- --database path/to/db.db --overview

# Check for duplicates
cargo run --bin duplicate-tester -- --database path/to/db.db
```

## 🧪 Testing

### Running Tests

```bash
# Run all tests
cargo test -p reev-db

# Run specific test modules
cargo test -p reev-db --test config_tests
cargo test -p reev-db --test writer_tests
```

### Concurrency Testing

For understanding Turso's limitations, run the consolidated test:

```bash
cd turso-test/step_by_step
cargo run --bin consolidated_upsert_concurrency
```

This test demonstrates:
1. ✅ Working proof of upsert functionality
2. ❌ Expected failures with high concurrency (10-20+ items)

## 📋 Do's and Don'ts

### ✅ Do's

- Use sequential processing for all database operations
- Create one `DatabaseWriter` instance per application
- Use proper error handling with `DatabaseError` types
- Implement retry logic for connection issues
- Use the `QueryFilter` builder for dynamic queries
- Check for duplicates periodically in production

### ❌ Don'ts

- Use parallel/concurrent database writes
- Share connections across async tasks without care
- Assume database operations are thread-safe by default
- Use connection pooling for write operations
- Ignore error messages about locking or connection issues
- Process more than 10 items concurrently

## 🔧 Advanced Usage

### Batch Operations

```rust
// Process multiple benchmarks sequentially
let batch_results = db.sync_benchmarks_from_dir("benchmarks/").await?;
println!("Processed: {}, New: {}, Updated: {}",
    batch_results.processed_count,
    batch_results.new_count,
    batch_results.updated_count
);
```

### Custom Queries

```rust
// Execute custom SQL with parameters
let mut rows = conn.query(
    "SELECT * FROM benchmarks WHERE score >= ? AND agent_type = ?",
    [0.8, "llm"]
).await?;

while let Some(row) = rows.next().await? {
    let name: String = row.get(0)?;
    let score: f64 = row.get(1)?;
    println!("{}: {}", name, score);
}
```

## 🐛 Troubleshooting

### Common Issues

1. **"Database is locked" errors**
   - Cause: Concurrent access attempts
   - Solution: Use sequential processing

2. **Connection timeouts**
   - Cause: Network issues or overloaded database
   - Solution: Increase timeout in configuration

3. **Duplicate records**
   - Cause: Bypassing the upsert logic
   - Solution: Always use `upsert_benchmark` method

### Debug Mode

Enable debug logging:

```rust
use tracing_subscriber;

tracing_subscriber::fmt::init();
```

This will show detailed database operation logs.

## 📄 License

This project is part of the Reev ecosystem and follows the same licensing terms.

## 🤝 Contributing

When contributing to reev-db:

1. Always test with sequential processing patterns
2. Add comprehensive error handling
3. Include tests that demonstrate concurrency limitations
4. Update documentation for any API changes
5. Run `cargo clippy --fix --allow-dirty` before commits

---

**Key Takeaway**: reev-db provides robust database operations when used sequentially. Avoid concurrent operations to prevent reliability issues with Turso.
