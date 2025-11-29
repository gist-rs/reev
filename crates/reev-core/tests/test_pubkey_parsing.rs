//! Test pubkey parsing to understand the "String is the wrong size" error

use reev_lib::get_keypair;
use rig::tool::Tool;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signer;
use std::str::FromStr;

#[tokio::test(flavor = "multi_thread")]
async fn test_pubkey_parsing() {
    println!("Starting pubkey parsing test");

    // Get a test keypair
    let keypair = get_keypair().expect("Failed to get keypair");
    let pubkey = keypair.pubkey();
    let pubkey_str = pubkey.to_string();

    println!("Original pubkey: {:?}", pubkey);
    println!(
        "Pubkey string: '{}' (length: {})",
        pubkey_str,
        pubkey_str.len()
    );

    // Try to parse the string back to a pubkey
    match Pubkey::from_str(&pubkey_str) {
        Ok(parsed_pubkey) => {
            println!("✅ Successfully parsed pubkey: {:?}", parsed_pubkey);
            println!("✅ Pubkeys match: {}", pubkey == parsed_pubkey);
        }
        Err(e) => {
            println!("❌ Failed to parse pubkey: {}", e);
        }
    }

    // Try to parse a hardcoded known good pubkey
    let known_good = "11111111111111111111111111111112";
    println!(
        "Known good pubkey: '{}' (length: {})",
        known_good,
        known_good.len()
    );

    match Pubkey::from_str(known_good) {
        Ok(parsed_pubkey) => {
            println!(
                "✅ Successfully parsed known good pubkey: {:?}",
                parsed_pubkey
            );
        }
        Err(e) => {
            println!("❌ Failed to parse known good pubkey: {}", e);
        }
    }

    // Try to parse the pubkey from the failing test
    let test_pubkey = "3F42CLVYyxuMYNTBRKuCQ6o3XnzPky6raWTPHtW8myLr";
    println!(
        "Test pubkey: '{}' (length: {})",
        test_pubkey,
        test_pubkey.len()
    );

    match Pubkey::from_str(test_pubkey) {
        Ok(parsed_pubkey) => {
            println!("✅ Successfully parsed test pubkey: {:?}", parsed_pubkey);
        }
        Err(e) => {
            println!("❌ Failed to parse test pubkey: {}", e);
        }
    }

    // Test creating JupiterSwapArgs with the test pubkey
    println!("Creating JupiterSwapArgs with test pubkey...");
    let swap_args = reev_tools::tools::jupiter_swap::JupiterSwapArgs {
        user_pubkey: test_pubkey.to_string(),
        input_mint: "So11111111111111111111111111111111111111112".to_string(),
        output_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        amount: 100_000_000, // 0.1 SOL
        slippage_bps: Some(100),
    };

    println!(
        "JupiterSwapArgs created successfully: {:?}",
        swap_args.user_pubkey
    );

    // Test calling JupiterSwapTool with the args
    println!("Creating JupiterSwapTool...");
    let key_map = std::collections::HashMap::new();
    let jupiter_swap_tool = reev_tools::tools::jupiter_swap::JupiterSwapTool { key_map };

    println!("Calling JupiterSwapTool with tokio::task::spawn_blocking...");
    let result = tokio::task::spawn_blocking(move || {
        // Use the blocking runtime to call the tool
        tokio::runtime::Handle::current().block_on(jupiter_swap_tool.call(swap_args))
    })
    .await
    .unwrap();

    match result {
        Ok(tool_result) => {
            println!("✅ JupiterSwapTool call successful: {}", tool_result);
        }
        Err(e) => {
            println!("❌ JupiterSwapTool call failed: {}", e);
        }
    }
}
