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
    /// The amount to lend, in the smallest unit of the token.
    #[serde(with = "serde_helpers::field_as_string")]
    pub amount: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResponse {
    #[serde(with = "serde_helpers::field_as_string")]
    pub input_mint: Pubkey,
    #[serde(with = "serde_helpers::field_as_string")]
    pub in_amount: u64,
    #[serde(with = "serde_helpers::field_as_string")]
    pub output_mint: Pubkey, // This would be the "receipt" or "lending" token
    #[serde(with = "serde_helpers::field_as_string")]
    pub out_amount: u64,
    #[serde(default)]
    pub context_slot: u64,
    #[serde(default)]
    pub time_taken: f64,
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

    /// Gets a quote for a lending operation
    pub async fn quote(
        &self,
        quote_request: &LendQuoteRequest,
    ) -> Result<QuoteResponse, ClientError> {
        let url = format!("{}/quote", self.base_path);
        let response = Client::new().get(url).query(quote_request).send().await?;
        check_status_code_and_deserialize(response).await
    }

    /// Gets a transaction for a lending operation
    pub async fn lend(&self, lend_request: &LendRequest) -> Result<LendResponse, ClientError> {
        let response = Client::new()
            .post(format!("{}/lend-transaction", self.base_path))
            .json(lend_request)
            .send()
            .await?;
        check_status_code_and_deserialize(response).await
    }
}
