use crate::refiner::RefinedPrompt;
use crate::yml_generator::operation_types::{
    is_sell_all, LendParams, SwapParams, SwapThenLendParams, TransferParams,
};
use crate::yml_schema::{
    YmlAssertion, YmlFlow, YmlGroundTruth, YmlStep, YmlToolCall, YmlWalletInfo,
};
use anyhow::Result;
use reev_types::flow::WalletContext;
use reev_types::tools::ToolName;
use uuid::Uuid;

/// Build a swap flow from parameters
pub async fn build_swap_flow(
    refined_prompt: &RefinedPrompt,
    wallet_context: &WalletContext,
    params: SwapParams,
) -> Result<YmlFlow> {
    let flow_id = Uuid::now_v7().to_string();

    // Create wallet info
    let mut wallet_info =
        YmlWalletInfo::new(wallet_context.owner.clone(), wallet_context.sol_balance)
            .with_total_value(wallet_context.total_value_usd);

    // Add each token balance to the wallet info
    for token in wallet_context.token_balances.values() {
        wallet_info = wallet_info.with_token(token.clone());
    }

    // Calculate amount in SOL for display
    let amount_sol = if params.from_token == "SOL" {
        // Account for gas reserve when calculating display amount
        let gas_reserve_lamports = 50_000_000u64; // 0.05 SOL
        let amount_in_lamports = params.amount * 1_000_000_000.0;
        let display_amount = if amount_in_lamports > gas_reserve_lamports as f64 {
            amount_in_lamports - gas_reserve_lamports as f64
        } else {
            amount_in_lamports / 2.0
        };
        display_amount / 1_000_000_000.0
    } else {
        params.amount
    };

    // Create swap step
    let step_prompt = if is_sell_all(&refined_prompt.refined) {
        format!(
            "swap ALL {amount_sol:.1} {} to {}",
            params.from_token, params.to_token
        )
    } else {
        format!(
            "swap {amount_sol:.1} {} to {}",
            params.from_token, params.to_token
        )
    };

    let step = YmlStep::new(
        "swap".to_string(),
        step_prompt.clone(),
        format!(
            "Exchange {amount_sol:.1} {} for {}",
            params.from_token, params.to_token
        ),
    )
    .with_tool_call(YmlToolCall::new(ToolName::JupiterSwap, true))
    .with_expected_tools(vec![ToolName::JupiterSwap])
    .with_critical(true);

    // Create ground truth
    let ground_truth = YmlGroundTruth::new()
        .with_assertion(
            YmlAssertion::new("SolBalanceChange".to_string())
                .with_pubkey(wallet_context.owner.clone())
                .with_expected_change_gte(
                    -(amount_sol * 1_000_000_000.0 + 50_000_000.0 + 10_000_000.0),
                ),
        ) // Account for swap amount + gas reserve + transaction fees
        .with_tool_call(YmlToolCall::new(ToolName::JupiterSwap, true))
        .with_error_tolerance(0.01);

    // Create flow
    let flow = YmlFlow::new(flow_id, refined_prompt.original.clone(), wallet_info)
        .with_step(step)
        .with_ground_truth(ground_truth)
        .with_refined_prompt(refined_prompt.refined.clone());

    Ok(flow)
}

/// Build a transfer flow from parameters
pub async fn build_transfer_flow(
    refined_prompt: &RefinedPrompt,
    wallet_context: &WalletContext,
    params: TransferParams,
) -> Result<YmlFlow> {
    let flow_id = Uuid::now_v7().to_string();

    // Create wallet info
    let mut wallet_info =
        YmlWalletInfo::new(wallet_context.owner.clone(), wallet_context.sol_balance)
            .with_total_value(wallet_context.total_value_usd);

    // Add each token balance to the wallet info
    for token in wallet_context.token_balances.values() {
        wallet_info = wallet_info.with_token(token.clone());
    }

    // Create transfer step
    let step = YmlStep::new(
        "transfer".to_string(),
        format!("transfer {} SOL to {}", params.amount, params.recipient),
        format!(
            "Transfer {} SOL to recipient {}",
            params.amount, params.recipient
        ),
    )
    .with_tool_call(YmlToolCall::new(ToolName::SolTransfer, true))
    .with_expected_tools(vec![ToolName::SolTransfer])
    .with_critical(true);

    // Create ground truth
    let ground_truth = YmlGroundTruth::new()
        .with_assertion(
            YmlAssertion::new("SolBalanceChange".to_string())
                .with_pubkey(wallet_context.owner.clone())
                .with_expected_change_lte(-(params.amount * 1_000_000_000.0 + 5_000_000.0)), // Account for fees
        )
        .with_tool_call(YmlToolCall::new(ToolName::SolTransfer, true))
        .with_error_tolerance(0.01);

    // Create flow
    let flow = YmlFlow::new(flow_id, refined_prompt.original.clone(), wallet_info)
        .with_step(step)
        .with_ground_truth(ground_truth)
        .with_refined_prompt(refined_prompt.refined.clone());

    Ok(flow)
}

/// Build a lend flow from parameters
pub async fn build_lend_flow(
    refined_prompt: &RefinedPrompt,
    wallet_context: &WalletContext,
    params: LendParams,
) -> Result<YmlFlow> {
    let flow_id = Uuid::now_v7().to_string();

    // Create wallet info
    let mut wallet_info =
        YmlWalletInfo::new(wallet_context.owner.clone(), wallet_context.sol_balance)
            .with_total_value(wallet_context.total_value_usd);

    // Add each token balance to the wallet info
    for token in wallet_context.token_balances.values() {
        wallet_info = wallet_info.with_token(token.clone());
    }

    // Create lend step
    let step = YmlStep::new(
        "lend".to_string(),
        format!("lend {} {} to jupiter", params.amount, params.token),
        format!(
            "Deposit {} {} in Jupiter earn for yield",
            params.amount, params.token
        ),
    )
    .with_tool_call(YmlToolCall::new(ToolName::JupiterLendEarnDeposit, true))
    .with_expected_tools(vec![ToolName::JupiterLendEarnDeposit])
    .with_critical(true);

    // Create ground truth
    let ground_truth = YmlGroundTruth::new()
        .with_tool_call(YmlToolCall::new(ToolName::JupiterLendEarnDeposit, true))
        .with_error_tolerance(0.01);

    // Create flow
    let flow = YmlFlow::new(flow_id, refined_prompt.original.clone(), wallet_info)
        .with_step(step)
        .with_ground_truth(ground_truth)
        .with_refined_prompt(refined_prompt.refined.clone());

    Ok(flow)
}
