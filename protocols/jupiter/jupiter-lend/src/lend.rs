
use reqwest::{Client, Response};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use thiserror::Error;

use crate::serde_helpers;
use crate::transaction_config::TransactionConfig;

// --- Structs for Quote ---

#[derive(Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LendQuoteRequest {
    #[serde(with = "serde_helpers::field_as_string")]
    pub input_mint: Pubkey,
    #[serde(with = "serde_helpers::field_as_string")]
    pub output_mint: Pubkey,
    /// The amount to lend, in the smallest unit of the token.
    #[serde(with = "serde_helpers::field_as_string")]
    pub amount: u64,
    /// Slippage tolerance in basis points.
    pub slippage_bps: u16,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResponse {
    #[serde(with = "serde_helpers::field_as_string")]
    pub input_mint: Pubkey,
    #[serde(with = "serde_helpers::field_as_string")]
    pub in_amount: u64,
    #[serde(with = "serde_helpers::field_as_string")]
    pub output_mint: Pubkey,
    #[serde(with = "serde_helpers::field_as_string")]
    pub out_amount: u64,
    #[serde(default)]
    pub context_slot: u64,
    #[serde(default)]
    pub time_taken: f64,
    // Note: A real QuoteResponse from Jupiter has many more fields.
    // This is a simplified version for this example.
    // We need to add `routePlan` for the swap call to work.
    pub route_plan: serde_json::Value,
    #[serde(with = "serde_helpers::field_as_string")]
    pub other_amount_threshold: u64,
    pub swap_mode: String,
    pub slippage_bps: u16,
    pub price_impact_pct: String,
}

// --- Structs for Lend Transaction ---

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LendRequest {
    #[serde(with = "serde_helpers::field_as_string")]
    pub user_public_key: Pubkey,
    pub quote_response: QuoteResponse,
    #[serde(flatten)]
    pub config: TransactionConfig,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LendResponse {
    #[serde(with = "base64_serialize_deserialize")]
    pub lend_transaction: Vec<u8>,
    pub last_valid_block_height: u64,
}

pub mod base64_serialize_deserialize {
    use base64::{engine::general_purpose::STANDARD, Engine};
    use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(v: &[u8], s: S) -> Result<S::Ok, S::Error> {
        let base64_str = STANDARD.encode(v);
        String::serialize(&base64_str, s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let field_string = String::deserialize(deserializer)?;
        STANDARD
            .decode(field_string)
            .map_err(|e| de::Error::custom(format!("base64 decoding error: {e:?}")))
    }
}

// --- API Client ---

#[derive(Clone)]
pub struct JupiterLendApiClient {
    pub base_path: String,
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Request failed with status {status}: {body}")]
    RequestFailed {
        status: reqwest::StatusCode,
        body: String,
    },
    #[error("Failed to deserialize response: {0}")]
    DeserializationError(#[from] reqwest::Error),
}

async fn check_is_success(response: Response) -> Result<Response, ClientError> {
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(ClientError::RequestFailed { status, body });
    }
    Ok(response)
}

async fn check_status_code_and_deserialize<T: DeserializeOwned>(
    response: Response,
) -> Result<T, ClientError> {
    let response = check_is_success(response).await?;
    response
        .json::<T>()
        .await
        .map_err(ClientError::DeserializationError)
}

impl JupiterLendApiClient {
    pub fn new(base_path: String) -> Self {
        Self { base_path }
    }

    /// Gets a quote for a lending operation by treating it as a swap.
    pub async fn quote(
        &self,
        quote_request: &LendQuoteRequest,
    ) -> Result<QuoteResponse, ClientError> {
        let url = format!("{}/quote", self.base_path);
        let response = Client::new().get(url).query(quote_request).send().await?;
        check_status_code_and_deserialize(response).await
    }

    /// Gets a transaction for a lending operation.
    /// This is effectively a swap transaction from the input mint to the deposit receipt mint.
    pub async fn lend(&self, lend_request: &LendRequest) -> Result<LendResponse, ClientError> {
        let response = Client::new()
            .post(format!("{}/swap", self.base_path)) // Using the /swap endpoint
            .json(lend_request)
            .send()
            .await?;

        // The /swap endpoint returns a SwapResponse, so we deserialize that
        // and convert it to our LendResponse.
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct SwapResponse {
            #[serde(with = "base64_serialize_deserialize")]
            swap_transaction: Vec<u8>,
            last_valid_block_height: u64,
        }

        let swap_response: SwapResponse = check_status_code_and_deserialize(response).await?;
        Ok(LendResponse {
            lend_transaction: swap_response.swap_transaction,
            last_valid_block_height: swap_response.last_valid_block_height,
        })
    }
}
