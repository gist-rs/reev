//! This module handles all HTTP interactions with the Jupiter Tokens API.

use anyhow::Result;

use crate::{
    api_client, config,
    models::{TokenInfo, TokenSearchParams},
};

/// Searches for tokens using the Jupiter Tokens API.
///
/// # Arguments
///
/// * `params` - The search parameters including the query string
///
/// # Returns
///
/// Returns a vector of `TokenInfo` matching the search query
///
/// # Example
///
/// ```no_run
/// use jup_sdk::{api::tokens::search_tokens, models::TokenSearchParams};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let params = TokenSearchParams {
///         query: "So11111111111111111111111111111111111111112,EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
///     };
///
///     let tokens = search_tokens(&params).await?;
///     println!("Found {} tokens", tokens.len());
///
///     Ok(())
/// }
/// ```
pub async fn search_tokens(params: &TokenSearchParams) -> Result<Vec<TokenInfo>> {
    let client = api_client::api_client();
    let search_url = format!(
        "{}/tokens/v2/search?query={}",
        config::base_url(),
        urlencoding::encode(&params.query)
    );

    let search_response = client
        .get(&search_url)
        .headers(api_client::json_headers())
        .send()
        .await?
        .error_for_status()?
        .json::<Vec<TokenInfo>>()
        .await?;

    Ok(search_response)
}
