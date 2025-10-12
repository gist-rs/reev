//! # GLM TUI Integration Tests
//!
//! This test file validates that the TUI properly handles GLM 4.6 agent selection
//! and correctly disables/enables the GLM tab based on environment variable availability.

use reev_tui::SelectedAgent;
use strum::IntoEnumIterator;

#[test]
fn test_glm_agent_enum() {
    // Test that GLM agent is properly defined in the enum
    let glm_agent = SelectedAgent::Glm46;
    assert_eq!(glm_agent.to_agent_name(), "glm-4.6");
    assert_eq!(glm_agent.to_string(), " GLM 4.6 ");
}

#[test]
fn test_glm_agent_navigation() {
    // Test GLM agent navigation (previous/next)
    let glm_agent = SelectedAgent::Glm46;

    // Test previous navigation
    let previous_agent = glm_agent.previous();
    assert_eq!(previous_agent, SelectedAgent::Local);

    // Test next navigation (should stay at GLM since it's the last one)
    let next_agent = glm_agent.next();
    assert_eq!(next_agent, SelectedAgent::Glm46);
}

#[test]
fn test_glm_agent_disabled_when_env_vars_missing() {
    // Test that GLM agent is disabled when environment variables are not set
    let glm_agent = SelectedAgent::Glm46;

    // Ensure environment variables are not set
    unsafe {
        std::env::remove_var("GLM_API_KEY");
        std::env::remove_var("GLM_API_URL");
    }

    // GLM should be disabled when not running is false (i.e., when benchmarks can run)
    assert!(glm_agent.is_disabled(false));

    // GLM should also be disabled when benchmarks are running
    assert!(glm_agent.is_disabled(true));
}

#[test]
fn test_glm_agent_enabled_when_env_vars_present() {
    // Test that GLM agent is enabled when both environment variables are set
    let glm_agent = SelectedAgent::Glm46;

    // Set environment variables
    unsafe {
        std::env::set_var("GLM_API_KEY", "test_key");
        std::env::set_var("GLM_API_URL", "https://api.example.com");
    }

    // GLM should be enabled when not running benchmarks
    assert!(!glm_agent.is_disabled(false));

    // GLM should still be disabled when benchmarks are running (like all agents)
    assert!(glm_agent.is_disabled(true));

    // Clean up
    unsafe {
        std::env::remove_var("GLM_API_KEY");
        std::env::remove_var("GLM_API_URL");
    }
}

#[test]
fn test_glm_agent_disabled_when_only_one_env_var_present() {
    // Test that GLM agent is disabled when only one environment variable is set

    // Test case 1: Only GLM_API_KEY set
    unsafe {
        std::env::set_var("GLM_API_KEY", "test_key");
        std::env::remove_var("GLM_API_URL");
    }

    let glm_agent = SelectedAgent::Glm46;
    assert!(glm_agent.is_disabled(false));

    // Test case 2: Only GLM_API_URL set
    unsafe {
        std::env::remove_var("GLM_API_KEY");
        std::env::set_var("GLM_API_URL", "https://api.example.com");
    }

    assert!(glm_agent.is_disabled(false));

    // Clean up
    unsafe {
        std::env::remove_var("GLM_API_KEY");
        std::env::remove_var("GLM_API_URL");
    }
}

#[test]
fn test_other_agents_not_affected_by_glm_env_vars() {
    // Test that other agents are not affected by GLM environment variables

    // Set GLM environment variables
    unsafe {
        std::env::set_var("GLM_API_KEY", "test_key");
        std::env::set_var("GLM_API_URL", "https://api.example.com");
    }

    // Other agents should only be disabled when running
    let deterministic = SelectedAgent::Deterministic;
    let gemini = SelectedAgent::Gemini;
    let local = SelectedAgent::Local;

    assert!(!deterministic.is_disabled(false));
    assert!(deterministic.is_disabled(true));

    assert!(!gemini.is_disabled(false));
    assert!(gemini.is_disabled(true));

    assert!(!local.is_disabled(false));
    assert!(local.is_disabled(true));

    // Clean up
    unsafe {
        std::env::remove_var("GLM_API_KEY");
        std::env::remove_var("GLM_API_URL");
    }
}

#[test]
fn test_glm_agent_enum_iteration() {
    // Test that GLM agent appears in enum iteration
    let agents: Vec<_> = SelectedAgent::iter().collect();
    assert!(agents.contains(&SelectedAgent::Glm46));

    // Verify the order: Deterministic, Gemini, Local, GLM
    assert_eq!(agents[0], SelectedAgent::Deterministic);
    assert_eq!(agents[1], SelectedAgent::Gemini);
    assert_eq!(agents[2], SelectedAgent::Local);
    assert_eq!(agents[3], SelectedAgent::Glm46);
}
