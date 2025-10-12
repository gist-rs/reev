use anyhow::Result;
use env_logger::Env;
use jupiter_perps_rs::{ClosePositionRequestParams, JupiterPerpsClient};
use solana_sdk::pubkey::Pubkey;

use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // Initialize client from environment variables
    let client = JupiterPerpsClient::from_env()?;

    println!("=== Closing Jupiter Perps Position ===");

    // Get current market data
    let sol_price = client.get_sol_price().await?;
    println!("Current SOL price: ${sol_price:.2}");

    // Example position to close (replace with actual position pubkey)
    let position_pubkey_str = std::env::var("POSITION_PUBKEY")
        .unwrap_or_else(|_| "YOUR_POSITION_PUBKEY_HERE".to_string());

    if position_pubkey_str == "YOUR_POSITION_PUBKEY_HERE" {
        println!("⚠️  Please set POSITION_PUBKEY environment variable to an actual position");
        println!("   You can get position pubkeys from the get_positions example");

        // For demo purposes, we'll continue with a placeholder
        println!("   Using placeholder for demonstration...\n");
    }

    let position_pubkey = match Pubkey::from_str(&position_pubkey_str) {
        Ok(pubkey) => pubkey,
        Err(_) => {
            println!("Invalid position pubkey format, using placeholder for demo");
            Pubkey::new_unique()
        }
    };

    let slippage_percent = 5.0; // 5% slippage tolerance

    // Example 1: Close Long Position and receive SOL
    println!("=== Example 1: Close Long Position (Receive SOL) ===");
    let sol_mint = Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();
    let price_slippage_sol = (sol_price * 1_000_000.0 * (1.0 - slippage_percent / 100.0)) as u64;

    let close_sol_params = ClosePositionRequestParams {
        position_pubkey,
        desired_mint: sol_mint,
        price_slippage: price_slippage_sol,
    };

    match client
        .create_market_close_position_request(close_sol_params)
        .await
    {
        Ok(_transaction) => {
            println!("✓ Successfully created close position transaction (receive SOL)");
            println!("Transaction ready to be signed and submitted");

            // Uncomment to actually sign and submit:
            // match client.sign_and_submit_transaction(transaction).await {
            //     Ok(signature) => {
            //         println!("✓ Transaction submitted: {}", signature);
            //         println!("View on Solscan: https://solscan.io/tx/{}", signature);
            //
            //         // Wait for keeper to fulfill the request
            //         println!("⏳ Waiting for keeper to fulfill position request...");
            //         tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            //
            //         // Check if position is closed
            //         match client.get_open_positions_for_wallet(&client.keypair.pubkey()).await {
            //             Ok(positions) => {
            //                 let is_closed = !positions.iter().any(|p| p.pubkey == position_pubkey);
            //                 if is_closed {
            //                     println!("✓ Position successfully closed");
            //                 } else {
            //                     println!("⏳ Position request still pending fulfillment");
            //                 }
            //             }
            //             Err(e) => {
            //                 eprintln!("Failed to check position status: {}", e);
            //             }
            //         }
            //     }
            //     Err(e) => {
            //         eprintln!("✗ Failed to submit transaction: {}", e);
            //     }
            // }
        }
        Err(e) => {
            eprintln!("✗ Failed to create close position transaction: {e}");
        }
    }

    // Example 2: Close Position and receive USDC
    println!("\n=== Example 2: Close Position (Receive USDC) ===");
    let usdc_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")?; // USDC mint
    let price_slippage_usdc = (sol_price * 1_000_000.0) as u64; // No slippage adjustment needed for stable coin

    let close_usdc_params = ClosePositionRequestParams {
        position_pubkey,
        desired_mint: usdc_mint,
        price_slippage: price_slippage_usdc,
    };

    match client
        .create_market_close_position_request(close_usdc_params)
        .await
    {
        Ok(_transaction) => {
            println!("✓ Successfully created close position transaction (receive USDC)");
            println!("Transaction ready to be signed and submitted");
        }
        Err(e) => {
            eprintln!("✗ Failed to create close position transaction: {e}");
        }
    }

    // Example 3: Partial Position Close (if supported)
    println!("\n=== Example 3: Partial Position Close ===");

    // For partial closes, you would set collateral_usd_delta and size_usd_delta
    // This is a simplified example - in practice you'd need to implement partial close logic
    println!("Note: Partial position closes require additional implementation");
    println!("The current implementation closes the entire position");

    // Example 4: Close Position with Stop Loss or Take Profit
    println!("\n=== Example 4: Stop Loss / Take Profit ===");

    let entry_price = 95.0; // Example entry price
    let current_price = sol_price;

    if current_price <= entry_price * 0.95 {
        println!(
            "🛑 Stop Loss Triggered: Price is {:.2}% below entry price",
            ((entry_price - current_price) / entry_price * 100.0)
        );

        // Create stop loss close request
        let stop_loss_params = ClosePositionRequestParams {
            position_pubkey,
            desired_mint: usdc_mint,
            price_slippage: (current_price * 1_000_000.0 * 0.98) as u64, // Tighter slippage for stop loss
        };

        match client
            .create_market_close_position_request(stop_loss_params)
            .await
        {
            Ok(_transaction) => {
                println!("✓ Stop loss transaction created");
                println!("Transaction ready to be signed and submitted");
            }
            Err(e) => {
                eprintln!("✗ Failed to create stop loss transaction: {e}");
            }
        }
    } else if current_price >= entry_price * 1.20 {
        println!(
            "🎯 Take Profit Triggered: Price is {:.2}% above entry price",
            ((current_price - entry_price) / entry_price * 100.0)
        );

        // Create take profit close request
        let take_profit_params = ClosePositionRequestParams {
            position_pubkey,
            desired_mint: sol_mint,
            price_slippage: (current_price * 1_000_000.0 * 0.99) as u64, // Tighter slippage for take profit
        };

        match client
            .create_market_close_position_request(take_profit_params)
            .await
        {
            Ok(_transaction) => {
                println!("✓ Take profit transaction created");
                println!("Transaction ready to be signed and submitted");
            }
            Err(e) => {
                eprintln!("✗ Failed to create take profit transaction: {e}");
            }
        }
    } else {
        println!(
            "Current price: ${current_price:.2} (Entry: ${entry_price:.2})"
        );
        println!("No stop loss or take profit triggered");
    }

    // Example 5: Calculate closing metrics
    println!("\n=== Closing Metrics ===");

    // Simulate position PnL calculation (simplified)
    let position_size_usd = 1000.0; // Example position size
    let collateral_usd = 100.0; // Example collateral
    let unrealized_pnl = if current_price > entry_price {
        (current_price - entry_price) / entry_price * position_size_usd
    } else {
        -((entry_price - current_price) / entry_price * position_size_usd)
    };

    println!("Position size: ${position_size_usd:.2}");
    println!("Collateral: ${collateral_usd:.2}");
    println!("Entry price: ${entry_price:.2}");
    println!("Current price: ${current_price:.2}");
    println!("Unrealized PnL: ${unrealized_pnl:.2}");
    println!("ROI: {:.2}%", (unrealized_pnl / collateral_usd) * 100.0);

    // Estimate closing fees
    let close_fee_bps = 10; // 0.1% closing fee
    let estimated_close_fee = position_size_usd * close_fee_bps as f64 / 10_000.0;
    println!("Estimated closing fee: ${estimated_close_fee:.4}");

    let net_pnl = unrealized_pnl - estimated_close_fee;
    println!("Net PnL after fees: ${net_pnl:.2}");

    // Example 6: Emergency Close (high slippage)
    println!("\n=== Example 6: Emergency Close ===");
    println!("In volatile market conditions, you might want to use higher slippage");

    let emergency_slippage = 15.0; // 15% slippage for emergency close
    let emergency_price_slippage = if current_price > entry_price {
        (current_price * 1_000_000.0 * (1.0 - emergency_slippage / 100.0)) as u64
    } else {
        (current_price * 1_000_000.0 * (1.0 + emergency_slippage / 100.0)) as u64
    };

    let emergency_params = ClosePositionRequestParams {
        position_pubkey,
        desired_mint: usdc_mint, // Emergency close to stable coin
        price_slippage: emergency_price_slippage,
    };

    match client
        .create_market_close_position_request(emergency_params)
        .await
    {
        Ok(_transaction) => {
            println!(
                "✓ Emergency close transaction created with {emergency_slippage:.0}% slippage"
            );
            println!("⚠️  High slippage may result in poor execution price");
        }
        Err(e) => {
            eprintln!("✗ Failed to create emergency close transaction: {e}");
        }
    }

    println!("\n=== Important Notes ===");
    println!("1. Position close requests are fulfilled by keepers, not immediately");
    println!("2. Monitor your transaction after submission for fulfillment");
    println!("3. Set appropriate slippage based on market conditions");
    println!("4. Consider gas fees when closing small positions");
    println!("5. Stop loss and take profit should be automated in production");
    println!("6. Always verify position is closed after fulfillment");
    println!("7. Keep some SOL in your wallet for gas fees during closes");

    Ok(())
}
