#![allow(dead_code)]
#![allow(unused_variables)]

use anyhow::Result;
use reev_db::writer::DatabaseWriterTrait;
use reev_types::{ExecutionRequest, ExecutionState, ExecutionStatus, RunnerConfig, TimeoutConfig};
use serde_json::Value;
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
    /// **Smart Mode Detection:**
    /// - Auto-detect: Uses release binary if `./target/release/reev-runner` exists
    /// - Development: Uses `cargo watch` for fast recompilation when no release binary
    /// - Production: Uses release binary for maximum performance
    ///
    /// **Environment Variables:**
    /// - `REEV_USE_RELEASE`:
    ///   - "true": Force release binary mode
    ///   - "false": Force development mode with cargo watch
    ///   - "auto" (default): Auto-detect based on binary availability
    /// - `RUST_LOG`: Set to "info" for development logging
    /// - `REEV_ENHANCED_OTEL_FILE`: Enhanced OTEL logging path
    ///
    /// **Usage:**
    /// ```bash
    /// # Build release binary for production
    /// cargo build --release -p reev-runner
    ///
    /// # Force development mode even with release binary
    /// REEV_USE_RELEASE=false cargo run -p reev-api
    ///
    /// # Force production mode
    /// REEV_USE_RELEASE=true cargo run -p reev-api
    /// ```
    pub fn new_with_default(db: Arc<T>) -> Self {
        Self::new(
            db,
            RunnerConfig {
                runner_binary_path: "./target/release/reev-runner".to_string(),
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

        // Only store final state to database if execution was successful
        // Don't store to database on CLI failures to avoid lock conflicts
        if execution_state.status == ExecutionStatus::Completed {
            if let Err(e) = self.store_execution_state(&execution_state).await {
                warn!("Failed to store successful execution state: {}", e);
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

        if params.shared_surfpool {
            args.push("--shared-surfpool".to_string());
        }

        // Execute CLI command
        let result = self
            .execute_cli_command(args, &execution_state.execution_id)
            .await?;

        // Read session file to get actual results
        if let Err(e) = self.read_session_file_results(execution_state).await {
            warn!(
                "Failed to read session file results: {}, using CLI result as fallback",
                e
            );

            // Fallback to CLI result if session file reading fails
            self.update_execution_state_from_cli_result(execution_state, &result);
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
        let max_attempts = 10;
        let delay_ms = 100;

        for attempt in 1..=max_attempts {
            if session_file.exists() {
                break;
            }

            if attempt == max_attempts {
                return Err(anyhow::anyhow!(
                    "Session file not found after {max_attempts} attempts: {session_file:?}"
                ));
            }

            debug!(
                "Session file not found (attempt {}/{}), waiting {}ms...",
                attempt, max_attempts, delay_ms
            );
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
    async fn execute_cli_command(
        &self,
        args: Vec<String>,
        context_id: &str,
    ) -> Result<reev_types::ProcessExecutionResult> {
        use std::process::Stdio;
        use tokio::process::Command as TokioCommand;
        use tokio::time::timeout;

        let execution_id = format!("cli-{}-{}", context_id, uuid::Uuid::new_v4());
        let timeout_seconds = self
            .timeout_config
            .default_timeout_seconds
            .min(self.timeout_config.max_timeout_seconds);

        // Set up environment
        // Smart detection: use release binary if available, otherwise cargo watch
        // Priority: 1) Manual override via REEV_USE_RELEASE=true
        //          2) Auto-detect release binary exists
        //          3) Fallback to cargo watch for development
        let use_release_manual =
            std::env::var("REEV_USE_RELEASE").unwrap_or_else(|_| "auto".to_string());

        let release_binary_exists = std::path::Path::new(&self.config.runner_binary_path).exists();

        let (use_release, runner_path, mode) = match use_release_manual.as_str() {
            "true" if release_binary_exists => {
                let path = self.config.runner_binary_path.clone();
                (true, path, "production (manual)".to_string())
            }
            "false" => (
                false,
                "cargo watch --quiet -x 'run -p reev-runner --'".to_string(),
                "development (manual)".to_string(),
            ),
            "auto" if release_binary_exists => {
                info!("Auto-detected release binary, using production mode");
                let path = self.config.runner_binary_path.clone();
                (true, path, "production (auto-detected)".to_string())
            }
            "auto" => {
                info!("No release binary found, using development mode with cargo watch");
                (
                    false,
                    "cargo watch --quiet -x 'run -p reev-runner --'".to_string(),
                    "development (auto)".to_string(),
                )
            }
            _ => {
                warn!(
                    "Invalid REEV_USE_RELEASE value: {}, defaulting to auto",
                    use_release_manual
                );
                if release_binary_exists {
                    let path = self.config.runner_binary_path.clone();
                    (true, path, "production (auto-fallback)".to_string())
                } else {
                    (
                        false,
                        "cargo watch --quiet -x 'run -p reev-runner --'".to_string(),
                        "development (auto-fallback)".to_string(),
                    )
                }
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
                error!("CLI command failed to start: {} - {}", execution_id, e);
            }
            Err(_) => {
                result.timed_out = true;
                result.stderr = format!("Command timed out after {timeout_seconds} seconds");
                warn!(
                    "CLI command timed out: {} - {}s",
                    execution_id, timeout_seconds
                );
            }
        }

        Ok(result)
    }

    /// Store execution state in database
    async fn store_execution_state(&self, state: &ExecutionState) -> Result<()> {
        self.db
            .store_execution_state(state)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to store execution state: {e}"))
    }
}

/// Type alias for BenchmarkExecutor with PooledDatabaseWriter
pub type PooledBenchmarkExecutor = BenchmarkExecutor<reev_lib::db::PooledDatabaseWriter>;
