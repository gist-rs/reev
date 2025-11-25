//! YML Generator for Phase 1 of V3 Plan
//!
//! This module implements the rule-based YML generation component in Phase 1 of the V3 plan.
//! It uses refined prompts from the LanguageRefiner to generate structured YML flows with
//! appropriate expected_tools hints for the rig agent.

use anyhow::{anyhow, Result};
use reev_types::flow::WalletContext;
use tracing::{debug, info, instrument};
use uuid::Uuid;

use crate::refiner::RefinedPrompt;
use crate::yml_schema::{
    YmlAssertion, YmlFlow, YmlGroundTruth, YmlStep, YmlToolCall, YmlWalletInfo,
};
use reev_types::tools::ToolName;

/// YML generator for creating structured flows from refined prompts
pub struct YmlGenerator {
    /// Default error tolerance for validation
    default_error_tolerance: f64,
}

impl Default for YmlGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl YmlGenerator {
    /// Create a new YML generator
    pub fn new() -> Self {
        Self {
            default_error_tolerance: 0.01, // 1% default tolerance
        }
    }

    /// Create a YML generator with custom error tolerance
    pub fn with_error_tolerance(error_tolerance: f64) -> Self {
        Self {
            default_error_tolerance: error_tolerance,
        }
    }

    /// Generate a YML flow from a refined prompt and wallet context
    #[instrument(skip(self, refined_prompt, wallet_context))]
    pub async fn generate_flow(
        &self,
        refined_prompt: &RefinedPrompt,
        wallet_context: &WalletContext,
    ) -> Result<YmlFlow> {
        info!(
            "Generating YML flow from refined prompt: {}",
            refined_prompt.refined
        );

        // Parse the refined prompt to determine operation type
        let operation_type = self.parse_operation_type(&refined_prompt.refined)?;
        debug!("Detected operation type: {:?}", operation_type);

        // Generate flow based on operation type
        let flow = match operation_type {
            OperationType::Swap(params) => {
                self.generate_swap_flow(refined_prompt, wallet_context, params)
                    .await?
            }
            OperationType::Transfer(params) => {
                self.generate_transfer_flow(refined_prompt, wallet_context, params)
                    .await?
            }
            OperationType::Lend(params) => {
                self.generate_lend_flow(refined_prompt, wallet_context, params)
                    .await?
            }
            OperationType::SwapThenLend(params) => {
                self.generate_swap_then_lend_flow(refined_prompt, wallet_context, params)
                    .await?
            }
            OperationType::Unknown => {
                return Err(anyhow!(
                    "Unable to determine operation type from prompt: {}",
                    refined_prompt.refined
                ));
            }
        };

        info!("Generated YML flow with ID: {}", flow.flow_id);
        Ok(flow)
    }

    /// Parse operation type from refined prompt
    fn parse_operation_type(&self, refined_prompt: &str) -> Result<OperationType> {
        let prompt_lower = refined_prompt.to_lowercase();

        // Check for swap operations
        if prompt_lower.contains("swap") || prompt_lower.contains("exchange") {
            if let Ok(params) = self.parse_swap_params(refined_prompt) {
                return Ok(OperationType::Swap(params));
            }
        }

        // Check for transfer operations
        if prompt_lower.contains("transfer") || prompt_lower.contains("send") {
            if let Ok(params) = self.parse_transfer_params(refined_prompt) {
                return Ok(OperationType::Transfer(params));
            }
        }

        // Check for lend operations
        if prompt_lower.contains("lend") || prompt_lower.contains("deposit") {
            if let Ok(params) = self.parse_lend_params(refined_prompt) {
                return Ok(OperationType::Lend(params));
            }
        }

        // Check for swap then lend operations
        if (prompt_lower.contains("swap") && prompt_lower.contains("lend"))
            || (prompt_lower.contains("exchange") && prompt_lower.contains("deposit"))
        {
            if let Ok(params) = self.parse_swap_then_lend_params(refined_prompt) {
                return Ok(OperationType::SwapThenLend(params));
            }
        }

        Ok(OperationType::Unknown)
    }

    /// Generate a swap flow
    async fn generate_swap_flow(
        &self,
        refined_prompt: &RefinedPrompt,
        wallet_context: &WalletContext,
        params: SwapParams,
    ) -> Result<YmlFlow> {
        let flow_id = Uuid::now_v7().to_string();

        // Create wallet info
        let mut wallet_info =
            YmlWalletInfo::new(wallet_context.owner.clone(), wallet_context.sol_balance)
                .with_total_value(wallet_context.total_value_usd);

        // Add tokens if they exist
        for token in wallet_context.token_balances.values() {
            wallet_info = wallet_info.with_token(token.clone());
        }

        // Create swap step with expected_tools hint
        let mut step = YmlStep::new(
            "swap".to_string(),
            refined_prompt.original.clone(),
            refined_prompt.refined.clone(),
        );
        step.context = format!(
            "Exchange {} {} for {}",
            params.amount, params.from_token, params.to_token
        );
        let step = step
            .with_expected_tools(vec![ToolName::JupiterSwap])
            .with_tool_call(YmlToolCall::new(ToolName::JupiterSwap, true));

        // Create ground truth
        let ground_truth = YmlGroundTruth::new()
            .with_error_tolerance(self.default_error_tolerance)
            .with_assertion(
                YmlAssertion::new("TokenBalanceChange".to_string())
                    .with_pubkey(wallet_context.owner.clone())
                    .with_expected_change_lte(-(params.amount + 0.1) * 1_000_000_000.0), // Account for fees
            )
            .with_tool_call(YmlToolCall::new(ToolName::JupiterSwap, true));

        // Create flow
        let flow = YmlFlow::new(flow_id, refined_prompt.original.clone(), wallet_info)
            .with_refined_prompt(refined_prompt.refined.clone())
            .with_step(step)
            .with_ground_truth(ground_truth)
            .with_metadata(
                crate::yml_schema::FlowMetadata::new()
                    .with_category("swap".to_string())
                    .with_tag("jupiter".to_string())
                    .with_complexity(2),
            );

        Ok(flow)
    }

    /// Generate a transfer flow
    async fn generate_transfer_flow(
        &self,
        refined_prompt: &RefinedPrompt,
        wallet_context: &WalletContext,
        params: TransferParams,
    ) -> Result<YmlFlow> {
        let flow_id = Uuid::now_v7().to_string();

        // Create wallet info
        let mut wallet_info =
            YmlWalletInfo::new(wallet_context.owner.clone(), wallet_context.sol_balance)
                .with_total_value(wallet_context.total_value_usd);

        // Add tokens if they exist
        for token in wallet_context.token_balances.values() {
            wallet_info = wallet_info.with_token(token.clone());
        }

        // Create transfer step with expected_tools hint
        let mut step = YmlStep::new(
            "transfer".to_string(),
            refined_prompt.original.clone(),
            refined_prompt.refined.clone(),
        );
        step.context = format!("Transfer {} SOL to {}", params.amount, params.recipient);
        let step = step
            .with_expected_tools(vec![ToolName::SolTransfer])
            .with_tool_call(YmlToolCall::new(ToolName::SolTransfer, true));

        // Create ground truth
        let ground_truth = YmlGroundTruth::new()
            .with_error_tolerance(self.default_error_tolerance)
            .with_assertion(
                YmlAssertion::new("SolBalanceChange".to_string())
                    .with_pubkey(wallet_context.owner.clone())
                    .with_expected_change_lte(-(params.amount + 0.01) * 1_000_000_000.0), // Account for fees
            )
            .with_tool_call(YmlToolCall::new(ToolName::SolTransfer, true));

        // Create flow
        let flow = YmlFlow::new(flow_id, refined_prompt.original.clone(), wallet_info)
            .with_refined_prompt(refined_prompt.refined.clone())
            .with_step(step)
            .with_ground_truth(ground_truth)
            .with_metadata(
                crate::yml_schema::FlowMetadata::new()
                    .with_category("transfer".to_string())
                    .with_complexity(1),
            );

        Ok(flow)
    }

    /// Generate a lend flow
    async fn generate_lend_flow(
        &self,
        refined_prompt: &RefinedPrompt,
        wallet_context: &WalletContext,
        params: LendParams,
    ) -> Result<YmlFlow> {
        let flow_id = Uuid::now_v7().to_string();

        // Create wallet info
        let mut wallet_info =
            YmlWalletInfo::new(wallet_context.owner.clone(), wallet_context.sol_balance)
                .with_total_value(wallet_context.total_value_usd);

        // Add tokens if they exist
        for token in wallet_context.token_balances.values() {
            wallet_info = wallet_info.with_token(token.clone());
        }

        // Create lend step with expected_tools hint
        let mut step = YmlStep::new(
            "lend".to_string(),
            refined_prompt.original.clone(),
            refined_prompt.refined.clone(),
        );
        step.context = format!(
            "Deposit {} {} in Jupiter earn for yield",
            params.amount, params.token
        );
        let step = step
            .with_expected_tools(vec![ToolName::JupiterLendEarnDeposit])
            .with_tool_call(YmlToolCall::new(ToolName::JupiterLendEarnDeposit, true));

        // Create ground truth
        let ground_truth = YmlGroundTruth::new()
            .with_error_tolerance(self.default_error_tolerance)
            .with_assertion(
                YmlAssertion::new("TokenBalanceChange".to_string())
                    .with_pubkey(wallet_context.owner.clone())
                    .with_expected_change_lte(-(params.amount + 0.1) * 1_000_000_000.0), // Account for fees
            )
            .with_tool_call(YmlToolCall::new(ToolName::JupiterLendEarnDeposit, true));

        // Create flow
        let flow = YmlFlow::new(flow_id, refined_prompt.original.clone(), wallet_info)
            .with_refined_prompt(refined_prompt.refined.clone())
            .with_step(step)
            .with_ground_truth(ground_truth)
            .with_metadata(
                crate::yml_schema::FlowMetadata::new()
                    .with_category("lend".to_string())
                    .with_tag("jupiter".to_string())
                    .with_complexity(2),
            );

        Ok(flow)
    }

    /// Generate a swap then lend flow
    async fn generate_swap_then_lend_flow(
        &self,
        refined_prompt: &RefinedPrompt,
        wallet_context: &WalletContext,
        params: SwapThenLendParams,
    ) -> Result<YmlFlow> {
        let flow_id = Uuid::now_v7().to_string();
        let _swapped_amount_var = "SWAPPED_AMOUNT".to_string();

        // Create wallet info
        let mut wallet_info =
            YmlWalletInfo::new(wallet_context.owner.clone(), wallet_context.sol_balance)
                .with_total_value(wallet_context.total_value_usd);

        // Add tokens if they exist
        for token in wallet_context.token_balances.values() {
            wallet_info = wallet_info.with_token(token.clone());
        }

        // Create swap step with expected_tools hint
        let mut step1 = YmlStep::new(
            "swap".to_string(),
            refined_prompt.original.clone(),
            refined_prompt.refined.clone(),
        );
        step1.context = format!(
            "Exchange {} {} for {}",
            params.amount, params.from_token, params.to_token
        );
        let step1 = step1
            .with_expected_tools(vec![ToolName::JupiterSwap])
            .with_tool_call(YmlToolCall::new(ToolName::JupiterSwap, true));

        // Create lend step with expected_tools hint
        let mut step2 = YmlStep::new(
            "lend".to_string(),
            refined_prompt.original.clone(),
            refined_prompt.refined.clone(),
        );
        step2.context = format!("Lend swapped {} for yield", params.to_token);
        let step2 = step2
            .with_expected_tools(vec![ToolName::JupiterLendEarnDeposit])
            .with_tool_call(YmlToolCall::new(ToolName::JupiterLendEarnDeposit, true));

        // Create ground truth
        let ground_truth = YmlGroundTruth::new()
            .with_error_tolerance(self.default_error_tolerance)
            .with_assertion(
                YmlAssertion::new("SolBalanceChange".to_string())
                    .with_pubkey(wallet_context.owner.clone())
                    .with_expected_change_lte(-(params.amount + 0.1) * 1_000_000_000.0), // Account for fees
            )
            .with_tool_call(YmlToolCall::new(ToolName::JupiterSwap, true))
            .with_tool_call(YmlToolCall::new(ToolName::JupiterLendEarnDeposit, true));

        // Create flow
        let flow = YmlFlow::new(flow_id, refined_prompt.original.clone(), wallet_info)
            .with_refined_prompt(refined_prompt.refined.clone())
            .with_step(step1)
            .with_step(step2)
            .with_ground_truth(ground_truth)
            .with_metadata(
                crate::yml_schema::FlowMetadata::new()
                    .with_category("swap_then_lend".to_string())
                    .with_tag("jupiter".to_string())
                    .with_tag("compound".to_string())
                    .with_complexity(3),
            );

        Ok(flow)
    }

    /// Parse swap parameters from refined prompt
    fn parse_swap_params(&self, refined_prompt: &str) -> Result<SwapParams> {
        // Simple regex-based parsing for demonstration
        // In a real implementation, this would be more sophisticated
        use regex::Regex;

        // Parse amount
        let amount = if let Some(captures) =
            Regex::new(r"(\d+(?:\.\d+)?)\s*(\w+)\s+(?:swap|exchange|to)\s+(\w+)")
                .unwrap()
                .captures(refined_prompt)
        {
            captures
                .get(1)
                .unwrap()
                .as_str()
                .parse::<f64>()
                .unwrap_or(0.0)
        } else {
            0.0
        };

        // Parse from token
        let from_token = if let Some(captures) =
            Regex::new(r"(\d+(?:\.\d+)?)\s*(\w+)\s+(?:swap|exchange|to)\s+(\w+)")
                .unwrap()
                .captures(refined_prompt)
        {
            captures.get(2).unwrap().as_str().to_uppercase()
        } else {
            "SOL".to_string() // Default
        };

        // Parse to token
        let to_token = if let Some(captures) =
            Regex::new(r"(\d+(?:\.\d+)?)\s*(\w+)\s+(?:swap|exchange|to)\s+(\w+)")
                .unwrap()
                .captures(refined_prompt)
        {
            captures.get(3).unwrap().as_str().to_uppercase()
        } else {
            "USDC".to_string() // Default
        };

        Ok(SwapParams {
            amount,
            from_token,
            to_token,
        })
    }

    /// Parse transfer parameters from refined prompt
    fn parse_transfer_params(&self, refined_prompt: &str) -> Result<TransferParams> {
        // Simple regex-based parsing for demonstration
        // In a real implementation, this would be more sophisticated
        use regex::Regex;

        // Parse amount
        let amount = if let Some(captures) = Regex::new(r"(\d+(?:\.\d+)?)\s*sol")
            .unwrap()
            .captures(refined_prompt)
        {
            captures
                .get(1)
                .unwrap()
                .as_str()
                .parse::<f64>()
                .unwrap_or(0.0)
        } else {
            1.0 // Default to 1 SOL
        };

        // Parse recipient
        let recipient = if let Some(captures) = Regex::new(r"[1-9A-HJ-NP-Za-km-z]{32,44}")
            .unwrap()
            .captures(refined_prompt)
        {
            captures.get(0).unwrap().as_str().to_string()
        } else {
            "gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq".to_string() // Default for testing
        };

        Ok(TransferParams { amount, recipient })
    }

    /// Parse lend parameters from refined prompt
    fn parse_lend_params(&self, refined_prompt: &str) -> Result<LendParams> {
        // Simple regex-based parsing for demonstration
        // In a real implementation, this would be more sophisticated
        use regex::Regex;

        // Parse amount
        let amount = if let Some(captures) = Regex::new(r"(\d+(?:\.\d+)?)\s*(\w+)")
            .unwrap()
            .captures(refined_prompt)
        {
            captures
                .get(1)
                .unwrap()
                .as_str()
                .parse::<f64>()
                .unwrap_or(0.0)
        } else {
            0.0
        };

        // Parse token
        let token = if let Some(captures) = Regex::new(r"(\d+(?:\.\d+)?)\s*(\w+)")
            .unwrap()
            .captures(refined_prompt)
        {
            captures.get(2).unwrap().as_str().to_uppercase()
        } else {
            "USDC".to_string() // Default
        };

        Ok(LendParams { amount, token })
    }

    /// Parse swap then lend parameters from refined prompt
    fn parse_swap_then_lend_params(&self, refined_prompt: &str) -> Result<SwapThenLendParams> {
        // Use the swap parser to get swap parameters
        let swap_params = self.parse_swap_params(refined_prompt)?;

        Ok(SwapThenLendParams {
            amount: swap_params.amount,
            from_token: swap_params.from_token,
            to_token: swap_params.to_token,
        })
    }
}

/// Operation type determined from refined prompt
#[derive(Debug, Clone)]
enum OperationType {
    Swap(SwapParams),
    Transfer(TransferParams),
    Lend(LendParams),
    SwapThenLend(SwapThenLendParams),
    Unknown,
}

/// Parameters for swap operations
#[derive(Debug, Clone)]
struct SwapParams {
    amount: f64,
    from_token: String,
    to_token: String,
}

/// Parameters for transfer operations
#[derive(Debug, Clone)]
struct TransferParams {
    amount: f64,
    recipient: String,
}

/// Parameters for lend operations
#[derive(Debug, Clone)]
struct LendParams {
    amount: f64,
    token: String,
}

/// Parameters for swap then lend operations
#[derive(Debug, Clone)]
struct SwapThenLendParams {
    amount: f64,
    from_token: String,
    to_token: String,
}
