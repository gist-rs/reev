use anyhow::Result;
use env_logger::Env;
use jupiter_perps_rs::{JupiterPerpsClient, PositionSide};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // Initialize client from environment variables
    let client = JupiterPerpsClient::from_env()?;

    // Example 1: Get all open positions across the protocol
    println!("=== All Open Positions ===");
    match client.get_open_positions().await {
        Ok(positions) => {
            println!("Found {} open positions:", positions.len());
            for (i, position) in positions.iter().enumerate() {
                println!(
                    "  {}. Position: {} | Owner: {} | Custody: {} | Size: ${:.2} | Side: {:?}",
                    i + 1,
                    position.pubkey,
                    position.account.owner,
                    position.account.custody,
                    position.account.size_usd as f64 / 1_000_000.0,
                    position.account.side
                );
            }
        }
        Err(e) => {
            eprintln!("Failed to get open positions: {e}");
        }
    }

    // Example 2: Get positions for a specific wallet
    println!("\n=== Positions for Specific Wallet ===");
    let wallet_pubkey =
        std::env::var("WALLET_PUBKEY").unwrap_or_else(|_| "YOUR_WALLET_PUBKEY_HERE".to_string());

    if let Ok(wallet) = Pubkey::from_str(&wallet_pubkey) {
        match client.get_open_positions_for_wallet(&wallet).await {
            Ok(positions) => {
                println!(
                    "Found {} open positions for wallet {}:",
                    positions.len(),
                    wallet
                );
                for (i, position) in positions.iter().enumerate() {
                    println!(
                    "  {}. Position: {} | Custody: {} | Size: ${:.2} | Collateral: ${:.2} | Side: {:?}",
                    i + 1,
                    position.pubkey,
                    position.account.custody,
                    position.account.size_usd as f64 / 1_000_000.0,
                    position.account.collateral_usd as f64 / 1_000_000.0,
                    position.account.side
                );

                    // Calculate PnL (simplified)
                    let pnl = position
                        .account
                        .size_usd
                        .saturating_sub(position.account.collateral_usd);
                    println!("      Estimated PnL: ${:.2}", pnl as f64 / 1_000_000.0);
                }
            }
            Err(e) => {
                eprintln!("Failed to get positions for wallet: {e}");
            }
        }
    } else {
        println!("Invalid wallet pubkey format");
    }

    // Example 3: Get custody information
    println!("\n=== Custody Information ===");
    match client.get_all_custodies().await {
        Ok(custodies) => {
            println!("Found {} custody accounts:", custodies.len());
            for custody in custodies {
                println!(
                    "  Custody: {} | Mint: {} | Is Asset: {} | Total Amount: {} | Oracle Price: ${:.6}",
                    custody.pubkey,
                    custody.account.mint,
                    custody.account.is_asset,
                    custody.account.total_amount,
                    custody.account.oracle_price as f64 / 1_000_000.0
                );
            }
        }
        Err(e) => {
            eprintln!("Failed to get custodies: {e}");
        }
    }

    // Example 4: Generate position PDA for a potential new position
    println!("\n=== Generate Position PDA ===");
    let sol_custody = Pubkey::from_str("7xS2gz2bTp3fwCC7knJvUWTEU9Tycczu6VhJYKgi1wdz")?;
    let usdc_custody = Pubkey::from_str("G18jKKXQwBbrHeiK3C9MRXhkHsLHf7XgCSisykV46EZa")?;

    match client.generate_position_pda(&sol_custody, &usdc_custody, PositionSide::Long) {
        Ok((position_pda, bump)) => {
            println!(
                "Generated Long Position PDA: {position_pda} (bump: {bump})"
            );
        }
        Err(e) => {
            eprintln!("Failed to generate position PDA: {e}");
        }
    }

    match client.generate_position_pda(&sol_custody, &usdc_custody, PositionSide::Short) {
        Ok((position_pda, bump)) => {
            println!(
                "Generated Short Position PDA: {position_pda} (bump: {bump})"
            );
        }
        Err(e) => {
            eprintln!("Failed to generate position PDA: {e}");
        }
    }

    // Example 5: Get current SOL price
    println!("\n=== Market Data ===");
    match client.get_sol_price().await {
        Ok(price) => {
            println!("Current SOL price: ${price:.2}");

            // Calculate position sizes
            let usd_size = 1000 * 1_000_000; // $1000 in 6 decimal places
            let sol_amount = client.calculate_token_amount_from_usd(usd_size, price, 9);
            println!("For $1000 position, you need: {sol_amount} SOL");

            let reverse_usd = client.calculate_position_size_usd(sol_amount, price, 9);
            println!(
                "{} SOL equals: ${:.2}",
                sol_amount,
                reverse_usd as f64 / 1_000_000.0
            );
        }
        Err(e) => {
            eprintln!("Failed to get SOL price: {e}");
        }
    }

    Ok(())
}
