//! Tests for refiner module

use reev_core::refiner::RefinedPrompt;

#[tokio::test]
async fn test_refined_prompt_creation() {
    let original = "send 1 SOL to test".to_string();
    let refined = "transfer 1 SOL to test".to_string();
    let changes_detected = true;

    let refined_prompt =
        RefinedPrompt::new_for_test(original.clone(), refined.clone(), changes_detected);

    assert_eq!(refined_prompt.original, original);
    assert_eq!(refined_prompt.refined, refined);
    assert_eq!(refined_prompt.changes_detected, changes_detected);
    assert_eq!(refined_prompt.get_confidence(), 0.8); // Default confidence for testing
}

#[tokio::test]
async fn test_refined_prompt_no_changes() {
    let original = "send 1 SOL to test".to_string();
    let refined = original.clone();
    let changes_detected = false;

    let refined_prompt =
        RefinedPrompt::new_for_test(original.clone(), refined.clone(), changes_detected);

    assert_eq!(refined_prompt.original, original);
    assert_eq!(refined_prompt.refined, refined);
    assert_eq!(refined_prompt.changes_detected, changes_detected);
    assert_eq!(refined_prompt.get_confidence(), 0.8); // Default confidence for testing
}
