use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::post,
    Router,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod lend;
mod serde_helpers;
mod transaction_config;

use lend::{JupiterLendApiClient, LendQuoteRequest, LendRequest};
use transaction_config::TransactionConfig;

/// The shared state for our application, primarily holding the API client.
#[derive(Clone)]
struct AppState {
    jupiter_client: Arc<JupiterLendApiClient>,
}

/// Defines the structure of the JSON payload for a lend request.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct LendApiRequest {
    user_public_key: String,
    input_mint: String,
    amount: u64,
}

/// Defines the structure of a successful JSON response, containing the transaction.
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct LendApiResponse {
    lend_transaction: String, // base64 encoded transaction
    last_valid_block_height: u64,
}

#[tokio::main]
async fn main() {
    // Initialize the tracing subscriber for structured logging.
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "jupiter_lend=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // The base URL for the Jupiter API.
    // TODO: Make this configurable via environment variables.
    let api_base_url = "https://quote-api.jup.ag/v6".to_string();

    // Create a shared instance of the Jupiter Lend API client.
    let jupiter_client = Arc::new(JupiterLendApiClient::new(api_base_url));

    let app_state = AppState { jupiter_client };

    // Configure the application's routes.
    let app = Router::new()
        .route("/build-lend-transaction", post(build_lend_transaction))
        .with_state(app_state);

    // Start the server on localhost:3000.
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Server listening on {}", addr);
    let listener = TcpListener::bind(addr).await.unwrap();
    if let Err(e) = axum::serve(listener, app.into_make_service()).await {
        error!("Server failed: {}", e);
    }
}

/// The Axum handler for building a Jupiter lend transaction.
/// It takes a JSON request, calls the Jupiter API for a quote and then the transaction,
/// and returns the serialized transaction.
async fn build_lend_transaction(
    State(state): State<AppState>,
    Json(payload): Json<LendApiRequest>,
) -> impl IntoResponse {
    info!("Received request to build lend transaction: {:?}", payload);

    // --- 1. Parse and validate public keys from the request ---
    let user_public_key = match payload.user_public_key.parse::<Pubkey>() {
        Ok(key) => key,
        Err(e) => {
            error!("Invalid user_public_key format: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Invalid user_public_key" })),
            )
                .into_response();
        }
    };
    let input_mint = match payload.input_mint.parse::<Pubkey>() {
        Ok(key) => key,
        Err(e) => {
            error!("Invalid input_mint format: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Invalid input_mint" })),
            )
                .into_response();
        }
    };

    // --- 2. Get a lending quote from the Jupiter API ---
    let quote_request = LendQuoteRequest {
        amount: payload.amount,
        input_mint,
    };

    let quote_response = match state.jupiter_client.quote(&quote_request).await {
        Ok(response) => response,
        Err(e) => {
            error!("Failed to get Jupiter quote: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to get quote: {}", e) })),
            )
                .into_response();
        }
    };

    // --- 3. Build the final lend transaction ---
    let lend_request = LendRequest {
        user_public_key,
        quote_response,
        config: TransactionConfig::default(),
    };

    match state.jupiter_client.lend(&lend_request).await {
        Ok(lend_response) => {
            info!("Successfully built lend transaction.");
            let response = LendApiResponse {
                lend_transaction: STANDARD.encode(&lend_response.lend_transaction),
                last_valid_block_height: lend_response.last_valid_block_height,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to get Jupiter lend transaction: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to build transaction: {}", e) })),
            )
                .into_response()
        }
    }
}
