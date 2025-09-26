use jupiter_swap_api_client::{
    quote::QuoteRequest, swap::SwapRequest, transaction_config::TransactionConfig,
    JupiterSwapApiClient,
};
use reev_lib::agent::RawInstruction;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use thiserror::Error;

/// The arguments for the Jupiter swap tool, which will be provided by the AI model.
#[derive(Deserialize, Debug)]
pub struct JupiterSwapArgs {
    pub user_pubkey: String,
    pub input_mint: String,
    pub output_mint: String,
    pub amount: u64,
    pub slippage_bps: u16,
}

/// A custom error type for the Jupiter swap tool.
#[derive(Debug, Error)]
pub enum JupiterSwapError {
    #[error("Failed to parse pubkey: {0}")]
    PubkeyParse(String),
    #[error("Jupiter API call failed: {0}")]
    Api(String),
    #[error("Failed to serialize instruction: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// A `rig` tool for performing swaps using the Jupiter API.
#[derive(Deserialize, Serialize, Default)]
pub struct JupiterSwapTool;

impl Tool for JupiterSwapTool {
    const NAME: &'static str = "jupiter_swap";
    type Error = JupiterSwapError;
    type Args = JupiterSwapArgs;
    type Output = String; // The tool will return the raw instruction as a JSON string.

    /// Defines the tool's schema and description for the AI model.
    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Swap one token for another using the Jupiter aggregator. This finds the best price across many decentralized exchanges.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "user_pubkey": {
                        "type": "string",
                        "description": "The public key of the user's wallet performing the swap. This wallet must sign the transaction."
                    },
                    "input_mint": {
                        "type": "string",
                        "description": "The mint address of the token to be swapped FROM. For native SOL, use 'So11111111111111111111111111111111111111112'."
                    },
                    "output_mint": {
                        "type": "string",
                        "description": "The mint address of the token to be swapped TO. For native SOL, use 'So11111111111111111111111111111111111111112'."
                    },
                    "amount": {
                        "type": "number",
                        "description": "The amount of the input token to swap, in its smallest denomination (e.g., lamports for SOL)."
                    },
                    "slippage_bps": {
                        "type": "number",
                        "description": "The slippage tolerance in basis points (e.g., 50 for 0.5%)."
                    }
                },
                "required": ["user_pubkey", "input_mint", "output_mint", "amount", "slippage_bps"],
            }),
        }
    }

    /// Executes the tool's logic: calls the Jupiter API and serializes the resulting instruction.
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let user_pubkey = Pubkey::from_str(&args.user_pubkey)
            .map_err(|e| JupiterSwapError::PubkeyParse(e.to_string()))?;
        let input_mint = Pubkey::from_str(&args.input_mint)
            .map_err(|e| JupiterSwapError::PubkeyParse(e.to_string()))?;
        let output_mint = Pubkey::from_str(&args.output_mint)
            .map_err(|e| JupiterSwapError::PubkeyParse(e.to_string()))?;

        let jupiter_client = JupiterSwapApiClient::new("https://quote-api.jup.ag/v6".to_string());

        let quote_request = QuoteRequest {
            amount: args.amount,
            input_mint,
            output_mint,
            slippage_bps: args.slippage_bps,
            ..Default::default()
        };

        let quote_response = jupiter_client
            .quote(&quote_request)
            .await
            .map_err(|e| JupiterSwapError::Api(e.to_string()))?;

        let swap_instructions = jupiter_client
            .swap_instructions(&SwapRequest {
                user_public_key: user_pubkey,
                quote_response,
                config: TransactionConfig::default(),
            })
            .await
            .map_err(|e| JupiterSwapError::Api(e.to_string()))?;

        let raw_instruction: RawInstruction = swap_instructions.swap_instruction.into();

        let output = serde_json::to_string(&raw_instruction)?;

        Ok(output)
    }
}
