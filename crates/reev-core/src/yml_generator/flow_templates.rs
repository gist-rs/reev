//! Flow Templates for Common Patterns
//!
//! This module implements template-based flow generation for common operation
//! patterns as specified in the V3 plan. It provides a template system that
//! can be extended for new patterns without changing core logic.

use crate::refiner::RefinedPrompt;
use crate::yml_generator::operation_parser::{FlowTemplate, Operation};
use crate::yml_generator::step_builders::{
    create_wallet_info, LendStepBuilder, SwapStepBuilder, TransferStepBuilder,
};
use crate::yml_schema::{YmlFlow, YmlGroundTruth};
use anyhow::Result;
use reev_types::flow::WalletContext;
use std::collections::HashMap;
use uuid::Uuid;

/// Template manager for flow templates
pub struct FlowTemplateManager {
    /// Cache of templates by name
    templates: HashMap<String, FlowTemplateDefinition>,
}

/// Definition of a flow template
#[derive(Debug, Clone)]
pub struct FlowTemplateDefinition {
    /// Name of the template
    pub name: String,
    /// Description of the template
    pub description: String,
    /// Step pattern for this template
    pub step_pattern: Vec<String>,
}

impl Default for FlowTemplateManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FlowTemplateManager {
    /// Create a new flow template manager with default templates
    pub fn new() -> Self {
        let mut manager = Self {
            templates: HashMap::new(),
        };

        // Register default templates
        manager.register_default_templates();
        manager
    }

    /// Register default flow templates
    fn register_default_templates(&mut self) {
        // Single operation template
        let single_op = FlowTemplateDefinition {
            name: "single_operation".to_string(),
            description: "Template for a single operation flow".to_string(),
            step_pattern: vec!["operation".to_string()],
        };
        self.templates
            .insert("single_operation".to_string(), single_op);

        // Multi-operation template
        let multi_op = FlowTemplateDefinition {
            name: "multi_operation".to_string(),
            description: "Template for multiple operations in sequence".to_string(),
            step_pattern: vec![
                "operation1".to_string(),
                "operation2".to_string(),
                "...".to_string(),
            ],
        };
        self.templates
            .insert("multi_operation".to_string(), multi_op);

        // Swap then lend template
        let swap_then_lend = FlowTemplateDefinition {
            name: "swap_then_lend".to_string(),
            description: "Template for swapping tokens then lending the result".to_string(),
            step_pattern: vec!["swap".to_string(), "lend".to_string()],
        };
        self.templates
            .insert("swap_then_lend".to_string(), swap_then_lend);

        // Lend then swap template
        let lend_then_swap = FlowTemplateDefinition {
            name: "lend_then_swap".to_string(),
            description: "Template for lending tokens then swapping the result".to_string(),
            step_pattern: vec!["lend".to_string(), "swap".to_string()],
        };
        self.templates
            .insert("lend_then_swap".to_string(), lend_then_swap);

        // Complex multi-operation template
        let complex = FlowTemplateDefinition {
            name: "complex_multi_operation".to_string(),
            description: "Template for complex multi-operation flows".to_string(),
            step_pattern: vec![
                "operation1".to_string(),
                "operation2".to_string(),
                "operation3".to_string(),
                "...".to_string(),
            ],
        };
        self.templates
            .insert("complex_multi_operation".to_string(), complex);
    }

    /// Register a custom flow template
    pub fn register_template(&mut self, template: FlowTemplateDefinition) {
        self.templates.insert(template.name.clone(), template);
    }

    /// Get a template by name
    pub fn get_template(&self, name: &str) -> Option<&FlowTemplateDefinition> {
        self.templates.get(name)
    }

    /// Generate a flow from a template and operations
    pub async fn generate_flow_from_template(
        &self,
        refined_prompt: &RefinedPrompt,
        wallet_context: &WalletContext,
        operations: Vec<Operation>,
        template: FlowTemplate,
    ) -> Result<YmlFlow> {
        let flow_id = Uuid::now_v7().to_string();

        // Create wallet info
        let wallet_info = create_wallet_info(wallet_context);

        // Get the template name
        let template_name = match template {
            FlowTemplate::SingleOperation => "single_operation",
            FlowTemplate::MultiOperation => "multi_operation",
            FlowTemplate::SwapThenLend => "swap_then_lend",
            FlowTemplate::Custom(ref name) => name,
        };

        // Get the template definition
        let _template_def = self
            .get_template(template_name)
            .ok_or_else(|| anyhow::anyhow!("Template '{template_name}' not found"))?;

        // Generate steps based on the template and operations
        let mut steps = Vec::new();
        let mut ground_truth = YmlGroundTruth::new();

        match template_name {
            "single_operation" => {
                // Create a single step from the single operation
                if let Some(operation) = operations.first() {
                    let step = self
                        .create_step_from_operation(refined_prompt, wallet_context, operation)
                        .await?;
                    let step_ground_truth = self
                        .create_ground_truth_from_operation(wallet_context, operation)
                        .await?;

                    steps.push(step);
                    ground_truth = ground_truth.merge(step_ground_truth);
                }
            }
            "swap_then_lend" => {
                // Create swap then lend steps
                for operation in &operations {
                    let step = self
                        .create_step_from_operation(refined_prompt, wallet_context, operation)
                        .await?;
                    let step_ground_truth = self
                        .create_ground_truth_from_operation(wallet_context, operation)
                        .await?;

                    steps.push(step);
                    ground_truth = ground_truth.merge(step_ground_truth);
                }
            }
            "multi_operation" => {
                // Create steps for each operation in sequence
                for operation in &operations {
                    let step = self
                        .create_step_from_operation(refined_prompt, wallet_context, operation)
                        .await?;
                    let step_ground_truth = self
                        .create_ground_truth_from_operation(wallet_context, operation)
                        .await?;

                    steps.push(step);
                    ground_truth = ground_truth.merge(step_ground_truth);
                }
            }
            "lend_then_swap" => {
                // Create lend then swap steps
                for operation in &operations {
                    let step = self
                        .create_step_from_operation(refined_prompt, wallet_context, operation)
                        .await?;
                    let step_ground_truth = self
                        .create_ground_truth_from_operation(wallet_context, operation)
                        .await?;

                    steps.push(step);
                    ground_truth = ground_truth.merge(step_ground_truth);
                }
            }
            "complex_multi_operation" => {
                // Create steps for each operation in sequence
                for operation in &operations {
                    let step = self
                        .create_step_from_operation(refined_prompt, wallet_context, operation)
                        .await?;
                    let step_ground_truth = self
                        .create_ground_truth_from_operation(wallet_context, operation)
                        .await?;

                    steps.push(step);
                    ground_truth = ground_truth.merge(step_ground_truth);
                }
            }
            _ => {
                return Err(anyhow::anyhow!("Unknown template: {template_name}"));
            }
        }

        // Create the flow
        let flow = YmlFlow::new(flow_id, refined_prompt.original.clone(), wallet_info)
            .with_steps(steps)
            .with_ground_truth(ground_truth)
            .with_refined_prompt(refined_prompt.refined.clone());

        Ok(flow)
    }

    /// Create a step from an operation based on its type
    async fn create_step_from_operation(
        &self,
        refined_prompt: &RefinedPrompt,
        wallet_context: &WalletContext,
        operation: &Operation,
    ) -> Result<crate::yml_schema::YmlStep> {
        match operation {
            Operation::Swap { from, to, amount } => {
                SwapStepBuilder::create_step(refined_prompt, wallet_context, from, to, *amount)
                    .await
            }
            Operation::Transfer { mint, to, amount } => {
                TransferStepBuilder::create_step(refined_prompt, wallet_context, mint, to, *amount)
                    .await
            }
            Operation::Lend { mint, amount } => {
                LendStepBuilder::create_step(refined_prompt, wallet_context, mint, *amount).await
            }
        }
    }

    /// Create ground truth from an operation based on its type
    async fn create_ground_truth_from_operation(
        &self,
        wallet_context: &WalletContext,
        operation: &Operation,
    ) -> Result<YmlGroundTruth> {
        match operation {
            Operation::Swap { from, to, amount } => {
                SwapStepBuilder::create_ground_truth(wallet_context, from, to, *amount).await
            }
            Operation::Transfer { mint, to, amount } => {
                TransferStepBuilder::create_ground_truth(wallet_context, mint, to, *amount).await
            }
            Operation::Lend { mint, amount } => {
                LendStepBuilder::create_ground_truth(wallet_context, mint, *amount).await
            }
        }
    }
}
