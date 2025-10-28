use serde::{Deserialize, Serialize};

use uuid::Uuid;

/// JSON-RPC 2.0 request wrapper for inter-process communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// JSON-RPC version, always "2.0"
    pub jsonrpc: String,
    /// Request identifier for correlation
    pub id: RequestId,
    /// Method name to invoke
    pub method: String,
    /// Method parameters
    pub params: serde_json::Value,
}

/// JSON-RPC 2.0 response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// JSON-RPC version, always "2.0"
    pub jsonrpc: String,
    /// Request identifier (must match request)
    pub id: RequestId,
    /// Result data if successful, null if error
    pub result: Option<serde_json::Value>,
    /// Error data if unsuccessful, null if success
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC error structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Error code
    pub code: i32,
    /// Human readable error message
    pub message: String,
    /// Optional error data
    pub data: Option<serde_json::Value>,
}

/// Request identifier for correlation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum RequestId {
    /// Null identifier (notifications)
    Null,
    /// String identifier
    String(String),
    /// Numeric identifier
    Number(i64),
    /// UUID identifier
    Uuid(Uuid),
}

impl RequestId {
    pub fn generate() -> Self {
        Self::Uuid(Uuid::new_v4())
    }
}

/// RPC method names
pub struct RpcMethods;

impl RpcMethods {
    /// Execute a benchmark via runner
    pub const RUN_BENCHMARK: &'static str = "runner.run_benchmark";
    /// Get execution status
    pub const GET_STATUS: &'static str = "runner.get_status";
    /// Stop execution
    pub const STOP_EXECUTION: &'static str = "runner.stop_execution";
    /// List available benchmarks
    pub const LIST_BENCHMARKS: &'static str = "runner.list_benchmarks";
    /// List available agents
    pub const LIST_AGENTS: &'static str = "runner.list_agents";
}

/// Parameters for run_benchmark RPC call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunBenchmarkParams {
    /// Path to benchmark file
    pub benchmark_path: String,
    /// Agent type to use
    pub agent: String,
    /// Execution ID for tracking
    pub execution_id: Option<String>,
    /// Whether to use shared surfpool
    pub shared_surfpool: Option<bool>,
}

/// Parameters for get_status RPC call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetStatusParams {
    /// Execution ID to check
    pub execution_id: String,
}

/// Parameters for stop_execution RPC call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopExecutionParams {
    /// Execution ID to stop
    pub execution_id: String,
}

impl JsonRpcRequest {
    /// Create a new JSON-RPC request
    pub fn new<T: Serialize>(
        method: impl Into<String>,
        params: T,
        id: RequestId,
    ) -> Result<Self, serde_json::Error> {
        Ok(Self {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.into(),
            params: serde_json::to_value(params)?,
        })
    }

    /// Create a notification (no response expected)
    pub fn notification<T: Serialize>(
        method: impl Into<String>,
        params: T,
    ) -> Result<Self, serde_json::Error> {
        Ok(Self {
            jsonrpc: "2.0".to_string(),
            id: RequestId::Null,
            method: method.into(),
            params: serde_json::to_value(params)?,
        })
    }
}

impl JsonRpcResponse {
    /// Create a successful response
    pub fn success<T: Serialize>(id: RequestId, result: T) -> Result<Self, serde_json::Error> {
        Ok(Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(serde_json::to_value(result)?),
            error: None,
        })
    }

    /// Create an error response
    pub fn error(id: RequestId, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(error),
        }
    }

    /// Create an error response from components
    pub fn error_with_message(id: RequestId, code: i32, message: impl Into<String>) -> Self {
        Self::error(
            id,
            JsonRpcError {
                code,
                message: message.into(),
                data: None,
            },
        )
    }
}

impl JsonRpcError {
    /// Standard error codes
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;

    /// Custom error codes for runner
    pub const BENCHMARK_NOT_FOUND: i32 = -32001;
    pub const AGENT_NOT_FOUND: i32 = -32002;
    pub const EXECUTION_TIMEOUT: i32 = -32003;
    pub const EXECUTION_FAILED: i32 = -32004;
}
