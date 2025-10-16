use anyhow::Result;
use chrono::Utc;
use turso::Builder;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🧪 Step 2: Basic Table Creation and INSERT");

    // Create in-memory database for this example
    let db = Builder::new_local(":memory:").build().await?;
    let conn = db.connect()?;
    println!("✅ Connected successfully");

    // Drop table if exists (for clean test)
    conn.execute("DROP TABLE IF EXISTS test_benchmarks", ()).await?;
    println!("🗑️  Cleaned existing table");

    // Create table similar to our benchmarks table
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

    // Test basic INSERT
    let insert_sql = "
        INSERT INTO test_benchmarks (id, benchmark_name, prompt, content, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
    ";

    let timestamp = Utc::now().to_rfc3339();
    let result = conn.execute(
        insert_sql,
        [
            "test-id-1".to_string(),
            "001-test-benchmark".to_string(),
            "Test prompt".to_string(),
            "Test content".to_string(),
            timestamp.clone(),
            timestamp.clone(),
        ],
    ).await?;

    println!("✅ Basic INSERT result: {result}");

    // Insert multiple records
    let test_data = [("test-id-2", "002-sol-transfer", "Transfer SOL", "Transfer 1 SOL to recipient"),
        ("test-id-3", "003-spl-token", "SPL Token operation", "Create and transfer SPL token"),
        ("test-id-4", "004-jupiter-swap", "Jupiter Swap", "Swap tokens via Jupiter")];

    for (i, (id, name, prompt, content)) in test_data.iter().enumerate() {
        let timestamp = Utc::now().to_rfc3339();
        let result = conn.execute(
            insert_sql,
            [
                id.to_string(),
                name.to_string(),
                prompt.to_string(),
                content.to_string(),
                timestamp.clone(),
                timestamp.clone(),
            ],
        ).await?;
        println!("✅ Inserted record {}: {} (result: {})", i + 2, name, result);
    }

    // Verify all records were inserted
    let mut rows = conn.query("SELECT COUNT(*) FROM test_benchmarks", ()).await?;
    let row = rows.next().await?.unwrap();
    let count: i64 = row.get(0)?;
    println!("📊 Total records in table: {count}");

    if count == 4 {
        println!("✅ Step 2 completed: Basic INSERT works");

        // Show all inserted records
        println!("\n📋 All records:");
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
    } else {
        println!("❌ Step 2 failed: Expected 4 records, got {count}");
    }

    println!("\n💡 Next step: Try step3_on_conflict to learn about ON CONFLICT handling");
    Ok(())
}
