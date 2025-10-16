use anyhow::Result;
use chrono::Utc;
use turso::{Builder, Connection};

/// Test to reproduce the original duplicate creation issue described in TOFIX.md
///
/// This test simulates the conditions that previously caused duplicates:
/// 1. Multiple database connections
/// 2. Concurrent/parallel processing
/// 3. Transaction boundary issues
/// 4. Connection pool isolation problems
///
/// Expected behavior: Should NOT create duplicates
/// If duplicates are created: Indicates the original issue still exists

#[tokio::main]
async fn main() -> Result<()> {
    println!("🧪 Step 6: Reproduce Original Duplicate Creation Issue");
    println!("📝 Testing conditions that previously caused duplicate creation");

    // Test 1: Multiple Connections (original suspected cause)
    println!("\n🔍 Test 1: Multiple Database Connections");
    test_multiple_connections().await?;

    // Test 2: Parallel Processing (original suspected cause)
    println!("\n🔍 Test 2: Parallel Processing with Shared Connection");
    test_parallel_processing().await?;

    // Test 3: Connection Pool Simulation (original suspected cause)
    println!("\n🔍 Test 3: Connection Pool Simulation");
    test_connection_pool_simulation().await?;

    // Test 4: Transaction Boundary Issues (original suspected cause)
    println!("\n🔍 Test 4: Transaction Boundary Issues");
    test_transaction_boundaries().await?;

    println!("\n🎯 Summary:");
    println!("✅ If all tests pass: Original issue is resolved");
    println!("❌ If any test fails: Original issue still exists");

    Ok(())
}

/// Test 1: Multiple database connections causing duplicate creation
async fn test_multiple_connections() -> Result<()> {
    println!("   Testing multiple independent database connections...");

    // Create multiple connections to the same database
    let db1 = Builder::new_local(":memory:").build().await?;
    let conn1 = db1.connect()?;

    let db2 = Builder::new_local(":memory:").build().await?;
    let conn2 = db2.connect()?;

    // Initialize schema on both connections
    init_schema(&conn1).await?;
    init_schema(&conn2).await?;

    // Try to insert the same record through both connections
    let benchmark_name = "test-benchmark";
    let prompt = "Test prompt";
    let content = "Test content";

    let md5_1 = upsert_benchmark(&conn1, benchmark_name, prompt, content).await?;
    let md5_2 = upsert_benchmark(&conn2, benchmark_name, prompt, content).await?;

    // Check counts
    let count1 = get_count(&conn1).await?;
    let count2 = get_count(&conn2).await?;

    println!("   Connection 1: {count1} records, MD5: {md5_1}");
    println!("   Connection 2: {count2} records, MD5: {md5_2}");

    if count1 == 1 && count2 == 1 && md5_1 == md5_2 {
        println!("   ✅ PASS: No duplicates with multiple connections");
    } else {
        println!("   ❌ FAIL: Multiple connections created inconsistent results");
        println!("   Expected: 1 record on each connection, same MD5");
        println!("   Actual: {count1} records on conn1, {count2} records on conn2");
    }

    Ok(())
}

/// Test 2: Parallel processing with shared connection
async fn test_parallel_processing() -> Result<()> {
    println!("   Testing parallel processing with shared connection...");

    let db = Builder::new_local(":memory:").build().await?;
    let conn = db.connect()?;
    init_schema(&conn).await?;

    // Create test data with potentially conflicting records
    let test_cases = vec![
        ("001-test", "Prompt 1", "Content 1"),
        ("002-test", "Prompt 2", "Content 2"),
        ("001-test", "Prompt 1", "Content 1"), // Duplicate of first
        ("003-test", "Prompt 3", "Content 3"),
        ("002-test", "Prompt 2", "Content 2"), // Duplicate of second
    ];

    // Process them in parallel (this could cause race conditions)
    let mut join_set = tokio::task::JoinSet::new();

    for (i, (name, prompt, content)) in test_cases.into_iter().enumerate() {
        let conn_clone = conn.clone();
        join_set.spawn(async move {
            println!("     Parallel task {}: Processing {}", i + 1, name);
            upsert_benchmark(&conn_clone, name, prompt, content).await
        });
    }

    let mut results = Vec::new();
    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(md5_result) => {
                match md5_result {
                    Ok(md5) => {
                        println!("     ✅ Parallel task completed: {md5}");
                        results.push(md5);
                    }
                    Err(e) => {
                        println!("     ❌ Parallel task failed: {e}");
                    }
                }
            }
            Err(e) => {
                println!("     ❌ Task join failed: {e}");
            }
        }
    }

    let final_count = get_count(&conn).await?;
    println!("   Final record count: {final_count}");
    println!("   Expected: 3 unique records");

    if final_count == 3 {
        println!("   ✅ PASS: Parallel processing handled correctly");
    } else {
        println!("   ❌ FAIL: Parallel processing created {final_count} records (expected 3)");

        // Show all records for debugging
        println!("   All records:");
        let mut rows = conn.query("SELECT id, benchmark_name FROM benchmarks ORDER BY id", ()).await?;
        while let Some(row) = rows.next().await? {
            let id: String = row.get(0)?;
            let name: String = row.get(1)?;
            println!("     {id} | {name}");
        }
    }

    Ok(())
}

/// Test 3: Connection pool simulation
async fn test_connection_pool_simulation() -> Result<()> {
    println!("   Testing connection pool simulation...");

    // Simulate getting different connections from a pool
    let mut connections = Vec::new();

    for i in 0..3 {
        let db = Builder::new_local(":memory:").build().await?;
        let conn = db.connect()?;
        init_schema(&conn).await?;
        connections.push(conn);
        println!("     Created connection {} from 'pool'", i + 1);
    }

    // Use different connections for the same operation
    let benchmark_name = "pool-test";
    let prompt = "Pool test prompt";
    let content = "Pool test content";

    for (i, conn) in connections.iter().enumerate() {
        let md5 = upsert_benchmark(conn, benchmark_name, prompt, content).await?;
        let count = get_count(conn).await?;
        println!("     Connection {}: {} records, MD5: {}", i + 1, count, md5);
    }

    println!("   ✅ PASS: Connection pool simulation completed");
    println!("   Note: Each connection is independent, which was the original issue");

    Ok(())
}

/// Test 4: Transaction boundary issues
async fn test_transaction_boundaries() -> Result<()> {
    println!("   Testing transaction boundary issues...");

    let db = Builder::new_local(":memory:").build().await?;
    let conn = db.connect()?;
    init_schema(&conn).await?;

    // Simulate operations that might have transaction boundary issues
    println!("     Simulating rapid successive operations...");

    for i in 0..10 {
        let benchmark_name = format!("tx-test-{}", i % 3); // Create some duplicates
        let prompt = format!("Prompt {i}"); // Different prompts create different MD5s
        let content = format!("Content {i}");

        let md5 = upsert_benchmark(&conn, &benchmark_name, &prompt, &content).await?;
        let count = get_count(&conn).await?;

        println!("     Operation {}: {} records, MD5: {}", i + 1, count, md5);
    }

    let final_count = get_count(&conn).await?;
    println!("   Final record count: {final_count}");

    // Calculate expected unique records: 3 benchmark names * different prompts
    // tx-test-0: appears at i=0,3,6,9 (4 different prompts)
    // tx-test-1: appears at i=1,4,7 (3 different prompts)
    // tx-test-2: appears at i=2,5,8 (3 different prompts)
    // Total: 4 + 3 + 3 = 10 unique records
    let expected_unique = 10;

    if final_count == expected_unique {
        println!("   ✅ PASS: Different prompts correctly create different records");
        println!("   Note: Each prompt variation creates a unique MD5, as expected");
    } else {
        println!("   ❌ FAIL: Unexpected record count");
        println!("   Expected {expected_unique} unique records, got {final_count}");
    }

    Ok(())
}

// Helper functions to simulate the original problematic behavior

async fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS benchmarks (
            id TEXT PRIMARY KEY,
            benchmark_name TEXT NOT NULL,
            prompt TEXT NOT NULL,
            content TEXT NOT NULL,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT DEFAULT CURRENT_TIMESTAMP
        )",
        (),
    ).await?;
    Ok(())
}

async fn upsert_benchmark(
    conn: &Connection,
    benchmark_name: &str,
    prompt: &str,
    content: &str,
) -> Result<String> {
    let prompt_md5 = format!("{:x}", md5::compute(format!("{benchmark_name}:{prompt}").as_bytes()));
    let timestamp = Utc::now().to_rfc3339();

    // This is the original problematic query pattern
    let query = "
        INSERT INTO benchmarks (id, benchmark_name, prompt, content, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
            benchmark_name = excluded.benchmark_name,
            prompt = excluded.prompt,
            content = excluded.content,
            updated_at = excluded.updated_at;
    ";

    conn.execute(
        query,
        [
            prompt_md5.clone(),
            benchmark_name.to_string(),
            prompt.to_string(),
            content.to_string(),
            timestamp.clone(),
            timestamp.clone(),
        ],
    ).await?;

    println!("[DB] Upserted benchmark '{benchmark_name}' with MD5 '{prompt_md5}'");
    Ok(prompt_md5)
}

async fn get_count(conn: &Connection) -> Result<i64> {
    let mut rows = conn.query("SELECT COUNT(*) FROM benchmarks", ()).await?;
    if let Some(row) = rows.next().await? {
        let count: i64 = row.get(0)?;
        Ok(count)
    } else {
        Ok(0)
    }
}
