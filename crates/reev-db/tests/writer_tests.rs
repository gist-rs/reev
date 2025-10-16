//! Tests for database writer module

use reev_db::{DatabaseConfig, DatabaseWriter};
use tempfile::TempDir;
use tokio::fs;

#[tokio::test]
async fn test_database_writer_creation() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test.db");
    let config = DatabaseConfig::new(db_path.to_string_lossy());

    let writer = DatabaseWriter::new(config).await?;
    assert_eq!(writer.get_all_benchmark_count().await?, 0);

    Ok(())
}

#[tokio::test]
async fn test_upsert_no_duplicates() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test.db");
    let config = DatabaseConfig::new(db_path.to_string_lossy());

    let writer = DatabaseWriter::new(config).await?;

    // First upsert
    let md5_1 = writer
        .upsert_benchmark("test-benchmark", "Test prompt", "Test content")
        .await?;

    // Second upsert (should update, not create duplicate)
    let md5_2 = writer
        .upsert_benchmark("test-benchmark", "Test prompt", "Test content")
        .await?;

    assert_eq!(md5_1, md5_2);
    assert_eq!(writer.get_all_benchmark_count().await?, 1);

    Ok(())
}

#[tokio::test]
async fn test_sync_benchmarks() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let benchmarks_dir = temp_dir.path().join("benchmarks");
    fs::create_dir(&benchmarks_dir).await?;

    // Create test benchmark files
    let benchmark1 = benchmarks_dir.join("001-test.yml");
    let benchmark2 = benchmarks_dir.join("002-test.yml");

    fs::write(
        &benchmark1,
        "id: 001-test\ndescription: Test benchmark 1\ntags: [\"test\"]\ninitial_state:\n  - pubkey: \"USER_WALLET_PUBKEY\"\n    owner: \"11111111111111111111111111111111\"\n    lamports: 1000000000\nprompt: Test 1\nground_truth:\n  final_state_assertions: []\n  expected_instructions: []\n",
    )
    .await?;
    fs::write(
        &benchmark2,
        "id: 002-test\ndescription: Test benchmark 2\ntags: [\"test\"]\ninitial_state:\n  - pubkey: \"USER_WALLET_PUBKEY\"\n    owner: \"11111111111111111111111111111111\"\n    lamports: 1000000000\nprompt: Test 2\nground_truth:\n  final_state_assertions: []\n  expected_instructions: []\n",
    )
    .await?;

    let db_path = temp_dir.path().join("test.db");
    let config = DatabaseConfig::new(db_path.to_string_lossy());
    let writer = DatabaseWriter::new(config).await?;

    // First sync
    let result1 = writer.sync_benchmarks_from_dir(&benchmarks_dir).await?;
    assert_eq!(result1.processed_count, 2);
    assert_eq!(result1.new_count, 2);
    assert_eq!(result1.updated_count, 0);

    // Second sync (should update existing records)
    let result2 = writer.sync_benchmarks_from_dir(&benchmarks_dir).await?;
    assert_eq!(result2.processed_count, 2);
    assert_eq!(result2.new_count, 0);
    assert_eq!(result2.updated_count, 0); // No changes since content is same

    Ok(())
}
