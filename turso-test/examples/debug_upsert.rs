use anyhow::Result;
use chrono::Utc;
use turso::{Builder, Connection};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🧪 Debug Upsert Test - Step by Step Analysis");

    // Create in-memory database for testing
    let db = Builder::new_local(":memory:").build().await?;
    let conn = db.connect()?;
    println!("✅ Connected to in-memory database");

    // Clean and create table exactly like our production schema
    conn.execute("DROP TABLE IF EXISTS benchmarks", ()).await?;
    println!("🗑️  Cleaned existing table");

    let create_table_sql = "
        CREATE TABLE benchmarks (
            id TEXT PRIMARY KEY,  -- MD5 of prompt
            benchmark_name TEXT NOT NULL,  -- e.g., 001-sol-transfer
            prompt TEXT NOT NULL,
            content TEXT NOT NULL, -- Full YML content
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT DEFAULT CURRENT_TIMESTAMP
        )
    ";

    conn.execute(create_table_sql, ()).await?;
    println!("✅ Created benchmarks table");

    // Replicate our exact upsert_benchmark function logic
    async fn upsert_benchmark(
        conn: &Connection,
        benchmark_name: &str,
        prompt: &str,
        content: &str,
    ) -> Result<String> {
        let prompt_md5 =
            reev_db::shared::benchmark::BenchmarkUtils::generate_md5(&benchmark_name, &prompt);
        let timestamp = Utc::now().to_rfc3339();

        println!("🔢 Calculated MD5: {prompt_md5} for {benchmark_name}:{prompt:50}");

        // Use INSERT ... ON CONFLICT DO UPDATE pattern from our implementation
        let query = "
            INSERT INTO benchmarks (id, benchmark_name, prompt, content, created_at, updated_at)
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
        )
        .await?;

        println!(
            "[DB] Upserted benchmark '{benchmark_name}' with MD5 '{prompt_md5}' (prompt: {prompt:.50}...) - Result: {result:?}"
        );
        Ok(prompt_md5)
    }

    // Test Data - simulate the problematic scenario
    let test_cases = [("001-spl-transfer", "Transfer 1 SOL to recipient", "content: spl transfer"),
        ("002-spl-transfer-fail", "Transfer 1 SOL to recipient (should fail)", "content: spl transfer fail"),
        ("003-spl-transfer-fail", "Transfer 1 SOL to recipient (should fail)", "content: spl transfer fail")];

    println!("\n🔄 Phase 1: First sync (create all records)");
    let mut first_sync_md5s = Vec::new();

    for (i, (name, prompt, content)) in test_cases.iter().enumerate() {
        println!("\n📝 First sync - Record {}: {}", i + 1, name);
        let md5 = upsert_benchmark(&conn, name, prompt, content).await?;
        first_sync_md5s.push(md5);

        // Check count after each insert
        let mut rows = conn.query("SELECT COUNT(*) FROM benchmarks", ()).await?;
        if let Some(row) = rows.next().await? {
            let count: i64 = row.get(0)?;
            println!("   Current record count: {count}");
        }
    }

    println!("\n📊 First sync complete. Total records: {}", first_sync_md5s.len());

    // Show all records after first sync
    println!("\n📋 Records after first sync:");
    let mut rows = conn.query("SELECT id, benchmark_name, SUBSTR(prompt, 1, 30) as prompt_preview, created_at FROM benchmarks ORDER BY id", ()).await?;
    while let Some(row) = rows.next().await? {
        let id: String = row.get(0)?;
        let name: String = row.get(1)?;
        let prompt_preview: String = row.get(2)?;
        let created_at: String = row.get(3)?;
        println!("   {id} | {name} | {prompt_preview}... | {created_at}");
    }

    println!("\n🔄 Phase 2: Second sync (simulate duplicate issue)");
    let mut second_sync_md5s = Vec::new();

    for (i, (name, prompt, content)) in test_cases.iter().enumerate() {
        println!("\n📝 Second sync - Record {}: {}", i + 1, name);
        let md5 = upsert_benchmark(&conn, name, prompt, content).await?;
        second_sync_md5s.push(md5);

        // Check count after each insert
        let mut rows = conn.query("SELECT COUNT(*) FROM benchmarks", ()).await?;
        if let Some(row) = rows.next().await? {
            let count: i64 = row.get(0)?;
            println!("   Current record count: {count}");
        }
    }

    println!("\n📊 Second sync complete. Total records attempted: {}", second_sync_md5s.len());

    // Final analysis
    let mut rows = conn.query("SELECT COUNT(*) FROM benchmarks", ()).await?;
    let row = rows.next().await?.unwrap();
    let final_count: i64 = row.get(0)?;

    println!("\n🎯 Final Analysis:");
    println!("   Test cases: {}", test_cases.len());
    println!("   Final record count: {final_count}");

    if final_count == test_cases.len() as i64 {
        println!("✅ SUCCESS: No duplicates created! ON CONFLICT works correctly");
    } else {
        println!("❌ ISSUE DETECTED: Expected {} records, got {}", test_cases.len(), final_count);
        println!("   Duplicates created: {}", final_count - test_cases.len() as i64);

        // Show all records for debugging
        println!("\n🔍 All records (including duplicates):");
        let mut rows = conn.query("SELECT id, benchmark_name, SUBSTR(prompt, 1, 30) as prompt_preview, created_at, updated_at FROM benchmarks ORDER BY id, updated_at", ()).await?;
        while let Some(row) = rows.next().await? {
            let id: String = row.get(0)?;
            let name: String = row.get(1)?;
            let prompt_preview: String = row.get(2)?;
            let created_at: String = row.get(3)?;
            let updated_at: String = row.get(4)?;
            println!("   {id} | {name} | {prompt_preview}... | Created: {created_at} | Updated: {updated_at}");
        }

        // Group by ID to see duplicates
        println!("\n🔍 Grouped by ID:");
        let _grouped: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
        let mut rows = conn.query("SELECT id, benchmark_name, COUNT(*) as count FROM benchmarks GROUP BY id, benchmark_name ORDER BY id", ()).await?;
        while let Some(row) = rows.next().await? {
            let id: String = row.get(0)?;
            let name: String = row.get(1)?;
            let count: i64 = row.get(2)?;
            println!("   ID: {id} | Name: {name} | Count: {count}");
            if count > 1 {
                println!("      ⚠️  DUPLICATE DETECTED!");
            }
        }
    }

    // Test the specific MD5 collision mentioned in TOFIX.md
    println!("\n🔬 Testing MD5 collision issue:");
    println!("   002-spl-transfer-fail MD5: {}", first_sync_md5s[1]);
    println!("   003-spl-transfer-fail MD5: {}", first_sync_md5s[2]);

    if first_sync_md5s[1] == first_sync_md5s[2] {
        println!("⚠️  MD5 COLLISION DETECTED! Both have same MD5: {}", first_sync_md5s[1]);
        println!("This could cause the duplicate issue!");
    } else {
        println!("✅ No MD5 collision - different prompts generate different MD5s");
    }

    println!("\n🏁 Test complete");
    Ok(())
}
