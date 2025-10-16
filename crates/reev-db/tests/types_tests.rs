//! Tests for database types module

use reev_db::{BatchResult, DatabaseStats, QueryFilter};

#[test]
fn test_query_filter_builder() {
    let filter = QueryFilter::new()
        .benchmark_name("test")
        .agent_type("llm")
        .score_range(0.5, 1.0)
        .paginate(10, 0)
        .sort_by("score", "desc");

    assert_eq!(filter.benchmark_name, Some("test".to_string()));
    assert_eq!(filter.agent_type, Some("llm".to_string()));
    assert_eq!(filter.min_score, Some(0.5));
    assert_eq!(filter.max_score, Some(1.0));
    assert_eq!(filter.limit, Some(10));
    assert_eq!(filter.offset, Some(0));
    assert_eq!(filter.sort_by, Some("score".to_string()));
    assert_eq!(filter.sort_direction, Some("desc".to_string()));
}

#[test]
fn test_batch_result() {
    let mut result = BatchResult::new();

    result.add_success("item1");
    result.add_success("item2");
    result.add_failure("item3".to_string(), "Error message".to_string());

    assert_eq!(result.success_count, 2);
    assert_eq!(result.failure_count, 1);
    assert_eq!(result.total_count(), 3);
    assert!(!result.is_complete_success());
    assert_eq!(result.success_rate(), 66.66666666666667);
}

#[test]
fn test_database_stats_default() {
    let stats = DatabaseStats::default();

    assert_eq!(stats.total_benchmarks, 0);
    assert_eq!(stats.duplicate_count, 0);
    assert!(stats.duplicate_details.is_empty());
    assert!(stats.last_updated.starts_with("20"));
}
