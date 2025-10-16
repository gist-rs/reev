use turso::{Client, Config};
use std::env;
use tokio::task::JoinSet;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Step 5: Test Multiple Rapid Calls (simulate sync_benchmarks_to_db)");

    // Get database URL from environment
    let db_url = env::var("TURSO_DATABASE_URL")
        .unwrap_or_else(|_| "libsql://memory".to_string());

    let auth_token = env::var("TURSO_AUTH_TOKEN").ok();

    // Create client and connect
    let config = Config::new(db_url.clone());
    let client = match auth_token {
        Some(token) => Client::new(config).auth_token(token),
        None => Client::new(config),
    };

    let conn = client.connect().await?;
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

    // Replicate our exact upsert_benchmark function logic (async version)
    async fn upsert_benchmark(
        conn: &turso::Connection,
        benchmark_name: &str,
        prompt: &str,
        content: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let prompt_md5 = format!(
            "{:x}",
            md5::compute(format!("{}:{}", benchmark_name, prompt).as_bytes())
        );
        let timestamp = chrono::Utc::now().to_rfc3339();

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

        conn.execute(
            query,
            turso::params![
                prompt_md5.clone(),
                benchmark_name,
                prompt,
                content,
                timestamp.clone(),
                timestamp.clone()
            ],
        )
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        println!(
            "[DB] Upserted benchmark '{}' with MD5 '{}' (prompt: {:.50}...)",
            benchmark_name, prompt_md5, prompt
        );
        Ok(prompt_md5)
    }

    // Create test data - simulate multiple benchmark files
    let test_data = vec![
        ("001-spl-transfer", "Transfer 1 SOL to recipient", "content: Transfer SOL test"),
        ("002-spl-transfer-fail", "Transfer 1 SOL to recipient (should fail)", "content: Failed transfer test"),
        ("003-token-transfer", "Transfer tokens to recipient", "content: Token transfer test"),
    ];

    // Test 1: Sequential calls (like our current sync_benchmarks_to_db)
    println!("\n📝 Test 1: Sequential calls (current implementation)");
    let mut sequential_md5s = Vec::new();

    for (i, (name, prompt, content)) in test_data.iter().enumerate() {
        println!("Sequential call {} for {}", i + 1, name);
        let md5 = upsert_benchmark(&conn, name, prompt, content).await?;
        sequential_md5s.push(md5);
    }

    // Verify sequential results
    let rows = conn.query("SELECT COUNT(*) FROM test_benchmarks", ()).await?;
    let row = rows.next().await?.unwrap();
    let count_sequential: i64 = row.get(0)?;
    println!("Records after sequential calls: {}", count_sequential);

    // Test 2: Run the same sequential calls again (simulate second sync)
    println!("\n📝 Test 2: Second sequential run (simulate second sync)");
    let mut second_run_md5s = Vec::new();

    for (i, (name, prompt, content)) in test_data.iter().enumerate() {
        println!("Second run call {} for {}", i + 1, name);
        let md5 = upsert_benchmark(&conn, name, prompt, content).await?;
        second_run_md5s.push(md5);
    }

    // Verify no duplicates created in second run
    let rows = conn.query("SELECT COUNT(*) FROM test_benchmarks", ()).await?;
    let row = rows.next().await?.unwrap();
    let count_second_run: i64 = row.get(0)?;
    println!("Records after second sequential run: {}", count_second_run);

    // Test 3: Parallel calls (to test if concurrency causes issues)
    println!("\n📝 Test 3: Parallel calls (test concurrency)");

    // Clean table for parallel test
    conn.execute("DELETE FROM test_benchmarks", ()).await?;

    let mut join_set = JoinSet::new();
    let test_data_clone = test_data.clone();

    for (i, (name, prompt, content)) in test_data_clone.into_iter().enumerate() {
        let conn_clone = conn.clone();
        join_set.spawn(async move {
            println!("Parallel call {} for {}", i + 1, name);
            upsert_benchmark(&conn_clone, &name, &prompt, &content).await
        });
    }

    let mut parallel_md5s = Vec::new();
    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(md5_result) => match md5_result {
                Ok(md5) => parallel_md5s.push(md5),
                Err(e) => println!("❌ Parallel call failed: {}", e),
            },
            Err(e) => println!("❌ Task join failed: {}", e),
        }
    }

    // Verify parallel results
    let rows = conn.query("SELECT COUNT(*) FROM test_benchmarks", ()).await?;
    let row = rows.next().await?.unwrap();
    let count_parallel: i64 = row.get(0)?;
    println!("Records after parallel calls: {}", count_parallel);

    // Test 4: Parallel calls repeated (simulate concurrent syncs)
    println!("\n📝 Test 4: Parallel calls repeated (simulate concurrent syncs)");

    let mut join_set2 = JoinSet::new();
    let test_data_clone2 = test_data.clone();

    for (i, (name, prompt, content)) in test_data_clone2.into_iter().enumerate() {
        let conn_clone = conn.clone();
        join_set2.spawn(async move {
            println!("Parallel repeat call {} for {}", i + 1, name);
            upsert_benchmark(&conn_clone, &name, &prompt, &content).await
        });
    }

    let mut parallel_repeat_md5s = Vec::new();
    while let Some(result) = join_set2.join_next().await {
        match result {
            Ok(md5_result) => match md5_result {
                Ok(md5) => parallel_repeat_md5s.push(md5),
                Err(e) => println!("❌ Parallel repeat call failed: {}", e),
            },
            Err(e) => println!("❌ Task join failed: {}", e),
        }
    }

    // Verify parallel repeat results
    let rows = conn.query("SELECT COUNT(*) FROM test_benchmarks", ()).await?;
    let row = rows.next().await?.unwrap();
    let count_parallel_repeat: i64 = row.get(0)?;
    println!("Records after parallel repeat calls: {}", count_parallel_repeat);

    // Final analysis
    println!("\n🎯 Test Results:");
    println!("   Sequential first run: {} records", count_sequential);
    println!("   Sequential second run: {} records", count_second_run);
    println!("   Parallel first run: {} records", count_parallel);
    println!("   Parallel repeat run: {} records", count_parallel_repeat);

    if count_sequential == 3 && count_second_run == 3 && count_parallel == 3 && count_parallel_repeat == 3 {
        println!("✅ Step 5 completed: All tests passed - no duplicates created");
    } else {
        println!("❌ Step 5 failed: Duplicates detected in one or more tests");

        // Show all records for debugging
        println!("\n📊 All records in database:");
        let rows = conn.query("SELECT id, benchmark_name, LEFT(prompt, 30) as prompt_preview, created_at, updated_at FROM test_benchmarks ORDER BY id, updated_at", ()).await?;
        while let Some(row) = rows.next().await? {
            let id: String = row.get(0)?;
            let name: String = row.get(1)?;
            let prompt_preview: String = row.get(2)?;
            let created_at: String = row.get(3)?;
            let updated_at: String = row.get(4)?;
            println!("   {} | {} | {}... | Created: {} | Updated: {}", id, name, prompt_preview, created_at, updated_at);
        }
    }

    // Additional test: Check for MD5 collisions
    println!("\n🔍 MD5 Analysis:");
    let mut all_md5s = vec![];
    all_md5s.extend(sequential_md5s);
    all_md5s.extend(second_run_md5s);
    all_md5s.extend(parallel_md5s);
    all_md5s.extend(parallel_repeat_md5s);

    let mut unique_md5s = std::collections::HashSet::new();
    let mut duplicate_md5s = std::collections::HashSet::new();

    for md5 in &all_md5s {
        if !unique_md5s.insert(md5) {
            duplicate_md5s.insert(md5);
        }
    }

    if duplicate_md5s.is_empty() {
        println!("✅ No MD5 collisions detected");
    } else {
        println!("❌ MD5 collisions detected: {:?}", duplicate_md5s);
    }

    Ok(())
}
