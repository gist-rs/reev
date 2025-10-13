use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use reev_runner::db::Db;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use tracing::{debug, error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

/// API state containing database connection and execution state
#[derive(Clone)]
struct ApiState {
    db: Arc<Db>,
    executions: Arc<Mutex<HashMap<String, ExecutionState>>>,
    agent_configs: Arc<Mutex<HashMap<String, AgentConfig>>>,
}

/// Execution state for tracking benchmark runs
#[derive(Debug, Clone, Serialize)]
struct ExecutionState {
    id: String,
    benchmark_id: String,
    agent: String,
    status: ExecutionStatus,
    progress: u8,
    start_time: chrono::DateTime<chrono::Utc>,
    end_time: Option<chrono::DateTime<chrono::Utc>>,
    trace: String,
    logs: String,
    error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AgentConfig {
    agent_type: String,
    api_url: Option<String>,
    api_key: Option<String>,
}

/// Benchmark execution request
#[derive(Debug, Deserialize)]
struct BenchmarkExecutionRequest {
    agent: String,
    config: Option<AgentConfig>,
}

/// Execution response
#[derive(Debug, Serialize)]
struct ExecutionResponse {
    execution_id: String,
    status: String,
}

/// Health check response
#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    timestamp: String,
    version: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "reev_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize database
    let db_path =
        std::env::var("DATABASE_PATH").unwrap_or_else(|_| "db/reev_results.db".to_string());
    info!("Connecting to database at: {}", db_path);

    let db = Arc::new(Db::new(&db_path).await?);
    info!("Database connection established");

    // Create API state
    let state = ApiState {
        db,
        executions: Arc::new(Mutex::new(HashMap::new())),
        agent_configs: Arc::new(Mutex::new(HashMap::new())),
    };

    // Create router with state - simple approach for testing
    let app = Router::new()
        // Health check
        .route("/api/v1/health", get(health_check))
        // General routes
        .route("/api/v1/benchmarks", get(list_benchmarks))
        .route("/api/v1/agents", get(list_agents))
        .route("/api/v1/agent-performance", get(get_agent_performance))
        // Benchmark execution endpoints
        .route("/api/v1/benchmarks/{id}/run", post(run_benchmark))
        .route(
            "/api/v1/benchmarks/{id}/status/{execution_id}",
            get(get_execution_status),
        )
        .route(
            "/api/v1/benchmarks/{id}/stop/{execution_id}",
            post(stop_benchmark),
        )
        // Agent configuration endpoints
        .route("/api/v1/agents/config", post(save_agent_config))
        .route("/api/v1/agents/config/{agent_type}", get(get_agent_config))
        .route("/api/v1/agents/test", post(test_agent_connection))
        // Flow logs endpoints
        .route("/api/v1/flow-logs/{benchmark_id}", get(get_flow_log))
        .route(
            "/api/v1/parse-yml-to-testresult",
            post(parse_yml_to_testresult),
        )
        .route("/api/v1/render-ascii-tree", post(render_ascii_tree))
        // Test endpoint without JSON
        .route("/api/v1/test", get(test_endpoint))
        // Test POST endpoint without JSON
        .route("/api/v1/test-post", post(test_post_endpoint))
        // Simple CORS layer
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Start server
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3001".to_string()) // Changed from 3000 to 3001 to avoid macOS Apple services conflict
        .parse()
        .unwrap_or(3001); // Changed from 3000 to 3001

    let addr = format!("0.0.0.0:{port}");
    info!("Starting API server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("API server listening on {}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}

/// Health check endpoint
async fn health_check() -> Json<HealthResponse> {
    let response = HealthResponse {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        version: "0.1.0".to_string(),
    };
    Json(response)
}

/// List all available benchmarks
async fn list_benchmarks() -> Json<Vec<String>> {
    let benchmarks = vec![
        "001-sol-transfer".to_string(),
        "002-spl-transfer".to_string(),
        "003-spl-transfer-fail".to_string(),
        "004-partial-score-spl-transfer".to_string(),
        "100-jup-swap-sol-usdc".to_string(),
        "110-jup-lend-deposit-sol".to_string(),
        "111-jup-lend-deposit-usdc".to_string(),
        "112-jup-lend-withdraw-sol".to_string(),
        "113-jup-lend-withdraw-usdc".to_string(),
        "114-jup-positions-and-earnings".to_string(),
        "115-jup-lend-mint-usdc".to_string(),
        "116-jup-lend-redeem-usdc".to_string(),
        "200-jup-swap-then-lend-deposit".to_string(),
    ];
    Json(benchmarks)
}

/// List all available agents
async fn list_agents() -> Json<Vec<String>> {
    let agents = vec![
        "deterministic".to_string(),
        "local".to_string(),
        "gemini".to_string(),
        "glm-4.6".to_string(),
    ];
    Json(agents)
}

/// Get agent performance summary
async fn get_agent_performance(State(state): State<ApiState>) -> impl IntoResponse {
    info!("Getting agent performance summary");

    match state.db.get_agent_performance().await {
        Ok(summaries) => Json(summaries).into_response(),
        Err(e) => {
            error!("Failed to get agent performance: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to get agent performance",
            )
                .into_response()
        }
    }
}

/// Run a benchmark
/// Simple test endpoint
async fn test_endpoint() -> impl IntoResponse {
    Json(serde_json::json!({"status": "test working"}))
}

/// POST test endpoint without JSON
async fn test_post_endpoint() -> impl IntoResponse {
    Json(serde_json::json!({"status": "POST test working"}))
}

/// Run a benchmark
async fn run_benchmark(
    State(state): State<ApiState>,
    Path(benchmark_id): Path<String>,
    Json(request): Json<BenchmarkExecutionRequest>,
) -> impl IntoResponse {
    let execution_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now();

    let execution_state = ExecutionState {
        id: execution_id.clone(),
        benchmark_id: benchmark_id.clone(),
        agent: request.agent.clone(),
        status: ExecutionStatus::Pending,
        progress: 0,
        start_time: now,
        end_time: None,
        trace: String::new(),
        logs: String::new(),
        error: None,
    };

    // Store execution state
    {
        let mut executions = state.executions.lock().await;
        executions.insert(execution_id.clone(), execution_state);
    }

    // Save agent configuration if provided
    if let Some(config) = request.config {
        let mut configs = state.agent_configs.lock().await;
        configs.insert(request.agent.clone(), config);
    }

    info!(
        "Starting benchmark execution: {} for agent: {}",
        benchmark_id, request.agent
    );

    // Start the benchmark execution in background using blocking task for non-Send dependencies
    let state_clone = state.clone();
    let execution_id_clone = execution_id.clone();
    let benchmark_id_clone = benchmark_id.clone();
    let agent = request.agent.clone();

    tokio::spawn(async move {
        tokio::task::spawn_blocking(move || {
            // Use a blocking runtime for the benchmark runner
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                execute_benchmark_background(
                    state_clone,
                    execution_id_clone,
                    benchmark_id_clone,
                    agent,
                )
                .await;
            })
        })
        .await
        .unwrap_or_else(|e| {
            error!("Benchmark execution task failed: {}", e);
        });
    });

    Json(ExecutionResponse {
        execution_id,
        status: "started".to_string(),
    })
}

/// Get execution status
async fn get_execution_status(
    State(state): State<ApiState>,
    Path((_benchmark_id, execution_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let executions = state.executions.lock().await;

    match executions.get(&execution_id) {
        Some(execution) => Json(execution.clone()).into_response(),
        None => (StatusCode::NOT_FOUND, "Execution not found").into_response(),
    }
}

/// Stop a running benchmark
async fn stop_benchmark(
    State(state): State<ApiState>,
    Path((_benchmark_id, execution_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let mut executions = state.executions.lock().await;

    match executions.get_mut(&execution_id) {
        Some(execution) => {
            if matches!(execution.status, ExecutionStatus::Running) {
                execution.status = ExecutionStatus::Failed;
                execution.end_time = Some(chrono::Utc::now());
                execution.error = Some("Execution stopped by user".to_string());
                info!("Stopped benchmark execution: {}", execution_id);
                Json(serde_json::json!({"status": "stopped"})).into_response()
            } else {
                (StatusCode::BAD_REQUEST, "Execution is not running").into_response()
            }
        }
        None => (StatusCode::NOT_FOUND, "Execution not found").into_response(),
    }
}

/// Save agent configuration
async fn save_agent_config(
    State(state): State<ApiState>,
    Json(config): Json<AgentConfig>,
) -> impl IntoResponse {
    let mut configs = state.agent_configs.lock().await;
    configs.insert(config.agent_type.clone(), config.clone());

    info!("Saved configuration for agent: {}", config.agent_type);
    Json(serde_json::json!({"status": "saved"}))
}

/// Get agent configuration
async fn get_agent_config(
    Path(agent_type): Path<String>,
    State(state): State<ApiState>,
) -> impl IntoResponse {
    let configs = state.agent_configs.lock().await;

    match configs.get(&agent_type) {
        Some(config) => {
            // Mask API key for security
            let mut masked_config = config.clone();
            if let Some(ref api_key) = masked_config.api_key {
                if api_key.len() > 4 {
                    masked_config.api_key = Some(format!("***{}", &api_key[api_key.len() - 4..]));
                } else {
                    masked_config.api_key = Some("***".to_string());
                }
            }
            Json(masked_config).into_response()
        }
        None => (StatusCode::NOT_FOUND, "Configuration not found").into_response(),
    }
}

/// Test agent connection
async fn test_agent_connection(
    State(_state): State<ApiState>,
    Json(config): Json<AgentConfig>,
) -> impl IntoResponse {
    // For now, just validate the configuration format
    if config.agent_type == "deterministic" {
        Json(serde_json::json!({
            "status": "success",
            "message": "Deterministic agent is always available"
        }))
    } else {
        // Validate that API URL and API Key are provided for LLM agents
        match (&config.api_url, &config.api_key) {
            (Some(url), Some(key)) if !url.is_empty() && !key.is_empty() => {
                Json(serde_json::json!({
                    "status": "success",
                    "message": "Configuration appears valid"
                }))
            }
            _ => Json(serde_json::json!({
                "status": "error",
                "message": "API URL and API Key are required for LLM agents"
            })),
        }
    }
}

/// Background task to execute benchmark
async fn execute_benchmark_background(
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

                    // Debug: Log first and last parts of the trace
                    if ascii_trace.len() > 0 {
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

            // Store result in database
            let db_clone = state.db.clone();
            let benchmark_id_clone = benchmark_id.clone();
            let agent_clone = agent.clone();

            tokio::spawn(async move {
                if let Err(e) = store_benchmark_result(
                    &db_clone,
                    &benchmark_id_clone,
                    &agent_clone,
                    test_result.score,
                )
                .await
                {
                    error!("Failed to store benchmark result: {}", e);
                }

                // Store flow log in database
                if let Err(e) =
                    store_flow_log_from_result(&db_clone, &benchmark_id_clone, &test_result).await
                {
                    error!("Failed to store flow log: {}", e);
                }
            });
        }
        Err(e) => {
            error!("Benchmark execution failed: {}", e);
            update_execution_failed(&state, &execution_id, &format!("Execution failed: {e}")).await;
        }
    }

    info!("Benchmark execution completed: {}", execution_id);
}

/// Find benchmark file by ID
fn find_benchmark_file(benchmark_id: &str) -> Option<std::path::PathBuf> {
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
fn generate_transaction_logs(result: &reev_lib::results::TestResult) -> String {
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
async fn update_execution_failed(state: &ApiState, execution_id: &str, error_message: &str) {
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
async fn store_benchmark_result(
    db: &Db,
    benchmark_id: &str,
    agent: &str,
    score: f64,
) -> Result<()> {
    let timestamp = chrono::Utc::now().to_rfc3339();
    let execution_time_ms = 5000; // Mock execution time

    db.insert_agent_performance(
        benchmark_id,
        agent,
        score,
        "Succeeded",
        execution_time_ms,
        &timestamp,
        None,
    )
    .await?;

    Ok(())
}

/// Store flow log in database from test result
async fn store_flow_log_from_result(
    db: &Db,
    benchmark_id: &str,
    test_result: &reev_lib::results::TestResult,
) -> Result<()> {
    use reev_lib::flow::types::{
        EventContent, ExecutionResult, ExecutionStatistics, FlowEvent, FlowLog,
    };
    use std::time::SystemTime;

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

    let flow_log = FlowLog {
        session_id: uuid::Uuid::new_v4().to_string(),
        benchmark_id: benchmark_id.to_string(),
        agent_type: "deterministic".to_string(), // This should come from the execution
        start_time: SystemTime::now(),
        end_time: Some(SystemTime::now()),
        events: vec![FlowEvent {
            timestamp: SystemTime::now(),
            event_type: reev_lib::flow::types::FlowEventType::BenchmarkStateChange,
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
async fn store_flow_log(db: &Db, benchmark_id: &str, trace_data: &str) -> Result<()> {
    use reev_lib::flow::types::{
        EventContent, ExecutionResult, ExecutionStatistics, FlowEvent, FlowLog,
    };
    use std::time::SystemTime;

    let flow_log = FlowLog {
        session_id: uuid::Uuid::new_v4().to_string(),
        benchmark_id: benchmark_id.to_string(),
        agent_type: "deterministic".to_string(),
        start_time: SystemTime::now(),
        end_time: Some(SystemTime::now()),
        events: vec![FlowEvent {
            timestamp: SystemTime::now(),
            event_type: reev_lib::flow::types::FlowEventType::BenchmarkStateChange,
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

/// Get flow logs for a benchmark
async fn get_flow_log(
    State(state): State<ApiState>,
    Path(benchmark_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting YML flow logs for benchmark: {}", benchmark_id);

    match state.db.get_yml_flow_logs(&benchmark_id).await {
        Ok(yml_logs) => {
            info!(
                "Found {} YML logs for benchmark: {}",
                yml_logs.len(),
                benchmark_id
            );
            for (i, log) in yml_logs.iter().enumerate() {
                info!(
                    "YML log {}: length={}, preview={}",
                    i,
                    log.len(),
                    &log[..log.len().min(100)]
                );
            }
            Json(yml_logs).into_response()
        }
        Err(e) => {
            error!("Failed to get YML flow logs: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to get flow logs").into_response()
        }
    }
}

/// Render TestResult as ASCII tree
async fn render_ascii_tree(Json(test_result): Json<serde_json::Value>) -> impl IntoResponse {
    info!("Rendering ASCII tree for TestResult");

    // Parse the TestResult from JSON
    let test_result: reev_lib::results::TestResult = match serde_json::from_value(test_result) {
        Ok(result) => result,
        Err(e) => {
            error!("Failed to parse TestResult: {}", e);
            return (StatusCode::BAD_REQUEST, "Invalid TestResult format").into_response();
        }
    };

    // Render as ASCII tree
    let ascii_tree = reev_runner::renderer::render_result_as_tree(&test_result);

    info!("Successfully rendered ASCII tree");
    (StatusCode::OK, [("Content-Type", "text/plain")], ascii_tree).into_response()
}

/// Parse YML to TestResult
async fn parse_yml_to_testresult(yml_content: String) -> impl IntoResponse {
    info!("Parsing YML to TestResult");
    info!("YML content length: {} chars", yml_content.len());
    info!(
        "YML content preview: {}",
        &yml_content[..yml_content.len().min(200)]
    );

    // Log the first few lines to understand the format
    let lines: Vec<&str> = yml_content.lines().take(5).collect();
    info!("YML first 5 lines: {:?}", lines);

    // Parse YML to TestResult object
    let test_result: reev_lib::results::TestResult = match serde_yaml::from_str(&yml_content) {
        Ok(result) => {
            info!("Successfully parsed YML to TestResult");
            result
        }
        Err(e) => {
            error!("Failed to parse YML to TestResult: {}", e);
            error!(
                "YML content that failed: {}",
                &yml_content[..yml_content.len().min(500)]
            );
            return (
                StatusCode::BAD_REQUEST,
                format!("Invalid YML format: {}", e),
            )
                .into_response();
        }
    };

    info!("Successfully parsed YML to TestResult");
    Json(test_result).into_response()
}
