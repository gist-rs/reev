//! Health check and test endpoints handlers
use crate::types::*;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};

use tracing::{error, info};

/// Health check endpoint
pub async fn health_check() -> Json<HealthResponse> {
    let response = HealthResponse {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        version: "0.1.0".to_string(),
    };
    Json(response)
}

/// Simple test endpoint
/// Helper function to create error responses
#[allow(dead_code)]
pub fn create_error_response(
    status: StatusCode,
    message: String,
) -> (StatusCode, Json<ErrorResponse>) {
    let response = ErrorResponse {
        error: status.as_str().to_string(),
        message,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    (status, Json(response))
}

/// Debug endpoint to check benchmarks table
pub async fn debug_benchmarks(State(state): State<ApiState>) -> impl IntoResponse {
    let db = &state.db;

    // Get all benchmarks from database
    let filter = reev_db::QueryFilter::new();
    match db.list_benchmarks(&filter).await {
        Ok(benchmarks) => {
            let debug_info: Vec<serde_json::Value> = benchmarks
                .into_iter()
                .map(|b| {
                    serde_json::json!({
                        "id": b.id,
                        "prompt_preview": b.prompt.chars().take(50).collect::<String>(),
                        "benchmark_name": b.benchmark_name,
                        "created_at": b.created_at,
                    })
                })
                .collect();

            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "success": true,
                    "count": debug_info.len(),
                    "benchmarks": debug_info
                })),
            )
        }
        Err(e) => {
            error!("Failed to get benchmarks: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to get benchmarks: {}", e)
                })),
            )
        }
    }
}

/// Test ON CONFLICT behavior with simple data
pub async fn test_on_conflict(State(state): State<ApiState>) -> impl IntoResponse {
    let db = &state.db;

    // Test 1: Insert first record using existing upsert function
    let benchmark_data1 = reev_db::types::BenchmarkData {
        id: "test-id-1".to_string(),
        benchmark_name: "test-conflict-1".to_string(),
        prompt: "test-prompt-1".to_string(),
        content: "test-content-1".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    let result1 = db.upsert_benchmark(&benchmark_data1).await;

    // Test 2: Insert second record with SAME benchmark_name AND SAME prompt (should trigger conflict)
    let benchmark_data2 = reev_db::types::BenchmarkData {
        id: "test-id-1".to_string(), // Same MD5 as first record
        benchmark_name: "test-conflict-1".to_string(),
        prompt: "test-prompt-1".to_string(), // SAME prompt - should generate same MD5
        content: "test-content-1-updated".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    let result2 = db.upsert_benchmark(&benchmark_data2).await;

    // Check results using existing database functions
    let stats = db.get_database_stats().await.unwrap_or_default();
    let total_records = stats.total_benchmarks;

    let success = total_records == 1;
    let message = if success {
        "✅ Turso ON CONFLICT works correctly - no duplicates created"
    } else {
        "❌ Turso ON CONFLICT failed - duplicates created"
    };

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": success,
            "message": message,
            "test_results": {
                "first_insert_result": format!("{:?}", result1),
                "second_insert_result": format!("{:?}", result2),
                "total_records": total_records,
                "expected_records": 1
            }
        })),
    )
}

/// Sync benchmarks from filesystem to database
pub async fn sync_benchmarks(State(state): State<ApiState>) -> impl IntoResponse {
    let db = &state.db;
    let benchmarks_dir = "benchmarks";

    info!(
        "Starting manual benchmark sync from directory: {}",
        benchmarks_dir
    );

    match db.sync_benchmarks_from_dir(benchmarks_dir).await {
        Ok(synced_count) => {
            info!(
                "Successfully synced {:?} benchmarks to database",
                synced_count
            );
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "success": true,
                    "synced_count": synced_count,
                    "message": format!("Successfully synced {:?} benchmarks from {}", synced_count, benchmarks_dir)
                })),
            )
        }
        Err(e) => {
            error!("Failed to sync benchmarks: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to sync benchmarks: {}", e)
                })),
            )
        }
    }
}
