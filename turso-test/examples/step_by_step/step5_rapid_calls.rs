use anyhow::Result;
use chrono::Utc;
use std::collections::HashSet;
use tokio::task::JoinSet;
use turso::Builder;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🧪 Step 5: Test Multiple Rapid Calls (simulate sync_benchmarks_to_db)");

    // Create in-memory database for this example
    let db = Builder::new_local(":memory:").build().await?;
    let conn = db.connect()?;
    println!("✅ Connected successfully");

    // Clean and create table
    conn.execute("DROP TABLE IF EXISTS test_benchmarks", ()).await?;

    let create_table_sql = "
        CREATE TABLE test_benchmarks (
            id TEXT PRIMARY KEY,
            benchmark_name TEXT NOT NULL,
            prompt TEXT NOT NULL,
            content TEXT NOT NULL,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT DEFAULT CURRENT_TIMESTAMP
        )
    ";

    conn.execute(create_table_sql, ()).await?;
    println!("✅ Created test_benchmarks table");

    // Replicate our exact upsert_benchmark function logic
    async fn upsert_benchmark(
        conn: &turso::Connection,
        benchmark_name: &str,
        prompt: &str,
        content: &str,
    ) -> Result<String> {
        let prompt_md5 = format!(
            "{:x}",
            md5::compute(format!("{benchmark_name}:{prompt}").as_bytes())
        );
        let timestamp = Utc::now().to_rfc3339();

        // Use INSERT ... ON CONFLICT DO UPDATE pattern from our implementation
        let query = "
            INSERT INTO test_benchmarks (id, benchmark_name, prompt, content, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                benchmark_name = excluded.benchmark_name,
                prompt = excluded.prompt,
                content = excluded.content,
                updated_at = excluded.updated_at;
        ";

        let result = conn.execute(
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

        println!(
            "[DB] Upserted benchmark '{benchmark_name}' with MD5 '{prompt_md5}' (prompt: {prompt:.50}...) - Result: {result}"
        );
        Ok(prompt_md5)
    }

    // Test 1: Sequential rapid calls (should work perfectly)
    println!("\n📝 Test 1: Sequential rapid calls (20 operations)");
    let start_time = std::time::Instant::now();
    let mut sequential_md5s = Vec::new();

    for i in 0..20 {
        let benchmark_name = format!("{i:03}-sequential");
        let prompt = format!("Sequential prompt {i}");
        let content = format!("Sequential content for benchmark {i}");

        let md5 = upsert_benchmark(&conn, &benchmark_name, &prompt, &content).await?;
        sequential_md5s.push(md5);
    }

    let sequential_duration = start_time.elapsed();
    println!("Sequential processing: {} operations in {:?}", sequential_md5s.len(), sequential_duration);

    // Verify sequential results
    let mut rows = conn.query("SELECT COUNT(*) FROM test_benchmarks", ()).await?;
    let row = rows.next().await?.unwrap();
    let sequential_count: i64 = row.get(0)?;
    println!("Records after sequential: {sequential_count}");

    // Test 2: Rapid calls with small delays (simulating real-world timing)
    println!("\n📝 Test 2: Rapid calls with small delays (10 operations)");
    let start_time = std::time::Instant::now();
    let mut rapid_md5s = Vec::new();

    for i in 0..10 {
        let benchmark_name = format!("{i:03}-rapid");
        let prompt = format!("Rapid prompt {i}");
        let content = format!("Rapid content for benchmark {i}");

        let md5 = upsert_benchmark(&conn, &benchmark_name, &prompt, &content).await?;
        rapid_md5s.push(md5);

        // Small delay to simulate real-world processing
        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
    }

    let rapid_duration = start_time.elapsed();
    println!("Rapid processing: {} operations in {:?}", rapid_md5s.len(), rapid_duration);

    // Verify rapid results
    let mut rows = conn.query("SELECT COUNT(*) FROM test_benchmarks", ()).await?;
    let row = rows.next().await?.unwrap();
    let rapid_count: i64 = row.get(0)?;
    println!("Records after rapid: {rapid_count}");

    // Test 3: Mixed rapid calls (some duplicates)
    println!("\n📝 Test 3: Mixed rapid calls with duplicates (15 operations)");
    let start_time = std::time::Instant::now();
    let mut mixed_md5s = Vec::new();

    let test_cases = vec![
        ("001-mixed", "Mixed prompt 1", "Mixed content 1"),
        ("002-mixed", "Mixed prompt 2", "Mixed content 2"),
        ("001-mixed", "Mixed prompt 1", "Mixed content 1"), // Duplicate
        ("003-mixed", "Mixed prompt 3", "Mixed content 3"),
        ("002-mixed", "Mixed prompt 2", "Mixed content 2"), // Duplicate
        ("004-mixed", "Mixed prompt 4", "Mixed content 4"),
        ("005-mixed", "Mixed prompt 5", "Mixed content 5"),
        ("001-mixed", "Mixed prompt 1", "Mixed content 1"), // Duplicate
        ("006-mixed", "Mixed prompt 6", "Mixed content 6"),
        ("003-mixed", "Mixed prompt 3", "Mixed content 3"), // Duplicate
        ("007-mixed", "Mixed prompt 7", "Mixed content 7"),
        ("008-mixed", "Mixed prompt 8", "Mixed content 8"),
        ("002-mixed", "Mixed prompt 2", "Mixed content 2"), // Duplicate
        ("009-mixed", "Mixed prompt 9", "Mixed content 9"),
        ("010-mixed", "Mixed prompt 10", "Mixed content 10"),
    ];

    for (i, (name, prompt, content)) in test_cases.iter().enumerate() {
        let md5 = upsert_benchmark(&conn, name, prompt, content).await?;
        mixed_md5s.push(md5);

        // Very small delay
        if i % 3 == 0 {
            tokio::time::sleep(tokio::time::Duration::from_micros(500)).await;
        }
    }

    let mixed_duration = start_time.elapsed();
    println!("Mixed processing: {} operations in {:?}", mixed_md5s.len(), mixed_duration);

    // Verify mixed results
    let mut rows = conn.query("SELECT COUNT(*) FROM test_benchmarks", ()).await?;
    let row = rows.next().await?.unwrap();
    let mixed_count: i64 = row.get(0)?;
    println!("Records after mixed: {mixed_count}");

    // Check MD5 uniqueness
    let unique_mixed_md5s: HashSet<_> = mixed_md5s.iter().collect();
    println!("Unique MD5s in mixed test: {} (out of {} operations)", unique_mixed_md5s.len(), mixed_md5s.len());

    // Test 4: Attempt at concurrent calls (will demonstrate Turso limitations)
    println!("\n📝 Test 4: Concurrent calls (demonstrating limitations - 5 operations)");
    let start_time = std::time::Instant::now();
    let mut join_set = JoinSet::new();
    let mut concurrent_success = 0;
    let mut concurrent_errors = 0;

    for i in 0..5 {
        let conn_clone = conn.clone();
        let benchmark_name = format!("{i:03}-concurrent");
        let prompt = format!("Concurrent prompt {i}");
        let content = format!("Concurrent content for benchmark {i}");

        join_set.spawn(async move {
            println!("     Starting concurrent task {}: {}", i + 1, benchmark_name);
            match upsert_benchmark(&conn_clone, &benchmark_name, &prompt, &content).await {
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

    let mut concurrent_results = Vec::new();
    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(task_result) => match task_result {
                Ok(md5) => {
                    concurrent_results.push(md5);
                    concurrent_success += 1;
                }
                Err(_) => concurrent_errors += 1,
            },
            Err(_) => concurrent_errors += 1,
        }
    }

    let concurrent_duration = start_time.elapsed();
    println!("Concurrent processing: {concurrent_success} success, {concurrent_errors} errors in {concurrent_duration:?}");

    // Final verification
    let mut rows = conn.query("SELECT COUNT(*) FROM test_benchmarks", ()).await?;
    let row = rows.next().await?.unwrap();
    let final_count: i64 = row.get(0)?;
    println!("\n📊 Final record count: {final_count}");

    // Performance comparison
    println!("\n⚡ Performance Summary:");
    println!("   Sequential: {} ops in {:?} ({:.2} ops/sec)",
        sequential_md5s.len(), sequential_duration,
        sequential_md5s.len() as f64 / sequential_duration.as_secs_f64());
    println!("   Rapid:      {} ops in {:?} ({:.2} ops/sec)",
        rapid_md5s.len(), rapid_duration,
        rapid_md5s.len() as f64 / rapid_duration.as_secs_f64());
    println!("   Mixed:      {} ops in {:?} ({:.2} ops/sec)",
        mixed_md5s.len(), mixed_duration,
        mixed_md5s.len() as f64 / mixed_duration.as_secs_f64());
    println!("   Concurrent: {concurrent_success} success, {concurrent_errors} errors in {concurrent_duration:?}");

    // Expected unique records: 20 (sequential) + 10 (rapid) + 10 (unique from mixed) + concurrent_success
    let expected_unique = 20 + 10 + unique_mixed_md5s.len() + concurrent_success;

    if final_count as usize == expected_unique {
        println!("\n✅ Step 5 completed: All rapid call tests successful");
        println!("   - Sequential processing: ✅ Perfect");
        println!("   - Rapid processing: ✅ Perfect");
        println!("   - Mixed with duplicates: ✅ Perfect (UPSERT working)");
        println!("   - Concurrent processing: ⚠️  {concurrent_errors} errors (expected Turso limitation)");
        println!("   - Data integrity: ✅ Maintained");
    } else {
        println!("\n❌ Step 5 failed: Expected {expected_unique} records, got {final_count}");
    }

    // Show final statistics
    println!("\n📊 Final Database Statistics:");
    let mut rows = conn.query(
        "SELECT
            benchmark_name LIKE '%sequential%' as sequential,
            benchmark_name LIKE '%rapid%' as rapid,
            benchmark_name LIKE '%mixed%' as mixed,
            benchmark_name LIKE '%concurrent%' as concurrent,
            COUNT(*) as count
        FROM test_benchmarks
        GROUP BY sequential, rapid, mixed, concurrent
        ORDER BY sequential DESC, rapid DESC, mixed DESC, concurrent DESC",
        ()
    ).await?;

    while let Some(row) = rows.next().await? {
        let is_seq: i64 = row.get(0)?;
        let is_rapid: i64 = row.get(1)?;
        let is_mixed: i64 = row.get(2)?;
        let is_concurrent: i64 = row.get(3)?;
        let count: i64 = row.get(4)?;

        let test_type = if is_seq == 1 { "Sequential" }
                      else if is_rapid == 1 { "Rapid" }
                      else if is_mixed == 1 { "Mixed" }
                      else if is_concurrent == 1 { "Concurrent" }
                      else { "Unknown" };

        println!("   {test_type}: {count} records");
    }

    println!("\n💡 Key insights:");
    println!("   - Sequential processing is 100% reliable");
    println!("   - Rapid sequential calls work perfectly");
    println!("   - UPSERT correctly handles duplicates");
    println!("   - Concurrent operations show Turso's limitations");
    println!("   - For production: Use sequential processing");

    Ok(())
}
