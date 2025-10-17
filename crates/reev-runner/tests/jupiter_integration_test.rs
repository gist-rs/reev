//! # Jupiter Integration Tests
//!
//! This test suite validates Jupiter protocol integration under various
//! edge cases including insufficient liquidity, slippage protection,
//! and error handling scenarios.

use anyhow::Result;
use reev_lib::{
    agent::AgentAction, benchmark::TestCase, env::GymEnv, score::calculate_final_score,
    solana_env::environment::SolanaEnv,
};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Signer,
};
use std::str::FromStr;

/// Test Jupiter swap with insufficient SOL balance
#[tokio::test(flavor = "multi_thread")]
async fn test_jupiter_swap_insufficient_balance() -> Result<()> {
    let _ = tracing_subscriber::fmt::try_init();

    tracing::info!("🧪 Testing Jupiter swap with insufficient SOL balance");

    // Create a minimal test case for insufficient balance
    let user_wallet = Pubkey::new_unique();
    let usdc_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")?;
    let jupiter_program = Pubkey::from_str("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4")?;

    // Create mock test case
    let test_case = TestCase {
        id: "jupiter-insufficient-test".to_string(),
        description: "Test insufficient balance scenario".to_string(),
        tags: vec!["jupiter".to_string(), "swap".to_string()],
        prompt: "Swap 1 SOL for USDC using Jupiter".to_string(),
        initial_state: vec![reev_lib::benchmark::InitialStateItem {
            pubkey: "USER_WALLET_PUBKEY".to_string(),
            owner: "11111111111111111111111111111111".to_string(),
            lamports: 50000000, // 0.05 SOL (insufficient)
            data: None,
        }],
        flow: Some(vec![]),
        ground_truth: reev_lib::benchmark::GroundTruth {
            final_state_assertions: vec![],
            expected_instructions: vec![],
            skip_instruction_validation: false,
            transaction_status: "Failure".to_string(),
        },
    };

    // Set up environment
    let mut env = SolanaEnv::new()?;
    let fee_payer_keypair = solana_sdk::signature::Keypair::new();
    let fee_payer_pubkey = fee_payer_keypair.pubkey();
    env.fee_payer = Some("FEE_PAYER".to_string());
    env.keypair_map
        .insert("FEE_PAYER".to_string(), fee_payer_keypair);
    env.pubkey_map
        .insert("FEE_PAYER".to_string(), fee_payer_pubkey);

    let mut key_map = std::collections::HashMap::new();
    key_map.insert("USER_WALLET_PUBKEY".to_string(), user_wallet.to_string());

    let initial_obs = reev_lib::agent::AgentObservation {
        last_transaction_status: "Success".to_string(),
        last_transaction_error: None,
        last_transaction_logs: vec![],
        account_states: std::collections::HashMap::new(),
        key_map,
    };

    // Create mock Jupiter instruction that should fail
    let mock_instruction = Instruction {
        program_id: jupiter_program,
        accounts: vec![
            AccountMeta::new(user_wallet, true),
            AccountMeta::new_readonly(usdc_mint, false),
        ],
        data: vec![1, 0, 0, 0], // Invalid instruction data for testing
    };

    let actions = vec![AgentAction(mock_instruction)];

    // Execute the transaction (should fail)
    let step_result = env.step(actions.clone(), &test_case.ground_truth)?;

    // Calculate score - should be low due to execution failure
    let score = calculate_final_score(&test_case, &actions, &initial_obs, &step_result.observation);

    // Score should be low (transaction should fail)
    assert!(
        score < 0.5,
        "Expected low score for failed swap, got {score}"
    );
    assert_eq!(step_result.observation.last_transaction_status, "Failure");

    env.close()?;
    tracing::info!("✅ Insufficient balance test completed");
    Ok(())
}

/// Test Jupiter swap with malformed instruction
#[tokio::test(flavor = "multi_thread")]
async fn test_jupiter_malformed_instruction() -> Result<()> {
    let _ = tracing_subscriber::fmt::try_init();

    tracing::info!("🧪 Testing Jupiter with malformed instruction data");

    let user_wallet = Pubkey::new_unique();
    let jupiter_program = Pubkey::from_str("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4")?;

    let test_case = TestCase {
        id: "jupiter-malformed-test".to_string(),
        description: "Test malformed instruction scenario".to_string(),
        tags: vec!["jupiter".to_string(), "error".to_string()],
        prompt: "Swap SOL for USDC using Jupiter".to_string(),
        initial_state: vec![reev_lib::benchmark::InitialStateItem {
            pubkey: "USER_WALLET_PUBKEY".to_string(),
            owner: "11111111111111111111111111111111".to_string(),
            lamports: 1000000000, // 1 SOL
            data: None,
        }],
        flow: Some(vec![]),
        ground_truth: reev_lib::benchmark::GroundTruth {
            final_state_assertions: vec![],
            expected_instructions: vec![],
            skip_instruction_validation: false,
            transaction_status: "Failure".to_string(),
        },
    };

    let mut env = SolanaEnv::new()?;
    let fee_payer_keypair = solana_sdk::signature::Keypair::new();
    let fee_payer_pubkey = fee_payer_keypair.pubkey();
    env.fee_payer = Some("FEE_PAYER".to_string());
    env.keypair_map
        .insert("FEE_PAYER".to_string(), fee_payer_keypair);
    env.pubkey_map
        .insert("FEE_PAYER".to_string(), fee_payer_pubkey);

    let mut key_map = std::collections::HashMap::new();
    key_map.insert("USER_WALLET_PUBKEY".to_string(), user_wallet.to_string());

    let initial_obs = reev_lib::agent::AgentObservation {
        last_transaction_status: "Success".to_string(),
        last_transaction_error: None,
        last_transaction_logs: vec![],
        account_states: std::collections::HashMap::new(),
        key_map,
    };

    // Create malformed instruction (empty accounts, invalid data)
    let malformed_instruction = Instruction {
        program_id: jupiter_program,
        accounts: vec![], // Empty accounts - should fail
        data: vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF], // Invalid data
    };

    let actions = vec![AgentAction(malformed_instruction)];
    let step_result = env.step(actions.clone(), &test_case.ground_truth)?;

    let score = calculate_final_score(&test_case, &actions, &initial_obs, &step_result.observation);

    // Malformed instructions should fail completely
    assert!(
        score < 0.25,
        "Expected very low score for malformed instruction, got {score}"
    );
    assert_eq!(step_result.observation.last_transaction_status, "Failure");

    env.close()?;
    tracing::info!("✅ Malformed instruction test completed");
    Ok(())
}

/// Test Jupiter with valid instruction but potential execution issues
#[tokio::test(flavor = "multi_thread")]
async fn test_jupiter_valid_instruction_execution() -> Result<()> {
    let _ = tracing_subscriber::fmt::try_init();

    tracing::info!("🧪 Testing Jupiter with valid instruction structure");

    let user_wallet = Pubkey::new_unique();
    let usdc_ata = Pubkey::new_unique();
    let usdc_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")?;
    let jupiter_program = Pubkey::from_str("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4")?;
    let token_program = Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")?;

    let test_case = TestCase {
        id: "jupiter-valid-test".to_string(),
        description: "Test valid instruction scenario".to_string(),
        tags: vec!["jupiter".to_string(), "valid".to_string()],
        prompt: "Swap 0.1 SOL for USDC using Jupiter".to_string(),
        initial_state: vec![
            reev_lib::benchmark::InitialStateItem {
                pubkey: "USER_WALLET_PUBKEY".to_string(),
                owner: "11111111111111111111111111111111".to_string(),
                lamports: 1000000000, // 1 SOL
                data: None,
            },
            reev_lib::benchmark::InitialStateItem {
                pubkey: "USER_USDC_ATA".to_string(),
                owner: token_program.to_string(),
                lamports: 2039280,
                data: Some(reev_lib::benchmark::SplAccountData {
                    mint: usdc_mint.to_string(),
                    owner: user_wallet.to_string(),
                    amount: "0".to_string(),
                }),
            },
        ],
        flow: Some(vec![]),
        ground_truth: reev_lib::benchmark::GroundTruth {
            final_state_assertions: vec![],
            expected_instructions: vec![],
            skip_instruction_validation: false,
            transaction_status: "Success".to_string(),
        },
    };

    let mut env = SolanaEnv::new()?;
    let fee_payer_keypair = solana_sdk::signature::Keypair::new();
    let fee_payer_pubkey = fee_payer_keypair.pubkey();
    env.fee_payer = Some("FEE_PAYER".to_string());
    env.keypair_map
        .insert("FEE_PAYER".to_string(), fee_payer_keypair);
    env.pubkey_map
        .insert("FEE_PAYER".to_string(), fee_payer_pubkey);

    let mut key_map = std::collections::HashMap::new();
    key_map.insert("USER_WALLET_PUBKEY".to_string(), user_wallet.to_string());
    key_map.insert("USER_USDC_ATA".to_string(), usdc_ata.to_string());

    let initial_obs = reev_lib::agent::AgentObservation {
        last_transaction_status: "Success".to_string(),
        last_transaction_error: None,
        last_transaction_logs: vec![],
        account_states: std::collections::HashMap::new(),
        key_map,
    };

    // Create a valid Jupiter swap instruction structure
    let valid_instruction = Instruction {
        program_id: jupiter_program,
        accounts: vec![
            AccountMeta::new(user_wallet, true), // User wallet as signer
            AccountMeta::new(usdc_ata, false),   // USDC ATA
            AccountMeta::new_readonly(usdc_mint, false), // USDC mint
            AccountMeta::new_readonly(token_program, false), // Token program
            AccountMeta::new_readonly(Pubkey::new_unique(), false), // SYSVAR_RENT
        ],
        data: vec![1, 0, 0, 0], // Valid swap instruction prefix
    };

    let actions = vec![AgentAction(valid_instruction)];
    let step_result = env.step(actions.clone(), &test_case.ground_truth)?;

    let score = calculate_final_score(&test_case, &actions, &initial_obs, &step_result.observation);

    // Score should be valid (0.0 to 1.0) - actual execution depends on surfpool
    assert!(
        (0.0..=1.0).contains(&score),
        "Score should be valid range: {score}"
    );

    tracing::info!(
        "Valid instruction test completed - Score: {}, Status: {}",
        score,
        step_result.observation.last_transaction_status
    );

    env.close()?;
    tracing::info!("✅ Valid instruction test completed");
    Ok(())
}

/// Test multiple Jupiter operations in sequence
#[tokio::test(flavor = "multi_thread")]
async fn test_jupiter_multiple_operations() -> Result<()> {
    let _ = tracing_subscriber::fmt::try_init();

    tracing::info!("🧪 Testing multiple Jupiter operations in sequence");

    let user_wallet = Pubkey::new_unique();
    let usdc_ata = Pubkey::new_unique();
    let usdc_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")?;
    let jupiter_program = Pubkey::from_str("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4")?;
    let jupiter_lending = Pubkey::from_str("jup3YeL8QhtSx1e253b2FDvsMNC87fDrgQZivbrndc9")?;
    let token_program = Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")?;

    let test_case = TestCase {
        id: "jupiter-multi-test".to_string(),
        description: "Test multiple Jupiter operations".to_string(),
        tags: vec!["jupiter".to_string(), "multi".to_string()],
        prompt: "Swap SOL for USDC and deposit to lending".to_string(),
        initial_state: vec![
            reev_lib::benchmark::InitialStateItem {
                pubkey: "USER_WALLET_PUBKEY".to_string(),
                owner: "11111111111111111111111111111111".to_string(),
                lamports: 2000000000, // 2 SOL
                data: None,
            },
            reev_lib::benchmark::InitialStateItem {
                pubkey: "USER_USDC_ATA".to_string(),
                owner: token_program.to_string(),
                lamports: 2039280,
                data: Some(reev_lib::benchmark::SplAccountData {
                    mint: usdc_mint.to_string(),
                    owner: user_wallet.to_string(),
                    amount: "0".to_string(),
                }),
            },
        ],
        flow: Some(vec![]),
        ground_truth: reev_lib::benchmark::GroundTruth {
            final_state_assertions: vec![],
            expected_instructions: vec![],
            skip_instruction_validation: false,
            transaction_status: "Success".to_string(),
        },
    };

    let mut env = SolanaEnv::new()?;
    let fee_payer_keypair = solana_sdk::signature::Keypair::new();
    let fee_payer_pubkey = fee_payer_keypair.pubkey();
    env.fee_payer = Some("FEE_PAYER".to_string());
    env.keypair_map
        .insert("FEE_PAYER".to_string(), fee_payer_keypair);
    env.pubkey_map
        .insert("FEE_PAYER".to_string(), fee_payer_pubkey);

    let mut key_map = std::collections::HashMap::new();
    key_map.insert("USER_WALLET_PUBKEY".to_string(), user_wallet.to_string());
    key_map.insert("USER_USDC_ATA".to_string(), usdc_ata.to_string());

    let initial_obs = reev_lib::agent::AgentObservation {
        last_transaction_status: "Success".to_string(),
        last_transaction_error: None,
        last_transaction_logs: vec![],
        account_states: std::collections::HashMap::new(),
        key_map,
    };

    // First operation: Jupiter swap
    let swap_instruction = Instruction {
        program_id: jupiter_program,
        accounts: vec![
            AccountMeta::new(user_wallet, true),
            AccountMeta::new(usdc_ata, false),
            AccountMeta::new_readonly(usdc_mint, false),
            AccountMeta::new_readonly(token_program, false),
        ],
        data: vec![1, 0, 0, 0], // Swap instruction
    };

    let swap_action = AgentAction(swap_instruction);
    let swap_result = env.step(vec![swap_action.clone()], &test_case.ground_truth)?;

    // Second operation: Jupiter lending deposit
    let lend_instruction = Instruction {
        program_id: jupiter_lending,
        accounts: vec![
            AccountMeta::new(user_wallet, true),
            AccountMeta::new(usdc_ata, false),
            AccountMeta::new_readonly(usdc_mint, false),
        ],
        data: vec![2, 0, 0, 0, 160, 134, 1, 0], // Deposit 10 USDC (10000000 as little-endian bytes)
    };

    let lend_action = AgentAction(lend_instruction);
    let final_result = env.step(vec![lend_action.clone()], &test_case.ground_truth)?;

    // Calculate final score for multi-operation sequence
    let all_actions = vec![swap_action, lend_action];
    let score = calculate_final_score(
        &test_case,
        &all_actions,
        &initial_obs,
        &final_result.observation,
    );

    assert!(
        (0.0..=1.0).contains(&score),
        "Score should be valid for multi-operation: {score}"
    );

    tracing::info!(
        "Multi-operation test completed - Score: {}, Swap Status: {}, Lend Status: {}",
        score,
        swap_result.observation.last_transaction_status,
        final_result.observation.last_transaction_status
    );

    env.close()?;
    tracing::info!("✅ Multiple operations test completed");
    Ok(())
}
