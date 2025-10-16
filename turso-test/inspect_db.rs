use anyhow::Result;
use turso::{Builder, Connection};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 Inspecting Real Database State");

    // Connect to the actual database used by the API
    let db = Builder::new_local("../db/reev_results.db").build().await?;
    let conn = db.connect()?;

    println!("✅ Connected to real database: db/reev_results.db");

    // Get total count of benchmarks
    let mut rows = conn.query("SELECT COUNT(*) FROM benchmarks", ()).await?;
    if let Some(row) = rows.next().await? {
        let count: i64 = row.get(0)?;
        println!("📊 Total benchmark records: {}", count);
    }

    // Show all records grouped by ID to detect duplicates
    println!("\n📋 Records grouped by ID:");
    let mut rows = conn.query("SELECT id, benchmark_name, COUNT(*) as count FROM benchmarks GROUP BY id, benchmark_name ORDER BY id", ()).await?;
    let mut total_records = 0;
    let mut duplicate_count = 0;

    while let Some(row) = rows.next().await? {
        let id: String = row.get(0)?;
        let name: String = row.get(1)?;
        let count: i64 = row.get(2)?;
        total_records += 1;

        if count > 1 {
            duplicate_count += 1;
            println!("   ❌ DUPLICATE: {} | {} | Count: {}", id, name, count);
        } else {
            println!("   ✅ OK: {} | {} | Count: {}", id, name, count);
        }
    }

    println!("\n🎯 Summary:");
    println!("   Unique benchmark IDs: {}", total_records);
    println!("   Duplicates detected: {}", duplicate_count);

    if duplicate_count > 0 {
        println!("   ⚠️  ISSUE CONFIRMED: Duplicates exist in database");

        // Show all records for problematic cases
        println!("\n🔍 All records with duplicate IDs:");
        let mut rows = conn.query("SELECT id, benchmark_name, SUBSTR(prompt, 1, 50) as prompt_preview, created_at, updated_at FROM benchmarks ORDER BY id, updated_at", ()).await?;

        let mut last_id = String::new();
        let mut record_count = 0;

        while let Some(row) = rows.next().await? {
            let id: String = row.get(0)?;
            let name: String = row.get(1)?;
            let prompt_preview: String = row.get(2)?;
            let created_at: String = row.get(3)?;
            let updated_at: String = row.get(4)?;

            if id != last_id {
                if !last_id.is_empty() && record_count > 1 {
                    println!("   ⚠️  {} has {} records", last_id, record_count);
                }
                last_id = id.clone();
                record_count = 1;
            } else {
                record_count += 1;
            }

            println!("      {} | {} | {}... | Created: {} | Updated: {}",
                id, name, prompt_preview, created_at, updated_at);
        }

        if !last_id.is_empty() && record_count > 1 {
            println!("   ⚠️  {} has {} records", last_id, record_count);
        }
    } else {
        println!("   ✅ No duplicates found");
    }

    // Test the specific MD5 collision mentioned in TOFIX.md
    println!("\n🔬 Checking specific MD5 collision case:");
    let mut rows = conn.query("SELECT id, benchmark_name, prompt FROM benchmarks WHERE benchmark_name LIKE '%spl-transfer-fail%'", ()).await?;

    while let Some(row) = rows.next().await? {
        let id: String = row.get(0)?;
        let name: String = row.get(1)?;
        let prompt: String = row.get(2)?;
        println!("   {} | {} | {}", id, name, prompt);
    }

    Ok(())
}
