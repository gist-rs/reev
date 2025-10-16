//! Numeric constants for amounts and values used throughout the project

/// Token amounts (in smallest units)
pub mod tokens {
    /// USDC amounts (6 decimals)
    pub mod usdc {
        /// 1 USDC in smallest units (1,000,000)
        pub const ONE: u64 = 1_000_000;

        /// 10 USDC in smallest units
        pub const TEN: u64 = 10_000_000;

        /// 50 USDC in smallest units
        pub const FIFTY: u64 = 50_000_000;

        /// 40 USDC in smallest units
        pub const FORTY: u64 = 40_000_000;

        /// 100 USDC in smallest units
        pub const HUNDRED: u64 = 100_000_000;
    }

    /// SOL amounts (9 decimals)
    pub mod sol {
        /// 0.001 SOL in lamports (1,000,000)
        pub const ONE_MILLI: u64 = 1_000_000;

        /// 0.01 SOL in lamports (10,000,000)
        pub const ONE_CENTI: u64 = 10_000_000;

        /// 0.1 SOL in lamports (100,000,000)
        pub const ONE_DECI: u64 = 100_000_000;

        /// 0.5 SOL in lamports (500,000,000)
        pub const HALF: u64 = 500_000_000;

        /// 1 SOL in lamports (1,000,000,000)
        pub const ONE: u64 = 1_000_000_000;

        /// 5 SOL in lamports for transaction fees
        pub const FEE_RESERVE: u64 = 5_000_000_000;
    }
}

/// Slippage tolerance values
pub mod slippage {
    /// 5% slippage tolerance in basis points
    pub const FIVE_PERCENT: u16 = 500;

    /// 8% slippage tolerance in basis points
    pub const EIGHT_PERCENT: u16 = 800;

    /// 10% slippage tolerance in basis points
    pub const TEN_PERCENT: u16 = 1000;
}

/// Solana-specific constants
pub mod solana {
    /// Rent exemption amount for typical accounts
    pub const RENT_EXEMPTION: u64 = 2_039_280;

    /// Minimum account balance
    pub const MIN_BALANCE: u64 = 890_880;
}

/// Default amounts used in benchmarks
pub mod defaults {
    use super::tokens::sol;
    use super::tokens::usdc;

    /// Default SOL swap amount for small tests
    pub const SOL_SWAP_AMOUNT: u64 = sol::ONE_DECI; // 0.1 SOL

    /// Default SOL swap amount for medium tests
    pub const SOL_SWAP_AMOUNT_MEDIUM: u64 = sol::HALF; // 0.5 SOL

    /// Default USDC amount for lending tests
    pub const USDC_LEND_AMOUNT: u64 = usdc::TEN; // 10 USDC

    /// Default USDC amount for larger lending tests
    pub const USDC_LEND_AMOUNT_LARGE: u64 = 50_000_000; // ~50 USDC (accounting for swap output)

    /// Default USDC amount for mint/redeem tests
    pub const USDC_MINT_AMOUNT: u64 = usdc::FIFTY; // 50 USDC
}

/// Scoring constants
pub mod scoring {
    /// Minimum score threshold for passing benchmarks
    pub const MIN_PASSING_SCORE: f64 = 0.5;

    /// Maximum possible score
    pub const MAX_SCORE: f64 = 100.0;

    /// Score tolerance for validation (±5%)
    pub const SCORE_TOLERANCE: f64 = 5.0;
}
