use anyhow::Result;
use env_logger::Env;
use jupiter_perps_rs::{CreatePositionRequestParams, JupiterPerpsClient, PositionSide};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signer;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // Initialize client from environment variables
    let client = JupiterPerpsClient::from_env()?;

    println!("=== Opening Jupiter Perps Position ===");

    // Get current market data
    let sol_price = client.get_sol_price().await?;
    println!("Current SOL price: ${sol_price:.2}");

    // Position parameters
    let position_size_usd = 1000.0; // $1000 position size
    let collateral_amount_sol = 0.01; // 0.01 SOL as collateral
    let slippage_percent = 5.0; // 5% slippage tolerance

    // Get custody accounts
    let sol_custody = Pubkey::from_str("7xS2gz2bTp3fwCC7knJvUWTEU9Tycczu6VhJYKgi1wdz")?;
    let usdc_custody = Pubkey::from_str("G18jKKXQwBbrHeiK3C9MRXhkHsLHf7XgCSisykV46EZa")?;
    let sol_mint = Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();

    // Generate position PDA
    let (position_pubkey, _bump) =
        client.generate_position_pda(&sol_custody, &usdc_custody, PositionSide::Long)?;
    println!("Generated Position PDA: {position_pubkey}");

    // Calculate amounts in proper units
    let size_usd_delta = (position_size_usd * 1_000_000.0) as u64; // 6 decimal places for USD
    let collateral_token_delta = (collateral_amount_sol * 10_f64.powi(9)) as u64; // 9 decimals for SOL
    let price_slippage = (sol_price * 1_000_000.0 * (1.0 - slippage_percent / 100.0)) as u64;

    // Example 1: Open Long Position with SOL collateral
    println!("\n=== Example 1: Open Long Position ===");
    let long_params = CreatePositionRequestParams {
        custody: sol_custody,
        collateral_custody: usdc_custody,
        collateral_token_delta,
        input_mint: sol_mint,
        jupiter_minimum_out: None, // No swap needed if using USDC as collateral
        owner: client.get_keypair().unwrap().pubkey(),
        price_slippage,
        side: PositionSide::Long,
        size_usd_delta,
        position_pubkey,
    };

    match client
        .create_market_open_position_request(long_params)
        .await
    {
        Ok(_transaction) => {
            println!("✓ Successfully created open long position transaction");
            println!("Transaction ready to be signed and submitted");

            // Uncomment to actually sign and submit:
            // match client.sign_and_submit_transaction(transaction).await {
            //     Ok(signature) => {
            //         println!("✓ Transaction submitted: {}", signature);
            //         println!("View on Solscan: https://solscan.io/tx/{}", signature);
            //     }
            //     Err(e) => {
            //         eprintln!("✗ Failed to submit transaction: {}", e);
            //     }
            // }
        }
        Err(e) => {
            eprintln!("✗ Failed to create open position transaction: {e}");
        }
    }

    // Example 2: Open Short Position with USDC collateral
    println!("\n=== Example 2: Open Short Position ===");
    let collateral_amount_usdc = 500.0; // $500 USDC collateral
    let collateral_token_delta_usdc = (collateral_amount_usdc * 10_f64.powi(6)) as u64; // 6 decimals for USDC
    let usdc_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")?; // USDC mint

    let short_params = CreatePositionRequestParams {
        custody: sol_custody,
        collateral_custody: usdc_custody,
        collateral_token_delta: collateral_token_delta_usdc,
        input_mint: usdc_mint,
        jupiter_minimum_out: None,
        owner: client.get_keypair().unwrap().pubkey(),
        price_slippage: (sol_price * 1_000_000.0 * (1.0 + slippage_percent / 100.0)) as u64, // Higher slippage for shorts
        side: PositionSide::Short,
        size_usd_delta,
        position_pubkey,
    };

    match client
        .create_market_open_position_request(short_params)
        .await
    {
        Ok(_transaction) => {
            println!("✓ Successfully created open short position transaction");
            println!("Transaction ready to be signed and submitted");

            // Uncomment to actually sign and submit:
            // match client.sign_and_submit_transaction(transaction).await {
            //     Ok(signature) => {
            //         println!("✓ Transaction submitted: {}", signature);
            //         println!("View on Solscan: https://solscan.io/tx/{}", signature);
            //     }
            //     Err(e) => {
            //         eprintln!("✗ Failed to submit transaction: {}", e);
            //     }
            // }
        }
        Err(e) => {
            eprintln!("✗ Failed to create open short position transaction: {e}");
        }
    }

    // Example 3: Open Position with Token Swap (USDC -> SOL)
    println!("\n=== Example 3: Open Position with Token Swap ===");

    // Simulate getting Jupiter quote for USDC -> SOL swap
    let usdc_amount_for_swap = 200.0; // $200 USDC to swap for SOL
    let usdc_swap_amount = (usdc_amount_for_swap * 10_f64.powi(6)) as u64;

    // In a real implementation, you would call Jupiter Quote API:
    // let quote = jupiter_quote_api.get_quote(
    //     input_mint: usdc_mint,
    //     output_mint: sol_mint,
    //     amount: usdc_swap_amount,
    //     slippage: 5
    // );
    // let jupiter_minimum_out = quote.out_amount;

    // For this example, we'll use a placeholder
    let estimated_sol_out = usdc_amount_for_swap / sol_price * 10_f64.powi(9);
    let jupiter_minimum_out = (estimated_sol_out * 0.95) as u64; // 5% slippage

    let swap_params = CreatePositionRequestParams {
        custody: sol_custody,
        collateral_custody: usdc_custody,
        collateral_token_delta: usdc_swap_amount,
        input_mint: usdc_mint,
        jupiter_minimum_out: Some(jupiter_minimum_out),
        owner: client.get_keypair().unwrap().pubkey(),
        price_slippage,
        side: PositionSide::Long,
        size_usd_delta,
        position_pubkey,
    };

    match client
        .create_market_open_position_request(swap_params)
        .await
    {
        Ok(_transaction) => {
            println!("✓ Successfully created open position with swap transaction");
            println!("Input: {usdc_amount_for_swap} USDC");
            println!(
                "Expected SOL output: {:.6}",
                jupiter_minimum_out as f64 / 10_f64.powi(9)
            );
            println!("Transaction ready to be signed and submitted");
        }
        Err(e) => {
            eprintln!(
                "✗ Failed to create open position with swap transaction: {e}"
            );
        }
    }

    // Example 4: Calculate position metrics
    println!("\n=== Position Metrics ===");
    let leverage = size_usd_delta as f64 / 1_000_000.0 / collateral_amount_sol / sol_price;
    println!("Position size: ${position_size_usd:.2}");
    println!(
        "Collateral: {:.6} SOL (${:.2})",
        collateral_amount_sol,
        collateral_amount_sol * sol_price
    );
    println!("Leverage: {leverage:.2}x");

    // Estimate fees (simplified)
    let open_fee_bps = 10; // 0.1% opening fee
    let estimated_open_fee = position_size_usd * open_fee_bps as f64 / 10_000.0;
    println!("Estimated opening fee: ${estimated_open_fee:.4}");

    println!("\n=== Notes ===");
    println!("1. All transactions are created but not submitted by default");
    println!("2. Uncomment the sign_and_submit_transaction code to execute trades");
    println!("3. Make sure your wallet has sufficient SOL for gas fees");
    println!("4. Make sure your wallet has sufficient collateral tokens");
    println!("5. Jupiter swap integration requires calling Jupiter Quote API first");
    println!("6. Position requests are fulfilled by keepers, not immediately");

    Ok(())
}
