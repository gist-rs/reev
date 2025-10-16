//! Tests for reev-flow website_exporter module

use reev_flow::types::FlowLog;
use reev_flow::website_exporter::WebsiteExporter;
use std::path::PathBuf;
use std::time::SystemTime;

#[test]
fn test_website_exporter_creation() {
    let _exporter = WebsiteExporter::new(PathBuf::from("/tmp"));
    // Test that exporter can be created successfully
    // We can't directly access output_path as it's private
}

#[test]
fn test_export_empty_flows() {
    let exporter = WebsiteExporter::new(PathBuf::from("/tmp"));
    let flows = vec![];

    // This should not panic even with empty flows
    let result = exporter.export_for_website(&flows);
    assert!(result.is_ok());

    // Verify the exported data structure
    let website_data = result.unwrap();
    assert!(website_data.flows.is_empty());
}

#[test]
fn test_export_with_flows() {
    let exporter = WebsiteExporter::new(PathBuf::from("/tmp"));
    let flows = vec![FlowLog {
        session_id: "test".to_string(),
        benchmark_id: "test".to_string(),
        agent_type: "test".to_string(),
        start_time: SystemTime::now(),
        end_time: Some(SystemTime::now()),
        events: vec![],
        final_result: None,
    }];

    let result = exporter.export_for_website(&flows);
    assert!(result.is_ok());

    let website_data = result.unwrap();
    assert_eq!(website_data.flows.len(), 1);
    assert_eq!(website_data.flows[0].session_id, "test");
}
