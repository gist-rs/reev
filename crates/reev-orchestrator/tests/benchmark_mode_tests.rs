//! Tests for benchmark_mode module

use reev_orchestrator::benchmark_mode::*;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_benchmark_path_construction() {
    let result = get_static_benchmark_path("100-jup-swap-sol-usdc");
    assert!(result.is_ok());

    let path = result.unwrap();
    assert_eq!(
        path,
        PathBuf::from("../../benchmarks/100-jup-swap-sol-usdc.yml")
    );
}

#[test]
fn test_benchmark_exists() {
    // Test with non-existent benchmark
    assert!(!benchmark_exists("non-existent-benchmark"));
}

#[test]
fn test_list_benchmarks() -> anyhow::Result<()> {
    // Create temporary benchmarks directory
    let temp_dir = TempDir::new()?;
    let _benchmarks_dir = temp_dir.path();

    // This test would need to mock the benchmarks directory
    // For now, just test that the function doesn't panic
    let _benchmarks = list_static_benchmarks();

    Ok(())
}
