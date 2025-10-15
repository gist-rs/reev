mod handlers;
mod services;
mod types;

use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use reev_lib::db::{DatabaseConfig, DatabaseWriter};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use handlers::*;
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

    // Initialize database
    let db_path =
        std::env::var("DATABASE_PATH").unwrap_or_else(|_| "db/reev_results.db".to_string());
    info!("Connecting to database at: {}", db_path);

    let db_config = DatabaseConfig::new(&db_path);
    let db = Arc::new(DatabaseWriter::new(db_config).await?);
    info!("Database connection established");

    // Sync benchmarks to database on startup
    let benchmarks_dir = "benchmarks";
    info!("Syncing benchmarks from directory: {}", benchmarks_dir);
    let synced_count = db
        .sync_benchmarks_to_db(benchmarks_dir)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to sync benchmarks: {e}"))?;
    info!(
        "Successfully synced {} benchmarks to database",
        synced_count
    );

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
        // Debug endpoints
        .route("/api/v1/debug/benchmarks", get(debug_benchmarks))
        .route(
            "/api/v1/debug/test-prompt-lookup",
            post(test_prompt_md5_lookup),
        )
        .route("/api/v1/agents/test", post(test_agent_connection))
        // Flow logs endpoints
        .route("/api/v1/flow-logs/{benchmark_id}", get(get_flow_log))
        .route(
            "/api/v1/parse-yml-to-testresult",
            post(parse_yml_to_testresult),
        )
        .route("/api/v1/render-ascii-tree", post(render_ascii_tree))
        // Benchmark management endpoints
        .route("/api/v1/upsert-yml", post(upsert_yml))
        .route("/api/v1/sync", post(sync_benchmarks))
        .route("/api/v1/test-on-conflict", post(test_on_conflict))
        // YML TestResult endpoints for historical access (removed - use ascii-tree endpoint instead)
        .route(
            "/api/v1/ascii-tree/{benchmark_id}/{agent_type}",
            get(get_ascii_tree_direct),
        )
        // Temporary endpoint for ASCII tree from execution state
        .route(
            "/api/v1/ascii-tree-state/{benchmark_id}/{agent_type}",
            get(get_ascii_tree_from_state),
        )
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
