mod handlers;
mod services;
mod types;

use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use reev_lib::db::{DatabaseConfig, PooledDatabaseWriter};
use reev_lib::server_utils::kill_existing_api;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use handlers::*;
use services::*;
use types::ApiState;

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

    // Clean up any existing API processes on the default port
    let default_port = std::env::var("PORT")
        .unwrap_or_else(|_| "3001".to_string())
        .parse()
        .unwrap_or(3001);

    kill_existing_api(default_port).await?;

    // Initialize database
    let db_path =
        std::env::var("DATABASE_PATH").unwrap_or_else(|_| "db/reev_results.db".to_string());
    info!("Connecting to database at: {}", db_path);

    let db_config = DatabaseConfig::new(&db_path);
    let db = PooledDatabaseWriter::new(db_config, 10).await?;
    info!("Database connection pool established");

    // Sync benchmarks to database on startup
    let benchmarks_dir = "benchmarks";
    info!("Syncing benchmarks from directory: {}", benchmarks_dir);
    let synced_count = db
        .sync_benchmarks_from_dir(benchmarks_dir)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to sync benchmarks: {e}"))?;
    info!(
        "Successfully synced {:?} benchmarks to database",
        synced_count
    );

    // Initialize benchmark executor
    let benchmark_executor = Arc::new(PooledBenchmarkExecutor::new(
        Arc::new(db.clone()),
        reev_types::RunnerConfig::default(),
        reev_types::TimeoutConfig::default(),
    ));

    // Create API state
    let state = ApiState {
        db: db.clone(),
        executions: Arc::new(Mutex::new(HashMap::new())),
        agent_configs: Arc::new(Mutex::new(HashMap::new())),
        benchmark_executor,
    };

    // Create router with state - simple approach for testing
    let app = Router::new()
        // Health check
        .route("/api/v1/health", get(health_check))
        // General routes
        .route("/api/v1/benchmarks", get(list_benchmarks))
        .route("/api/v1/agents", get(list_agents))
        .route("/api/v1/agent-performance", get(get_agent_performance))
        // Debug endpoints
        .route(
            "/api/v1/debug/agent-performance-raw",
            get(debug_agent_performance_raw),
        )
        .route(
            "/api/v1/debug/execution-sessions",
            get(debug_execution_sessions),
        )
        .route(
            "/api/v1/debug/insert-test-data",
            get(debug_insert_test_data),
        )
        // Benchmark execution endpoints
        .route("/api/v1/benchmarks/{id}/run", post(run_benchmark))
        .route(
            "/api/v1/benchmarks/{id}/status/{execution_id}",
            get(get_execution_status),
        )
        .route(
            "/api/v1/benchmarks/{id}/status",
            get(get_execution_status_no_id),
        )
        .route(
            "/api/v1/benchmarks/{id}/stop/{execution_id}",
            post(stop_benchmark),
        )
        // Agent configuration endpoints
        .route("/api/v1/agents/config", post(save_agent_config))
        .route("/api/v1/agents/config/{agent_type}", get(get_agent_config))
        // Debug endpoints
        .route("/api/v1/debug/benchmarks", get(debug_benchmarks))
        .route("/api/v1/agents/test", post(test_agent_connection))
        // Flow logs endpoints
        .route("/api/v1/flow-logs/{benchmark_id}", get(get_flow_log))
        .route("/api/v1/flows/{session_id}", get(get_flow))
        .route(
            "/api/v1/transaction-logs/{benchmark_id}",
            get(get_transaction_logs),
        )
        .route(
            "/api/v1/transaction-logs/demo",
            get(get_transaction_logs_demo),
        )
        // Execution trace endpoints
        .route(
            "/api/v1/execution-logs/{benchmark_id}",
            get(get_execution_trace),
        )
        // Benchmark management endpoints
        .route("/api/v1/upsert-yml", post(upsert_yml))
        .route("/api/v1/sync", post(sync_benchmarks))
        .route("/api/v1/test-on-conflict", post(test_on_conflict))
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

    // Graceful shutdown handling
    let graceful_shutdown = async move {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
        info!("Received Ctrl+C signal, shutting down gracefully...");

        // Gracefully shutdown database connections
        info!("Shutting down database connection pool...");
        if let Err(e) = db.shutdown().await {
            tracing::warn!("Error shutting down database: {}", e);
        } else {
            info!("Database connections closed successfully");
        }
    };

    axum::serve(listener, app)
        .with_graceful_shutdown(graceful_shutdown)
        .await?;

    info!("API server shutdown complete");
    Ok(())
}
