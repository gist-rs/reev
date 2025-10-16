//! Tests for shared benchmark utilities

use reev_db::shared::benchmark::{BenchmarkUtils, BenchmarkYml};
use serde_yaml::Value;

#[test]
fn test_benchmark_md5_generation() {
    let md5_1 = BenchmarkUtils::generate_md5("test", "prompt");
    let md5_2 = BenchmarkUtils::generate_md5("test", "prompt");
    let md5_3 = BenchmarkUtils::generate_md5("test", "different");

    assert_eq!(md5_1, md5_2);
    assert_ne!(md5_1, md5_3);
}

#[test]
fn test_benchmark_validation() {
    let valid_benchmark = BenchmarkYml {
        id: "test-1".to_string(),
        description: "Test benchmark".to_string(),
        tags: vec!["test".to_string()],
        prompt: "Test prompt".to_string(),
        initial_state: vec![Value::Null],
        ground_truth: Value::Null,
    };

    assert!(BenchmarkUtils::validate_benchmark(&valid_benchmark).is_ok());

    let invalid_benchmark = BenchmarkYml {
        id: "".to_string(),
        description: "Test benchmark".to_string(),
        tags: vec![],
        prompt: "".to_string(),
        initial_state: vec![],
        ground_truth: Value::Null,
    };

    assert!(BenchmarkUtils::validate_benchmark(&invalid_benchmark).is_err());
}

#[test]
fn test_benchmark_difficulty() {
    let benchmark = BenchmarkYml {
        id: "test-1".to_string(),
        description: "Test benchmark".to_string(),
        tags: vec!["test".to_string()],
        prompt: "A simple test prompt".to_string(),
        initial_state: vec![Value::Null],
        ground_truth: Value::Null,
    };

    let difficulty = BenchmarkUtils::calculate_difficulty(&benchmark);
    assert!(difficulty > 0.0);
    assert!(difficulty <= 5.0);
}
