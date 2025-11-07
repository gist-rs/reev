//! Integration tests for consolidation API endpoints
//!
//! This module tests the consolidated session retrieval and management
//! endpoints added as part of Phase 4 - API Integration.

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use reev_api::handlers::{
    get_consolidated_session, get_consolidation_status, get_execution_consolidated_session,
};
use reev_api::types::ApiState;
use reev_db::writer::DatabaseWriterTrait;
use serde_json::{json, Value};
use std::sync::Arc;
use tempfile::TempDir;
use tower::ServiceExt;

/// Create a test database and API state
async fn setup_test_state() -> (ApiState, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");

    let config = reev_db::DatabaseConfig::new(db_path.to_str().unwrap());
    let db = reev_db::PooledDatabaseWriter::new(config, 5)
        .await
        .expect("Failed to create test database");

    let state = ApiState {
        db: db.clone(),
        agent_configs: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        benchmark_executor: Arc::new(reev_services::PooledBenchmarkExecutor::new_with_default(
            Arc::new(db),
        )),
    };

    (state, temp_dir)
}

/// Create an app with consolidation routes
fn create_app(state: ApiState) -> Router {
    Router::new()
        .route(
            "/api/v1/sessions/consolidated/:session_id",
            axum::routing::get(get_consolidated_session),
        )
        .route(
            "/api/v1/executions/:execution_id/consolidated",
            axum::routing::get(get_execution_consolidated_session),
        )
        .route(
            "/api/v1/consolidation/:execution_id/status",
            axum::routing::get(get_consolidation_status),
        )
        .with_state(state)
}

/// Test retrieving a non-existent consolidated session
#[tokio::test]
async fn test_get_nonexistent_consolidated_session() {
    let (state, _temp_dir) = setup_test_state().await;
    let app = create_app(state);

    let request = Request::builder()
        .uri("/api/v1/sessions/consolidated/nonexistent")
        .method("GET")
        .header("content-type", "application/json")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let error_response: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(
        error_response["error"],
        "Consolidated session not found"
    );
    assert_eq!(error_response["session_id"], "nonexistent");
}

/// Test retrieving consolidated session for non-existent execution
#[tokio::test]
async fn test_get_nonexistent_execution_consolidated() {
    let (state, _temp_dir) = setup_test_state().await;
    let app = create_app(state);

    let request = Request::builder()
        .uri("/api/v1/executions/nonexistent/consolidated")
        .method("GET")
        .header("content-type", "application/json")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let error_response: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(
        error_response["error"],
        "No consolidated sessions found for this execution"
    );
    assert_eq!(error_response["execution_id"], "nonexistent");
}

/// Test consolidation status for non-existent execution
#[tokio::test]
async fn test_consolidation_status_nonexistent() {
    let (state, _temp_dir) = setup_test_state().await;
    let app = create_app(state);

    let request = Request::builder()
        .uri("/api/v1/consolidation/nonexistent/status")
        .method("GET")
        .header("content-type", "application/json")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let status_response: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(status_response["execution_id"], "nonexistent");
    assert_eq!(status_response["complete"], false);
    assert_eq!(status_response["status"], "pending");
    assert_eq!(
        status_response["message"],
        "No step sessions found for consolidation"
    );
    assert!(status_response["metadata"].is_null());
}

/// Test storing and retrieving a consolidated session
#[tokio::test]
async fn test_store_and_retrieve_consolidated_session() {
    let (state, _temp_dir) = setup_test_state().await;

    // Store test step sessions first
    let execution_id = "test-execution-123";
    let session_id_1 = "session-1";
    let session_id_2 = "session-2";

    let session_content_1 = r#"{
        "session_id": "session-1",
        "tool_calls": [
            {
                "tool_name": "get_account_balance",
                "success": true,
                "start_time": 1000,
                "end_time": 2000
            }
        ]
    }"#;

    let session_content_2 = r#"{
        "session_id": "session-2",
        "tool_calls": [
            {
                "tool_name": "jupiter_swap",
                "success": true,
                "start_time": 3000,
                "end_time": 5000
            }
        ]
    }"#;

    // Store step sessions
    state
        .db
        .store_step_session(execution_id, 0, session_id_1, session_content_1)
        .await
        .expect("Failed to store step session 1");

    state
        .db
        .store_step_session(execution_id, 1, session_id_2, session_content_2)
        .await
        .expect("Failed to store step session 2");

    // Store consolidated session
    let consolidated_id = "consolidated-123";
    let consolidated_content = json!({
        "execution_id": execution_id,
        "consolidated": true,
        "steps": [
            {
                "step_index": 0,
                "session_id": "session-1",
                "tool_name": "get_account_balance",
                "success": true,
                "duration_ms": 1000
            },
            {
                "step_index": 1,
                "session_id": "session-2",
                "tool_name": "jupiter_swap",
                "success": true,
                "duration_ms": 2000
            }
        ],
        "metadata": {
            "avg_score": 0.85,
            "total_tools": 2,
            "success_rate": 100.0,
            "execution_duration_ms": 3000
        }
    });

    let metadata = reev_db::shared::performance::ConsolidationMetadata::with_values(
        Some(0.85),
        Some(2),
        Some(100.0),
        Some(3000),
    );

    state
        .db
        .store_consolidated_session(
            consolidated_id,
            execution_id,
            &consolidated_content.to_string(),
            &metadata,
        )
        .await
        .expect("Failed to store consolidated session");

    // Test retrieval by session ID
    let app = create_app(state.clone());
    let request = Request::builder()
        .uri(&format!("/api/v1/sessions/consolidated/{}", consolidated_id))
        .method("GET")
        .header("content-type", "application/json")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let session_response: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(session_response["session_id"], consolidated_id);
    assert_eq!(session_response["execution_id"], execution_id);
    assert_eq!(session_response["success"], true);
    assert!(session_response["content"].is_string());
    assert_eq!(session_response["metadata"]["avg_score"], 0.85);
    assert_eq!(session_response["metadata"]["total_tools"], Some(2));
    assert_eq!(session_response["metadata"]["success_rate"], Some(100.0));
    assert_eq!(session_response["metadata"]["execution_duration_ms"], Some(3000));
    assert_eq!(session_response["metadata"]["session_count"], 2);

    // Test retrieval by execution ID
    let request = Request::builder()
        .uri(&format!("/api/v1/executions/{}/consolidated", execution_id))
        .method("GET")
        .header("content-type", "application/json")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let exec_response: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(exec_response["session_id"], consolidated_id);
    assert_eq!(exec_response["execution_id"], execution_id);
    assert_eq!(exec_response["success"], true);

    // Test consolidation status
    let request = Request::builder()
        .uri(&format!("/api/v1/consolidation/{}/status", execution_id))
        .method("GET")
        .header("content-type", "application/json")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let status_response: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(status_response["execution_id"], execution_id);
    assert_eq!(status_response["complete"], true);
    assert_eq!(status_response["status"], "completed");
    assert_eq!(status_response["consolidated_session_id"], consolidated_id);
    assert_eq!(
        status_response["message"],
        "Consolidation completed successfully"
    );
    assert!(!status_response["metadata"].is_null());
    assert_eq!(status_response["metadata"]["avg_score"], 0.85);
}

/// Test consolidation status for in-progress execution
#[tokio::test]
async fn test_consolidation_status_in_progress() {
    let (state, _temp_dir) = setup_test_state().await;

    // Store step sessions only (no consolidation)
    let execution_id = "in-progress-execution";
    let session_id = "session-1";

    let session_content = r#"{
        "session_id": "session-1",
        "tool_calls": [
            {
                "tool_name": "get_account_balance",
                "success": true,
                "start_time": 1000,
                "end_time": 2000
            }
        ]
    }"#;

    state
        .db
        .store_step_session(execution_id, 0, session_id, session_content)
        .await
        .expect("Failed to store step session");

    let app = create_app(state);
    let request = Request::builder()
        .uri(&format!("/api/v1/consolidation/{}/status", execution_id))
        .method("GET")
        .header("content-type", "application/json")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let status_response: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(status_response["execution_id"], execution_id);
    assert_eq!(status_response["complete"], false);
    assert_eq!(status_response["status"], "in_progress");
    assert_eq!(
        status_response["message"],
        "Consolidation in progress (60s timeout)"
    );
    assert_eq!(status_response["consolidated_session_id"], Value::Null);
    assert!(status_response["metadata"].is_null());
}
