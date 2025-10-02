//! This module handles all HTTP interactions with the Jupiter Lend API.

use anyhow::Result;
use solana_sdk::pubkey::Pubkey;

use crate::{
    api_client, config,
    models::{ApiResponse, DepositParams, WithdrawParams},
};

/// Fetches deposit instructions from the Jupiter Lend API.
pub async fn get_deposit_instructions(
    user_public_key: &Pubkey,
    params: &DepositParams,
) -> Result<ApiResponse> {
    let client = api_client::api_client();
    let data = serde_json::json!({
        "asset": params.asset_mint.to_string(),
        "signer": user_public_key.to_string(),
        "amount": params.amount.to_string(),
        "cluster": "mainnet"
    });

    let response = client
        .post(format!(
            "{}/lend/v1/earn/deposit-instructions",
            config::base_url()
        ))
        .headers(api_client::json_headers())
        .json(&data)
        .send()
        .await?
        .error_for_status()?
        .json::<ApiResponse>()
        .await?;

    Ok(response)
}

/// Fetches withdraw instructions from the Jupiter Lend API.
pub async fn get_withdraw_instructions(
    user_public_key: &Pubkey,
    params: &WithdrawParams,
) -> Result<ApiResponse> {
    let client = api_client::api_client();
    let data = serde_json::json!({
        "asset": params.asset_mint.to_string(),
        "signer": user_public_key.to_string(),
        "amount": params.amount.to_string(),
        "cluster": "mainnet"
    });

    let response = client
        .post(format!(
            "{}/lend/v1/earn/withdraw-instructions",
            config::base_url()
        ))
        .headers(api_client::json_headers())
        .json(&data)
        .send()
        .await?
        .error_for_status()?
        .json::<ApiResponse>()
        .await?;

    Ok(response)
}
