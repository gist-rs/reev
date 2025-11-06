//! YML Generator for Dynamic Flows
//!
//! This module handles generating YML benchmark files from dynamic flow plans,
//! enabling bridge mode compatibility with existing runner infrastructure.

use crate::Result;
use reev_types::flow::{DynamicFlowPlan, WalletContext};
use reev_types::tools::ToolName;

use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::NamedTempFile;
use tokio::sync::Mutex;
use tracing::{debug, instrument};

/// YML Generator for creating benchmark files
#[derive(Debug)]
pub struct YmlGenerator {
    /// Template directory for YML templates (for future template system)
    #[allow(dead_code)]
    pub template_dir: PathBuf,
    /// Keep generated files alive
    generated_files: Arc<Mutex<Vec<NamedTempFile>>>,
}

impl YmlGenerator {
    /// Create a new YML generator
    pub fn new() -> Self {
        Self {
            template_dir: PathBuf::from("templates"),
            generated_files: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Generate YML file from flow plan
    #[instrument(skip(self))]
    pub async fn generate_yml(&self, flow_plan: &DynamicFlowPlan) -> Result<String> {
        debug!("Generating YML for flow: {}", flow_plan.flow_id);

        // Generate YML content first
        let yml_content = self.generate_yml_content(flow_plan)?;

        // Create temporary file and write content immediately
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(yml_content.as_bytes())?;
        temp_file.flush()?;

        // Get path
        let temp_path = temp_file.path().to_string_lossy().to_string();

        // Keep file handle alive by storing it
        self.generated_files.lock().await.push(temp_file);

        debug!("Generated YML file: {}", temp_path);
        Ok(temp_path)
    }

    /// Generate YML content from flow plan
    pub fn generate_yml_content(&self, flow_plan: &DynamicFlowPlan) -> Result<String> {
        let mut yml = serde_yaml::Mapping::new();

        // Basic metadata
        yml.insert(
            serde_yaml::Value::String("id".to_string()),
            serde_yaml::Value::String(format!("dynamic-{}", flow_plan.flow_id)),
        );
        yml.insert(
            serde_yaml::Value::String("description".to_string()),
            serde_yaml::Value::String(format!("Dynamic flow: {}", flow_plan.user_prompt)),
        );

        // Add required tags field
        let tags = serde_yaml::Value::Sequence(vec![
            serde_yaml::Value::String("dynamic".to_string()),
            serde_yaml::Value::String("jupiter".to_string()),
        ]);
        yml.insert(serde_yaml::Value::String("tags".to_string()), tags);

        // Add flow_type field for dynamic flows
        yml.insert(
            serde_yaml::Value::String("flow_type".to_string()),
            serde_yaml::Value::String("dynamic".to_string()),
        );

        // Generate initial state from context
        let initial_state = self.generate_initial_state(&flow_plan.context);
        yml.insert(
            serde_yaml::Value::String("initial_state".to_string()),
            serde_yaml::Value::Sequence(initial_state),
        );

        // Use the enhanced prompt from first step
        let prompt = flow_plan
            .steps
            .first()
            .map(|step| step.prompt_template.clone())
            .unwrap_or_else(|| flow_plan.user_prompt.clone());

        yml.insert(
            serde_yaml::Value::String("prompt".to_string()),
            serde_yaml::Value::String(prompt),
        );

        // Generate basic ground truth
        let ground_truth = self.generate_ground_truth(&flow_plan.steps);
        yml.insert(
            serde_yaml::Value::String("ground_truth".to_string()),
            serde_yaml::Value::Mapping(ground_truth),
        );

        // Convert to string
        Ok(serde_yaml::to_string(&yml)?)
    }

    /// Generate initial state from wallet context
    pub fn generate_initial_state(&self, context: &WalletContext) -> Vec<serde_yaml::Value> {
        let mut state = Vec::new();

        // User wallet with SOL balance
        let mut user_wallet = serde_yaml::Mapping::new();
        user_wallet.insert(
            serde_yaml::Value::String("pubkey".to_string()),
            serde_yaml::Value::String("USER_WALLET_PUBKEY".to_string()),
        );
        user_wallet.insert(
            serde_yaml::Value::String("owner".to_string()),
            serde_yaml::Value::String(context.owner.clone()),
        );
        user_wallet.insert(
            serde_yaml::Value::String("lamports".to_string()),
            serde_yaml::Value::Number(serde_yaml::Number::from(context.sol_balance)),
        );
        state.push(serde_yaml::Value::Mapping(user_wallet));

        // USDC ATA with zero balance
        let mut usdc_ata = serde_yaml::Mapping::new();
        usdc_ata.insert(
            serde_yaml::Value::String("pubkey".to_string()),
            serde_yaml::Value::String("USER_USDC_ATA".to_string()),
        );
        usdc_ata.insert(
            serde_yaml::Value::String("owner".to_string()),
            serde_yaml::Value::String("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string()),
        );
        usdc_ata.insert(
            serde_yaml::Value::String("lamports".to_string()),
            serde_yaml::Value::Number(serde_yaml::Number::from(2039280)), // Standard rent
        );

        let mut ata_data = serde_yaml::Mapping::new();
        ata_data.insert(
            serde_yaml::Value::String("mint".to_string()),
            serde_yaml::Value::String("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()),
        );
        ata_data.insert(
            serde_yaml::Value::String("owner".to_string()),
            serde_yaml::Value::String("USER_WALLET_PUBKEY".to_string()),
        );
        ata_data.insert(
            serde_yaml::Value::String("amount".to_string()),
            serde_yaml::Value::String("0".to_string()),
        );

        usdc_ata.insert(
            serde_yaml::Value::String("data".to_string()),
            serde_yaml::Value::Mapping(ata_data),
        );
        state.push(serde_yaml::Value::Mapping(usdc_ata));

        state
    }

    /// Generate basic ground truth assertions
    pub fn generate_ground_truth(
        &self,
        steps: &[reev_types::flow::DynamicStep],
    ) -> serde_yaml::Mapping {
        let mut ground_truth = serde_yaml::Mapping::new();

        // For now, generate basic assertions that work for most flows
        let mut assertions = Vec::new();

        // Check if any step involves swap
        let has_swap = steps.iter().any(|step| {
            step.required_tools.contains(&ToolName::SolTransfer)
                || step.description.to_lowercase().contains("swap")
        });

        if has_swap {
            // Add USDC balance assertion
            let mut usdc_assertion = serde_yaml::Mapping::new();
            usdc_assertion.insert(
                serde_yaml::Value::String("type".to_string()),
                serde_yaml::Value::String("TokenAccountBalance".to_string()),
            );
            usdc_assertion.insert(
                serde_yaml::Value::String("pubkey".to_string()),
                serde_yaml::Value::String("USER_USDC_ATA".to_string()),
            );
            usdc_assertion.insert(
                serde_yaml::Value::String("expected_gte".to_string()),
                serde_yaml::Value::Number(serde_yaml::Number::from(1)),
            );
            usdc_assertion.insert(
                serde_yaml::Value::String("weight".to_string()),
                serde_yaml::Value::Number(serde_yaml::Number::from(1)),
            );
            assertions.push(serde_yaml::Value::Mapping(usdc_assertion));
        }

        // Check if any step involves lend/earn
        let has_lend = steps.iter().any(|step| {
            step.required_tools.contains(&ToolName::JupiterEarn)
                || step.description.to_lowercase().contains("lend")
                || step.description.to_lowercase().contains("deposit")
        });

        if has_lend {
            // Add SOL balance change assertion for lend operations
            let mut sol_assertion = serde_yaml::Mapping::new();
            sol_assertion.insert(
                serde_yaml::Value::String("type".to_string()),
                serde_yaml::Value::String("SolBalanceChange".to_string()),
            );
            sol_assertion.insert(
                serde_yaml::Value::String("pubkey".to_string()),
                serde_yaml::Value::String("USER_WALLET_PUBKEY".to_string()),
            );
            sol_assertion.insert(
                serde_yaml::Value::String("expected_change_gte".to_string()),
                serde_yaml::Value::Number(serde_yaml::Number::from(-100005000i64)), // Account for fees
            );
            sol_assertion.insert(
                serde_yaml::Value::String("weight".to_string()),
                serde_yaml::Value::Number(serde_yaml::Number::from(1)),
            );
            assertions.push(serde_yaml::Value::Mapping(sol_assertion));
        }

        ground_truth.insert(
            serde_yaml::Value::String("min_score".to_string()),
            serde_yaml::Value::Number(serde_yaml::Number::from(0.6)),
        );

        ground_truth.insert(
            serde_yaml::Value::String("final_state_assertions".to_string()),
            serde_yaml::Value::Sequence(assertions),
        );

        ground_truth
    }

    /// Generate system prompt from wallet context
    pub fn generate_system_prompt(&self, context: &WalletContext) -> String {
        format!(
            "You are a DeFi assistant helping the user manage their Solana wallet.\n\
             \n\
             Current Wallet State:\n\
             - SOL Balance: {:.2} SOL\n\
             - Total Value: ${:.2}\n\
             - Wallet: {}\n\
             \n\
             Available Protocols:\n\
             - Jupiter DEX for token swaps\n\
             - Jupiter Lending for earning yield\n\
             \n\
             Execute the user's request efficiently and provide clear feedback.",
            context.sol_balance_sol(),
            context.total_value_usd,
            context.owner
        )
    }

    /// Generate tools configuration from flow steps
    pub fn generate_tools_config(
        &self,
        steps: &[reev_types::flow::DynamicStep],
    ) -> Vec<serde_yaml::Value> {
        let mut tool_names = std::collections::HashSet::new();

        for step in steps {
            for tool in &step.required_tools {
                tool_names.insert(tool.as_str());
            }
        }

        // Convert to tool configurations
        tool_names
            .into_iter()
            .map(|tool_name| {
                let mut tool_config = serde_yaml::Mapping::new();
                tool_config.insert(
                    serde_yaml::Value::String("name".to_string()),
                    serde_yaml::Value::String(tool_name.to_string()),
                );
                serde_yaml::Value::Mapping(tool_config)
            })
            .collect()
    }

    /// Generate steps configuration
    pub fn generate_steps_config(
        &self,
        steps: &[reev_types::flow::DynamicStep],
    ) -> Vec<serde_yaml::Value> {
        steps
            .iter()
            .enumerate()
            .map(|(index, step)| {
                let mut step_config = serde_yaml::Mapping::new();
                step_config.insert(
                    serde_yaml::Value::String("id".to_string()),
                    serde_yaml::Value::String(step.step_id.clone()),
                );
                step_config.insert(
                    serde_yaml::Value::String("order".to_string()),
                    serde_yaml::Value::Number(serde_yaml::Number::from(index + 1)),
                );
                step_config.insert(
                    serde_yaml::Value::String("description".to_string()),
                    serde_yaml::Value::String(step.description.clone()),
                );
                step_config.insert(
                    serde_yaml::Value::String("prompt".to_string()),
                    serde_yaml::Value::String(step.prompt_template.clone()),
                );
                step_config.insert(
                    serde_yaml::Value::String("critical".to_string()),
                    serde_yaml::Value::Bool(step.critical),
                );
                serde_yaml::Value::Mapping(step_config)
            })
            .collect()
    }
}

impl Default for YmlGenerator {
    fn default() -> Self {
        Self::new()
    }
}
