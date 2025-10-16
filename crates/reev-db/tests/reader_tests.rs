//! Tests for database reader module

use reev_db::{DatabaseConfig, DatabaseReader, DatabaseWriter};
use tempfile::TempDir;

#[tokio::test]
async fn test_reader_creation() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test.db");
    let config = DatabaseConfig::new(db_path.to_string_lossy());

    // Create database first
    let writer = DatabaseWriter::new(config.clone()).await?;
    writer.upsert_benchmark("test", "prompt", "content").await?;

    // Create reader
    let reader = DatabaseReader::from_config(config).await?;
    let stats = reader.get_benchmark_stats().await?;

    assert_eq!(stats.total_benchmarks, 1);
    Ok(())
}

#[tokio::test]
async fn test_search_benchmarks() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test.db");
    let config = DatabaseConfig::new(db_path.to_string_lossy());

    let writer = DatabaseWriter::new(config.clone()).await?;
    writer
        .upsert_benchmark("test-search", "Searchable prompt", "Searchable content")
        .await?;

    let reader = DatabaseReader::from_config(config).await?;
    let results = reader.search_benchmarks("searchable").await?;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].benchmark_name, "test-search");

    Ok(())
}
