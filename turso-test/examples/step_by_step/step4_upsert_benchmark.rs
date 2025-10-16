use turso::{Client, Config};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Step 4: Test Exact upsert_benchmark Logic");

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

    // Replicate our exact upsert_benchmark function logic
    fn upsert_benchmark(
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

        // Note: This is synchronous for testing, our real function is async
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
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
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
            })
        })?;

        println!(
            "[DB] Upserted benchmark '{}' with MD5 '{}' (prompt: {:.50}...)",
            benchmark_name, prompt_md5, prompt
        );
        Ok(prompt_md5)
    }

    // Test 1: First upsert (should create)
    println!("\n📝 Test 1: First upsert (create)");
    let md5_1 = upsert_benchmark(&conn, "001-spl-transfer", "Transfer 1 SOL to recipient", "content1")?;
    println!("First upsert MD5: {}", md5_1);

    // Verify first record
    let rows = conn.query("SELECT COUNT(*) FROM test_benchmarks", ()).await?;
    let row = rows.next().await?.unwrap();
    let count1: i64 = row.get(0)?;
    println!("Records after first upsert: {}", count1);

    // Test 2: Second upsert with same content (should update, not create)
    println!("\n📝 Test 2: Second upsert with same content (update)");
    let md5_2 = upsert_benchmark(&conn, "001-spl-transfer", "Transfer 1 SOL to recipient", "content1")?;
    println!("Second upsert MD5: {}", md5_2);

    // Verify no duplicate was created
    let rows = conn.query("SELECT COUNT(*) FROM test_benchmarks", ()).await?;
    let row = rows.next().await?.unwrap();
    let count2: i64 = row.get(0)?;
    println!("Records after second upsert: {}", count2);

    // Test 3: Third upsert with same benchmark name but different prompt (should create new record)
    println!("\n📝 Test 3: Different prompt for same benchmark name (create new)");
    let md5_3 = upsert_benchmark(&conn, "001-spl-transfer", "Transfer 2 SOL to recipient", "content2")?;
    println!("Third upsert MD5: {}", md5_3);

    // Verify new record was created
    let rows = conn.query("SELECT COUNT(*) FROM test_benchmarks", ()).await?;
    let row = rows.next().await?.unwrap();
    let count3: i64 = row.get(0)?;
    println!("Records after third upsert: {}", count3);

    // Test 4: Fourth upsert identical to third (should update, not create)
    println!("\n📝 Test 4: Fourth upsert identical to third (update)");
    let md5_4 = upsert_benchmark(&conn, "001-spl-transfer", "Transfer 2 SOL to recipient", "content2")?;
    println!("Fourth upsert MD5: {}", md5_4);

    // Verify no duplicate was created
    let rows = conn.query("SELECT COUNT(*) FROM test_benchmarks", ()).await?;
    let row = rows.next().await?.unwrap();
    let count4: i64 = row.get(0)?;
    println!("Records after fourth upsert: {}", count4);

    // Final verification
    if count1 == 1 && count2 == 1 && count3 == 2 && count4 == 2 {
        println!("✅ Step 4 completed: upsert_benchmark logic works correctly");
        println!("   - First upsert: Created 1 record");
        println!("   - Second upsert: Updated existing record (no duplicate)");
        println!("   - Third upsert: Created new record (different prompt = different MD5)");
        println!("   - Fourth upsert: Updated existing record (no duplicate)");
        println!("   - Total records: 2 (as expected)");
    } else {
        println!("❌ Step 4 failed:");
        println!("   - Expected counts: 1, 1, 2, 2");
        println!("   - Actual counts: {}, {}, {}, {}", count1, count2, count3, count4);
    }

    // Show all records for debugging
    println!("\n📊 All records in database:");
    let rows = conn.query("SELECT id, benchmark_name, LEFT(prompt, 30) as prompt_preview FROM test_benchmarks ORDER BY id", ()).await?;
    while let Some(row) = rows.next().await? {
        let id: String = row.get(0)?;
        let name: String = row.get(1)?;
        let prompt_preview: String = row.get(2)?;
        println!("   {} | {} | {}...", id, name, prompt_preview);
    }

    Ok(())
}
