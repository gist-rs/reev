use std::env;

use anyhow::Result;
use jupiter_swap_api_client::{
    quote::QuoteRequest, swap::SwapRequest, transaction_config::TransactionConfig,
    JupiterSwapApiClient,
};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{pubkey, transaction::VersionedTransaction};
use solana_sdk::{pubkey::Pubkey, signature::NullSigner};
use tracing::info;

const USDC_MINT: Pubkey = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const NATIVE_MINT: Pubkey = pubkey!("So11111111111111111111111111111111111111112");

pub const TEST_WALLET: Pubkey = pubkey!("2AQdpHJ2JpcEgPiATUXjQxA8QmafFegfQwSLWSprPicm"); // Coinbase 2 wallet

pub async fn run_fast_check() -> Result<()> {
    let api_base_url = env::var("API_BASE_URL").unwrap_or("https://quote-api.jup.ag/v6".into());
    info!("Using base url: {api_base_url}");

    let jupiter_swap_api_client = JupiterSwapApiClient::new(api_base_url);

    let quote_request = QuoteRequest {
        amount: 1_000_000,
        input_mint: USDC_MINT,
        output_mint: NATIVE_MINT,
        slippage_bps: 50,
        ..QuoteRequest::default()
    };

    // GET /quote
    info!("Getting quote...");
    let quote_response = jupiter_swap_api_client.quote(&quote_request).await?;
    info!("Got quote.");

    // POST /swap
    info!("Getting swap transaction...");
    let swap_response = jupiter_swap_api_client
        .swap(
            &SwapRequest {
                user_public_key: TEST_WALLET,
                quote_response,
                config: TransactionConfig::default(),
            },
            None,
        )
        .await?;

    info!("Raw tx len: {}", swap_response.swap_transaction.len());

    let versioned_transaction: VersionedTransaction =
        bincode::deserialize(&swap_response.swap_transaction)?;

    // Replace with a keypair or other struct implementing signer
    let null_signer = NullSigner::new(&TEST_WALLET);
    let signed_versioned_transaction =
        VersionedTransaction::try_new(versioned_transaction.message, &[&null_signer])?;

    info!("Sending transaction with NullSigner, expecting signature verification failure...");
    let rpc_client = RpcClient::new("https://api.mainnet-beta.solana.com".into());

    // This is expected to fail with "Transaction signature verification failure"
    // because we signed with a NullSigner. This is the success condition for this test.
    let error = rpc_client
        .send_and_confirm_transaction(&signed_versioned_transaction)
        .await
        .unwrap_err();

    let error_string = error.to_string();

    if error_string.contains("Transaction signature verification failure") {
        info!("✅ Successfully received expected error: {error_string}");
    } else {
        anyhow::bail!(
            "Expected 'Transaction signature verification failure', but got: {error_string}"
        );
    }

    Ok(())
}
