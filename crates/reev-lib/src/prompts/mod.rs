//! Prompt templates and management for reev-core

use anyhow::{anyhow, Result};
use std::fs;
use std::path::Path;
use tracing::{debug, warn};

/// Prompt template manager
pub struct PromptManager {
    /// Directory containing prompt templates
    templates_dir: String,
}

impl PromptManager {
    /// Create a new prompt manager
    pub fn new(templates_dir: String) -> Self {
        Self { templates_dir }
    }

    /// Get the templates directory path
    pub fn templates_dir(&self) -> &str {
        &self.templates_dir
    }

    /// Load a prompt template by name
    pub fn load_template(&self, template_name: &str) -> Result<String> {
        let template_path = Path::new(&self.templates_dir).join(format!("{template_name}.yml"));

        if !template_path.exists() {
            warn!("Template file not found: {:?}", template_path);
            return Err(anyhow::anyhow!("Template not found: {template_name}"));
        }

        let content = fs::read_to_string(&template_path)
            .map_err(|e| anyhow!("Failed to read template {template_name}: {e}"))?;

        debug!("Loaded template: {}", template_name);
        Ok(content)
    }

    /// Load refine user prompt template
    pub fn load_refine_template(&self) -> Result<String> {
        self.load_template("refine_user_prompt")
    }

    /// Load tool execution template
    pub fn load_tool_execution_template(&self) -> Result<String> {
        self.load_template("tool_execution")
    }
}
// Tests moved to tests/prompts_tests.rs
