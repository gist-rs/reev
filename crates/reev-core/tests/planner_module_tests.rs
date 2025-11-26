//! Tests for planner module

use reev_core::planner::LlmClient;
use reev_core::refiner::RefinedPrompt;
use reev_core::yml_generator::YmlGenerator;
use reev_types::WalletContext;

// Mock LLM implementation for testing
#[allow(dead_code)]
struct MockLLMClient;

#[async_trait::async_trait]
impl LlmClient for MockLLMClient {
    async fn generate_flow(&self, prompt: &str) -> Result<String, anyhow::Error> {
        // Generate a simple mock YML flow based on the prompt
        let flow_id = uuid::Uuid::new_v4();
        let yml_response = format!(
            r#"flow_id: {flow_id}
user_prompt: "{prompt}"
subject_wallet_info:
  pubkey: "USER_WALLET_PUBKEY"
  lamports: 4000000000
  tokens: []
  total_value_usd: 100.0
steps:
  - step_id: "1"
    prompt: "{prompt}"
    context: "Test context"
    critical: true
    estimated_time_seconds: 5
ground_truth:
  final_state_assertions: []
  expected_tool_calls: []"#
        );
        Ok(yml_response)
    }
}

#[tokio::test]
async fn test_refine_and_plan() {
    // Create a mock wallet context instead of trying to resolve
    let mut wallet_context = WalletContext::new("test_wallet".to_string());
    wallet_context.sol_balance = 5_000_000_000; // 5 SOL

    let refined_prompt =
        RefinedPrompt::new_for_test("test prompt".to_string(), "test prompt".to_string(), false);

    let yml_generator = YmlGenerator::new();
    let result = yml_generator
        .generate_flow(&refined_prompt, &wallet_context)
        .await;

    assert!(result.is_ok());
}
