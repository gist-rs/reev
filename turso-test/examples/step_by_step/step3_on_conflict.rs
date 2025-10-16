use anyhow::Result;
use chrono::Utc;
use turso::Builder;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🧪 Step 3: Test ON CONFLICT with Simple INSERT");

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

    // Test 1: First INSERT
    println!("\n📝 Test 1: First INSERT");
    let timestamp1 = Utc::now().to_rfc3339();
    let result1 = conn.execute(
        "INSERT INTO test_benchmarks (id, benchmark_name, prompt, content, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?)",
        [
            "same-id".to_string(),
            "001-first".to_string(),
            "First prompt".to_string(),
            "First content".to_string(),
            timestamp1.clone(),
            timestamp1.clone(),
        ],
    ).await?;
    println!("First INSERT result: {result1}");

    // Verify first insert
    let mut rows = conn.query("SELECT COUNT(*) FROM test_benchmarks", ()).await?;
    let row = rows.next().await?.unwrap();
    let count1: i64 = row.get(0)?;
    println!("Records after first insert: {count1}");

    // Test 2: Second INSERT with same ID (should fail without ON CONFLICT)
    println!("\n📝 Test 2: Second INSERT without ON CONFLICT (should fail)");
    let timestamp2 = Utc::now().to_rfc3339();
    match conn.execute(
        "INSERT INTO test_benchmarks (id, benchmark_name, prompt, content, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?)",
        [
            "same-id".to_string(),
            "002-second".to_string(),
            "Second prompt".to_string(),
            "Second content".to_string(),
            timestamp2.clone(),
            timestamp2.clone(),
        ],
    ).await {
        Ok(result) => {
            println!("❌ Unexpected success: Second INSERT worked (should have failed): {result}");
        }
        Err(e) => {
            println!("✅ Expected failure: Second INSERT failed as expected: {e}");
        }
    }

    // Verify no duplicate was created
    let mut rows = conn.query("SELECT COUNT(*) FROM test_benchmarks", ()).await?;
    let row = rows.next().await?.unwrap();
    let count2: i64 = row.get(0)?;
    println!("Records after failed second insert: {count2}");

    // Test 3: INSERT with ON CONFLICT DO UPDATE (should work)
    println!("\n📝 Test 3: INSERT with ON CONFLICT DO UPDATE");
    let timestamp3 = Utc::now().to_rfc3339();
    let result3 = conn.execute(
        "INSERT INTO test_benchmarks (id, benchmark_name, prompt, content, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?)
         ON CONFLICT(id) DO UPDATE SET
             benchmark_name = excluded.benchmark_name,
             prompt = excluded.prompt,
             content = excluded.content,
             updated_at = excluded.updated_at",
        [
            "same-id".to_string(),
            "002-updated".to_string(),
            "Updated prompt".to_string(),
            "Updated content".to_string(),
            timestamp3.clone(),
            timestamp3.clone(),
        ],
    ).await?;
    println!("ON CONFLICT INSERT result: {result3}");

    // Verify update worked
    let mut rows = conn.query("SELECT COUNT(*) FROM test_benchmarks", ()).await?;
    let row = rows.next().await?.unwrap();
    let count3: i64 = row.get(0)?;
    println!("Records after ON CONFLICT: {count3}");

    // Verify content was updated
    let mut rows = conn.query(
        "SELECT benchmark_name, prompt FROM test_benchmarks WHERE id = ?",
        ["same-id"]
    ).await?;
    let row = rows.next().await?.unwrap();
    let name: String = row.get(0)?;
    let prompt: String = row.get(1)?;
    println!("Updated record - Name: {name}, Prompt: {prompt}");

    // Test 4: Multiple ON CONFLICT operations
    println!("\n📝 Test 4: Multiple ON CONFLICT operations");
    let test_cases = vec![
        ("multi-1", "Multi Test 1", "First multi test"),
        ("multi-2", "Multi Test 2", "Second multi test"),
        ("multi-1", "Multi Test 1 Updated", "Updated first multi test"), // Should update
        ("multi-3", "Multi Test 3", "Third multi test"),
        ("multi-2", "Multi Test 2 Updated", "Updated second multi test"), // Should update
    ];

    for (i, (id, name, content)) in test_cases.iter().enumerate() {
        let timestamp = Utc::now().to_rfc3339();
        let result = conn.execute(
            "INSERT INTO test_benchmarks (id, benchmark_name, prompt, content, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
                 benchmark_name = excluded.benchmark_name,
                 prompt = excluded.prompt,
                 content = excluded.content,
                 updated_at = excluded.updated_at",
            [
                id.to_string(),
                name.to_string(),
                name.to_string(), // Using name as prompt for simplicity
                content.to_string(),
                timestamp.clone(),
                timestamp.clone(),
            ],
        ).await?;
        println!("Multi test {} ({}): result = {}", i + 1, id, result);
    }

    // Final verification
    let mut rows = conn.query("SELECT COUNT(*) FROM test_benchmarks", ()).await?;
    let row = rows.next().await?.unwrap();
    let final_count: i64 = row.get(0)?;
    println!("\n📊 Final record count: {final_count}");

    // Show all unique records
    println!("\n📋 All records after multiple ON CONFLICT operations:");
    let mut rows = conn.query(
        "SELECT id, benchmark_name, SUBSTR(prompt, 1, 20) as prompt_preview FROM test_benchmarks ORDER BY id",
        ()
    ).await?;
    while let Some(row) = rows.next().await? {
        let id: String = row.get(0)?;
        let name: String = row.get(1)?;
        let prompt_preview: String = row.get(2)?;
        println!("   {id} | {name} | {prompt_preview}...");
    }

    if count3 == 1 && name == "002-updated" && prompt == "Updated prompt" && final_count == 4 {
        println!("\n✅ Step 3 completed: ON CONFLICT works correctly");
        println!("   - Basic INSERT: ✅");
        println!("   - Duplicate prevention: ✅");
        println!("   - ON CONFLICT DO UPDATE: ✅");
        println!("   - Multiple ON CONFLICT operations: ✅");
    } else {
        println!("\n❌ Step 3 failed: Expected 4 total records, got {final_count}");
    }

    println!("\n💡 Next step: Try step4_upsert_benchmark to see real-world UPSERT implementation");
    Ok(())
}
