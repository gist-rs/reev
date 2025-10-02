pub fn base_url() -> String {
    dotenvy::var("JUPITER_BASE_URL").unwrap_or_else(|_| "https://lite-api.jup.ag".to_string())
}

pub fn public_rpc_url() -> String {
    dotenvy::var("SOLANA_PUBLIC_RPC")
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}
