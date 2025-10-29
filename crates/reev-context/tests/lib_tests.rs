//! Context library tests
//!
//! Tests for context resolver functionality.

use reev_context::*;

#[test]
fn test_initial_state_creation() {
    // Test InitialState struct creation
    let initial_state = InitialState {
        pubkey: "TEST_PUBKEY".to_string(),
        owner: "11111111111111111111111111111111".to_string(),
        lamports: 1000000000,
        data: None,
    };

    assert_eq!(initial_state.pubkey, "TEST_PUBKEY");
    assert_eq!(initial_state.lamports, 1000000000);
    assert_eq!(initial_state.owner, "11111111111111111111111111111111");
    assert!(initial_state.data.is_none());
}

#[test]
fn test_agent_context_creation() {
    // Test AgentContext struct creation
    let mut key_map = std::collections::HashMap::new();
    key_map.insert("TEST".to_string(), "TEST_RESOLVED".to_string());

    let context = AgentContext {
        key_map,
        account_states: std::collections::HashMap::new(),
        fee_payer_placeholder: Some("TEST_PUBKEY".to_string()),
        current_step: Some(0),
        step_results: std::collections::HashMap::new(),
    };

    assert!(context.key_map.contains_key("TEST"));
    assert_eq!(context.current_step, Some(0));
    assert_eq!(
        context.fee_payer_placeholder,
        Some("TEST_PUBKEY".to_string())
    );
}
