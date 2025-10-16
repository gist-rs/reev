# Turso Test Suite

A comprehensive test suite for evaluating Turso database behavior, particularly focusing on UPSERT operations and concurrency handling.

## 🎯 Purpose

This test suite was created to investigate and demonstrate Turso's behavior with:
- Basic database operations (INSERT, UPDATE, SELECT)
- UPSERT operations using `ON CONFLICT DO UPDATE`
- Concurrency limitations and database locking issues
- Connection sharing across async tasks

## 📁 Project Structure

```
turso-test/
├── src/
│   └── main.rs              # Basic usage examples
├── tests/
│   ├── step_by_step/        # Step-by-step tutorial tests
│   │   ├── turso_upsert_concurrency_test.rs
│   │   ├── step6_reproduce_original_issue.rs
│   │   └── ...
│   ├── debug_test.rs        # Pure SQLite vs Turso comparison
│   ├── debug_upsert.rs      # Detailed upsert debugging
│   ├── inspect_db.rs        # Real database inspection
│   └── minimal_test.rs      # Minimal reproduction test
├── Cargo.toml               # Project configuration
└── README.md                # This file
```

## 🚀 Quick Start

### Basic Usage Example

```bash
cargo run --bin simple_test
```

This demonstrates:
- Database connection setup
- Table creation
- Basic INSERT operations
- UPSERT with `ON CONFLICT DO UPDATE`
- Data reading and verification

### Concurrency Testing

```bash
cargo run --bin turso_upsert_concurrency_test
```

This comprehensive test suite demonstrates:
- ✅ Sequential processing: Works perfectly (20/20 success)
- ⚠️ Moderate concurrency: Limited success with database locking errors
- ⚠️ High concurrency: Significant issues with `BorrowMutError` panics

## 🔍 Key Findings

### 1. Turso Concurrency Limitations

**Sequential Processing**: ✅ Recommended
- 100% success rate
- No database locking issues
- Predictable behavior

**Parallel Processing**: ❌ Not Recommended
- `BorrowMutError` panics with 10+ concurrent operations
- Database locking conflicts
- Connection sharing issues

**Root Cause**: Turso's internal database locking mechanisms don't handle concurrent access to the same connection well.

### 2. UPSERT Behavior

**Basic UPSERT**: ✅ Works correctly
```sql
INSERT INTO benchmarks (id, name, content) 
VALUES (?, ?, ?) 
ON CONFLICT(id) DO UPDATE SET 
    name = excluded.name, 
    content = excluded.content;
```

**Concurrent UPSERT**: ❌ Problematic
- Multiple tasks sharing same connection cause panics
- Database locking prevents proper concurrent updates
- Use sequential processing instead

## 📋 Available Tests

### Main Examples
- `cargo run --bin simple_test` - Basic usage demonstration
- `cargo run --bin turso_upsert_concurrency_test` - Full concurrency test suite

### Debug Tests
- `cargo run --bin debug_test` - Pure SQLite vs Turso comparison
- `cargo run --bin debug_upsert` - Detailed upsert operation debugging
- `cargo run --bin inspect_db` - Real database state inspection
- `cargo run --bin minimal_test` - Minimal reproduction of issues

### Step-by-Step Tutorials
Located in `tests/step_by_step/`:
- `step1_connection.rs` - Database connection basics
- `step2_basic_insert.rs` - Basic INSERT operations
- `step3_on_conflict.rs` - ON CONFLICT clause usage
- `step4_upsert_benchmark.rs` - Benchmark upsert implementation
- `step5_rapid_calls.rs` - Rapid successive calls
- `step6_reproduce_original_issue.rs` - Original issue reproduction

## 🛠️ Technical Details

### Database Schema
```sql
CREATE TABLE benchmarks (
    id TEXT PRIMARY KEY,                    -- MD5 hash of benchmark_name:prompt
    benchmark_name TEXT NOT NULL,          -- e.g., "001-sol-transfer"
    prompt TEXT NOT NULL,                  -- The actual prompt text
    content TEXT NOT NULL,                 -- Full YML content
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
);
```

### MD5 Generation
```rust
let prompt_md5 = format!("{:x}", md5::compute(format!("{}:{}", benchmark_name, prompt).as_bytes()));
```

### Key Dependencies
- `turso = "0.1.5"` - Database driver
- `tokio = { version = "1.0", features = ["full"] }` - Async runtime
- `anyhow = "1.0"` - Error handling
- `chrono = { version = "0.4", features = ["serde"] }` - Timestamps
- `md5 = "0.7"` - Hash generation

## 🚨 Known Issues

1. **Concurrency Limitations**: Turso doesn't handle concurrent writes well on the same connection
2. **Database Locking**: Multiple async tasks sharing connections cause `BorrowMutError`
3. **API Incompatibility**: Some tutorials use older Turso API versions

## ✅ Best Practices

1. **Use Sequential Processing**: For production code, process database operations sequentially
2. **One Connection Per Writer**: Avoid sharing connections across concurrent tasks
3. **Proper Error Handling**: Implement retry logic for database operations
4. **Connection Management**: Create new connections for different writer instances

## 🔧 Running Tests

### Prerequisites
- Rust 1.70+
- SQLite3 (for some comparison tests)

### Commands
```bash
# Run basic example
cargo run

# Run specific test
cargo run --bin <test_name>

# Run with logging
RUST_LOG=debug cargo run --bin <test_name>

# Clean build
cargo clean && cargo run
```

## 📚 Learn More

- [Turso Documentation](https://docs.turso.tech/)
- [SQLite ON CONFLICT Clause](https://www.sqlite.org/lang_conflict.html)
- [Rust Async/Await Patterns](https://rust-lang.github.io/async-book/)

## 🤝 Contributing

This test suite is designed to help understand Turso's behavior and limitations. Feel free to:
- Add new test cases
- Improve existing tests
- Update documentation
- Report new findings

---

**Note**: This test suite demonstrates that while Turso's basic functionality works well, it has significant limitations for concurrent operations. Use sequential processing for production workloads.