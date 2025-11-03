//! Mock Data System for Testing Dynamic Flows
//!
//! This module provides mock wallet contexts, token data, and price information
//! extracted from Jupiter SDK tests to enable comprehensive testing without external dependencies.

use reev_types::benchmark::TokenBalance;
use reev_types::flow::WalletContext;

/// Mock token information based on Jupiter SDK data
pub struct MockToken {
    pub mint: &'static str,
    pub usd_price: f64,
}

/// Common mock tokens based on Jupiter SDK test data
pub const MOCK_TOKENS: &[MockToken] = &[
    MockToken {
        mint: "So11111111111111111111111111111111111111112",
        usd_price: 150.0,
    },
    MockToken {
        mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
        usd_price: 1.0,
    },
    MockToken {
        mint: "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
        usd_price: 1.0,
    },
    MockToken {
        mint: "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263",
        usd_price: 0.000025,
    },
    MockToken {
        mint: "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So",
        usd_price: 148.5,
    },
];

/// Mock wallet scenarios for comprehensive testing
pub struct MockWalletScenario {
    pub name: &'static str,
    pub pubkey: &'static str,
    pub sol_balance: u64,
}

/// Common wallet scenarios covering DeFi patterns
pub const MOCK_WALLET_SCENARIOS: &[MockWalletScenario] = &[
    MockWalletScenario {
        name: "empty_wallet",
        pubkey: "empty_wallet_test",
        sol_balance: 1_000_000_000, // 1 SOL
    },
    MockWalletScenario {
        name: "sol_only_wallet",
        pubkey: "sol_only_test",
        sol_balance: 10_000_000_000, // 10 SOL
    },
    MockWalletScenario {
        name: "balanced_portfolio",
        pubkey: "balanced_test",
        sol_balance: 5_000_000_000, // 5 SOL
    },
    MockWalletScenario {
        name: "defi_power_user",
        pubkey: "defi_power_test",
        sol_balance: 50_000_000_000, // 50 SOL
    },
    MockWalletScenario {
        name: "small_holder",
        pubkey: "small_holder_test",
        sol_balance: 500_000_000, // 0.5 SOL
    },
];

/// Create mock wallet context from scenario
pub fn create_mock_wallet_context(scenario: &MockWalletScenario) -> WalletContext {
    let mut context = WalletContext::new(scenario.pubkey.to_string());
    context.sol_balance = scenario.sol_balance;

    // Add token prices
    for token in MOCK_TOKENS {
        context.add_token_price(token.mint.to_string(), token.usd_price);
    }

    // Add token balances for specific scenarios
    match scenario.name {
        "balanced_portfolio" => {
            // Add 5000 USDC
            let usdc_balance = TokenBalance {
                mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
                balance: 5_000_000_000,
                decimals: Some(6),
                symbol: Some("USDC".to_string()),
                formatted_amount: Some("5000.0 USDC".to_string()),
                owner: Some(scenario.pubkey.to_string()),
            };
            context.add_token_balance(usdc_balance.mint.clone(), usdc_balance);

            // Add 2000 USDT
            let usdt_balance = TokenBalance {
                mint: "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".to_string(),
                balance: 2_000_000_000,
                decimals: Some(6),
                symbol: Some("USDT".to_string()),
                formatted_amount: Some("2000.0 USDT".to_string()),
                owner: Some(scenario.pubkey.to_string()),
            };
            context.add_token_balance(usdt_balance.mint.clone(), usdt_balance);
        }
        "defi_power_user" => {
            // Add 100k USDC
            let usdc_balance = TokenBalance {
                mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
                balance: 100_000_000_000,
                decimals: Some(6),
                symbol: Some("USDC".to_string()),
                formatted_amount: Some("100000.0 USDC".to_string()),
                owner: Some(scenario.pubkey.to_string()),
            };
            context.add_token_balance(usdc_balance.mint.clone(), usdc_balance);

            // Add 50k USDT
            let usdt_balance = TokenBalance {
                mint: "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".to_string(),
                balance: 50_000_000_000,
                decimals: Some(6),
                symbol: Some("USDT".to_string()),
                formatted_amount: Some("50000.0 USDT".to_string()),
                owner: Some(scenario.pubkey.to_string()),
            };
            context.add_token_balance(usdt_balance.mint.clone(), usdt_balance);

            // Add 1M Bonk
            let bonk_balance = TokenBalance {
                mint: "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263".to_string(),
                balance: 100_000_000_000,
                decimals: Some(5),
                symbol: Some("Bonk".to_string()),
                formatted_amount: Some("1000000.0 Bonk".to_string()),
                owner: Some(scenario.pubkey.to_string()),
            };
            context.add_token_balance(bonk_balance.mint.clone(), bonk_balance);

            // Add 10 mSOL
            let msol_balance = TokenBalance {
                mint: "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So".to_string(),
                balance: 10_000_000_000,
                decimals: Some(9),
                symbol: Some("marinade".to_string()),
                formatted_amount: Some("10.0 marinade".to_string()),
                owner: Some(scenario.pubkey.to_string()),
            };
            context.add_token_balance(msol_balance.mint.clone(), msol_balance);
        }
        "small_holder" => {
            // Add 100 USDC
            let usdc_balance = TokenBalance {
                mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
                balance: 100_000_000,
                decimals: Some(6),
                symbol: Some("USDC".to_string()),
                formatted_amount: Some("100.0 USDC".to_string()),
                owner: Some(scenario.pubkey.to_string()),
            };
            context.add_token_balance(usdc_balance.mint.clone(), usdc_balance);

            // Add 100k Bonk
            let bonk_balance = TokenBalance {
                mint: "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263".to_string(),
                balance: 10_000_000_000,
                decimals: Some(5),
                symbol: Some("Bonk".to_string()),
                formatted_amount: Some("100000.0 Bonk".to_string()),
                owner: Some(scenario.pubkey.to_string()),
            };
            context.add_token_balance(bonk_balance.mint.clone(), bonk_balance);
        }
        _ => {} // No token balances for empty/sol_only wallets
    }

    // Calculate total value
    context.calculate_total_value();

    context
}

/// Get mock token by mint address
pub fn get_mock_token(mint: &str) -> Option<&'static MockToken> {
    MOCK_TOKENS.iter().find(|t| t.mint == mint)
}

/// Get mock wallet scenario by name
pub fn get_mock_scenario(name: &str) -> Option<&'static MockWalletScenario> {
    MOCK_WALLET_SCENARIOS.iter().find(|s| s.name == name)
}

/// Get all mock scenarios for comprehensive testing
pub fn all_mock_scenarios() -> Vec<WalletContext> {
    MOCK_WALLET_SCENARIOS
        .iter()
        .map(create_mock_wallet_context)
        .collect()
}

/// Mock price response for testing price resolution
pub fn get_mock_price_response(token_mint: &str) -> Option<f64> {
    get_mock_token(token_mint).map(|t| t.usd_price)
}

// Mock transaction responses removed as they were unused

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_token_creation() {
        let context = create_mock_wallet_context(&MOCK_WALLET_SCENARIOS[2]); // balanced_portfolio
        assert_eq!(context.owner, "balanced_test");
        assert_eq!(context.sol_balance_sol(), 5.0);
        assert_eq!(context.total_value_usd, 7750.0);
    }

    #[test]
    fn test_mock_price_resolution() {
        assert_eq!(
            get_mock_price_response("So11111111111111111111111111111111111111112"),
            Some(150.0)
        );
        assert_eq!(
            get_mock_price_response("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
            Some(1.0)
        );
        assert_eq!(get_mock_price_response("unknown"), None);
    }

    #[test]
    fn test_all_scenarios_coverage() {
        let contexts = all_mock_scenarios();
        assert_eq!(contexts.len(), MOCK_WALLET_SCENARIOS.len());

        // Verify each scenario covers different patterns
        for (i, context) in contexts.iter().enumerate() {
            assert!(
                context.total_value_usd > 0.0,
                "Scenario {i} should have value"
            );
        }
    }
}
