//! This module handles all HTTP interactions with the Jupiter Swap API.

use anyhow::Result;
use serde_json::{Value, json};

use crate::{api_client, config, models::SwapParams};
use solana_sdk::pubkey::Pubkey;

/// Fetches a swap quote from the Jupiter API.
pub async fn get_quote(params: &SwapParams) -> Result<Value> {
    let client = api_client::api_client();
    let quote_url = format!(
        "{}/swap/v1/quote?inputMint={}&outputMint={}&amount={}&slippageBps={}&onlyDirectRoutes=true",
        config::base_url(),
        params.input_mint,
        params.output_mint,
        params.amount,
        params.slippage_bps
    );

    let quote_response = client
        .get(&quote_url)
        .headers(api_client::json_headers())
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(quote_response)
}

/// Fetches the swap instructions from the Jupiter API using a quote.
pub async fn get_swap_instructions(
    user_public_key: &Pubkey,
    quote_response: &Value,
) -> Result<Value> {
    let client = api_client::api_client();

    let swap_request = json!({
        "userPublicKey": user_public_key.to_string(),
        "quoteResponse": quote_response,
        "prioritizationFeeLamports": {
            "priorityLevelWithMaxLamports": {
                "maxLamports": 10000000,
                "priorityLevel": "veryHigh"
            }
        },
        "dynamicComputeUnitLimit": true
    });

    let instructions_response = client
        .post(format!("{}/swap/v1/swap-instructions", config::base_url()))
        .headers(api_client::json_headers())
        .json(&swap_request)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(instructions_response)
}
