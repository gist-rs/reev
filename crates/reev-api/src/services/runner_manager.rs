#![allow(dead_code)]

use anyhow::{anyhow, Result};
use reev_db::DatabaseWriter;
use reev_types::{
    ExecutionRequest, ExecutionState, ExecutionStatus, ProcessExecutionResult, RunnerCommand,
    RunnerConfig, RunnerProcessState, TimeoutConfig,
};
use std::collections::HashMap;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command as TokioCommand;
use tokio::time::timeout;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Manager for runner process communication via CLI
pub struct RunnerProcessManager {
    config: RunnerConfig,
    db: DatabaseWriter,
    timeout_config: TimeoutConfig,
    active_processes: HashMap<String, RunnerProcessState>,
}

impl RunnerProcessManager {
    /// Create new runner process manager
    pub fn new(config: RunnerConfig, db: DatabaseWriter, timeout_config: TimeoutConfig) -> Self {
        Self {
            config,
            db,
            timeout_config,
            active_processes: HashMap::new(),
        }
    }

    /// Execute a benchmark via CLI process
    pub async fn execute_benchmark(&self, params: ExecutionRequest) -> Result<String> {
        let execution_id = params
            .execution_id
            .clone()
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        info!("Starting benchmark execution via CLI: {}", execution_id);

        // Create execution state in database
        let mut execution_state = ExecutionState::new(
            execution_id.clone(),
            params.benchmark_path.clone(),
            params.agent.clone(),
        );
        execution_state.update_status(ExecutionStatus::Queued);

        self.store_execution_state(&execution_state).await?;

        // Build runner command
        let command = self.build_runner_command(&params)?;

        // Execute CLI command with timeout
        let result = self.execute_cli_command(command, &execution_id).await?;

        // Update execution state based on result
        if result.is_success() {
            execution_state.update_status(ExecutionStatus::Completed);
            execution_state.complete(serde_json::json!({
                "stdout": result.stdout,
                "duration_ms": result.duration_ms,
            }));
        } else {
            execution_state.set_error(format!(
                "CLI execution failed: {}",
                result.get_combined_output()
            ));
            if result.timed_out {
                execution_state.update_status(ExecutionStatus::Timeout);
            }
        }

        self.store_execution_state(&execution_state).await?;

        info!(
            "Benchmark execution completed: {} - Success: {}",
            execution_id,
            result.is_success()
        );
        Ok(execution_id)
    }

    /// List available benchmarks via CLI
    pub async fn list_benchmarks(&self, directory: Option<&str>) -> Result<Vec<String>> {
        let command = RunnerCommand::ListBenchmarks {
            directory: directory.map(|d| d.to_string()),
        };

        let result = self.execute_cli_command(command, "list-benchmarks").await?;

        if !result.is_success() {
            return Err(anyhow!("Failed to list benchmarks: {}", result.stderr));
        }

        // Parse benchmarks from stdout (expecting one per line)
        let benchmarks: Vec<String> = result
            .stdout
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.trim().to_string())
            .collect();

        Ok(benchmarks)
    }

    /// List available agents via CLI
    pub async fn list_agents(&self) -> Result<Vec<String>> {
        let command = RunnerCommand::ListAgents;
        let result = self.execute_cli_command(command, "list-agents").await?;

        if !result.is_success() {
            return Err(anyhow!("Failed to list agents: {}", result.stderr));
        }

        // Parse agents from stdout (expecting one per line)
        let agents: Vec<String> = result
            .stdout
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.trim().to_string())
            .collect();

        Ok(agents)
    }

    /// Get execution status
    pub async fn get_execution_status(&self, _execution_id: &str) -> Result<ExecutionState> {
        // TODO: Implement when database trait has get_execution_state method
        Err(anyhow!("get_execution_status not yet implemented"))
    }

    /// Stop running execution
    pub async fn stop_execution(&self, execution_id: &str) -> Result<()> {
        // Try to stop via CLI first
        let command = RunnerCommand::StopExecution {
            execution_id: execution_id.to_string(),
        };

        let result = self
            .execute_cli_command(command, &format!("stop-{execution_id}"))
            .await?;

        if result.is_success() {
            // Update state in database
            if let Ok(mut execution_state) = self.get_execution_status(execution_id).await {
                execution_state.update_status(ExecutionStatus::Stopped);
                self.store_execution_state(&execution_state).await?;
            }
        } else {
            warn!("CLI stop command failed: {}", result.stderr);
        }

        Ok(())
    }

    /// Build CLI command from execution request
    fn build_runner_command(&self, params: &ExecutionRequest) -> Result<RunnerCommand> {
        Ok(RunnerCommand::RunBenchmark {
            benchmark_path: params.benchmark_path.clone(),
            agent: params.agent.clone(),
            shared_surfpool: params.shared_surfpool,
        })
    }

    /// Execute CLI command with timeout and monitoring
    async fn execute_cli_command(
        &self,
        command: RunnerCommand,
        context_id: &str,
    ) -> Result<ProcessExecutionResult> {
        let execution_id = format!("cli-{}-{}", context_id, Uuid::new_v4());
        let args = command.to_cli_args();
        let timeout_seconds = command
            .timeout_seconds()
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
        let mut result = ProcessExecutionResult::new(
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
                result.pid = None; // Not available from tokio::process

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

        // Log execution result for debugging
        if !result.is_success() {
            error!("CLI command failed: {}", result.get_combined_output());
        } else {
            debug!("CLI command succeeded: {}", execution_id);
        }

        Ok(result)
    }

    /// Store execution state in database
    async fn store_execution_state(&self, state: &ExecutionState) -> Result<()> {
        // TODO: Implement when database trait has store_execution_state method
        debug!(
            "Storing execution state: {} (placeholder)",
            state.execution_id
        );
        Ok(())
    }

    /// Check if runner binary is available
    pub async fn is_runner_available(&self) -> bool {
        // Simple check by trying to run --help
        let result = self
            .execute_cli_command(
                RunnerCommand::ListAgents, // This is a quick command
                "availability-check",
            )
            .await;

        match result {
            Ok(proc_result) => proc_result.exit_code == Some(0),
            Err(_) => false,
        }
    }

    /// Get metrics about runner process manager
    pub fn get_metrics(&self) -> RunnerMetrics {
        RunnerMetrics {
            active_processes: self.active_processes.len(),
            max_concurrent: self.config.max_concurrent_executions,
            runner_binary_path: self.config.runner_binary_path.clone(),
            working_directory: self.config.working_directory.clone(),
        }
    }
}

/// Metrics for runner process manager
#[derive(Debug, Clone)]
pub struct RunnerMetrics {
    pub active_processes: usize,
    pub max_concurrent: usize,
    pub runner_binary_path: String,
    pub working_directory: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_runner_command_builder() {
        let command = RunnerCommand::RunBenchmark {
            benchmark_path: "benchmarks/test.yml".to_string(),
            agent: "deterministic".to_string(),
            shared_surfpool: false,
        };

        let args = command.to_cli_args();
        assert_eq!(args, vec!["benchmarks/test.yml", "--agent=deterministic"]);

        let command = RunnerCommand::RunBenchmark {
            benchmark_path: "benchmarks/test.yml".to_string(),
            agent: "glm-4.6".to_string(),
            shared_surfpool: true,
        };

        let args = command.to_cli_args();
        assert!(args.contains(&"--shared-surfpool".to_string()));
    }

    #[tokio::test]
    async fn test_execution_request() {
        let request = ExecutionRequest::new(
            "benchmarks/test.yml".to_string(),
            "deterministic".to_string(),
        );

        assert_eq!(request.benchmark_path, "benchmarks/test.yml");
        assert_eq!(request.agent, "deterministic");
    }

    #[tokio::test]
    async fn test_runner_config_default() {
        let config = RunnerConfig::default();
        assert_eq!(config.runner_binary_path, "cargo run -p reev-runner --");
        assert_eq!(config.default_timeout_seconds, 300);
        assert_eq!(config.max_concurrent_executions, 5);
    }
}
