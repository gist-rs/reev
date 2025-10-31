//! Transaction log parser for converting blockchain transaction data to ASCII format
//!
//! This module provides functionality to parse blockchain transaction logs from execution data
//! and convert them into human-readable ASCII tree formats. It supports:
//! - Transaction logs from completed executions
//! - Structured blockchain transaction data
//! - Program call hierarchies with proper nesting
//!
//! The parser formats transaction logs with visual indicators for:
//! - Program calls and instructions
//! - Account operations
//! - Compute unit usage
//! - Success/failure status

use anyhow::Result;
use ascii_tree::Tree;
use reev_lib::results::TestResult;
use serde_json::Value;
use tracing::debug;

/// Transaction log parser for converting blockchain transaction data to ASCII format
#[derive(Debug, Clone)]
pub struct TransactionLogParser {
    /// Show compute units in the output
    show_compute_units: bool,
}

impl Default for TransactionLogParser {
    fn default() -> Self {
        Self::new()
    }
}

impl TransactionLogParser {
    /// Create a new transaction log parser
    pub fn new() -> Self {
        Self {
            show_compute_units: true,
        }
    }

    /// Create parser with specific compute unit display setting
    pub fn with_compute_units(show_compute_units: bool) -> Self {
        Self { show_compute_units }
    }

    /// Generate transaction logs from TestResult
    pub fn generate_transaction_logs(&self, test_result: &TestResult) -> Result<String> {
        let mut trees = Vec::new();

        for (step_idx, step) in test_result.trace.steps.iter().enumerate() {
            // Extract transaction logs from the step observation
            if !step.observation.last_transaction_logs.is_empty() {
                let step_tree = self.create_step_tree(
                    step_idx + 1,
                    &step.observation.last_transaction_logs,
                    step.observation.last_transaction_error.as_deref(),
                )?;
                trees.push(step_tree);
            }
        }

        if trees.is_empty() {
            return Ok("📝 No transaction logs found".to_string());
        }

        // Combine all steps into a single tree
        let root = Tree::Node("🔗 Blockchain Transactions".to_string(), trees);
        self.render_tree(&root)
    }

    /// Generate transaction logs from execution result JSON
    pub fn generate_from_result_data(&self, result_data: &Value) -> Result<String> {
        // Try to parse as TestResult first
        if let Ok(test_result_str) = serde_json::to_string(result_data) {
            if let Ok(test_result) = serde_json::from_str::<TestResult>(&test_result_str) {
                debug!("Successfully parsed as TestResult, generating transaction logs");
                return self.generate_transaction_logs(&test_result);
            } else {
                debug!("Failed to parse as TestResult");
            }
        }

        // Fallback: extract logs directly from JSON structure
        debug!("Using fallback JSON structure extraction");
        self.extract_from_json_structure(result_data)
    }

    /// Extract transaction logs from JSON structure
    fn extract_from_json_structure(&self, result_data: &Value) -> Result<String> {
        let mut trees = Vec::new();
        let mut step_counter = 1;

        // Check different possible structures
        if let Some(final_result) = result_data.get("final_result") {
            if let Some(data) = final_result.get("data") {
                debug!("Found final_result.data structure");
                self.extract_from_steps_data(data, &mut trees, &mut step_counter)?;
            }
        }

        if let Some(trace) = result_data.get("trace") {
            if let Some(steps) = trace.get("steps") {
                debug!("Found trace.steps structure");
                self.extract_from_steps_data(steps, &mut trees, &mut step_counter)?;
            }
        }

        if trees.is_empty() {
            return Ok("📝 No transaction logs found in execution result".to_string());
        }

        let root = Tree::Node("🔗 Blockchain Transactions".to_string(), trees);
        self.render_tree(&root)
    }

    /// Extract transaction logs from steps data
    fn extract_from_steps_data(
        &self,
        steps_data: &Value,
        trees: &mut Vec<Tree>,
        step_counter: &mut usize,
    ) -> Result<()> {
        if let Some(steps_array) = steps_data.as_array() {
            for step in steps_array {
                if let Some(observation) = step.get("observation") {
                    if let Some(tx_logs) = observation.get("last_transaction_logs") {
                        if let Some(tx_logs_array) = tx_logs.as_array() {
                            let logs: Vec<String> = tx_logs_array
                                .iter()
                                .filter_map(|log| log.as_str())
                                .map(|s| s.to_string())
                                .collect();

                            let error = observation
                                .get("last_transaction_error")
                                .and_then(|e| e.as_str());

                            let step_tree = self.create_step_tree(*step_counter, &logs, error)?;
                            trees.push(step_tree);
                            *step_counter += 1;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Create a tree for a single step's transaction logs with proper ASCII tree structure
    fn create_step_tree(
        &self,
        step_num: usize,
        logs: &[String],
        error: Option<&str>,
    ) -> Result<Tree> {
        let mut children = Vec::new();

        // Add error if present
        if let Some(err) = error {
            children.push(Tree::Leaf(vec![format!("❌ Error: {}", err)]));
        }

        // Parse transaction logs and create tree structure
        let mut current_indent = 0;
        let mut log_groups = Vec::new();
        let mut current_group = Vec::new();

        for log_line in logs {
            let indent_level = (log_line.len() - log_line.trim_start().len()) / 2;

            // If indentation changes, start a new group
            if indent_level != current_indent && !current_group.is_empty() {
                if !current_group.is_empty() {
                    log_groups.push((current_indent, current_group.clone()));
                    current_group.clear();
                }
                current_indent = indent_level;
            }

            current_group.push(log_line.clone());
        }

        // Add the last group
        if !current_group.is_empty() {
            log_groups.push((current_indent, current_group));
        }

        // Convert each group to a tree node
        for (indent_level, group_logs) in log_groups {
            if group_logs.len() == 1 {
                // Single log entry
                let formatted = self.format_single_log(&group_logs[0]);
                children.push(Tree::Leaf(vec![formatted]));
            } else {
                // Multiple log entries at same level - create a group
                let group_children: Vec<Tree> = group_logs
                    .iter()
                    .map(|log| Tree::Leaf(vec![self.format_single_log(log)]))
                    .collect();

                let group_label = self.get_group_label(&group_logs[0], indent_level);
                children.push(Tree::Node(group_label, group_children));
            }
        }

        Ok(Tree::Node(
            format!("Step {step_num}: Transaction Execution"),
            children,
        ))
    }

    /// Get appropriate label for a group of logs
    fn get_group_label(&self, first_log: &str, indent_level: usize) -> String {
        let trimmed = first_log.trim();

        if trimmed.contains("invoke [") {
            if let Some(program_name) = self.extract_program_name(trimmed) {
                return format!(
                    "{}{} {}",
                    "  ".repeat(indent_level),
                    self.get_program_icon_for_name(&program_name),
                    program_name
                );
            }
        }

        format!("{}Program Operations", "  ".repeat(indent_level))
    }

    /// Extract program name from log line
    fn extract_program_name(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find("invoke [") {
            let after_invoke = &line[start + "invoke [".len()..];
            if let Some(end) = after_invoke.find(']') {
                let instruction = &after_invoke[..end];
                return Some(instruction.to_string());
            }
        }
        None
    }

    /// Format individual transaction log line with appropriate icons and styling
    fn format_single_log(&self, log_str: &str) -> String {
        let trimmed = log_str.trim();

        // Add icons for different types of transaction logs
        if trimmed.contains("invoke [") {
            let program_name = self
                .extract_program_name(trimmed)
                .unwrap_or_else(|| "Unknown".to_string());
            let icon = self.get_program_icon_for_name(&program_name);
            format!(
                "{}{} invoke",
                "  ".repeat((log_str.len() - trimmed.len()) / 2),
                icon
            )
        } else if trimmed.contains("success") {
            format!(
                "{}✅ {}",
                "  ".repeat((log_str.len() - trimmed.len()) / 2),
                self.get_program_icon_for_line(trimmed)
            )
        } else if trimmed.contains("compute units") {
            format!(
                "{}⚡ {}",
                "  ".repeat((log_str.len() - trimmed.len()) / 2),
                trimmed
            )
        } else if trimmed.contains("Program log:") {
            format!(
                "{}📝 {}",
                "  ".repeat((log_str.len() - trimmed.len()) / 2),
                trimmed
            )
        } else if trimmed.contains("Program return:") {
            format!(
                "{}↩️ {}",
                "  ".repeat((log_str.len() - trimmed.len()) / 2),
                trimmed
            )
        } else {
            format!(
                "{}{}",
                "  ".repeat((log_str.len() - trimmed.len()) / 2),
                trimmed
            )
        }
    }

    /// Get program icon based on program name
    fn get_program_icon_for_name(&self, program_name: &str) -> &str {
        match program_name.to_lowercase().as_str() {
            "transfer" | "initializeaccount" | "initializeaccount2" | "initializeaccount3"
            | "mintto" | "transferchecked" | "closeaccount" => "🪙",
            "deposit" | "withdraw" | "operate" | "preoperate" => "💰",
            _ => "📦",
        }
    }

    /// Get program icon based on log line
    fn get_program_icon_for_line(&self, line: &str) -> &str {
        if line.contains("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") {
            "🪙"
        } else if line.contains("11111111111111111111111111111111111") {
            "🔧"
        } else if line.contains("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL") {
            "💱"
        } else if line.contains("jup3YeL8QhtSx1e253b2FDvsMNC87fDrgQZivbrndc9")
            || line.contains("jupeiUmn818Jg1ekPURTpr4mFo29p46vygyykFJ3wZC")
        {
            "💰"
        } else {
            "📦"
        }
    }

    /// Extract program ID from log line
    fn extract_program_id(&self, line: &str) -> Option<String> {
        // Look for program ID patterns
        if let Some(start) = line.find("invoke [") {
            let after_invoke = &line[start + "invoke [".len()..];
            if let Some(end) = after_invoke.find(']') {
                let potential_id = &after_invoke[..end];
                // Check if it looks like a base58 string
                if potential_id.len() > 30 && potential_id.chars().all(|c| c.is_alphanumeric()) {
                    return Some(potential_id.to_string());
                }
            }
        }
        None
    }

    /// Extract compute units from log line
    fn extract_compute_units(&self, line: &str) -> Option<u64> {
        // Look for CU usage patterns like "12345 compute units" or "12345 CU"
        if let Some(cu_start) = line.find("compute units") {
            let before_cu = &line[..cu_start].trim();
            if let Some(space_pos) = before_cu.rfind(' ') {
                if let Ok(cu) = before_cu[space_pos + 1..].parse::<u64>() {
                    return Some(cu);
                }
            }
        } else if let Some(cu_start) = line.find(" CU") {
            let before_cu = &line[..cu_start].trim();
            if let Some(space_pos) = before_cu.rfind(' ') {
                if let Ok(cu) = before_cu[space_pos + 1..].parse::<u64>() {
                    return Some(cu);
                }
            }
        }
        None
    }

    /// Get program icon based on program ID
    fn get_program_icon(&self, program_id: &str) -> &'static str {
        match program_id {
            "11111111111111111111111111111111111" => "🔧", // System Program
            "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA" => "🪙", // Token Program
            "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL" => "💱", // Associated Token Program
            "Sysvar111111111111111111111111111111111111" => "⚙️", // Sysvars
            "SysvarRent111111111111111111111111111111111111" => "🏦", // Rent Sysvar
            "SysvarC1ock11111111111111111111111111111111111" => "🕐", // Clock Sysvar
            "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4" => "💰", // Jupiter
            "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc" => "🏛️", // Orca
            _ if program_id.contains("11111111111111111111111111111") => "🔧", // Partial System Program
            _ if program_id.contains("Token") => "🪙", // Partial Token Program
            _ if program_id.contains("Sysvar") => "⚙️", // Partial Sysvar
            _ => "📦",                                 // Default program icon
        }
    }

    /// Render tree to string
    fn render_tree(&self, tree: &Tree) -> Result<String> {
        let mut buffer = String::new();
        ascii_tree::write_tree(&mut buffer, tree)?;
        Ok(buffer)
    }

    /// Generate error trace
    pub fn generate_error_trace(&self, error_message: &str, execution_id: &str) -> String {
        format!(
            "❌ Transaction Log Error\n\
             📋 Execution ID: {execution_id}\n\
             🚨 Error: {error_message}\n\
             \n\
             💡 This might indicate:\n\
             • No transactions were executed\n\
             • Transaction logs are not available\n\
             • Execution failed before transaction phase"
        )
    }
}

/// Transaction log entry structure (unused in new implementation but kept for compatibility)
#[derive(Debug, Clone)]
struct TransactionLogEntry {
    level: String,
    program_id: String,
    instruction: String,
    depth: usize,
    compute_units: Option<u64>,
    is_instruction: bool,
    is_success: bool,
    log_message: String,
}
