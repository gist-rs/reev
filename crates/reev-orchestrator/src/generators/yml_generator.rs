//! YML Generator for Dynamic Flows
//!
//! This module handles generating YML benchmark files from dynamic flow plans,
//! enabling bridge mode compatibility with existing runner infrastructure.

use crate::Result;
use reev_types::flow::{DynamicFlowPlan, WalletContext};

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
        yml.insert(
            serde_yaml::Value::String("agent".to_string()),
            serde_yaml::Value::String("glm-4.6".to_string()),
        );
        yml.insert(
            serde_yaml::Value::String("agent_config".to_string()),
            serde_yaml::Value::Mapping(serde_yaml::Mapping::new()),
        );

        // Create unified data from context
        let mut unified_data = serde_yaml::Mapping::new();

        // Add enhanced user request from first step
        let enhanced_request = flow_plan
            .steps
            .first()
            .map(|step| step.prompt_template.clone())
            .unwrap_or_else(|| flow_plan.user_prompt.clone());

        unified_data.insert(
            serde_yaml::Value::String("enhanced_user_request".to_string()),
            serde_yaml::Value::String(enhanced_request),
        );

        // Add wallet context as system prompt
        let system_prompt = self.generate_system_prompt(&flow_plan.context);
        unified_data.insert(
            serde_yaml::Value::String("system_prompt".to_string()),
            serde_yaml::Value::String(system_prompt),
        );

        // Add tools configuration
        let tools = self.generate_tools_config(&flow_plan.steps);
        unified_data.insert(
            serde_yaml::Value::String("tools".to_string()),
            serde_yaml::Value::Sequence(tools),
        );

        // Add step configuration
        let steps = self.generate_steps_config(&flow_plan.steps);
        unified_data.insert(
            serde_yaml::Value::String("steps".to_string()),
            serde_yaml::Value::Sequence(steps),
        );

        yml.insert(
            serde_yaml::Value::String("unified_data".to_string()),
            serde_yaml::Value::Mapping(unified_data),
        );

        // Convert to string
        Ok(serde_yaml::to_string(&yml)?)
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
