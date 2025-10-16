//! Tests for shared performance utilities

use reev_db::shared::performance::{PerformanceUtils, TrendDirection};

#[test]
fn test_success_rate_calculation() {
    assert_eq!(PerformanceUtils::calculate_success_rate(8, 10), 80.0);
    assert_eq!(PerformanceUtils::calculate_success_rate(0, 10), 0.0);
    assert_eq!(PerformanceUtils::calculate_success_rate(10, 10), 100.0);
    assert_eq!(PerformanceUtils::calculate_success_rate(0, 0), 0.0);
}

#[test]
fn test_average_score_calculation() {
    assert!(
        (PerformanceUtils::calculate_average_score(&[0.8, 0.9, 0.7]) - 0.8).abs() < f64::EPSILON
    );
    assert_eq!(PerformanceUtils::calculate_average_score(&[]), 0.0);
    assert_eq!(PerformanceUtils::calculate_average_score(&[1.0]), 1.0);
}

#[test]
fn test_outlier_detection() {
    let normal_values = vec![0.8, 0.9, 0.85, 0.95, 0.75];
    let with_outlier = vec![0.8, 0.9, 0.85, 0.95, 0.1]; // 0.1 is an outlier

    let normal_outliers = PerformanceUtils::detect_outliers(&normal_values, 2.0);
    assert_eq!(normal_outliers.len(), 0);

    let outlier_results = PerformanceUtils::detect_outliers(&with_outlier, 1.5);
    assert!(!outlier_results.is_empty());
}

#[test]
fn test_trend_determination() {
    let increasing = vec![0.7, 0.75, 0.8, 0.85, 0.9];
    let decreasing = vec![0.9, 0.85, 0.8, 0.75, 0.7];
    let stable = vec![0.8, 0.81, 0.79, 0.82, 0.8];

    assert!(matches!(
        PerformanceUtils::determine_trend(&increasing),
        TrendDirection::Improving
    ));
    assert!(matches!(
        PerformanceUtils::determine_trend(&decreasing),
        TrendDirection::Declining
    ));
    assert!(matches!(
        PerformanceUtils::determine_trend(&stable),
        TrendDirection::Stable
    ));
}
