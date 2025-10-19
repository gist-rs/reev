//! YML management handlers

use crate::types::ApiState;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use reev_lib::db::BenchmarkYml;

use serde::{Deserialize, Serialize};
use tracing::{error, info};

/// Upsert YML content to database
pub async fn upsert_yml(
    State(app_state): State<ApiState>,
    Json(payload): Json<UpsertYmlRequest>,
) -> impl IntoResponse {
    let db = &app_state.db;

    // Validate YML content
    let benchmark_data: BenchmarkYml = match serde_yaml::from_str(&payload.yml_content) {
        Ok(data) => data,
        Err(e) => {
            error!("Invalid YAML format: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("Invalid YAML format: {}", e)
                })),
            )
                .into_response();
        }
    };

    // Upsert to database
    let prompt_md5 = match db
        .upsert_benchmark(
            &benchmark_data.id,
            &benchmark_data.prompt,
            &payload.yml_content,
        )
        .await
    {
        Ok(id) => id,
        Err(e) => {
            error!("Failed to upsert benchmark: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to upsert benchmark: {}", e)
                })),
            )
                .into_response();
        }
    };

    info!("Upserted benchmark with MD5: {}", prompt_md5);

    (
        StatusCode::OK,
        Json(UpsertYmlResponse {
            success: true,
            benchmark_id: prompt_md5,
            message: "Benchmark upserted successfully".to_string(),
        }),
    )
        .into_response()
}

/// Request body for upsert_yml endpoint
#[derive(Debug, Deserialize)]
pub struct UpsertYmlRequest {
    pub yml_content: String,
}

/// Response body for upsert_yml endpoint
#[derive(Debug, Serialize)]
pub struct UpsertYmlResponse {
    pub success: bool,
    pub benchmark_id: String,
    pub message: String,
}
