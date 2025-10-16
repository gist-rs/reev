//! Tests for amount constants

use reev_lib::constants::amounts::{scoring, slippage, tokens};

#[test]
fn test_usdc_amounts() {
    assert_eq!(tokens::usdc::ONE, 1_000_000);
    assert_eq!(tokens::usdc::TEN, 10_000_000);
    assert_eq!(tokens::usdc::FIFTY, 50_000_000);
    assert_eq!(tokens::usdc::HUNDRED, 100_000_000);
}

#[test]
fn test_sol_amounts() {
    assert_eq!(tokens::sol::ONE_MILLI, 1_000_000);
    assert_eq!(tokens::sol::ONE_CENTI, 10_000_000);
    assert_eq!(tokens::sol::ONE_DECI, 100_000_000);
    assert_eq!(tokens::sol::HALF, 500_000_000);
    assert_eq!(tokens::sol::ONE, 1_000_000_000);
    assert_eq!(tokens::sol::FEE_RESERVE, 5_000_000_000);
}

#[test]
fn test_slippage_values() {
    assert_eq!(slippage::FIVE_PERCENT, 500);
    assert_eq!(slippage::EIGHT_PERCENT, 800);
    assert_eq!(slippage::TEN_PERCENT, 1000);
}

#[test]
fn test_scoring_constants() {
    assert_eq!(scoring::MIN_PASSING_SCORE, 0.5);
    assert_eq!(scoring::MAX_SCORE, 100.0);
    assert_eq!(scoring::SCORE_TOLERANCE, 5.0);
}
