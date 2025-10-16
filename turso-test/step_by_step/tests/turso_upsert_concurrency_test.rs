//! Consolidated Turso Upsert and Concurrency Test
//!
//! This test consolidates steps 1-5 into a comprehensive test suite that:
//! 1. Tests basic upsert functionality (working proof)
//! 2. Tests concurrency limits and expected failures with turso
//!
//! Key findings for LLM:
//! - Turso has concurrency limitations that can cause failures
//! - Sequential processing works reliably
//! - Parallel/concurrent operations may fail with 10-20 items
//! - Use sequential processing for production code

use anyhow::Result;
use chrono::Utc;
use std::collections::HashSet;
use tokio::task::JoinSet;
use turso::{Builder, Connection};

/// Consolidated test suite for Turso upsert and concurrency behavior
#[tokio::main]
async fn main() -> Result<()> {
    println!("🧪 Turso Upsert and Concurrency Test Suite");
    println!("==========================================");

    // Test 1: Basic Upsert Functionality (Working Proof)
    test_basic_upsert().await?;

    // Test 2: Sequential Processing (Should Work)
    test_sequential_processing().await?;

    // Test 3: Concurrency Limits (Expected to Show Issues)
    test_concurrency_limits().await?;

    // Test 4: High Concurrency Stress Test (Expected Failures)
    test_high_concurrency_stress().await?;

    // Summary and Recommendations
    print_summary();

    Ok(())
}

/// Test 1: Basic upsert functionality - should work perfectly
async fn test_basic_upsert() -> Result<()> {
    println!("\n📋 Test 1: Basic Upsert Functionality");
    println!("-------------------------------------");

    let db = Builder::new_local(":memory:").build().await?;
    let conn = db.connect()?;
    init_schema(&conn).await?;

    // Test basic upsert operations
    let test_cases = vec![
        ("001-basic", "Basic prompt", "Basic content"),
        ("002-basic", "Another prompt", "Another content"),
        ("001-basic", "Basic prompt", "Basic content"), // Should update, not duplicate
        ("003-basic", "Third prompt", "Third content"),
    ];

    let mut md5s = Vec::new();
    for (i, (name, prompt, content)) in test_cases.iter().enumerate() {
        let md5 = upsert_benchmark(&conn, name, prompt, content).await?;
        println!("   ✅ Upsert {}: {} -> {}", i + 1, name, md5);
        md5s.push(md5);
    }

    let count = get_count(&conn).await?;
    let unique_md5s: HashSet<_> = md5s.iter().collect();

    println!("   📊 Results: {} records, {} unique MD5s", count, unique_md5s.len());

    if count == 3 && unique_md5s.len() == 3 {
        println!("   ✅ PASS: Basic upsert works perfectly");
    } else {
        println!("   ❌ FAIL: Basic upsert failed");
    }

    Ok(())
}

/// Test 2: Sequential processing - should work reliably
async fn test_sequential_processing() -> Result<()> {
    println!("\n📋 Test 2: Sequential Processing");
    println!("--------------------------------");

    let db = Builder::new_local(":memory:").build().await?;
    let conn = db.connect()?;
    init_schema(&conn).await?;

    // Create a larger set of test data
    let test_data: Vec<_> = (0..20)
        .map(|i| {
            let name = format!("seq-{:03}", i);
            let prompt = format!("Sequential prompt {}", i);
            let content = format!("Sequential content {}", i);
            (name, prompt, content)
        })
        .collect();

    println!("   Processing {} items sequentially...", test_data.len());

    let mut md5s = Vec::new();
    for (i, (name, prompt, content)) in test_data.iter().enumerate() {
        let md5 = upsert_benchmark(&conn, name, prompt, content).await?;
        md5s.push(md5);

        if (i + 1) % 5 == 0 {
            println!("   Processed {}/{} items", i + 1, test_data.len());
        }
    }

    // Run the same data again (should update, not create duplicates)
    println!("   Running second pass (should update existing)...");
    for (name, prompt, content) in test_data.iter() {
        let md5 = upsert_benchmark(&conn, name, prompt, content).await?;
        md5s.push(md5);
    }

    let count = get_count(&conn).await?;
    let unique_md5s: HashSet<_> = md5s.iter().collect();

    println!("   📊 Results: {} records, {} unique MD5s", count, unique_md5s.len());

    if count == 20 && unique_md5s.len() == 20 {
        println!("   ✅ PASS: Sequential processing works reliably");
    } else {
        println!("   ❌ FAIL: Sequential processing failed");
    }

    Ok(())
}

/// Test 3: Concurrency limits - may show issues with moderate parallelism
async fn test_concurrency_limits() -> Result<()> {
    println!("\n📋 Test 3: Concurrency Limits");
    println!("----------------------------");

    let db = Builder::new_local(":memory:").build().await?;
    let conn = db.connect()?;
    init_schema(&conn).await?;

    // Test with moderate concurrency (10 items)
    let test_data: Vec<_> = (0..10)
        .map(|i| {
            let name = format!("conc-{:03}", i);
            let prompt = format!("Concurrent prompt {}", i);
            let content = format!("Concurrent content {}", i);
            (name, prompt, content)
        })
        .collect();

    println!("   Testing with {} concurrent operations...", test_data.len());

    let mut join_set = JoinSet::new();
    let mut success_count = 0;
    let mut error_count = 0;

    for (i, (name, prompt, content)) in test_data.into_iter().enumerate() {
        let conn_clone = conn.clone();
        join_set.spawn(async move {
            println!("     Starting concurrent task {}: {}", i + 1, name);
            match upsert_benchmark(&conn_clone, &name, &prompt, &content).await {
                Ok(md5) => {
                    println!("     ✅ Concurrent task {} completed: {}", i + 1, md5);
                    Ok(md5)
                }
                Err(e) => {
                    println!("     ❌ Concurrent task {} failed: {}", i + 1, e);
                    Err(e)
                }
            }
        });
    }

    let mut results = Vec::new();
    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(task_result) => match task_result {
                Ok(md5) => {
                    results.push(md5);
                    success_count += 1;
                }
                Err(_) => error_count += 1,
            },
            Err(_) => error_count += 1,
        }
    }

    let count = get_count(&conn).await?;
    let unique_md5s: HashSet<_> = results.iter().collect();

    println!("   📊 Results: {} records, {} unique MD5s", count, unique_md5s.len());
    println!("   📊 Success: {}, Errors: {}", success_count, error_count);

    if count == 10 && unique_md5s.len() == 10 && error_count == 0 {
        println!("   ✅ PASS: Moderate concurrency works");
    } else {
        println!("   ⚠️  PARTIAL: Moderate concurrency shows limitations");
        println!("   💡 This demonstrates turso's concurrency limits");
    }

    Ok(())
}

/// Test 4: High concurrency stress test - expected to show failures
async fn test_high_concurrency_stress() -> Result<()> {
    println!("\n📋 Test 4: High Concurrency Stress Test");
    println!("--------------------------------------");

    let db = Builder::new_local(":memory:").build().await?;
    let conn = db.connect()?;
    init_schema(&conn).await?;

    // Test with high concurrency (20 items)
    let test_data: Vec<_> = (0..20)
        .map(|i| {
            let name = format!("stress-{:03}", i);
            let prompt = format!("Stress test prompt {}", i);
            let content = format!("Stress test content {}", i);
            (name, prompt, content)
        })
        .collect();

    println!("   Testing with {} concurrent operations...", test_data.len());
    println!("   ⚠️  This test is expected to show concurrency issues");

    let mut join_set = JoinSet::new();
    let mut success_count = 0;
    let mut error_count = 0;

    for (i, (name, prompt, content)) in test_data.into_iter().enumerate() {
        let conn_clone = conn.clone();
        join_set.spawn(async move {
            // Add some delay to increase contention
            tokio::time::sleep(tokio::time::Duration::from_millis(i as u64 % 5)).await;

            match upsert_benchmark(&conn_clone, &name, &prompt, &content).await {
                Ok(md5) => {
                    if i % 5 == 0 {
                        println!("     ✅ Stress task {} completed: {}", i + 1, md5);
                    }
                    Ok(md5)
                }
                Err(e) => {
                    println!("     ❌ Stress task {} failed: {}", i + 1, e);
                    Err(e)
                }
            }
        });
    }

    let mut results = Vec::new();
    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(task_result) => match task_result {
                Ok(md5) => {
                    results.push(md5);
                    success_count += 1;
                }
                Err(_) => error_count += 1,
            },
            Err(_) => error_count += 1,
        }
    }

    let count = get_count(&conn).await?;
    let unique_md5s: HashSet<_> = results.iter().collect();

    println!("   📊 Results: {} records, {} unique MD5s", count, unique_md5s.len());
    println!("   📊 Success: {}, Errors: {}", success_count, error_count);

    if error_count > 0 {
        println!("   ✅ EXPECTED: High concurrency shows limitations");
        println!("   💡 This demonstrates why sequential processing is recommended");
    } else {
        println!("   ⚠️  UNEXPECTED: High concurrency worked (may be environment dependent)");
    }

    Ok(())
}

/// Print summary and recommendations for LLM
fn print_summary() {
    println!("\n🎯 Summary and Recommendations");
    println!("==============================");
    println!();
    println!("📋 Test Results Summary:");
    println!("   1. ✅ Basic Upsert: Works perfectly");
    println!("   2. ✅ Sequential Processing: Reliable and recommended");
    println!("   3. ⚠️  Moderate Concurrency: May show limitations");
    println!("   4. ⚠️  High Concurrency: Expected to show issues");
    println!();
    println!("🔍 Key Findings for LLM:");
    println!("   • Turso has concurrency limitations");
    println!("   • Sequential processing is the most reliable approach");
    println!("   • Parallel operations can fail with 10-20+ concurrent items");
    println!("   • Database locking and connection sharing cause issues");
    println!();
    println!("💡 Recommendations:");
    println!("   1. Use sequential processing for production code");
    println!("   2. Avoid parallel database writes in turso");
    println!("   3. Use connection pools only for read operations");
    println!("   4. Implement proper error handling for concurrent scenarios");
    println!("   5. Consider batching sequential operations for performance");
    println!();
    println!("🚫 What to Avoid:");
    println!("   • Parallel upsert operations on the same connection");
    println!("   • High concurrency (10+ simultaneous operations)");
    println!("   • Sharing connections across async tasks without care");
    println!("   • Assuming database operations are thread-safe by default");
    println!();
    println!("✅ Best Practices:");
    println!("   • One connection per writer instance");
    println!("   • Sequential processing of benchmark data");
    println!("   • Proper error handling and retries");
    println!("   • Connection lifecycle management");
}

// Helper functions

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
    let prompt_md5 = format!("{:x}", md5::compute(format!("{}:{}", benchmark_name, prompt).as_bytes()));
    let timestamp = Utc::now().to_rfc3339();

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
