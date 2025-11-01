use reev_tools::tools::native::SolTransferTool;
use reev_tools::tracker::otel_wrapper::{
    OtelMetricsCollector, SimpleToolWrapper, ToolExecutionMetrics,
};
use std::collections::HashMap;

#[tokio::test]
async fn test_simple_tool_wrapper() {
    // Create a mock tool
    let sol_tool = SolTransferTool {
        key_map: HashMap::new(),
    };

    // Wrap it with simple wrapper
    let wrapped_tool = SimpleToolWrapper::new(sol_tool, "sol_transfer_test");

    // Verify the wrapper
    assert_eq!(wrapped_tool.tool_name(), "sol_transfer_test");
}

#[test]
fn test_tool_execution_metrics() {
    let success_metrics = ToolExecutionMetrics::success("test_tool".to_string(), 100);
    assert!(success_metrics.success);
    assert_eq!(success_metrics.execution_time_ms, 100);

    let failure_metrics = ToolExecutionMetrics::failure("test_tool".to_string(), 50, "test error");
    assert!(!failure_metrics.success);
    assert_eq!(failure_metrics.execution_time_ms, 50);
    assert_eq!(
        failure_metrics.error_message,
        Some("test error".to_string())
    );
}

#[test]
fn test_metrics_collector() {
    let mut collector = OtelMetricsCollector::new();

    // Add some test metrics
    let metrics = ToolExecutionMetrics::success("test_tool".to_string(), 100);
    collector.add_metrics(metrics.clone());

    // Collect metrics
    let collected = collector.collect_tool_metrics("test_tool");
    assert_eq!(collected.len(), 1);
    assert_eq!(collected[0].tool_name, "test_tool");

    // Clear metrics
    collector.clear();
    let collected_after_clear = collector.collect_tool_metrics("test_tool");
    assert!(collected_after_clear.is_empty());
}
