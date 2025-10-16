//! Tests for main library module

use reev_db::{DatabaseConfig, DatabaseWriter, LibraryInfo};
use tempfile::TempDir;

#[tokio::test]
async fn test_library_info() -> Result<(), Box<dyn std::error::Error>> {
    let info = LibraryInfo::new();
    assert_eq!(info.name, "reev-db");
    assert!(!info.version.is_empty());
    assert!(!info.description.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_basic_operations() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test.db");
    let config = DatabaseConfig::new(db_path.to_string_lossy());

    let db = DatabaseWriter::new(config).await?;

    // Test basic upsert
    let md5 = db
        .upsert_benchmark("test-benchmark", "Test prompt", "Test content")
        .await?;

    assert!(!md5.is_empty());

    // Test duplicate prevention
    let md5_2 = db
        .upsert_benchmark("test-benchmark", "Test prompt", "Test content")
        .await?;

    assert_eq!(md5, md5_2);

    let count = db.get_all_benchmark_count().await?;
    assert_eq!(count, 1);

    Ok(())
}
