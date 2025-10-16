//! Tests for shared module structure and prelude

use reev_db::shared::prelude::*;

#[test]
fn test_shared_module_structure() {
    // Test that we can import and use shared types
    let flow_log = FlowLogUtils::create(
        "test-session".to_string(),
        "test-benchmark".to_string(),
        "test-agent".to_string(),
    );

    assert_eq!(flow_log.session_id(), "test-session");
    assert_eq!(flow_log.benchmark_id(), "test-benchmark");
    assert_eq!(flow_log.agent_type(), "test-agent");
}
