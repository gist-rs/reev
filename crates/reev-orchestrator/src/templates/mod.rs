//! Template System for Dynamic Flows
//!
//! This module provides Handlebars-based template rendering for generating
//! context-aware prompts for DeFi operations.

use anyhow::Result;
use reev_types::flow::WalletContext;

pub mod engine;
pub mod renderer;

pub use engine::TemplateEngine;
pub use renderer::TemplateRenderer;

/// Template types for different use cases
#[derive(Debug, Clone, PartialEq)]
pub enum TemplateType {
    /// Base templates for generic operations
    Base,
    /// Protocol-specific templates
    Protocol(String),
    /// Multi-step scenario templates
    Scenario(String),
}

/// Template metadata for validation and organization
#[derive(Debug, Clone)]
pub struct TemplateMetadata {
    pub name: String,
    pub template_type: TemplateType,
    pub description: String,
    pub required_variables: Vec<String>,
    pub optional_variables: Vec<String>,
}

impl TemplateMetadata {
    pub fn new(
        name: String,
        template_type: TemplateType,
        description: String,
        required_variables: Vec<String>,
        optional_variables: Vec<String>,
    ) -> Self {
        Self {
            name,
            template_type,
            description,
            required_variables,
            optional_variables,
        }
    }

    /// Validate that template has all required variables
    pub fn validate_variables(
        &self,
        context: &WalletContext,
        variables: &std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        // Check required variables
        for required in &self.required_variables {
            if !variables.contains_key(required) {
                return Err(anyhow::anyhow!("Missing required variable: {required}"));
            }
        }

        // Validate wallet-specific requirements
        if self.required_variables.contains(&"wallet".to_string()) && context.owner.is_empty() {
            return Err(anyhow::anyhow!("Wallet context is required but empty"));
        }

        Ok(())
    }
}

/// Template registration result
#[derive(Debug)]
pub struct TemplateRegistration {
    pub name: String,
    pub source: String,
    pub metadata: TemplateMetadata,
    pub compilation_time_ms: u64,
}

/// Template rendering result with metadata
#[derive(Debug, Clone)]
pub struct TemplateRenderResult {
    pub rendered: String,
    pub template_name: String,
    pub render_time_ms: u64,
    pub variables_used: Vec<String>,
}

/// Helper functions for Handlebars templates
pub mod helpers {
    use handlebars::{Context, Handlebars, Helper, Output, RenderContext, RenderError};
    use reev_types::flow::WalletContext;

    /// Helper to get token price from wallet context
    pub fn get_token_price(
        h: &Helper,
        _handlebars: &Handlebars,
        ctx: &Context,
        _render_context: &mut RenderContext,
        out: &mut dyn Output,
    ) -> Result<(), RenderError> {
        let token_mint = h
            .param(0)
            .ok_or_else(|| RenderError::new("Token mint parameter required"))?;
        let token_str = token_mint
            .value()
            .as_str()
            .ok_or_else(|| RenderError::new("Token must be string"))?;

        // Access the root data directly from the context
        let root_data = ctx.data();

        // Try to access wallet data through different paths
        if let Some(wallet_value) = root_data.get("wallet") {
            // Direct approach: extract prices from JSON without full deserialization
            if let Some(wallet_obj) = wallet_value.as_object() {
                if let Some(token_prices) = wallet_obj.get("token_prices") {
                    if let Some(prices_obj) = token_prices.as_object() {
                        if let Some(price_value) = prices_obj.get(token_str) {
                            if let Some(price) = price_value.as_f64() {
                                out.write(&format!("{price:.6}"))?;
                                return Ok(());
                            }
                        }
                    }
                }
            }

            // Fallback: try full deserialization
            if let Ok(wallet) = serde_json::from_value::<WalletContext>(wallet_value.clone()) {
                if let Some(price) = wallet.get_token_price(token_str) {
                    out.write(&format!("{price:.6}"))?;
                    return Ok(());
                }
            }
        }

        out.write("0.0")?;
        Ok(())
    }

    /// Helper to get token balance from wallet context
    pub fn get_token_balance(
        h: &Helper,
        _handlebars: &Handlebars,
        ctx: &Context,
        _render_context: &mut RenderContext,
        out: &mut dyn Output,
    ) -> Result<(), RenderError> {
        let token_mint = h
            .param(0)
            .ok_or_else(|| RenderError::new("Token mint parameter required"))?;
        let token_str = token_mint
            .value()
            .as_str()
            .ok_or_else(|| RenderError::new("Token must be string"))?;

        // Access the root data directly from the context
        let root_data = ctx.data();

        // Try to access wallet data through different paths
        if let Some(wallet_value) = root_data.get("wallet") {
            // Direct approach: extract balances from JSON without full deserialization
            if let Some(wallet_obj) = wallet_value.as_object() {
                if let Some(token_balances) = wallet_obj.get("token_balances") {
                    if let Some(balances_obj) = token_balances.as_object() {
                        if let Some(balance_value) = balances_obj.get(token_str) {
                            if let Some(balance_obj) = balance_value.as_object() {
                                if let Some(balance) = balance_obj.get("balance") {
                                    if let Some(balance_num) = balance.as_u64() {
                                        out.write(&format!("{balance_num}"))?;
                                        return Ok(());
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Fallback: try full deserialization
            if let Ok(wallet) = serde_json::from_value::<WalletContext>(wallet_value.clone()) {
                if let Some(balance) = wallet.get_token_balance(token_str) {
                    out.write(&format!("{}", balance.balance))?;
                    return Ok(());
                }
            }
        }

        out.write("0")?;
        Ok(())
    }

    /// Helper to format USD amounts
    pub fn format_usd(
        h: &Helper,
        _: &Handlebars,
        _: &Context,
        _: &mut RenderContext,
        out: &mut dyn Output,
    ) -> Result<(), RenderError> {
        let amount = h
            .param(0)
            .ok_or_else(|| RenderError::new("Amount parameter required"))?;
        let amount_val = amount
            .value()
            .as_f64()
            .ok_or_else(|| RenderError::new("Amount must be number"))?;

        out.write(&format!("${amount_val:.2}"))?;
        Ok(())
    }

    /// Helper to format token amounts with decimals
    pub fn format_token(
        h: &Helper,
        _: &Handlebars,
        _: &Context,
        _: &mut RenderContext,
        out: &mut dyn Output,
    ) -> Result<(), RenderError> {
        let amount = h
            .param(0)
            .ok_or_else(|| RenderError::new("Amount parameter required"))?;
        let decimals = h
            .param(1)
            .ok_or_else(|| RenderError::new("Decimals parameter required"))?;

        let amount_val = amount
            .value()
            .as_f64()
            .ok_or_else(|| RenderError::new("Amount must be number"))?;
        let decimals_val = decimals
            .value()
            .as_u64()
            .ok_or_else(|| RenderError::new("Decimals must be integer"))?;

        let divisor = 10_f64.powi(decimals_val as i32);
        let formatted_amount = amount_val / divisor;

        out.write(&format!("{formatted_amount}"))?;
        Ok(())
    }

    /// Register all helper functions
    pub fn register_all(handlebars: &mut Handlebars) -> anyhow::Result<()> {
        handlebars.register_helper("get_token_price", Box::new(get_token_price));
        handlebars.register_helper("get_token_balance", Box::new(get_token_balance));
        handlebars.register_helper("format_usd", Box::new(format_usd));
        handlebars.register_helper("format_token", Box::new(format_token));

        Ok(())
    }
}
// Tests moved to tests/template_metadata_tests.rs
