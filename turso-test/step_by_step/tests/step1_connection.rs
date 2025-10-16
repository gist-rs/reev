use turso::{Client, Config};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Step 1: Basic Turso Connection Test");

    // Get database URL from environment
    let db_url = env::var("TURSO_DATABASE_URL")
        .unwrap_or_else(|_| "libsql://memory".to_string());

    let auth_token = env::var("TURSO_AUTH_TOKEN").ok();

    println!("📝 Connecting to: {}", if db_url.contains("memory") { "in-memory database" } else { "Turso database" });

    // Create client
    let config = Config::new(db_url.clone());
    let client = match auth_token {
        Some(token) => Client::new(config).auth_token(token),
        None => Client::new(config),
    };

    // Connect
    let conn = client.connect().await?;
    println!("✅ Connected successfully");

    // Test basic query
    let result = conn.execute("SELECT 1 as test", ()).await?;
    println!("📊 Basic query result: {}", result);

    // Test connection persistence
    let rows = conn.query("SELECT 1 as test", ()).await?;
    let row = rows.next().await?.unwrap();
    let value: i64 = row.get(0)?;
    println!("📊 Query result: {}", value);

    println!("✅ Step 1 completed: Connection works");
    Ok(())
}
