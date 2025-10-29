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
