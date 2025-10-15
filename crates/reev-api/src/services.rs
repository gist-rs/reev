use crate::types::*;
use anyhow::Result;
use reev_lib::db::DatabaseWriter;
use reev_lib::flow::types::{
    EventContent, ExecutionResult, ExecutionStatistics, FlowEvent, FlowEventType, FlowLog,
};
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
            update_execution_failed(&state, &execution_id, "Benchmark file not found").await;
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

            // Agent performance is stored by FlowLogger::complete() in the runner
            // to avoid duplicates and maintain proper execution tracking
            let _final_status = match test_result.final_status {
                reev_lib::results::FinalStatus::Succeeded => "Succeeded",
                reev_lib::results::FinalStatus::Failed => "Failed",
            };

            // Store flow log in database
            if let Err(e) = store_flow_log_from_result(&state.db, &benchmark_id, &test_result).await
            {
                error!("Failed to store flow log: {}", e);
            }
        }
        Err(e) => {
            error!("Benchmark execution failed: {}", e);
            update_execution_failed(&state, &execution_id, &format!("Execution failed: {e}")).await;
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

/// Update execution as failed
pub async fn update_execution_failed(state: &ApiState, execution_id: &str, error_message: &str) {
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

    // Store the actual TestResult as YML in database
    let yml_content = serde_yaml::to_string(&test_result)
        .map_err(|e| anyhow::anyhow!("Failed to serialize TestResult to YML: {e}"))?;

    // Store YML directly in database
    db.insert_yml_flow_log(benchmark_id, &yml_content).await?;
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

    db.insert_flow_log(&flow_log).await?;
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
        "Attempting to store YML TestResult ({} chars) in database",
        yml_content.len()
    );

    // Use a method to insert YML TestResult instead of accessing private conn
    match db
        .insert_yml_testresult(benchmark_id, agent, &yml_content)
        .await
    {
        Ok(_) => {
            info!("Successfully stored YML TestResult in database");
        }
        Err(e) => {
            error!("YML TestResult insertion error: {:?}", e);
            error!("YML content length: {} chars", yml_content.len());
            error!(
                "YML content preview: {}",
                &yml_content[..yml_content.len().min(200)]
            );
        }
    }

    info!("YML TestResult stored for benchmark: {benchmark_id} by agent: {agent}");
    Ok(())
}
