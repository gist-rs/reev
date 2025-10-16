use turso::{Client, Config};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Step 2: Basic Table Creation and INSERT");

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

    let timestamp = chrono::Utc::now().to_rfc3339();
    let result = conn.execute(
        insert_sql,
        turso::params!["test-id-1", "001-test-benchmark", "Test prompt", "Test content", timestamp.clone(), timestamp.clone()]
    ).await?;

    println!("✅ Basic INSERT result: {}", result);

    // Verify the record was inserted
    let rows = conn.query("SELECT COUNT(*) FROM test_benchmarks", ()).await?;
    let row = rows.next().await?.unwrap();
    let count: i64 = row.get(0)?;
    println!("📊 Records in table: {}", count);

    if count == 1 {
        println!("✅ Step 2 completed: Basic INSERT works");
    } else {
        println!("❌ Step 2 failed: Expected 1 record, got {}", count);
    }

    Ok(())
}
