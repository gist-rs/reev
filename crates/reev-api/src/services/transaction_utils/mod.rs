//! Transaction utilities for processing and formatting transaction logs
//!
//! This module provides utilities for generating and formatting transaction logs
//! from benchmark execution results, supporting both plain text and YAML formats.

#![allow(dead_code)]

use anyhow::Result;
use reev_lib::results::TestResult;

/// Generate transaction logs from test result
pub fn generate_transaction_logs(result: &TestResult) -> String {
    let mut logs = String::new();

    for (i, step) in result.trace.steps.iter().enumerate() {
        logs.push_str(&format!("Step {}:\n", i + 1));

        for log in &step.observation.last_transaction_logs {
            logs.push_str(&format!("  {log}\n"));
        }

        if let Some(error) = &step.observation.last_transaction_error {
            logs.push_str(&format!("  Error: {error}\n"));
        }
    }

    logs
}

/// Generate transaction logs as YAML format (placeholder implementation)
pub fn generate_transaction_logs_yaml(logs: &[String], _show_cu: bool) -> Result<String> {
    let parsed_logs = parse_transaction_logs(logs);

    // For now, just return a simple formatted string
    let mut output = String::new();
    output.push_str("Transaction Logs:\n");

    for (i, entry) in parsed_logs.iter().enumerate() {
        output.push_str(&format!(
            "  {}: [{}] {}\n",
            i + 1,
            entry.program_name,
            entry.instruction
        ));
    }

    Ok(output)
}

/// Log entry structure for parsed transaction logs
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub level: String,
    pub program_id: String,
    pub program_name: String,
    pub instruction: String,
    pub log_message: String,
    pub compute_units: Option<u64>,
    pub is_instruction: bool,
    pub is_success: bool,
    pub is_last_child: bool,
    pub return_data: Option<String>,
}

/// Parse transaction logs into structured entries
pub fn parse_transaction_logs(logs: &[String]) -> Vec<LogEntry> {
    let mut entries = Vec::new();

    for log_line in logs {
        if let Ok(entry) = parse_log_entry(log_line) {
            entries.push(entry);
        }
    }

    entries
}

/// Parse individual log entry
fn parse_log_entry(line: &str) -> Result<LogEntry> {
    // Simple parsing logic - this would need to match the actual log format
    // For now, create a basic entry
    Ok(LogEntry {
        level: "INFO".to_string(),
        program_id: "unknown".to_string(),
        program_name: "unknown".to_string(),
        instruction: line.trim().to_string(),
        log_message: line.trim().to_string(),
        compute_units: None,
        is_instruction: false,
        is_success: true,
        is_last_child: false,
        return_data: None,
    })
}

/// Create simple structure from parsed logs (placeholder)
pub fn create_tree_from_logs(_entries: &[LogEntry], _show_cu: bool) -> Result<String> {
    // Placeholder implementation - will be replaced when text_trees is available
    Ok("Transaction tree placeholder".to_string())
}

/// Create entry representation (placeholder)
pub fn create_entry_node(entry: &LogEntry, _show_cu: bool) -> String {
    format!(
        "[{}] {} - {}",
        get_program_icon(entry.program_id.as_str()),
        entry.program_name,
        entry.instruction
    )
}

/// Get program icon based on program ID
fn get_program_icon(program_id: &str) -> &'static str {
    match program_id {
        "11111111111111111111111111111111111" => "🔧", // System Program
        "TokenkeQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA" => "🪙", // Token Program
        "9xQeWcvG8sn3phJHPwE4VG5pZxCch9u4AEhUk" => "💱", // Associated Token Program
        "Sysvar111111111111111111111111111111111111" => "⚙️", // Sysvars
        "SysvarRent111111111111111111111111111111111111" => "🏦", // Rent Sysvar
        "SysvarC1ock11111111111111111111111111111111111" => "🕐", // Clock Sysvar
        _ => "📦",                                     // Default program icon
    }
}
