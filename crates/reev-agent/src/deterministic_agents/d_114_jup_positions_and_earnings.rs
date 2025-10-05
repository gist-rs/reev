use anyhow::{Context, Result};
use serde_json::json;
use std::collections::HashMap;
use tracing::info;

/// Handles the deterministic logic for the `114-JUP-POSITIONS-AND-EARNINGS` benchmark.
///
/// This is a multi-step flow benchmark that demonstrates fetching Jupiter positions
/// and then getting earnings data. For the deterministic agent, we return mock
/// responses that simulate the real API calls.
pub(crate) async fn handle_jup_positions_and_earnings(
    key_map: &HashMap<String, String>,
) -> Result<serde_json::Value> {
    info!("[reev-agent] Matched '114-JUP-POSITIONS-AND-EARNINGS' id. Creating deterministic multi-step flow response.");

    let user_pubkey = key_map
        .get("USER_WALLET_PUBKEY")
        .context("USER_WALLET_PUBKEY not found in key_map")?;

    // Step 1: Mock Jupiter positions response
    let positions_response = json!({
        "total_positions": 6,
        "positions_with_balance": 1,
        "summary": [
            {
                "token": {
                    "symbol": "jlUSDC",
                    "name": "jupiter lend USDC",
                    "address": "9BEcn9aPEmhSPbPQeFGjidRiEKki46fVQDyPpSQXPA2D",
                    "asset_address": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                    "decimals": 6
                },
                "asset": {
                    "symbol": "USDC",
                    "name": "USD Coin",
                    "price": "0.99970715345",
                    "logo_url": "https://coin-images.coingecko.com/coins/images/6319/large/usdc.png?1696506694"
                },
                "position": {
                    "shares": "916115281",
                    "underlying_assets": "927065854",
                    "underlying_balance": "0",
                    "underlying_balance_decimal": 927.065854,
                    "usd_value": 926.79,
                    "allowance": "0"
                },
                "rates": {
                    "supply_rate_pct": 5.19,
                    "total_rate_pct": 8.68,
                    "rewards_rate": "349"
                },
                "liquidity": {
                    "total_assets": "348342806597852",
                    "withdrawable": "36750926351916",
                    "withdrawal_limit": "260762024082453"
                }
            },
            {
                "token": {
                    "symbol": "jlUSDT",
                    "name": "jupiter lend USDT",
                    "address": "Cmn4v2wipYV41dkakDvCgFJpxhtaaKt11NyWV8pjSE8A",
                    "asset_address": "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
                    "decimals": 6
                },
                "asset": {
                    "symbol": "USDT",
                    "name": "USDT",
                    "price": "1.000066084195",
                    "logo_url": "https://coin-images.coingecko.com/coins/images/325/large/Tether.png?1696501661"
                },
                "position": {
                    "shares": "0",
                    "underlying_assets": "0",
                    "underlying_balance": "0",
                    "underlying_balance_decimal": 0.0,
                    "usd_value": 0.0,
                    "allowance": "0"
                },
                "rates": {
                    "supply_rate_pct": 4.18,
                    "total_rate_pct": 8.16,
                    "rewards_rate": "398"
                },
                "liquidity": {
                    "total_assets": "30511937495039",
                    "withdrawable": "3979829602520",
                    "withdrawal_limit": "22820545754791"
                }
            },
            {
                "token": {
                    "symbol": "jlSOL",
                    "name": "jupiter lend WSOL",
                    "address": "2uQsyo1fXXQkDtcpXnLofWy88PxcvnfH2L8FPSE62FVU",
                    "asset_address": "So11111111111111111111111111111111111111112",
                    "decimals": 9
                },
                "asset": {
                    "symbol": "WSOL",
                    "name": "Wrapped SOL",
                    "price": "233.723465903757",
                    "logo_url": "https://coin-images.coingecko.com/coins/images/21629/large/solana.jpg?1696520989"
                },
                "position": {
                    "shares": "0",
                    "underlying_assets": "0",
                    "underlying_balance": "0",
                    "underlying_balance_decimal": 0.0,
                    "usd_value": 0.0,
                    "allowance": "0"
                },
                "rates": {
                    "supply_rate_pct": 4.86,
                    "total_rate_pct": 4.86,
                    "rewards_rate": "0"
                },
                "liquidity": {
                    "total_assets": "39140566297455",
                    "withdrawable": "39143684935924",
                    "withdrawal_limit": "39143684935924"
                }
            }
        ],
        "raw_positions": []
    });

    // Step 2: Mock Jupiter earnings response
    let earnings_response = json!({
        "user_pubkey": user_pubkey,
        "position_filter": null,
        "total_positions": 1,
        "summary": {
            "total_earnings": {
                "raw": "5755419",
                "decimal": 5755419.0
            },
            "total_deposits": {
                "raw": "3333314252",
                "decimal": 3333314252.0
            },
            "total_withdraws": {
                "raw": "2412000000",
                "decimal": 2412000000.0
            },
            "current_balance": {
                "raw": "916115281",
                "decimal": 916115281.0
            }
        },
        "positions": [
            {
                "position_address": "9BEcn9aPEmhSPbPQeFGjidRiEKki46fVQDyPpSQXPA2D",
                "owner_address": user_pubkey,
                "earnings": {
                    "raw": "5755419",
                    "decimal": 5755419.0
                },
                "deposits": {
                    "raw": "3333314252",
                    "decimal": 3333314252.0
                },
                "withdraws": {
                    "raw": "2412000000",
                    "decimal": 2412000000.0
                },
                "current_balance": {
                    "raw": "916115281",
                    "decimal": 916115281.0
                },
                "total_assets": "927069671",
                "slot": 371334523
            }
        ],
        "raw_earnings": [
            {
                "address": "9BEcn9aPEmhSPbPQeFGjidRiEKki46fVQDyPpSQXPA2D",
                "ownerAddress": user_pubkey,
                "totalDeposits": "3333314252",
                "totalWithdraws": "2412000000",
                "totalBalance": "916115281",
                "totalAssets": "927069671",
                "earnings": "5755419",
                "slot": 371334523
            }
        ]
    });

    // Create a comprehensive response that includes both steps
    let response = json!({
        "step_1_result": positions_response,
        "step_2_result": earnings_response,
        "flow_completed": true,
        "summary": {
            "total_positions": 6,
            "positions_with_balance": 1,
            "total_earnings_usd": 5.76,
            "total_deposits_usd": 3333.31,
            "total_withdraws_usd": 2412.00,
            "current_balance_usd": 916.12,
            "active_positions": ["jlUSDC"],
            "highest_yielding_position": {
                "symbol": "jlUSDC",
                "supply_rate_pct": 5.19,
                "total_rate_pct": 8.68
            }
        }
    });

    info!(
        "[reev-agent] Successfully created deterministic multi-step flow response with {} positions and ${:.2} in total earnings.",
        response["step_1_result"]["total_positions"],
        response["summary"]["total_earnings_usd"].as_f64().unwrap_or(0.0)
    );

    Ok(response)
}
