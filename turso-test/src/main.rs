use anyhow::Result;
use chrono::Utc;
use turso::Builder;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🧪 Testing Turso ON CONFLICT behavior...");
    
    // Create in-memory database
    let conn = Builder::new_local().build()?.connect()?;
    
    // Create test table
    conn.execute(
        "CREATE TABLE test_table (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            value TEXT NOT NULL,
            updated_at TEXT
        )",
        (),
    ).await?;
    
    let timestamp = Utc::now().to_rfc3339();
    
    println!("
📝 Test 1: Insert first record");
    let result1 = conn.execute(
        "INSERT INTO test_table (id, name, value, updated_at) VALUES (?, ?, ?, ?)",
        ["test-id", "name1", "value1", &timestamp]
    ).await?;
    println!("✅ First insert result: {:?}", result1);
    
    // Check count after first insert
    let mut rows = conn.query("SELECT COUNT(*) FROM test_table", ()).await?;
    if let Some(row) = rows.next().await? {
        let count: i64 = row.get(0)?;
        println!("📊 Record count after first insert: {}", count);
    }
    
    println!("
📝 Test 2: Insert second record with SAME ID using ON CONFLICT");
    let result2 = conn.execute(
        "INSERT INTO test_table (id, name, value, updated_at) VALUES (?, ?, ?, ?)
         ON CONFLICT(id) DO UPDATE SET
             name = excluded.name,
             value = excluded.value,
             updated_at = excluded.updated_at",
        ["test-id", "name2-updated", "value2-updated", &timestamp]
    ).await?;
    println!("✅ Second insert result: {:?}", result2);
    
    // Check final count
    let mut rows = conn.query("SELECT COUNT(*) FROM test_table", ()).await?;
    if let Some(row) = rows.next().await? {
        let count: i64 = row.get(0)?;
        println!("📊 Final record count: {}", count);
        
        if count == 1 {
            println!("✅ SUCCESS: ON CONFLICT worked correctly - only 1 record exists");
        } else {
            println!("❌ FAILURE: ON CONFLICT failed - {} records exist (should be 1)", count);
        }
    }
    
    // Show actual data
    println!("
📋 Actual records in database:");
    let mut rows = conn.query("SELECT id, name, value FROM test_table", ()).await?;
    while let Some(row) = rows.next().await? {
        let id: String = row.get(0)?;
        let name: String = row.get(1)?;
        let value: String = row.get(2)?;
        println!("  - {}: {} | {}", id, name, value);
    }
    
    Ok(())
}
