//! Reev Core Library - Simplified Architecture
//!
//! This library implements the new reev-core architecture with:
//! - 18-step deterministic flow processing
//! - Snapshot-based testing for reliability
//! - Modular design with clear separation of concerns
//! - Mock-based testing for CI/CD reliability

pub mod core;
pub mod prompts;
pub mod test_snapshots;
pub mod types;

// Re-export main types for convenience
pub use core::*;
pub use test_snapshots::*;
pub use types::*;

// Legacy modules that are kept for compatibility (to be removed later)
pub mod constants;
pub mod env;

// Remove obsolete modules - they cause errors and are not needed in new architecture
// - agent.rs (obsolete)
// - balance_validation.rs (obsolete)
// - benchmark.rs (obsolete)
// - db.rs (obsolete)
// - flow.rs (obsolete)
// - instruction_score.rs (obsolete)
// - llm_agent.rs (obsolete)
// - mock.rs (obsolete)
// - otel_extraction.rs (obsolete)
// - parsing.rs (obsolete)
// - results.rs (obsolete)
// - score.rs (obsolete)
// - server_utils.rs (obsolete)
// - session_logger.rs (obsolete)
// - solana_env.rs (obsolete)
// - test_scenarios.rs (obsolete)
// - trace.rs (obsolete)

// Legacy modules kept for compatibility (to be refactored later)
pub mod actions;
pub mod benchmark;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_types_creation() {
        // Test that core types can be created without errors
        let wallet_state = WalletState::new();
        assert_eq!(wallet_state.sol_amount, 0);
        assert_eq!(wallet_state.usdc_amount, 0);

        let api_service = CachedApiService::new("./cache".to_string(), true, false);
        assert!(!api_service.mock_mode);
        assert!(api_service.real_jupiter_client);

        let refined_prompt = RefinedPrompt::new(1, "test".to_string(), "reasoning".to_string());
        assert_eq!(refined_prompt.step, 1);
        assert_eq!(refined_prompt.prompt, "test");

        let execution_result = ExecutionResult::new("test_id".to_string(), "test_tool".to_string());
        assert_eq!(execution_result.execution_id, "test_id");
        assert_eq!(execution_result.tool_name, "test_tool");
        assert!(!execution_result.success);
    }

    #[test]
    fn test_constants() {
        // Verify constants are valid Solana addresses (44 characters)
        assert_eq!(SOL_MINT.len(), 44, "SOL_MINT should be 44 characters");
        assert_eq!(USDC_MINT.len(), 44, "USDC_MINT should be 44 characters");
        assert_ne!(
            SOL_MINT, USDC_MINT,
            "SOL_MINT and USDC_MINT should be different"
        );

        // Verify they're valid base58 format (don't panic)
        bs58::decode(SOL_MINT)
            .into_vec()
            .expect("SOL_MINT should be valid base58");
        bs58::decode(USDC_MINT)
            .into_vec()
            .expect("USDC_MINT should be valid base58");
    }
}
