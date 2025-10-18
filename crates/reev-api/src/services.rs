use crate::types::*;
use anyhow::Result;
use reev_flow::{
    EventContent, ExecutionResult, ExecutionStatistics, FlowEvent, FlowEventType, FlowLog,
};
use reev_lib::db::DatabaseWriter;
use reev_lib::results::TestResult;
use std::path::PathBuf;
use std::time::SystemTime;
use tracing::{debug, error, info};
use uuid::Uuid;

/// Background task to execute benchmark
pub async fn execute_benchmark_background(
    state: ApiState,
    execution_id: String,
    benchmark_id: String,
    agent: String,
) {
    // Create database session with Running status
    let session_id = uuid::Uuid::new_v4().to_string();
    let session_info = reev_db::types::SessionInfo {
        session_id: session_id.clone(),
        benchmark_id: benchmark_id.clone(),
        agent_type: agent.clone(),
        interface: "Web".to_string(),
        start_time: chrono::Utc::now().timestamp(),
        end_time: None,
        status: "running".to_string(),
        score: None,
        final_status: None,
    };

    match state.db.create_session(&session_info).await {
        Ok(_) => {
            info!(
                "Created database session: {} for benchmark: {}",
                session_id, benchmark_id
            );
        }
        Err(e) => {
            error!("Failed to create database session: {}", e);
        }
    }

    // Update status to running
    {
        let mut executions = state.executions.lock().await;
        if let Some(execution) = executions.get_mut(&execution_id) {
            execution.status = ExecutionStatus::Running;
            execution.progress = 10;
            execution.trace = format!("Starting benchmark {benchmark_id} with agent {agent}\n");
        }
    }

    info!(
        "Executing benchmark: {} with agent: {}",
        benchmark_id, agent
    );

    // Find the benchmark file
    let benchmark_path = find_benchmark_file(&benchmark_id);
    let benchmark_path = match benchmark_path {
        Some(path) => path,
        None => {
            error!("Benchmark file not found: {}", benchmark_id);
            update_execution_failed(
                &state,
                &execution_id,
                &session_id,
                "Benchmark file not found",
            )
            .await;
            return;
        }
    };

    // Update progress
    {
        let mut executions = state.executions.lock().await;
        if let Some(execution) = executions.get_mut(&execution_id) {
            execution.progress = 30;
            execution.trace.push_str(&format!(
                "Found benchmark file: {}\n",
                benchmark_path.display()
            ));
        }
    }

    // Update progress - starting dependencies
    {
        let mut executions = state.executions.lock().await;
        if let Some(execution) = executions.get_mut(&execution_id) {
            execution.progress = 40;
            execution.trace.push_str("Initializing dependencies...\n");
        }
    }

    // Execute the benchmark using the real runner
    let execution_result = match reev_runner::run_benchmarks(benchmark_path.clone(), &agent).await {
        Ok(mut results) => {
            if let Some(result) = results.pop() {
                Ok(result)
            } else {
                Err(anyhow::anyhow!("Benchmark runner returned no results"))
            }
        }
        Err(e) => Err(e),
    };

    // Update progress - benchmark execution complete
    {
        let mut executions = state.executions.lock().await;
        if let Some(execution) = executions.get_mut(&execution_id) {
            execution.progress = 80;
            execution
                .trace
                .push_str("Benchmark execution completed, processing results...\n");
        }
    }

    match execution_result {
        Ok(test_result) => {
            // Update progress - generating results
            {
                let mut executions = state.executions.lock().await;
                if let Some(execution) = executions.get_mut(&execution_id) {
                    execution.progress = 90;
                    execution
                        .trace
                        .push_str("Generating execution trace and logs...\n");
                }
            }

            // Generate ASCII tree trace from the actual result
            info!("Generating ASCII tree trace from test result...");
            let ascii_trace = reev_runner::renderer::render_result_as_tree(&test_result);
            info!("ASCII tree generated, length: {} chars", ascii_trace.len());

            // Generate transaction logs from the trace
            let transaction_logs = generate_transaction_logs(&test_result);
            info!(
                "Transaction logs generated, length: {} chars",
                transaction_logs.len()
            );

            // Calculate score as percentage
            let score_percentage = test_result.score * 100.0;

            // Agent performance is stored by FlowLogger::complete() in the runner
            // to avoid duplicates and maintain proper execution tracking
            let final_status = match test_result.final_status {
                reev_lib::results::FinalStatus::Succeeded => "Succeeded",
                reev_lib::results::FinalStatus::Failed => "Failed",
            };

            // Update database session with final status and full execution log
            let full_execution_log = format!(
                "{}\nExecution completed with status: {}\nScore: {:.1}%\n\n{}",
                ascii_trace,
                final_status,
                score_percentage,
                transaction_logs.clone()
            );

            // Store the complete execution log
            if let Err(e) = state
                .db
                .store_complete_log(&session_id, &full_execution_log)
                .await
            {
                error!("Failed to store execution log: {}", e);
            }

            // Complete the session with results
            let session_result = reev_db::types::SessionResult {
                end_time: chrono::Utc::now().timestamp(),
                score: test_result.score,
                final_status: final_status.to_string(),
            };

            if let Err(e) = state
                .db
                .complete_session(&session_id, &session_result)
                .await
            {
                error!("Failed to complete database session: {}", e);
            } else {
                info!(
                    "Completed database session {} with final status: {}",
                    session_id, final_status
                );
            }

            {
                let mut executions = state.executions.lock().await;
                if let Some(execution) = executions.get_mut(&execution_id) {
                    execution.status = ExecutionStatus::Completed;
                    execution.progress = 100;
                    execution.end_time = Some(chrono::Utc::now());
                    execution.trace = ascii_trace.clone();
                    execution.logs = transaction_logs;

                    info!(
                        "Benchmark {} completed with score: {:.1}%, trace length: {}",
                        benchmark_id,
                        score_percentage,
                        ascii_trace.len()
                    );

                    // Store YML TestResult in database for historical access
                    if let Err(e) =
                        store_yml_testresult(&state.db, &benchmark_id, &agent, &test_result).await
                    {
                        error!("Failed to store YML TestResult in database: {}", e);
                    } else {
                        info!("YML TestResult stored in database for historical access");
                    }

                    // Debug: Log first and last parts of the trace
                    if !ascii_trace.is_empty() {
                        let first_part = if ascii_trace.len() > 100 {
                            ascii_trace.chars().take(100).collect::<String>()
                        } else {
                            ascii_trace.clone()
                        };
                        let last_part = if ascii_trace.len() > 100 {
                            ascii_trace
                                .chars()
                                .skip(ascii_trace.len() - 100)
                                .collect::<String>()
                        } else {
                            String::new()
                        };
                        debug!("Trace first 100 chars: {}", first_part);
                        debug!("Trace last 100 chars: {}", last_part);
                    }
                }
            }

            // Store flow log in database
            if let Err(e) = store_flow_log_from_result(&state.db, &benchmark_id, &test_result).await
            {
                error!("Failed to store flow log: {}", e);
            }
        }
        Err(e) => {
            error!("Benchmark execution failed: {}", e);
            update_execution_failed(
                &state,
                &execution_id,
                &session_id,
                &format!("Execution failed: {e}"),
            )
            .await;
        }
    }

    info!("Benchmark execution completed: {}", execution_id);
}

/// Find benchmark file by ID
pub fn find_benchmark_file(benchmark_id: &str) -> Option<PathBuf> {
    let benchmarks_dir = std::path::Path::new("benchmarks");

    if benchmarks_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(benchmarks_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(file_name) = path.file_name() {
                        if let Some(name_str) = file_name.to_str() {
                            if name_str.starts_with(benchmark_id)
                                || name_str == benchmark_id
                                || name_str == format!("{benchmark_id}.yml")
                                || name_str == format!("{benchmark_id}.yaml")
                            {
                                return Some(path);
                            }
                        }
                    }
                }
            }
        }
    }

    // Try direct path
    let direct_path = std::path::Path::new(benchmark_id);
    if direct_path.exists() {
        return Some(direct_path.to_path_buf());
    }

    None
}

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

        logs.push('\n');
    }

    logs
}

/// Generate beautiful ASCII tree visualization of transaction logs
pub fn generate_transaction_logs_tree(result: &TestResult) -> String {
    let mut tree = String::new();

    // Header
    tree.push_str(&format!(
        "🔄 TRANSACTION: {} | ID: {}\n\n",
        result
            .trace
            .prompt
            .split_whitespace()
            .next()
            .unwrap_or("unknown"),
        &result.id
    ));

    let mut total_programs = std::collections::HashSet::new();
    let mut total_instructions = 0;
    let mut total_compute_units = 0u64;
    let mut has_errors = false;

    for (step_idx, step) in result.trace.steps.iter().enumerate() {
        if step_idx > 0 {
            tree.push('\n');
        }

        tree.push_str(&format!("Step {}\n", step_idx + 1));

        // Parse and render transaction logs as tree
        let parsed_logs = parse_transaction_logs(&step.observation.last_transaction_logs);
        for (log_idx, log_entry) in parsed_logs.iter().enumerate() {
            render_log_entry(&mut tree, log_entry, log_idx == parsed_logs.len() - 1, 1);

            // Collect statistics
            if let Some(program_id) = &log_entry.program_id {
                total_programs.insert(program_id.clone());
            }
            if log_entry.is_instruction {
                total_instructions += 1;
            }
            total_compute_units += log_entry.compute_units.unwrap_or(0);
            if log_entry.is_error {
                has_errors = true;
            }
        }

        if let Some(error) = &step.observation.last_transaction_error {
            tree.push_str(&format!("  ❌ Error: {error}\n"));
            has_errors = true;
        }
    }

    // Summary section
    tree.push_str("\n📊 SUMMARY\n");
    tree.push_str(&format!("├─ Total Programs: {}\n", total_programs.len()));
    tree.push_str(&format!("├─ Total Instructions: {total_instructions}\n"));
    tree.push_str(&format!(
        "├─ Total Compute Units: ~{}K\n",
        total_compute_units / 1000
    ));
    tree.push_str(&format!(
        "└─ Status: {}\n",
        if has_errors {
            "❌ FAILED"
        } else {
            "✅ SUCCESS"
        }
    ));

    tree
}

/// Parsed transaction log entry
#[derive(Debug, Clone)]
struct LogEntry {
    level: usize,
    program_id: Option<String>,
    program_name: Option<String>,
    instruction: Option<String>,
    log_message: Option<String>,
    compute_units: Option<u64>,
    is_instruction: bool,
    is_success: bool,
    is_error: bool,
    is_last_child: bool,
    return_data: Option<String>,
}

/// Parse raw transaction log lines into structured entries
fn parse_transaction_logs(logs: &[String]) -> Vec<LogEntry> {
    let mut entries = Vec::new();
    let mut program_stack: Vec<(String, usize)> = Vec::new();

    // Pre-compile regex patterns for performance
    let program_invoke_regex =
        regex::Regex::new(r"Program ([a-zA-Z0-9]+) invoke \[(\d+)\]").unwrap();
    let compute_units_regex = regex::Regex::new(r"consumed (\d+) of (\d+) compute units").unwrap();
    let program_return_regex = regex::Regex::new(r"Program return: ([a-zA-Z0-9]+) (.+)").unwrap();

    for log_line in logs {
        let trimmed = log_line.trim();

        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }

        // Parse program invocation
        if let Some(caps) = program_invoke_regex.captures(trimmed) {
            let program_id = caps[1].to_string();
            let level = caps[2].parse::<usize>().unwrap_or(0);

            // Update stack for nesting
            while program_stack.len() > level {
                program_stack.pop();
            }
            program_stack.push((program_id.clone(), entries.len()));

            entries.push(LogEntry {
                level,
                program_id: Some(program_id.clone()),
                program_name: get_program_name(&program_id),
                instruction: None,
                log_message: None,
                compute_units: None,
                is_instruction: false,
                is_success: false,
                is_error: false,
                is_last_child: false,
                return_data: None,
            });
            continue;
        }

        // Parse program success
        if trimmed.contains("Program") && trimmed.contains("success") {
            if let Some(program_id) = extract_program_id_from_success(trimmed) {
                // Find the level of this program in the stack
                let level = program_stack
                    .iter()
                    .position(|(id, _)| id == &program_id)
                    .unwrap_or(0);

                entries.push(LogEntry {
                    level,
                    program_id: Some(program_id.clone()),
                    program_name: get_program_name(&program_id),
                    instruction: None,
                    log_message: None,
                    compute_units: None,
                    is_instruction: false,
                    is_success: true,
                    is_error: false,
                    is_last_child: false,
                    return_data: None,
                });
            }
            continue;
        }

        // Parse compute units
        if let Some(caps) = compute_units_regex.captures(trimmed) {
            let compute_units = caps[1].parse::<u64>().unwrap_or(0);
            // Find the most recent program entry
            for entry in entries.iter_mut().rev() {
                if !entry.is_success && entry.compute_units.is_none() {
                    entry.compute_units = Some(compute_units);
                    break;
                }
            }
            continue;
        }

        // Parse instruction log
        if trimmed.contains("Program log: Instruction:") {
            let instruction = trimmed
                .replace("Program log: Instruction:", "")
                .trim()
                .to_string();
            if let Some(entry) = entries.last_mut() {
                entry.instruction = Some(instruction);
                entry.is_instruction = true;
            }
            continue;
        }

        // Parse program log
        if trimmed.contains("Program log:") && !trimmed.contains("Instruction:") {
            let log_msg = trimmed.replace("Program log:", "").trim().to_string();
            if let Some(entry) = entries.last_mut() {
                entry.log_message = Some(log_msg);
            }
            continue;
        }

        // Parse program return
        if trimmed.contains("Program return:") {
            if let Some(caps) = program_return_regex.captures(trimmed) {
                let _program_id = caps[1].to_string();
                let return_data = caps[2].to_string();
                if let Some(entry) = entries.last_mut() {
                    entry.return_data = Some(return_data);
                }
            }
            continue;
        }
    }

    // Mark last children for proper tree rendering
    mark_last_children(&mut entries);
    entries
}

/// Get human-readable program name from program ID
fn get_program_name(program_id: &str) -> Option<String> {
    match program_id {
        "11111111111111111111111111111111" => Some("System".to_string()),
        "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA" => Some("SPL Token".to_string()),
        "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL" => Some("Associated Token".to_string()),
        "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4" => Some("Jupiter Router".to_string()),
        "TessVdML9pBGgG9yGks7o4HewRaXVAMuoVj4x83GLQH" => Some("Tessellate".to_string()),
        "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM" => Some("Serum DEX".to_string()),
        "SysvarRent111111111111111111111111111111111" => Some("Sysvar Rent".to_string()),
        "SysvarC1ock11111111111111111111111111111111" => Some("Sysvar Clock".to_string()),
        _ => None,
    }
}

/// Extract program ID from success message
fn extract_program_id_from_success(message: &str) -> Option<String> {
    regex::Regex::new(r"Program ([a-zA-Z0-9]+) success")
        .unwrap()
        .captures(message)
        .map(|caps| caps[1].to_string())
}

/// Mark which entries are the last child of their parent
fn mark_last_children(entries: &mut [LogEntry]) {
    let mut i = 0;
    while i < entries.len() {
        let current_level = entries[i].level;
        let mut j = i + 1;

        // Find next entry at same or higher level
        while j < entries.len() && entries[j].level > current_level {
            j += 1;
        }

        // If no next entry at same level, this is last child
        if j >= entries.len() || entries[j].level < current_level {
            entries[i].is_last_child = true;
        }

        i += 1;
    }
}

/// Render a single log entry as ASCII tree
fn render_log_entry(
    tree: &mut String,
    entry: &LogEntry,
    _is_last_in_step: bool,
    _base_indent: usize,
) {
    let indent = "  ".repeat(entry.level);
    let connector = if entry.level == 0 {
        ""
    } else if entry.is_last_child {
        "└─ "
    } else {
        "├─ "
    };

    // Get appropriate icon
    let icon = get_program_icon(&entry.program_id);

    // Render program invocation
    if entry.program_name.is_some() && !entry.is_success {
        tree.push_str(&format!(
            "{}{}{} {} ({})\n",
            indent,
            connector,
            icon,
            entry.program_name.as_deref().unwrap_or("Unknown"),
            entry.program_id.as_deref().unwrap_or("")
        ));

        // Render instruction if present
        if let Some(instruction) = &entry.instruction {
            tree.push_str(&format!(
                "{}{}  ├─ 📋 Instruction: {}\n",
                indent,
                if entry.is_last_child { "   " } else { "│  " },
                instruction
            ));
        }

        // Render log message if present
        if let Some(log_msg) = &entry.log_message {
            if log_msg.starts_with("Please upgrade") {
                tree.push_str(&format!(
                    "{}{}  ├─ ⚠️  Log: {}\n",
                    indent,
                    if entry.is_last_child { "   " } else { "│  " },
                    log_msg
                ));
            } else {
                tree.push_str(&format!(
                    "{}{}  ├─ 📝 Log: {}\n",
                    indent,
                    if entry.is_last_child { "   " } else { "│  " },
                    log_msg
                ));
            }
        }

        // Render return data if present
        if let Some(return_data) = &entry.return_data {
            tree.push_str(&format!(
                "{}{}  ├─ 💾 Return: {}\n",
                indent,
                if entry.is_last_child { "   " } else { "│  " },
                return_data
            ));
        }

        // Render success if present
        if let Some(cu) = entry.compute_units {
            tree.push_str(&format!(
                "{}{}  └─ ✅ Success ({} CU)\n",
                indent,
                if entry.is_last_child { "   " } else { "│  " },
                cu
            ));
        }
    }
    // Render standalone success entry
    else if entry.is_success {
        let parent_indent = if entry.level > 0 {
            "  ".repeat(entry.level)
        } else {
            String::new()
        };
        let connector = if entry.level == 0 { "" } else { "└─ " };

        if let Some(cu) = entry.compute_units {
            tree.push_str(&format!(
                "{parent_indent}{connector}{icon} ✅ Success ({cu} CU)\n"
            ));
        } else {
            tree.push_str(&format!("{parent_indent}{connector}{icon} ✅ Success\n"));
        }
    }
}

/// Get appropriate icon for program type
fn get_program_icon(program_id: &Option<String>) -> &'static str {
    match program_id.as_deref() {
        Some("11111111111111111111111111111111") => "🔹",
        Some("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") => "🪙",
        Some("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL") => "🏦",
        Some("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4") => "🚀",
        Some("TessVdML9pBGgG9yGks7o4HewRaXVAMuoVj4x83GLQH") => "🔸",
        Some("9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM") => "📈",
        Some(_) => "📄",
        None => "❓",
    }
}

/// Test function to generate tree visualization with mock data
pub fn generate_mock_transaction_logs_tree() -> String {
    let mock_logs = vec![
        "Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL invoke [1]".to_string(),
        "Program log: CreateIdempotent".to_string(),
        "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]".to_string(),
        "Program log: Instruction: GetAccountDataSize".to_string(),
        "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 1569 of 997595 compute units".to_string(),
        "Program return: TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA pQAAAAAAAAA=".to_string(),
        "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success".to_string(),
        "Program 11111111111111111111111111111111 invoke [2]".to_string(),
        "Program 11111111111111111111111111111111 success".to_string(),
        "Program log: Initialize the associated token account".to_string(),
        "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]".to_string(),
        "Program log: Instruction: InitializeImmutableOwner".to_string(),
        "Program log: Please upgrade to SPL Token 2022 for immutable owner support".to_string(),
        "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 1405 of 991008 compute units".to_string(),
        "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success".to_string(),
        "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]".to_string(),
        "Program log: Instruction: InitializeAccount3".to_string(),
        "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 3158 of 987126 compute units".to_string(),
        "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success".to_string(),
        "Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL consumed 19315 of 1003000 compute units".to_string(),
        "Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL success".to_string(),
        "Program 11111111111111111111111111111111 invoke [1]".to_string(),
        "Program 11111111111111111111111111111111 success".to_string(),
        "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [1]".to_string(),
        "Program log: Instruction: SyncNative".to_string(),
        "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 3045 of 983535 compute units".to_string(),
        "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success".to_string(),
    ];

    let mut tree = String::new();
    tree.push_str("🔄 TRANSACTION: 100-jup-swap-sol-usdc | ID: mock-test\n\n");

    let parsed_logs = parse_transaction_logs(&mock_logs);
    for (log_idx, log_entry) in parsed_logs.iter().enumerate() {
        render_log_entry(&mut tree, log_entry, log_idx == parsed_logs.len() - 1, 1);
    }

    tree.push_str("\n📊 SUMMARY\n");
    tree.push_str("├─ Total Programs: 3\n");
    tree.push_str("├─ Total Instructions: 6\n");
    tree.push_str("├─ Total Compute Units: ~28K\n");
    tree.push_str("└─ Status: ✅ SUCCESS\n");

    tree
}

/// Update execution as failed
pub async fn update_execution_failed(
    state: &ApiState,
    execution_id: &str,
    session_id: &str,
    error_message: &str,
) {
    let mut executions = state.executions.lock().await;
    if let Some(execution) = executions.get_mut(execution_id) {
        execution.status = ExecutionStatus::Failed;
        execution.progress = 100;
        execution.end_time = Some(chrono::Utc::now());
        execution.error = Some(error_message.to_string());
        execution
            .trace
            .push_str(&format!("ERROR: {error_message}\n"));
    }

    // Update database session with failed status
    if let Err(e) = state.db.update_session_status(session_id, "failed").await {
        error!(
            "Failed to update database session with failed status: {}",
            e
        );
    } else {
        info!("Updated database session {} with failed status", session_id);
    }

    // Store error log
    let error_log = format!("Execution failed: {error_message}");
    if let Err(e) = state.db.store_complete_log(session_id, &error_log).await {
        error!("Failed to store error log: {}", e);
    }
}

/// Store benchmark result in database
// Benchmark result storage is now handled by FlowLogger::complete() to avoid duplicates
// This function has been removed to prevent duplicate entries in agent_performance table
/// Store flow log in database from test result
pub async fn store_flow_log_from_result(
    db: &DatabaseWriter,
    benchmark_id: &str,
    test_result: &TestResult,
) -> Result<()> {
    let start_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let total_time_ms = (SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        - start_time)
        * 1000;

    let _flow_log = FlowLog {
        session_id: Uuid::new_v4().to_string(),
        benchmark_id: benchmark_id.to_string(),
        agent_type: "deterministic".to_string(), // This should come from the execution
        start_time: SystemTime::now(),
        end_time: Some(SystemTime::now()),
        events: vec![FlowEvent {
            timestamp: SystemTime::now(),
            event_type: FlowEventType::BenchmarkStateChange,
            depth: 0,
            content: EventContent {
                data: serde_json::json!({
                    "trace": test_result,
                    "status": if test_result.final_status == reev_lib::results::FinalStatus::Succeeded {
                        "completed"
                    } else {
                        "failed"
                    }
                }),
                metadata: std::collections::HashMap::new(),
            },
        }],
        final_result: Some(ExecutionResult {
            success: test_result.final_status == reev_lib::results::FinalStatus::Succeeded,
            score: test_result.score,
            total_time_ms,
            statistics: ExecutionStatistics {
                total_llm_calls: 0, // These should come from actual execution stats
                total_tool_calls: test_result.trace.steps.len() as u32,
                total_tokens: 0,
                tool_usage: std::collections::HashMap::new(),
                max_depth: 0,
            },
            scoring_breakdown: None,
        }),
    };

    // Store the execution trace as session log in new architecture
    let log_content = serde_json::to_string(&test_result.trace)
        .map_err(|e| anyhow::anyhow!("Failed to serialize execution trace to JSON: {e}"))?;

    // Create a session for this result
    let session_id = uuid::Uuid::new_v4().to_string();
    let start_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let session_info = reev_db::types::SessionInfo {
        session_id: session_id.clone(),
        benchmark_id: benchmark_id.to_string(),
        agent_type: "api".to_string(),
        interface: "web".to_string(),
        start_time: start_time as i64,
        end_time: Some(start_time as i64 + total_time_ms as i64 / 1000),
        status: if test_result.final_status == reev_lib::results::FinalStatus::Succeeded {
            "completed".to_string()
        } else {
            "failed".to_string()
        },
        score: Some(test_result.score),
        final_status: Some(format!("{:?}", test_result.final_status)),
    };

    // Create and complete session
    db.create_session(&session_info).await?;
    db.store_complete_log(&session_id, &log_content).await?;

    let session_result = reev_db::types::SessionResult {
        end_time: (start_time + total_time_ms / 1000) as i64,
        score: test_result.score,
        final_status: format!("{:?}", test_result.final_status),
    };

    db.complete_session(&session_id, &session_result).await?;
    Ok(())
}

/// Store flow log in database (legacy method for trace string)
#[allow(dead_code)]
pub async fn store_flow_log(
    db: &DatabaseWriter,
    benchmark_id: &str,
    trace_data: &str,
) -> Result<()> {
    let flow_log = FlowLog {
        session_id: Uuid::new_v4().to_string(),
        benchmark_id: benchmark_id.to_string(),
        agent_type: "deterministic".to_string(),
        start_time: SystemTime::now(),
        end_time: Some(SystemTime::now()),
        events: vec![FlowEvent {
            timestamp: SystemTime::now(),
            event_type: FlowEventType::BenchmarkStateChange,
            depth: 0,
            content: EventContent {
                data: serde_json::json!({
                    "trace": trace_data,
                    "status": "completed"
                }),
                metadata: std::collections::HashMap::new(),
            },
        }],
        final_result: Some(ExecutionResult {
            success: true,
            score: 1.0,
            total_time_ms: 5000,
            statistics: ExecutionStatistics {
                total_llm_calls: 0,
                total_tool_calls: 0,
                total_tokens: 0,
                tool_usage: std::collections::HashMap::new(),
                max_depth: 0,
            },
            scoring_breakdown: None,
        }),
    };

    // Store flow log as session data in new architecture
    let session_id = uuid::Uuid::new_v4().to_string();
    let start_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let session_info = reev_db::types::SessionInfo {
        session_id: session_id.clone(),
        benchmark_id: format!("flow_{}", flow_log.benchmark_id),
        agent_type: flow_log.agent_type.clone(),
        interface: "web".to_string(),
        start_time: start_time as i64,
        end_time: flow_log.end_time.map(|et| {
            et.duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64
        }),
        status: if flow_log
            .final_result
            .as_ref()
            .map(|r| r.success)
            .unwrap_or(false)
        {
            "completed".to_string()
        } else {
            "failed".to_string()
        },
        score: flow_log
            .final_result
            .as_ref()
            .map(|r| Some(r.score))
            .unwrap_or(None),
        final_status: Some(
            if flow_log
                .final_result
                .as_ref()
                .map(|r| r.success)
                .unwrap_or(false)
            {
                "Success".to_string()
            } else {
                "Failed".to_string()
            },
        ),
    };

    // Store session and log
    db.create_session(&session_info).await?;

    let log_content = serde_json::to_string(&flow_log)
        .map_err(|e| anyhow::anyhow!("Failed to serialize flow log to JSON: {e}"))?;

    db.store_complete_log(&session_id, &log_content).await?;
    Ok(())
}

/// Store YML TestResult in database for historical access
pub async fn store_yml_testresult(
    db: &DatabaseWriter,
    benchmark_id: &str,
    agent: &str,
    test_result: &TestResult,
) -> Result<()> {
    // Convert TestResult to YML string
    let yml_content = serde_yaml::to_string(test_result)
        .map_err(|e| anyhow::anyhow!("Failed to serialize TestResult to YML: {e}"))?;

    info!(
        "Attempting to store YML TestResult ({} chars) as session data",
        yml_content.len()
    );

    // Store YML TestResult as session data in new architecture
    let session_id = uuid::Uuid::new_v4().to_string();
    let start_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let session_info = reev_db::types::SessionInfo {
        session_id: session_id.clone(),
        benchmark_id: benchmark_id.to_string(),
        agent_type: agent.to_string(),
        interface: "web".to_string(),
        start_time: start_time as i64,
        end_time: Some(start_time as i64),
        status: if test_result.final_status == reev_lib::results::FinalStatus::Succeeded {
            "completed".to_string()
        } else {
            "failed".to_string()
        },
        score: Some(test_result.score),
        final_status: Some(format!("{:?}", test_result.final_status)),
    };

    // Create and complete session with YML content as log
    db.create_session(&session_info).await.map_err(|e| {
        error!("Failed to create session for YML TestResult: {:?}", e);
        e
    })?;

    db.store_complete_log(&session_id, &yml_content)
        .await
        .map_err(|e| {
            error!("Failed to store YML TestResult as session log: {:?}", e);
            e
        })?;

    let session_result = reev_db::types::SessionResult {
        end_time: start_time as i64,
        score: test_result.score,
        final_status: format!("{:?}", test_result.final_status),
    };

    db.complete_session(&session_id, &session_result)
        .await
        .map_err(|e| {
            error!("Failed to complete session for YML TestResult: {:?}", e);
            e
        })?;

    info!("Successfully stored YML TestResult as session data");

    info!("YML TestResult stored for benchmark: {benchmark_id} by agent: {agent}");
    Ok(())
}
