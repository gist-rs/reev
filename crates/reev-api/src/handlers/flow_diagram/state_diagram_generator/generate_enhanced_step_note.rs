//! Generate Enhanced Step Note Module
//!
//! This module provides utility functions for generating enhanced step notes for diagram visualizations.

use reev_types::ToolName;

/// Generate enhanced step notes for 300-series flows
pub fn generate_enhanced_step_note(
    tool_call: &crate::handlers::flow_diagram::session_parser::ParsedToolCall,
    step_index: usize,
) -> String {
    match tool_call.tool_name.as_str() {
        name if name == ToolName::GetAccountBalance.to_string() => {
            if let Some(result_data) = &tool_call.result_data {
                if let (Some(sol_balance), Some(usdc_balance), Some(total_value)) = (
                    result_data.get("sol_balance").and_then(|v| v.as_f64()),
                    result_data.get("usdc_balance").and_then(|v| v.as_f64()),
                    result_data.get("total_value_usd").and_then(|v| v.as_f64()),
                ) {
                    return format!(
                        "Portfolio Snapshot<br/>SOL: {:.6}<br/>USDC: {:.2}<br/>Total: ${:.2}",
                        sol_balance / 1_000_000_000.0,
                        usdc_balance / 1_000_000.0,
                        total_value
                    );
                }
            }
            "Step 1: Portfolio Assessment<br/>Check wallet balances<br/>Calculate available capital"
                .to_string()
        }
        name if name == ToolName::JupiterSwap.to_string() => {
            if let Some(result_data) = &tool_call.result_data {
                if let (Some(input_amount), Some(output_amount), Some(signature)) = (
                    result_data.get("input_amount").and_then(|v| v.as_f64()),
                    result_data.get("output_amount").and_then(|v| v.as_f64()),
                    result_data.get("signature").and_then(|v| v.as_str()),
                ) {
                    return format!(
                        "Step {}: Jupiter Swap<br/>Input: {:.6} SOL<br/>Output: {:.2} USDC<br/>TX: {}...",
                        step_index + 1,
                        input_amount,
                        output_amount,
                        &signature[..8]
                    );
                }
            }

            // Fallback to parameters
            if let Some(amount) = tool_call.params.get("amount").and_then(|v| v.as_u64()) {
                let sol_amount = amount as f64 / 1_000_000_000.0;
                let estimated_usdc = sol_amount * 150.0; // Approximate price
                return format!(
                    "Step {}: SOL → USDC Swap<br/>Amount: {:.6} SOL<br/>Expected: {:.2} USDC<br/>DEX: Jupiter",
                    step_index + 1,
                    sol_amount,
                    estimated_usdc
                );
            }
            "Step 3: Jupiter DEX Swap<br/>Convert SOL to USDC<br/>Execute with slippage tolerance"
                .to_string()
        }
        name if name == ToolName::JupiterLendEarnDeposit.to_string() => {
            if let Some(result_data) = &tool_call.result_data {
                if let (Some(deposit_amount), Some(apy), Some(position_value)) = (
                    result_data.get("deposit_amount").and_then(|v| v.as_f64()),
                    result_data.get("apy").and_then(|v| v.as_f64()),
                    result_data.get("position_value").and_then(|v| v.as_f64()),
                ) {
                    return format!(
                        "Step {}: Lend Position<br/>Deposit: {:.2} USDC<br/>APY: {:.1}%<br/>Position: ${:.2}",
                        step_index + 1,
                        deposit_amount / 1_000_000.0,
                        apy,
                        position_value
                    );
                }
            }

            // Fallback to parameters
            if let Some(amount) = tool_call.params.get("amount").and_then(|v| v.as_u64()) {
                let usdc_amount = amount as f64 / 1_000_000.0;
                let daily_yield = usdc_amount * 0.085 / 365.0; // 8.5% APY
                return format!(
                    "Step 4: USDC Lending<br/>Deposit: {usdc_amount:.2} USDC<br/>Daily Yield: ${daily_yield:.4}<br/>Protocol: Jupiter"
                );
            }
            "Step 4: Jupiter Lending<br/>Deposit USDC for yield<br/>Create lending position"
                .to_string()
        }
        name if name.contains("check_positions") => {
            "Step 5: Position Verification<br/>Validate final state<br/>Check yield generation"
                .to_string()
        }
        _ => {
            format!(
                "Step {}: {}<br/>Executing tool call<br/>Processing parameters",
                step_index + 1,
                tool_call.tool_name
            )
        }
    }
}
