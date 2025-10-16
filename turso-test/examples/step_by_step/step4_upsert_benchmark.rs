use anyhow::Result;
use chrono::Utc;
use turso::Builder;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🧪 Step 4: Test Exact upsert_benchmark Logic");

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

    // Test 1: First upsert (should create)
    println!("\n📝 Test 1: First upsert (create)");
    let md5_1 = upsert_benchmark(&conn, "001-spl-transfer", "Transfer 1 SOL to recipient", "content1").await?;
    println!("First upsert MD5: {md5_1}");

    // Verify first record
    let mut rows = conn.query("SELECT COUNT(*) FROM test_benchmarks", ()).await?;
    let row = rows.next().await?.unwrap();
    let count1: i64 = row.get(0)?;
    println!("Records after first upsert: {count1}");

    // Test 2: Second upsert with same content (should update, not create)
    println!("\n📝 Test 2: Second upsert with same content (update)");
    let md5_2 = upsert_benchmark(&conn, "001-spl-transfer", "Transfer 1 SOL to recipient", "content1").await?;
    println!("Second upsert MD5: {md5_2}");

    // Verify no duplicate was created
    let mut rows = conn.query("SELECT COUNT(*) FROM test_benchmarks", ()).await?;
    let row = rows.next().await?.unwrap();
    let count2: i64 = row.get(0)?;
    println!("Records after second upsert: {count2}");

    // Test 3: Third upsert with same benchmark name but different prompt (should create new record)
    println!("\n📝 Test 3: Different prompt for same benchmark name (create new)");
    let md5_3 = upsert_benchmark(&conn, "001-spl-transfer", "Transfer 2 SOL to recipient", "content2").await?;
    println!("Third upsert MD5: {md5_3}");

    // Verify new record was created
    let mut rows = conn.query("SELECT COUNT(*) FROM test_benchmarks", ()).await?;
    let row = rows.next().await?.unwrap();
    let count3: i64 = row.get(0)?;
    println!("Records after third upsert: {count3}");

    // Test 4: Fourth upsert identical to third (should update, not create)
    println!("\n📝 Test 4: Fourth upsert identical to third (update)");
    let md5_4 = upsert_benchmark(&conn, "001-spl-transfer", "Transfer 2 SOL to recipient", "content2").await?;
    println!("Fourth upsert MD5: {md5_4}");

    // Verify no duplicate was created
    let mut rows = conn.query("SELECT COUNT(*) FROM test_benchmarks", ()).await?;
    let row = rows.next().await?.unwrap();
    let count4: i64 = row.get(0)?;
    println!("Records after fourth upsert: {count4}");

    // Test 5: Multiple different benchmarks
    println!("\n📝 Test 5: Multiple different benchmarks");
    let test_cases = vec![
        ("002-swap", "Swap SOL to USDC", "Swap content"),
        ("003-stake", "Stake SOL", "Stake content"),
        ("001-spl-transfer", "Transfer 1 SOL to recipient", "content1"), // Should update first
        ("004-unstake", "Unstake SOL", "Unstake content"),
    ];

    let mut md5s = Vec::new();
    for (i, (name, prompt, content)) in test_cases.iter().enumerate() {
        let md5 = upsert_benchmark(&conn, name, prompt, content).await?;
        md5s.push(md5);
        println!("Test 5-{}: {} -> {}", i + 1, name, md5s[i]);
    }

    // Final verification
    let mut rows = conn.query("SELECT COUNT(*) FROM test_benchmarks", ()).await?;
    let row = rows.next().await?.unwrap();
    let final_count: i64 = row.get(0)?;
    println!("\n📊 Final record count: {final_count}");

    // Check MD5 uniqueness
    let unique_md5s: std::collections::HashSet<_> = md5s.iter().collect();
    println!("📊 Unique MD5s in test 5: {}", unique_md5s.len());

    if count1 == 1 && count2 == 1 && count3 == 2 && count4 == 2 && final_count == 5 {
        println!("\n✅ Step 4 completed: upsert_benchmark logic works correctly");
        println!("   - First upsert: Created 1 record");
        println!("   - Second upsert: Updated existing record (no duplicate)");
        println!("   - Third upsert: Created new record (different prompt = different MD5)");
        println!("   - Fourth upsert: Updated existing record (no duplicate)");
        println!("   - Multiple benchmarks: Created 3 new records, updated 1 existing");
        println!("   - Total records: 5 (as expected)");
    } else {
        println!("\n❌ Step 4 failed:");
        println!("   - Expected counts: 1, 1, 2, 2, final=5");
        println!("   - Actual counts: {count1}, {count2}, {count3}, {count4}, final={final_count}");
    }

    // Show all records for debugging
    println!("\n📊 All records in database:");
    let mut rows = conn.query(
        "SELECT id, benchmark_name, SUBSTR(prompt, 1, 30) as prompt_preview, created_at FROM test_benchmarks ORDER BY id",
        ()
    ).await?;
    while let Some(row) = rows.next().await? {
        let id: String = row.get(0)?;
        let name: String = row.get(1)?;
        let prompt_preview: String = row.get(2)?;
        let created_at: String = row.get(3)?;
        println!("   {id} | {name} | {prompt_preview}... | {created_at}");
    }

    // Show MD5 collision analysis
    println!("\n🔬 MD5 Analysis:");
    let mut rows = conn.query(
        "SELECT id, benchmark_name, prompt FROM test_benchmarks WHERE benchmark_name = '001-spl-transfer' ORDER BY id",
        ()
    ).await?;
    while let Some(row) = rows.next().await? {
        let id: String = row.get(0)?;
        let name: String = row.get(1)?;
        let prompt: String = row.get(2)?;
        println!("   {id} | {name} | {prompt}");
    }

    println!("\n💡 Key insight: Different prompts for the same benchmark name create different records");
    println!("💡 Next step: Try step5_rapid_calls to test rapid successive operations");

    Ok(())
}
