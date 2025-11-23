//! Tests for planner module

use reev_core::context::ContextResolver;
use reev_core::planner::{LlmClient, Planner, UserIntent};
// MockLlmClient is defined in the test file itself

#[tokio::test]
async fn test_parse_swap_intent() {
    let context_resolver = ContextResolver::default();
    let planner = Planner::new(context_resolver);

    let intent = planner.parse_intent("swap 1 SOL to USDC").unwrap();

    match intent {
        UserIntent::Swap {
            from,
            to,
            amount,
            percentage,
        } => {
            assert_eq!(from, "SOL");
            assert_eq!(to, "USDC");
            assert_eq!(amount, 1.0);
            assert!(percentage.is_none());
        }
        _ => panic!("Expected Swap intent"),
    }
}

#[tokio::test]
async fn test_parse_lend_intent() {
    let context_resolver = ContextResolver::default();
    let planner = Planner::new(context_resolver);

    let intent = planner.parse_intent("lend 100 USDC to jupiter").unwrap();

    match intent {
        UserIntent::Lend {
            mint,
            amount,
            percentage,
        } => {
            assert_eq!(mint, "USDC");
            assert_eq!(amount, 100.0);
            assert!(percentage.is_none());
        }
        _ => panic!("Expected Lend intent"),
    }
}

#[tokio::test]
async fn test_parse_swap_then_lend_intent() {
    let context_resolver = ContextResolver::default();
    let planner = Planner::new(context_resolver);

    let intent = planner
        .parse_intent("swap 50% SOL to USDC and lend 50%")
        .unwrap();

    match intent {
        UserIntent::SwapThenLend {
            from,
            to,
            amount,
            percentage,
        } => {
            assert_eq!(from, "SOL");
            assert_eq!(to, "USDC");
            assert_eq!(amount, 50.0);
            assert!(percentage.is_some());
        }
        _ => panic!("Expected SwapThenLend intent"),
    }
}

#[tokio::test]
async fn test_extract_percentage() {
    let context_resolver = ContextResolver::default();
    let planner = Planner::new(context_resolver);

    assert_eq!(
        planner.extract_percentage("swap 50% SOL to USDC"),
        Some(50.0)
    );
    assert_eq!(
        planner.extract_percentage("swap 25.5% SOL to USDC"),
        Some(25.5)
    );
    assert_eq!(planner.extract_percentage("swap 1 SOL to USDC"), None);
}

#[tokio::test]
async fn test_token_to_mint() {
    let context_resolver = ContextResolver::default();
    let planner = Planner::new(context_resolver);

    assert_eq!(
        planner.token_to_mint("SOL").unwrap(),
        "So11111111111111111111111111111111111111112".to_string()
    );
    assert_eq!(
        planner.token_to_mint("USDC").unwrap(),
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()
    );
    assert!(planner.token_to_mint("UNKNOWN").is_err());
}

#[tokio::test]
async fn test_refine_and_plan_with_llm() {
    use reev_types::flow::WalletContext;

    struct MockLlmClient;

    #[async_trait::async_trait]
    impl LlmClient for MockLlmClient {
        async fn generate_flow(&self, _prompt: &str) -> Result<String, anyhow::Error> {
            Ok(r#"
flow_id: "test-flow-id"
user_prompt: "swap 1 SOL to USDC"
subject_wallet_info:
  pubkey: "11111111111111111111111111111112"
  lamports: 1000000000
  total_value_usd: 150.0
  tokens:
    - mint: "So11111111111111111111111111111111111111112"
      balance: 1000000000
      decimals: 9
      symbol: "SOL"
steps:
  - step_id: "swap"
    prompt: "swap 1 SOL to USDC"
    context: "Exchange 1 SOL for USDC"
    critical: true
ground_truth:
  final_state_assertions:
    - assertion_type: "SolBalanceChange"
      pubkey: "11111111111111111111111111111112"
      expected_change_gte: -1010000000
  error_tolerance: 0.01
metadata:
  category: "swap"
  complexity_score: 1
  tags: []
  version: "1.0"
created_at: "2023-01-01T00:00:00Z"
"#
            .to_string())
        }
    }

    let context_resolver = ContextResolver::default();
    let planner = Planner::new(context_resolver);
    let mock_client = MockLlmClient;

    // Directly test the LLM client integration without wallet resolution
    // This avoids blocking RPC call and SURFPOOL dependency
    let mut wallet_context = WalletContext::new("test-pubkey".to_string());
    wallet_context.sol_balance = 1_000_000_000; // 1 SOL
    wallet_context.total_value_usd = 150.0;

    let flow = planner
        .generate_flow_with_llm("swap 1 SOL to USDC", &wallet_context, &mock_client)
        .await
        .unwrap();

    assert_eq!(flow.flow_id, "test-flow-id");
    assert_eq!(flow.user_prompt, "swap 1 SOL to USDC");
    assert_eq!(
        flow.subject_wallet_info.pubkey,
        "11111111111111111111111111111112"
    );
    assert_eq!(flow.steps.len(), 1);
    assert_eq!(flow.steps[0].step_id, "swap");
    assert!(flow.ground_truth.is_some());
}
