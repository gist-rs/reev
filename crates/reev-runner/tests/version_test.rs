use reev_runner::version::{VersionInfo, get_agent_version, set_agent_version};

#[test]
fn test_version_info() {
    let version_info = VersionInfo::new();

    assert!(!version_info.runner_version.is_empty());
    assert!(!version_info.agent_version.is_empty());
}

#[test]
fn test_custom_agent_version() {
    let custom_agent = "glm-4.6-custom".to_string();
    let version_info = VersionInfo::with_agent_version(custom_agent.clone());

    assert_eq!(version_info.agent_version, custom_agent);
    assert!(!version_info.runner_version.is_empty());
}

#[test]
fn test_version_setting() {
    set_agent_version("test-agent-version");
    assert_eq!(get_agent_version(), "test-agent-version");
}
