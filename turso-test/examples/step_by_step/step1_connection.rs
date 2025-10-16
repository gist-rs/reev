use anyhow::Result;
use turso::Builder;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🧪 Step 1: Basic Turso Connection Test");

    // Create in-memory database for this example
    let db = Builder::new_local(":memory:").build().await?;
    let conn = db.connect()?;
    println!("✅ Connected successfully");

    // Test basic query - just count
    let mut rows = conn.query("SELECT 1", ()).await?;
    if let Some(row) = rows.next().await? {
        let value: i64 = row.get(0)?;
        println!("📊 Basic query result: {value}");
    }

    // Create a simple test table
    conn.execute(
        "CREATE TABLE test_connection (
            id INTEGER PRIMARY KEY,
            message TEXT
        )",
        (),
    ).await?;
    println!("✅ Test table created successfully");

    // Insert a test record
    let insert_result = conn.execute(
        "INSERT INTO test_connection (message) VALUES (?)",
        ["Connection test successful"]
    ).await?;
    println!("📊 Insert result: {insert_result} row(s) affected");

    // Read back the test record
    let mut rows = conn.query("SELECT id, message FROM test_connection", ()).await?;
    if let Some(row) = rows.next().await? {
        let id: i64 = row.get(0)?;
        let message: String = row.get(1)?;
        println!("📊 Test record: ID={id}, Message='{message}'");
    }

    // Verify count
    let mut rows = conn.query("SELECT COUNT(*) FROM test_connection", ()).await?;
    if let Some(row) = rows.next().await? {
        let count: i64 = row.get(0)?;
        println!("📊 Total records: {count}");
    }

    println!("✅ Step 1 completed: Connection and basic operations work");
    println!("💡 Next step: Try step2_basic_insert to learn more about INSERT operations");

    Ok(())
}
