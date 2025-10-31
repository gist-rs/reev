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

/// Transaction log parser for converting blockchain transaction data to ASCII format
#[derive(Debug, Clone)]
pub struct TransactionLogParser {
    /// Show compute units in the output
    show_compute_units: bool,
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
                println!(
                    "DEBUG: Found {} transaction logs for step {}",
                    step.observation.last_transaction_logs.len(),
                    step_idx + 1
                );
                let step_tree = self.create_step_tree(
                    step_idx + 1,
                    &step.observation.last_transaction_logs,
                    step.observation.last_transaction_error.as_deref(),
                )?;
                trees.push(step_tree);
            } else {
                println!("DEBUG: No transaction logs found for step {}", step_idx + 1);
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
                println!("DEBUG: Successfully parsed as TestResult, generating transaction logs");
                return self.generate_transaction_logs(&test_result);
            } else {
                println!("DEBUG: Failed to parse as TestResult");
            }
        }

        // Fallback: extract logs directly from JSON structure
        println!("DEBUG: Using fallback JSON structure extraction");
        self.extract_from_json_structure(result_data)
    }

    /// Extract transaction logs from JSON structure
    fn extract_from_json_structure(&self, result_data: &Value) -> Result<String> {
        let mut trees = Vec::new();
        let mut step_counter = 1;

        println!("DEBUG: Extracting from JSON structure");

        // Check different possible structures
        if let Some(final_result) = result_data.get("final_result") {
            if let Some(data) = final_result.get("data") {
                println!("DEBUG: Found final_result.data structure");
                self.extract_from_steps_data(data, &mut trees, &mut step_counter)?;
            }
        }

        if trees.is_empty() {
            return Ok("📝 No transaction logs found in execution result".to_string());
        }

        if let Some(trace) = result_data.get("trace") {
            if let Some(steps) = trace.get("steps") {
                println!("DEBUG: Found trace.steps structure");
                self.extract_from_steps_data(steps, &mut trees, &mut step_counter)?;
            }
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
    ) -> Result<(), anyhow::Error> {
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

                            println!(
                                "DEBUG: Found {} transaction logs for step {}",
                                logs.len(),
                                step_counter
                            );

                            let error = observation
                                .get("last_transaction_error")
                                .and_then(|e| e.as_str());

                            let step_tree = self.create_step_tree(*step_counter, &logs, error)?;
                            trees.push(step_tree);
                            *step_counter += 1;
                        }
                    } else {
                        println!("DEBUG: No last_transaction_logs found in observation");
                    }
                } else {
                    println!("DEBUG: No observation found in step");
                }
            }
        }
        Ok(())
    }

    /// Create a tree for a single step's transaction logs
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

        // Parse and add transaction logs
        let parsed_logs = self.parse_transaction_logs(logs);
        for log_entry in parsed_logs {
            let node = self.create_log_node(&log_entry)?;
            children.push(node);
        }

        Ok(Tree::Node(
            format!("Step {}: Transaction Execution", step_num),
            children,
        ))
    }

    /// Parse transaction logs into structured entries
    fn parse_transaction_logs(&self, logs: &[String]) -> Vec<TransactionLogEntry> {
        let mut entries = Vec::new();
        let mut current_depth = 0;

        for log_line in logs {
            if let Ok(entry) = self.parse_log_entry(log_line, &mut current_depth) {
                entries.push(entry);
            }
        }

        entries
    }

    /// Parse individual log entry
    fn parse_log_entry(
        &self,
        line: &str,
        current_depth: &mut usize,
    ) -> Result<TransactionLogEntry> {
        let trimmed = line.trim();

        // Detect instruction calls
        if trimmed.contains("invoke [") {
            let depth = (line.len() - trimmed.len()) / 2;
            *current_depth = depth;

            if let Some(start) = trimmed.find('[') {
                if let Some(end) = trimmed.find(']') {
                    let instruction = &trimmed[start + 1..end];
                    let program_id = self.extract_program_id(trimmed);

                    return Ok(TransactionLogEntry {
                        level: "INFO".to_string(),
                        program_id: program_id.unwrap_or("unknown".to_string()),
                        instruction: instruction.to_string(),
                        depth,
                        compute_units: self.extract_compute_units(trimmed),
                        is_instruction: true,
                        is_success: !trimmed.contains("failed"),
                        log_message: trimmed.to_string(),
                    });
                }
            }
        }

        // Detect log entries
        if trimmed.contains("Program log:") {
            let depth = (line.len() - trimmed.len()) / 2;
            if let Some(log_start) = trimmed.find("Program log:") {
                let log_msg = &trimmed[log_start + "Program log:".len()..];
                return Ok(TransactionLogEntry {
                    level: "LOG".to_string(),
                    program_id: "program".to_string(),
                    instruction: "".to_string(),
                    depth: depth + 1, // Indent logs under instructions
                    compute_units: None,
                    is_instruction: false,
                    is_success: true,
                    log_message: log_msg.trim().to_string(),
                });
            }
        }

        // Default entry
        let depth = (line.len() - trimmed.len()) / 2;
        Ok(TransactionLogEntry {
            level: "INFO".to_string(),
            program_id: "system".to_string(),
            instruction: "".to_string(),
            depth,
            compute_units: None,
            is_instruction: false,
            is_success: true,
            log_message: trimmed.to_string(),
        })
    }

    /// Create a tree node from a log entry
    fn create_log_node(&self, entry: &TransactionLogEntry) -> Result<Tree> {
        let icon = self.get_program_icon(&entry.program_id);
        let status = if entry.is_success { "✅" } else { "❌" };

        let label = if entry.is_instruction {
            let cu_info = if self.show_compute_units {
                entry
                    .compute_units
                    .map(|cu| format!(" ({cu} CU)"))
                    .unwrap_or_default()
            } else {
                String::new()
            };
            format!("{} {} {}{}", status, icon, entry.instruction, cu_info)
        } else if !entry.log_message.is_empty() {
            format!("📝 {}", entry.log_message)
        } else {
            format!("{} {}", icon, entry.log_message)
        };

        Ok(Tree::Leaf(vec![label]))
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
            "11111111111111111111111111111111" => "🔧", // System Program
            "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA" => "🪙", // Token Program
            "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL" => "💱", // Associated Token Program
            "Sysvar111111111111111111111111111111111111" => "⚙️", // Sysvars
            "SysvarRent111111111111111111111111111111111111" => "🏦", // Rent Sysvar
            "SysvarC1ock11111111111111111111111111111111111" => "🕐", // Clock Sysvar
            "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4" => "💰", // Jupiter
            "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc" => "🏛️", // Orca
            _ if program_id.contains("11111111111111111111111111") => "🔧", // Partial System Program
            _ if program_id.contains("Token") => "🪙",                      // Partial Token Program
            _ if program_id.contains("Sysvar") => "⚙️",                     // Partial Sysvar
            _ => "📦",                                                      // Default program icon
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
             📋 Execution ID: {}\n\
             🚨 Error: {}\n\
             \n\
             💡 This might indicate:\n\
             • No transactions were executed\n\
             • Transaction logs are not available\n\
             • Execution failed before transaction phase",
            execution_id, error_message
        )
    }
}

/// Transaction log entry structure
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
