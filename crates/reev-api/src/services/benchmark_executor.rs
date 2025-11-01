#![allow(dead_code)]
#![allow(unused_variables)]

use anyhow::{anyhow, Result};
use chrono;
use reev_db::shared::performance::AgentPerformance;
use reev_db::writer::DatabaseWriterTrait;
use reev_flow::JsonlToYmlConverter;
use reev_types::{ExecutionRequest, ExecutionState, ExecutionStatus, RunnerConfig, TimeoutConfig};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::fs;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

/// Benchmark executor using CLI-based runner with real process management
pub struct BenchmarkExecutor<T>
where
    T: DatabaseWriterTrait + Send + Sync + 'static,
{
    db: Arc<T>,
    config: RunnerConfig,
    timeout_config: TimeoutConfig,
}

impl<T> BenchmarkExecutor<T>
where
    T: DatabaseWriterTrait + Send + Sync + 'static,
{
    /// Create new benchmark executor
    pub fn new(db: Arc<T>, config: RunnerConfig, timeout_config: TimeoutConfig) -> Self {
        Self {
            db,
            config,
            timeout_config,
        }
    }

    /// Create new benchmark executor with default config
    ///
    /// **Auto-Detection:**
    /// - Release mode: Uses `./target/release/reev-runner` if it exists
    /// - Development mode: Uses `cargo watch` for fast recompilation if no release binary
    ///
    /// **Environment Variables:**
    /// - `RUST_LOG`: Set to "info" for development logging
    /// - `REEV_ENHANCED_OTEL_FILE`: Enhanced OTEL logging path
    ///
    /// **Usage:**
    /// ```bash
    /// # Development: cargo watch (default)
    /// cargo run -p reev-api
    ///
    /// # Production: build release binary
    /// cargo build --release -p reev-runner
    ///
    /// # Now reev-api will auto-detect and use the release binary
    /// cargo run -p reev-api
    /// ```
    pub fn new_with_default(db: Arc<T>) -> Self {
        Self::new(
            db,
            RunnerConfig {
                runner_binary_path: "./target/debug/reev-runner".to_string(),
                working_directory: std::env::current_dir()
                    .unwrap_or_else(|_| std::path::PathBuf::from("."))
                    .to_string_lossy()
                    .to_string(),
                environment: std::collections::HashMap::new(),
                default_timeout_seconds: 300,
                max_concurrent_executions: 1,
            },
            TimeoutConfig {
                default_timeout_seconds: 300,
                max_timeout_seconds: 600,
                status_check_timeout_seconds: 30,
            },
        )
    }

    /// Execute a benchmark using CLI runner
    pub async fn execute_benchmark(&self, params: ExecutionRequest) -> Result<String> {
        let execution_id = params
            .execution_id
            .clone()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        info!("Starting benchmark execution via CLI: {}", execution_id);

        // Create execution state
        let mut execution_state = ExecutionState::new(
            execution_id.clone(),
            params.benchmark_path.clone(),
            params.agent.clone(),
        );
        execution_state.update_status(ExecutionStatus::Running);

        // Execute benchmark using CLI runner
        self.execute_cli_benchmark(&mut execution_state, params)
            .await?;

        // Store final execution state to database regardless of status
        // This ensures both successful and failed executions are tracked
        match self.store_execution_state(&execution_state).await {
            Ok(_) => {
                info!(
                    "✅ Final execution state stored successfully: {}",
                    execution_id
                );
            }
            Err(e) => {
                warn!("⚠️ Final execution state storage failed: {} - execution will complete but database state may be incomplete", e);
                // Continue execution completion despite database failure
                // This prevents the system from getting stuck in "Queued" state
            }
        }

        debug!("Benchmark execution completed: {}", execution_id);
        Ok(execution_id)
    }

    /// List available benchmarks via CLI
    pub async fn list_benchmarks(&self, directory: Option<&str>) -> Result<Vec<String>> {
        self.execute_cli_list_command("list-benchmarks", directory)
            .await
    }

    /// List available agents via CLI
    pub async fn list_agents(&self) -> Result<Vec<String>> {
        self.execute_cli_list_command("list-agents", None).await
    }

    /// Check if runner is available
    pub async fn is_runner_available(&self) -> bool {
        let result = self
            .execute_cli_command(vec!["--help".to_string()], "availability-check")
            .await;
        result.map(|r| r.exit_code == Some(0)).unwrap_or(false)
    }

    /// Execute benchmark via CLI
    async fn execute_cli_benchmark(
        &self,
        execution_state: &mut ExecutionState,
        params: ExecutionRequest,
    ) -> Result<()> {
        execution_state.update_status(ExecutionStatus::Running);

        // Build CLI command
        let mut args = vec![params.benchmark_path.clone()];
        args.push(format!("--agent={}", params.agent));
        args.push(format!("--execution-id={}", execution_state.execution_id));

        if params.shared_surfpool {
            args.push("--shared-surfpool".to_string());
        }

        // Execute CLI command
        let result = self
            .execute_cli_command(args, &execution_state.execution_id)
            .await?;

        // IMMEDIATELY log what runner actually outputted
        info!("🔍 RUNNER STDOUT: {}", result.stdout);
        info!("🔍 RUNNER STDERR: {}", result.stderr);
        info!("🔍 RUNNER EXIT CODE: {:?}", result.exit_code);
        info!("🔍 RUNNER TIMED OUT: {}", result.timed_out);

        // Add delay to ensure session file is completely written and closed
        // This prevents race condition where API reads file while runner is still writing
        info!("CLI command completed, waiting 2 seconds for session file to finalize...");
        sleep(Duration::from_secs(2)).await;

        // Read session file to get actual results
        if let Err(e) = self.read_session_file_results(execution_state).await {
            warn!(
                "Failed to read session file results: {}, using CLI result as fallback",
                e
            );

            // Fallback to CLI result if session file reading fails
            self.update_execution_state_from_cli_result(execution_state, &result);
        }

        // Store final execution state with session file results in database
        // This ensures "Completed" status and actual results are persisted
        if let Err(e) = self.store_execution_state(execution_state).await {
            warn!("Failed to store session file results in database: {e}");
        }

        Ok(())
    }

    /// Update execution state based on CLI process result (fallback method)
    fn update_execution_state_from_cli_result(
        &self,
        execution_state: &mut ExecutionState,
        result: &reev_types::ProcessExecutionResult,
    ) {
        if result.is_success() {
            execution_state.update_status(ExecutionStatus::Completed);
            execution_state.complete(serde_json::json!({
                "stdout": result.stdout,
                "duration_ms": result.duration_ms,
                "exit_code": result.exit_code,
            }));
            info!("CLI execution successful: {}", execution_state.execution_id);
        } else {
            execution_state.set_error(format!(
                "CLI execution failed: {}",
                result.get_combined_output()
            ));
            if result.timed_out {
                execution_state.update_status(ExecutionStatus::Timeout);
            } else {
                execution_state.update_status(ExecutionStatus::Failed);
            }
            error!("CLI execution failed: {}", result.get_combined_output());
        }
    }

    /// Read session file results and update execution state
    async fn read_session_file_results(&self, execution_state: &mut ExecutionState) -> Result<()> {
        let session_id = execution_state.execution_id.clone();
        let session_file = PathBuf::from(format!("logs/sessions/session_{session_id}.json"));

        debug!("Looking for session file: {:?}", session_file);

        // Wait for session file to be created (with timeout)
        // Runner can take up to 30 seconds, wait appropriately
        let max_attempts = 300; // 300 attempts × 100ms = 30 seconds total wait
        let delay_ms = 100;

        for attempt in 1..=max_attempts {
            if session_file.exists() {
                info!(
                    "✅ Session file found after {:.1}s (attempt {})",
                    (attempt * delay_ms) as f64 / 1000.0,
                    attempt
                );
                break;
            }

            if attempt == max_attempts {
                return Err(anyhow::anyhow!(
                    "Session file not found after {max_attempts} attempts ({:.1}s): {session_file:?}",
                    (max_attempts * delay_ms) as f64 / 1000.0
                ));
            }

            // Log every 10 attempts (1 second)
            if attempt % 10 == 0 {
                info!(
                    "Still waiting for session file... ({:.1}s elapsed, attempt {}/{})",
                    (attempt * delay_ms) as f64 / 1000.0,
                    attempt,
                    max_attempts
                );
            } else {
                debug!(
                    "Session file not found (attempt {}/{}), waiting {}ms... ({:.1}s elapsed)",
                    attempt,
                    max_attempts,
                    delay_ms,
                    (attempt * delay_ms) as f64 / 1000.0
                );
            }
            sleep(Duration::from_millis(delay_ms)).await;
        }

        // Read and parse session file
        let content = fs::read_to_string(&session_file).await?;
        let session_data: Value = serde_json::from_str(&content)?;

        debug!(
            "Session file content parsed successfully for {}",
            session_id
        );

        // Extract final result from session data
        if let Some(final_result) = session_data.get("final_result") {
            let success = final_result
                .get("success")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let score = final_result
                .get("score")
                .and_then(Value::as_f64)
                .unwrap_or(0.0);

            debug!(
                "Extracted from session file: success={}, score={}",
                success, score
            );

            // Update execution state with session file results
            if success {
                execution_state.update_status(ExecutionStatus::Completed);
                execution_state.complete(serde_json::json!({
                    "success": success,
                    "score": score,
                    "source": "session_file",
                    "final_result": final_result
                }));
                info!(
                    "Session file indicates success: {} (score: {})",
                    session_id, score
                );
            } else {
                execution_state.update_status(ExecutionStatus::Failed);
                execution_state
                    .set_error(format!("Session file indicates failure (score: {score})"));
                warn!(
                    "Session file indicates failure: {} (score: {})",
                    session_id, score
                );
            }
        } else {
            return Err(anyhow::anyhow!(
                "Session file missing 'final_result' field: {session_file:?}"
            ));
        }

        // Store agent performance data from session file (function handles errors gracefully)
        let _ = self
            .store_agent_performance_from_session(execution_state)
            .await;

        // Convert enhanced_otel JSONL to YML and store in database for flow diagrams
        let _ = self.convert_and_store_enhanced_otel(&session_id).await;

        Ok(())
    }

    /// Convert enhanced_otel JSONL file to YML format and store in database
    async fn convert_and_store_enhanced_otel(&self, session_id: &str) -> Result<()> {
        let jsonl_path = PathBuf::from(format!("logs/sessions/enhanced_otel_{session_id}.jsonl"));

        if !jsonl_path.exists() {
            debug!("No enhanced_otel file found for session: {}", session_id);
            return Ok(());
        }

        info!(
            "Converting enhanced_otel to YML for session: {}",
            session_id
        );

        // Convert JSONL to YML format using temporary file
        let temp_yml_path = jsonl_path.with_extension("yml");
        let session_data = JsonlToYmlConverter::convert_file(&jsonl_path, &temp_yml_path)
            .map_err(|e| anyhow!("Failed to convert enhanced_otel JSONL to YML: {e}"))?;

        // Read YML content for database storage
        let yml_content = tokio::fs::read_to_string(&temp_yml_path)
            .await
            .map_err(|e| anyhow!("Failed to read temporary YML file: {e}"))?;

        // Clean up temporary file
        let _ = tokio::fs::remove_file(&temp_yml_path).await;

        // Store session log in database
        if let Err(e) = self.db.store_session_log(session_id, &yml_content).await {
            warn!("Failed to store session log in database: {}", e);
        } else {
            info!("Stored session log in database for session: {}", session_id);
        }

        // Store individual tool calls in database
        for tool_call in &session_data.tool_calls {
            let tool_data = serde_json::json!({
                "tool_name": tool_call.tool_name,
                "start_time": tool_call.start_time,
                "end_time": tool_call.end_time,
                "duration_ms": tool_call.duration_ms,
                "input": tool_call.input,
                "output": tool_call.output,
                "success": tool_call.success,
                "error_message": tool_call.error_message
            });

            if let Err(e) = self
                .db
                .store_tool_call(session_id, &tool_call.tool_name, &tool_data)
                .await
            {
                warn!(
                    "Failed to store tool call {} in database: {}",
                    tool_call.tool_name, e
                );
            } else {
                debug!("Stored tool call {} in database", tool_call.tool_name);
            }
        }

        info!(
            "Successfully converted and stored enhanced_otel data for session: {}",
            session_id
        );
        Ok(())
    }

    /// Store agent performance data from session file results
    async fn store_agent_performance_from_session(
        &self,
        execution_state: &ExecutionState,
    ) -> Result<()> {
        let session_id = &execution_state.execution_id;
        let session_file = PathBuf::from(format!("logs/sessions/session_{session_id}.json"));

        debug!(
            "Storing agent performance from session file: {:?}",
            session_file
        );

        // Read session file
        let session_content = tokio::fs::read_to_string(&session_file)
            .await
            .map_err(|e| anyhow!("Failed to read session file {session_file:?}: {e}"))?;

        let session_data: Value = serde_json::from_str(&session_content)
            .map_err(|e| anyhow!("Failed to parse session file {session_file:?}: {e}"))?;

        // Extract data from session
        let benchmark_id = session_data
            .get("benchmark_id")
            .and_then(Value::as_str)
            .unwrap_or(&execution_state.benchmark_id)
            .to_string();

        let agent_type = session_data
            .get("agent_type")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();

        let final_result = session_data
            .get("final_result")
            .ok_or_else(|| anyhow!("Session file missing 'final_result' field"))?;

        let score = final_result
            .get("score")
            .and_then(Value::as_f64)
            .unwrap_or(0.0);

        let execution_time_ms = final_result
            .get("execution_time_ms")
            .and_then(Value::as_u64)
            .map(|v| v as i64);

        let status = final_result
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();

        // Create agent performance record
        let performance = AgentPerformance {
            id: None,
            session_id: session_id.clone(),
            benchmark_id,
            agent_type,
            score,
            final_status: status,
            execution_time_ms,
            timestamp: chrono::Utc::now().to_rfc3339(),
            flow_log_id: None,
            prompt_md5: None, // TODO: Extract from prompt if needed
            additional_metrics: HashMap::new(),
        };

        // Store in database with retry logic
        match self.db.insert_agent_performance(&performance).await {
            Ok(_) => {
                info!(
                    "✅ Stored agent performance for session: {} (score: {})",
                    session_id, score
                );
                Ok(())
            }
            Err(first_error) => {
                warn!(
                    "⚠️ First attempt to store agent performance failed: {} - {}",
                    session_id, first_error
                );

                // Retry once after brief delay
                tokio::time::sleep(Duration::from_millis(500)).await;

                match self.db.insert_agent_performance(&performance).await {
                    Ok(_) => {
                        info!(
                            "✅ Agent performance stored successfully on retry: {}",
                            session_id
                        );
                        Ok(())
                    }
                    Err(retry_error) => {
                        error!(
                            "❌ Failed to store agent performance after retry: {} - {}",
                            session_id, retry_error
                        );

                        // Continue execution despite database failure
                        warn!(
                            "🔄 Completing execution despite agent performance storage failure: {}",
                            session_id
                        );

                        // Don't fail the entire execution, return Ok to allow continuation
                        Ok(())
                    }
                }
            }
        }
    }

    /// Read session file results and update execution state
    pub async fn read_session_file_results_with_cache_update(
        &self,
        execution_state: &mut ExecutionState,
        update_cache: Option<impl Fn(ExecutionState) + Send + Sync>,
    ) -> Result<()> {
        // Read session file results first
        self.read_session_file_results(execution_state).await?;

        // Update in-memory cache if callback provided
        if let Some(update_fn) = update_cache {
            update_fn(execution_state.clone());
        }

        Ok(())
    }

    /// Execute CLI list command and parse output
    async fn execute_cli_list_command(
        &self,
        command_name: &str,
        directory: Option<&str>,
    ) -> Result<Vec<String>> {
        // For list-benchmarks: use "benchmarks" directory as path
        // For list-agents: use "--help" and parse agents from output
        let args = match command_name {
            "list-benchmarks" => vec!["benchmarks".to_string()],
            "list-agents" => vec!["--help".to_string()],
            _ => vec![command_name.to_string()],
        };

        let result = self.execute_cli_command(args, command_name).await?;

        if !result.is_success() {
            return Err(anyhow::anyhow!(
                "CLI list command failed: {}",
                result.stderr
            ));
        }

        // Parse output based on command type
        let items = match command_name {
            "list-agents" => {
                // Parse agent options from help text
                let help_text = result.stdout;
                let agent_line = help_text
                    .lines()
                    .find(|line| line.contains("agent") && line.contains("Can be"))
                    .unwrap_or("");

                if agent_line.is_empty() {
                    vec!["deterministic".to_string(), "local".to_string()]
                } else {
                    // Extract agent names from help text
                    agent_line
                        .split('`')
                        .filter_map(|part| {
                            let part = part.trim();
                            if part.contains(',')
                                || part == "deterministic"
                                || part == "local"
                                || part.contains('-')
                            {
                                Some(part.trim_end_matches(',').to_string())
                            } else {
                                None
                            }
                        })
                        .filter(|s| !s.is_empty())
                        .collect()
                }
            }
            "list-benchmarks" => {
                // Parse benchmark files from directory listing
                result
                    .stdout
                    .lines()
                    .filter(|line| {
                        let line = line.trim();
                        !line.is_empty()
                            && (line.contains(".yml")
                                || line.contains("benchmark")
                                || !line.contains("error"))
                    })
                    .map(|line| {
                        // Extract filename if it's a file listing
                        let line = line.trim();
                        if line.ends_with(".yml") {
                            line.strip_suffix(".yml").unwrap_or(line).to_string()
                        } else {
                            line.to_string()
                        }
                    })
                    .filter(|s| !s.is_empty())
                    .collect()
            }
            _ => {
                // Default parsing - one item per line
                result
                    .stdout
                    .lines()
                    .filter(|line| !line.trim().is_empty())
                    .map(|line| line.trim().to_string())
                    .collect()
            }
        };

        Ok(items)
    }

    /// Execute CLI command with timeout
    pub async fn execute_cli_command(
        &self,
        args: Vec<String>,
        context_id: &str,
    ) -> Result<reev_types::ProcessExecutionResult> {
        use std::process::Stdio;
        use tokio::process::Command as TokioCommand;
        use tokio::time::timeout;

        let execution_id = context_id.to_string();
        let timeout_seconds = self
            .timeout_config
            .default_timeout_seconds
            .min(self.timeout_config.max_timeout_seconds);

        // Set up environment
        // Auto-dectect: prefer debug binary over cargo watch, avoid hanging
        let configured_binary_exists =
            std::path::Path::new(&self.config.runner_binary_path).exists();

        // Try common debug path as fallback before cargo watch
        let debug_binary_path = "target/debug/reev-runner";
        let debug_binary_exists = std::path::Path::new(debug_binary_path).exists();

        let (use_release, runner_path, mode) = if configured_binary_exists {
            let path = self.config.runner_binary_path.clone();
            (true, path, "configured binary (auto-detected)".to_string())
        } else if debug_binary_exists {
            (
                true,
                debug_binary_path.to_string(),
                "debug binary (auto-detected)".to_string(),
            )
        } else {
            info!("No binary found, building debug binary to avoid cargo watch hanging");
            // Build debug binary first to avoid cargo watch issues
            if let Ok(output) = std::process::Command::new("cargo")
                .args(["build", "-p", "reev-runner"])
                .output()
            {
                if output.status.success() {
                    info!("Debug binary built successfully");
                    (
                        true,
                        debug_binary_path.to_string(),
                        "built debug binary".to_string(),
                    )
                } else {
                    warn!("Failed to build debug binary, falling back to cargo watch");
                    (
                        false,
                        "cargo watch --quiet -x 'run -p reev-runner --'".to_string(),
                        "development (cargo watch fallback)".to_string(),
                    )
                }
            } else {
                warn!("Failed to start cargo build, falling back to cargo watch");
                (
                    false,
                    "cargo watch --quiet -x 'run -p reev-runner --'".to_string(),
                    "development (cargo watch fallback)".to_string(),
                )
            }
        };

        info!("Using {} mode: {}", mode, runner_path);

        // Execute command differently based on type
        let mut cmd = if runner_path.starts_with("cargo watch") {
            // For cargo watch, we need to execute the command properly
            let mut cmd = TokioCommand::new("cargo");
            cmd.args([
                "watch",
                "--quiet",
                "-x",
                &format!("run -p reev-runner -- {}", args.join(" ")),
            ]);
            cmd
        } else {
            // For release binary, execute directly or via shell if needed
            if runner_path.contains(' ') {
                let mut cmd = TokioCommand::new("sh");
                cmd.arg("-c")
                    .arg(format!("{} {}", runner_path, args.join(" ")));
                cmd
            } else {
                let mut cmd = TokioCommand::new(&runner_path);
                cmd.args(&args);
                cmd
            }
        };

        cmd.current_dir(&self.config.working_directory)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        info!(
            "Executing CLI command: {} {} (timeout: {}s)",
            runner_path,
            args.join(" "),
            timeout_seconds
        );

        // Add environment variables
        for (key, value) in &self.config.environment {
            cmd.env(key, value);
        }

        // Set development environment variables for cargo watch mode
        if !use_release {
            cmd.env("RUST_LOG", "info");
            cmd.env(
                "REEV_ENHANCED_OTEL_FILE",
                "logs/sessions/enhanced_otel_{session_id}.jsonl",
            );
        } else {
            // Production mode - ensure enhanced OTEL is also available
            cmd.env(
                "REEV_ENHANCED_OTEL_FILE",
                std::env::var("REEV_ENHANCED_OTEL_FILE").unwrap_or_else(|_| {
                    "logs/sessions/enhanced_otel_{session_id}.jsonl".to_string()
                }),
            );
        }

        let start_time = std::time::Instant::now();
        let mut result = reev_types::ProcessExecutionResult::new(
            execution_id.clone(),
            self.config.runner_binary_path.clone(),
            args,
        );

        // Execute with timeout
        let execution_result = timeout(Duration::from_secs(timeout_seconds), cmd.output()).await;

        result.duration_ms = start_time.elapsed().as_millis() as u64;

        match execution_result {
            Ok(Ok(output)) => {
                result.exit_code = Some(output.status.code().unwrap_or(-1));
                result.stdout = String::from_utf8_lossy(&output.stdout).to_string();
                result.stderr = String::from_utf8_lossy(&output.stderr).to_string();
                result.pid = None;

                debug!(
                    "CLI command completed: {} - exit code: {:?}",
                    execution_id, result.exit_code
                );
            }
            Ok(Err(e)) => {
                result.timed_out = false;
                result.stderr = format!("Process execution failed: {e}");
                error!("❌ CLI command failed to start: {} - {}", execution_id, e);
            }
            Err(_) => {
                result.timed_out = true;
                result.stderr = format!("Command timed out after {timeout_seconds} seconds");
                warn!(
                    "⏰ CLI command timed out: {} - {}s",
                    execution_id, timeout_seconds
                );
            }
        }

        info!("🔍 FINAL RESULT - Is Success: {}", result.is_success());
        info!("🔍 FINAL RESULT - Is Timeout: {}", result.timed_out);
        info!(
            "🔍 FINAL RESULT - Process Duration: {}ms",
            result.duration_ms
        );

        Ok(result)
    }

    /// Store execution state in database with retry and graceful error handling
    async fn store_execution_state(&self, state: &ExecutionState) -> Result<()> {
        // First attempt
        match self.db.store_execution_state(state).await {
            Ok(_) => {
                debug!(
                    "Execution state stored successfully on first attempt: {}",
                    state.execution_id
                );
                Ok(())
            }
            Err(first_error) => {
                warn!(
                    "⚠️ First attempt to store execution state failed: {}",
                    first_error
                );

                // Retry once after brief delay
                tokio::time::sleep(Duration::from_millis(500)).await;

                match self.db.store_execution_state(state).await {
                    Ok(_) => {
                        info!(
                            "✅ Execution state stored successfully on retry: {}",
                            state.execution_id
                        );
                        Ok(())
                    }
                    Err(retry_error) => {
                        error!(
                            "❌ Failed to store execution state after retry: {} - {}",
                            state.execution_id, retry_error
                        );

                        // Complete execution with error but don't crash system
                        warn!(
                            "🔄 Completing execution despite database storage failure: {}",
                            state.execution_id
                        );

                        // Return a specific error that can be handled gracefully
                        Err(anyhow::anyhow!(
                            "Database storage failed after retry for {}: {}",
                            state.execution_id,
                            retry_error
                        ))
                    }
                }
            }
        }
    }
}

/// Type alias for BenchmarkExecutor with PooledDatabaseWriter
pub type PooledBenchmarkExecutor = BenchmarkExecutor<reev_lib::db::PooledDatabaseWriter>;

#[cfg(test)]
mod tests {
    use super::*;
    use reev_db::{DatabaseConfig, DatabaseWriter};
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_enhanced_otel_conversion() -> Result<(), Box<dyn std::error::Error>> {
        // Test enhanced_otel to YML conversion using test files
        let current_dir = std::env::current_dir()?;
        let test_files_dir = current_dir.join("tests");
        let jsonl_path =
            test_files_dir.join("enhanced_otel_057d2e4a-f687-469f-8885-ad57759817c0.jsonl");
        let temp_yml_path = PathBuf::from("test_conversion_output.yml");

        if !jsonl_path.exists() {
            println!("❌ enhanced_otel test file not found: {jsonl_path:?}");
            return Ok(());
        }

        println!("🔄 Converting enhanced_otel to YML...");

        // Convert JSONL to YML
        let session_data = JsonlToYmlConverter::convert_file(&jsonl_path, &temp_yml_path)?;

        println!("✅ Conversion successful!");
        println!("   Session ID: {}", session_data.session_id);
        println!("   Tool calls: {}", session_data.tool_calls.len());

        for (i, tool) in session_data.tool_calls.iter().enumerate() {
            println!(
                "   Tool {}: {} ({}ms) - success: {}",
                i + 1,
                tool.tool_name,
                tool.duration_ms,
                tool.success
            );
        }

        // Read YML content
        let yml_content = tokio::fs::read_to_string(&temp_yml_path).await?;
        println!("📄 YML content (first 1000 chars):");
        println!("{}", &yml_content[..yml_content.len().min(1000)]);

        // Test database storage
        println!("\n🗄️ Testing database storage...");

        // Initialize database
        let config = DatabaseConfig::default();
        let db = DatabaseWriter::new(config.clone()).await?;
        let db = std::sync::Arc::new(db);

        // Create pooled database writer for benchmark executor
        let pooled_db = reev_lib::db::PooledDatabaseWriter::new(config, 10).await?;
        let pooled_db = std::sync::Arc::new(pooled_db);

        // Create benchmark executor to test conversion
        let executor =
            PooledBenchmarkExecutor::new(pooled_db.clone(), Default::default(), Default::default());

        // Test conversion method
        if let Err(e) = executor
            .convert_and_store_enhanced_otel("057d2e4a-f687-469f-8885-ad57759817c0")
            .await
        {
            println!("❌ Enhanced_otel conversion failed: {e}");
        } else {
            println!("✅ Enhanced_otel conversion successful");
        }

        // Test with second test file
        let jsonl_path2 =
            test_files_dir.join("enhanced_otel_93aebfa7-cf08-4793-bc0e-7a8ef4cdddaa.jsonl");
        if jsonl_path2.exists() {
            println!("\n🔄 Testing second enhanced_otel file...");
            let session_data2 = JsonlToYmlConverter::convert_file(&jsonl_path2, &temp_yml_path)?;
            println!("✅ Second conversion successful!");
            println!("   Session ID: {}", session_data2.session_id);
            println!("   Tool calls: {}", session_data2.tool_calls.len());

            if let Err(e) = executor
                .convert_and_store_enhanced_otel("93aebfa7-cf08-4793-bc0e-7a8ef4cdddaa")
                .await
            {
                println!("❌ Second enhanced_otel conversion failed: {e}");
            } else {
                println!("✅ Second enhanced_otel conversion successful");
            }
        }

        // Test retrieval from pooled_db (same as API uses)
        if let Ok(Some(log_content)) = pooled_db.get_session_log(&session_data.session_id).await {
            println!("✅ Retrieved session log from pooled database");
            println!("   Content length: {} chars", log_content.len());

            // Test if our parser can read it
            use crate::handlers::flow_diagram::SessionParser;
            match SessionParser::parse_session_content(&log_content) {
                Ok(parsed) => {
                    println!("✅ Parser successfully read YML content");
                    println!("   Found {} tool calls", parsed.tool_calls.len());
                    for (i, tool) in parsed.tool_calls.iter().enumerate() {
                        println!(
                            "   Tool {}: {} ({}ms)",
                            i + 1,
                            tool.tool_name,
                            tool.duration_ms
                        );
                    }
                }
                Err(e) => {
                    println!("❌ Parser failed to read YML content: {e}");
                }
            }
        } else {
            println!("❌ Failed to retrieve session log from pooled database");
        }

        // Clean up
        tokio::fs::remove_file(&temp_yml_path).await.ok();

        println!("\n🎉 Enhanced_otel conversion test completed!");
        Ok(())
    }
}
