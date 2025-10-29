#![allow(dead_code)]
#![allow(unused_variables)]

use anyhow::Result;
use reev_db::writer::DatabaseWriterTrait;
use reev_types::{ExecutionRequest, ExecutionState, ExecutionStatus, RunnerConfig, TimeoutConfig};
use std::sync::Arc;
use std::time::Duration;
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
        execution_state.update_status(ExecutionStatus::Queued);

        // Store initial state
        self.store_execution_state(&execution_state).await?;

        // Execute benchmark using CLI runner
        self.execute_cli_benchmark(&mut execution_state, params)
            .await?;

        // Store final state
        self.store_execution_state(&execution_state).await?;

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
        self.store_execution_state(execution_state).await?;

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

        // Update execution state based on result
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

        info!(
            "Executing CLI command: {} {} (timeout: {}s)",
            self.config.runner_binary_path,
            args.join(" "),
            timeout_seconds
        );

        // Set up environment
        let mut cmd = TokioCommand::new("sh");
        cmd.arg("-c")
            .arg(format!(
                "{} {}",
                self.config.runner_binary_path,
                args.join(" ")
            ))
            .current_dir(&self.config.working_directory)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Add environment variables
        for (key, value) in &self.config.environment {
            cmd.env(key, value);
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

#[cfg(test)]
mod tests {
    use reev_types::RunnerConfig;

    #[tokio::test]
    async fn test_benchmark_executor_creation() {
        // Test with placeholder implementation
        // Will need proper mock database for full testing
        // DatabaseWriterTrait implementation completed
    }

    #[test]
    fn test_runner_config() {
        let config = RunnerConfig::default();
        assert_eq!(config.runner_binary_path, "cargo run -p reev-runner --");
        assert_eq!(config.default_timeout_seconds, 300);
        assert_eq!(config.max_concurrent_executions, 5);
    }
}
