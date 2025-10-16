use turso::{Client, Config};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Step 3: Test ON CONFLICT with Simple INSERT");

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

    // Test 1: First INSERT
    println!("\n📝 Test 1: First INSERT");
    let timestamp1 = chrono::Utc::now().to_rfc3339();
    let result1 = conn.execute(
        "INSERT INTO test_benchmarks (id, benchmark_name, prompt, content, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?)",
        turso::params!["same-id", "001-first", "First prompt", "First content", timestamp1.clone(), timestamp1.clone()]
    ).await?;
    println!("First INSERT result: {}", result1);

    // Verify first insert
    let rows = conn.query("SELECT COUNT(*) FROM test_benchmarks", ()).await?;
    let row = rows.next().await?.unwrap();
    let count1: i64 = row.get(0)?;
    println!("Records after first insert: {}", count1);

    // Test 2: Second INSERT with same ID (should fail without ON CONFLICT)
    println!("\n📝 Test 2: Second INSERT without ON CONFLICT (should fail)");
    let timestamp2 = chrono::Utc::now().to_rfc3339();
    match conn.execute(
        "INSERT INTO test_benchmarks (id, benchmark_name, prompt, content, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?)",
        turso::params!["same-id", "002-second", "Second prompt", "Second content", timestamp2.clone(), timestamp2.clone()]
    ).await {
        Ok(result) => {
            println!("❌ Unexpected success: Second INSERT worked (should have failed): {}", result);
        }
        Err(e) => {
            println!("✅ Expected failure: Second INSERT failed as expected: {}", e);
        }
    }

    // Verify no duplicate was created
    let rows = conn.query("SELECT COUNT(*) FROM test_benchmarks", ()).await?;
    let row = rows.next().await?.unwrap();
    let count2: i64 = row.get(0)?;
    println!("Records after failed second insert: {}", count2);

    // Test 3: INSERT with ON CONFLICT DO UPDATE (should work)
    println!("\n📝 Test 3: INSERT with ON CONFLICT DO UPDATE");
    let timestamp3 = chrono::Utc::now().to_rfc3339();
    let result3 = conn.execute(
        "INSERT INTO test_benchmarks (id, benchmark_name, prompt, content, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?)
         ON CONFLICT(id) DO UPDATE SET
             benchmark_name = excluded.benchmark_name,
             prompt = excluded.prompt,
             content = excluded.content,
             updated_at = excluded.updated_at",
        turso::params!["same-id", "002-updated", "Updated prompt", "Updated content", timestamp3.clone(), timestamp3.clone()]
    ).await?;
    println!("ON CONFLICT INSERT result: {}", result3);

    // Verify update worked
    let rows = conn.query("SELECT COUNT(*) FROM test_benchmarks", ()).await?;
    let row = rows.next().await?.unwrap();
    let count3: i64 = row.get(0)?;
    println!("Records after ON CONFLICT: {}", count3);

    // Verify content was updated
    let rows = conn.query("SELECT benchmark_name, prompt FROM test_benchmarks WHERE id = ?", turso::params!["same-id"]).await?;
    let row = rows.next().await?.unwrap();
    let name: String = row.get(0)?;
    let prompt: String = row.get(1)?;
    println!("Updated record - Name: {}, Prompt: {}", name, prompt);

    if count3 == 1 && name == "002-updated" && prompt == "Updated prompt" {
        println!("✅ Step 3 completed: ON CONFLICT works correctly - 1 record updated");
    } else {
        println!("❌ Step 3 failed: Expected 1 updated record, got {} with name='{}'", count3, name);
    }

    Ok(())
}
