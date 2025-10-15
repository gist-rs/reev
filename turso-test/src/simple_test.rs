use anyhow::Result;
use chrono::Utc;
use turso::Builder;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🧪 Simple Turso ON CONFLICT Test (No MD5)");

    // Create in-memory database
    let conn = Builder::new_local().build()?.connect()?;

    // Create simple test table
    conn.execute(
        "CREATE TABLE test_table (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL
        )",
        (),
    ).await?;

    println!("\n📝 Test 1: Insert first record");
    let result1 = conn.execute(
        "INSERT INTO test_table (id, name) VALUES (?, ?)",
        ["same-id", "first-name"]
    ).await?;
    println!("✅ First insert result: {:?}", result1);

    // Check count after first insert
    let mut rows = conn.query("SELECT COUNT(*) FROM test_table", ()).await?;
    if let Some(row) = rows.next().await? {
        let count: i64 = row.get(0)?;
        println!("📊 Record count after first insert: {}", count);
    }

    println!("\n📝 Test 2: Insert second record with SAME ID using ON CONFLICT");
    let result2 = conn.execute(
        "INSERT INTO test_table (id, name) VALUES (?, ?)
         ON CONFLICT(id) DO UPDATE SET name = excluded.name",
        ["same-id", "second-name"]
    ).await?;
    println!("✅ Second insert result: {:?}", result2);

    // Check final count
    let mut rows = conn.query("SELECT COUNT(*) FROM test_table", ()).await?;
    if let Some(row) = rows.next().await? {
        let count: i64 = row.get(0)?;
        println!("📊 Final record count: {}", count);

        if count == 1 {
            println!("✅ SUCCESS: ON CONFLICT worked correctly - only 1 record exists");

            // Show the actual record
            let mut rows = conn.query("SELECT id, name FROM test_table", ()).await?;
            while let Some(row) = rows.next().await? {
                let id: String = row.get(0)?;
                let name: String = row.get(1)?;
                println!("📋 Final record: {} -> {}", id, name);
            }
        } else {
            println!("❌ FAILURE: ON CONFLICT failed - {} records exist (should be 1)", count);

            // Show all records
            println!("📋 All records:");
            let mut rows = conn.query("SELECT id, name FROM test_table", ()).await?;
            while let Some(row) = rows.next().await? {
                let id: String = row.get(0)?;
                let name: String = row.get(1)?;
                println!("  - {} -> {}", id, name);
            }
        }
    }

    Ok(())
}
