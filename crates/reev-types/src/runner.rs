use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// CLI command types for runner communication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", content = "params")]
pub enum RunnerCommand {
    /// Run a benchmark file
    RunBenchmark {
        benchmark_path: String,
        agent: String,
        shared_surfpool: bool,
    },
    /// List available benchmarks
    ListBenchmarks { directory: Option<String> },
    /// List available agents
    ListAgents,
    /// Get execution status
    GetStatus { execution_id: String },
    /// Stop running execution
    StopExecution { execution_id: String },
    /// Render flow log as ASCII tree
    RenderFlow { flow_path: String },
}

/// CLI response types from runner
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum RunnerResponse {
    /// Success response with data
    Success { data: serde_json::Value },
    /// Error response with details
    Error {
        code: i32,
        message: String,
        details: Option<String>,
    },
    /// Progress update for long-running operations
    Progress {
        execution_id: String,
        progress: f64,
        message: Option<String>,
    },
    /// Benchmark execution completed
    BenchmarkCompleted {
        execution_id: String,
        success: bool,
        duration_ms: u64,
        result: Option<serde_json::Value>,
        error: Option<String>,
    },
}

/// Configuration for runner process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnerConfig {
    /// Path to runner binary
    pub runner_binary_path: String,
    /// Working directory for runner
    pub working_directory: String,
    /// Environment variables to set
    pub environment: HashMap<String, String>,
    /// Default timeout in seconds
    pub default_timeout_seconds: u64,
    /// Maximum concurrent executions
    pub max_concurrent_executions: usize,
}

impl Default for RunnerConfig {
    fn default() -> Self {
        Self {
            runner_binary_path: "cargo run -p reev-runner --".to_string(),
            working_directory: ".".to_string(),
            environment: HashMap::new(),
            default_timeout_seconds: 300,
            max_concurrent_executions: 5,
        }
    }
}

/// Process execution result from CLI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessExecutionResult {
    /// Unique execution identifier
    pub execution_id: String,
    /// Command that was executed
    pub command: String,
    /// Arguments passed to command
    pub args: Vec<String>,
    /// Working directory
    pub working_dir: String,
    /// Exit code
    pub exit_code: Option<i32>,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
    /// Whether execution timed out
    pub timed_out: bool,
    /// Process ID
    pub pid: Option<u32>,
}

impl ProcessExecutionResult {
    /// Create a new execution result
    pub fn new(execution_id: String, command: String, args: Vec<String>) -> Self {
        Self {
            execution_id,
            command,
            args,
            working_dir: std::env::current_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            duration_ms: 0,
            timed_out: false,
            pid: None,
        }
    }

    /// Check if execution was successful
    pub fn is_success(&self) -> bool {
        !self.timed_out && self.exit_code.is_some_and(|code| code == 0)
    }

    /// Get combined output
    pub fn get_combined_output(&self) -> String {
        format!("STDOUT:\n{}\nSTDERR:\n{}", self.stdout, self.stderr)
    }
}

/// Runner process manager state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnerProcessState {
    /// Process ID
    pub pid: Option<u32>,
    /// Process status
    pub status: ProcessStatus,
    /// Current command being executed
    pub current_command: Option<String>,
    /// Start timestamp
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Last activity timestamp
    pub last_activity_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Process status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProcessStatus {
    /// Process is idle
    Idle,
    /// Process is starting
    Starting,
    /// Process is running a command
    Running,
    /// Process finished successfully
    Completed,
    /// Process failed
    Failed,
    /// Process was killed
    Killed,
    /// Process timed out
    Timeout,
}

impl RunnerCommand {
    /// Convert command to CLI arguments
    pub fn to_cli_args(&self) -> Vec<String> {
        match self {
            RunnerCommand::RunBenchmark {
                benchmark_path,
                agent,
                shared_surfpool,
            } => {
                let mut args = vec![benchmark_path.clone()];
                args.push(format!("--agent={agent}"));
                if *shared_surfpool {
                    args.push("--shared-surfpool".to_string());
                }
                args
            }
            RunnerCommand::ListBenchmarks { directory } => {
                let mut args = vec!["list-benchmarks".to_string()];
                if let Some(dir) = directory {
                    args.push(dir.clone());
                }
                args
            }
            RunnerCommand::ListAgents => vec!["list-agents".to_string()],
            RunnerCommand::GetStatus { execution_id } => {
                vec!["get-status".to_string(), execution_id.clone()]
            }
            RunnerCommand::StopExecution { execution_id } => {
                vec!["stop".to_string(), execution_id.clone()]
            }
            RunnerCommand::RenderFlow { flow_path } => {
                vec!["--render-flow".to_string(), flow_path.clone()]
            }
        }
    }

    /// Get execution timeout in seconds
    pub fn timeout_seconds(&self) -> u64 {
        match self {
            RunnerCommand::RunBenchmark { .. } => 600, // 10 minutes for benchmarks
            RunnerCommand::ListBenchmarks { .. } => 30,
            RunnerCommand::ListAgents => 10,
            RunnerCommand::GetStatus { .. } => 5,
            RunnerCommand::StopExecution { .. } => 10,
            RunnerCommand::RenderFlow { .. } => 30,
        }
    }

    /// Generate unique execution ID
    pub fn generate_execution_id(&self) -> String {
        format!("runner-{}", Uuid::new_v4())
    }
}

impl RunnerResponse {
    /// Check if response indicates success
    pub fn is_success(&self) -> bool {
        matches!(self, RunnerResponse::Success { .. })
    }

    /// Check if response indicates an error
    pub fn is_error(&self) -> bool {
        matches!(self, RunnerResponse::Error { .. })
    }

    /// Extract success data
    pub fn into_success_data(self) -> Option<serde_json::Value> {
        match self {
            RunnerResponse::Success { data } => Some(data),
            _ => None,
        }
    }

    /// Extract error information
    pub fn into_error(self) -> Option<(i32, String, Option<String>)> {
        match self {
            RunnerResponse::Error {
                code,
                message,
                details,
            } => Some((code, message, details)),
            _ => None,
        }
    }
}

/// Command builder for creating runner commands
pub struct RunnerCommandBuilder;

impl RunnerCommandBuilder {
    /// Create run benchmark command
    pub fn run_benchmark(
        benchmark_path: impl Into<String>,
        agent: impl Into<String>,
        shared_surfpool: bool,
    ) -> RunnerCommand {
        RunnerCommand::RunBenchmark {
            benchmark_path: benchmark_path.into(),
            agent: agent.into(),
            shared_surfpool,
        }
    }

    /// Create list benchmarks command
    pub fn list_benchmarks(directory: Option<impl Into<String>>) -> RunnerCommand {
        RunnerCommand::ListBenchmarks {
            directory: directory.map(|d| d.into()),
        }
    }

    /// Create list agents command
    pub fn list_agents() -> RunnerCommand {
        RunnerCommand::ListAgents
    }

    /// Create get status command
    pub fn get_status(execution_id: impl Into<String>) -> RunnerCommand {
        RunnerCommand::GetStatus {
            execution_id: execution_id.into(),
        }
    }

    /// Create stop execution command
    pub fn stop_execution(execution_id: impl Into<String>) -> RunnerCommand {
        RunnerCommand::StopExecution {
            execution_id: execution_id.into(),
        }
    }

    /// Create render flow command
    pub fn render_flow(flow_path: impl Into<String>) -> RunnerCommand {
        RunnerCommand::RenderFlow {
            flow_path: flow_path.into(),
        }
    }
}
