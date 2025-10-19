//! ASCII tree rendering handlers
use crate::types::*;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use tracing::{error, info, warn};

/// Get ASCII tree directly from YML TestResult in database
pub async fn get_ascii_tree_direct(
    State(state): State<ApiState>,
    Path((benchmark_id, agent_type)): Path<(String, String)>,
) -> impl IntoResponse {
    info!(
        "Getting execution log for benchmark: {} by agent: {}",
        benchmark_id, agent_type
    );

    // Get the most recent session for this benchmark and agent
    let filter = reev_db::types::SessionFilter {
        benchmark_id: Some(benchmark_id.clone()),
        agent_type: Some(agent_type.clone()),
        interface: None,
        status: None,
        limit: Some(1), // Get the most recent session
    };

    match state.db.list_sessions(&filter).await {
        Ok(sessions) => {
            if let Some(session) = sessions.first() {
                info!(
                    "Found session for benchmark: {} by agent: {} with status: {}",
                    benchmark_id,
                    agent_type,
                    session.final_status.as_deref().unwrap_or("Unknown")
                );

                // Check execution status
                match session.final_status.as_deref() {
                    Some("Running") | Some("running") => (
                        StatusCode::OK,
                        [("Content-Type", "text/plain")],
                        "⏳ Execution in progress...".to_string(),
                    )
                        .into_response(),
                    Some("Completed") | Some("Succeeded") | Some("completed")
                    | Some("succeeded") => {
                        // Get the session log which contains the full execution output
                        match state.db.get_session_log(&session.session_id).await {
                            Ok(Some(log_content)) => {
                                if log_content.trim().is_empty() {
                                    return (
                                        StatusCode::OK,
                                        [("Content-Type", "text/plain")],
                                        "📝 No execution data available".to_string(),
                                    )
                                        .into_response();
                                }

                                // Parse the flow log and format it as readable trace
                                match serde_json::from_str::<reev_flow::FlowLog>(&log_content) {
                                    Ok(flow_log) => {
                                        // Format the flow log as a readable trace
                                        let mut formatted_trace = String::new();

                                        // Add prompt
                                        if !flow_log.events.is_empty() {
                                            // Try to extract prompt from first event or use the prompt from the parsed data
                                            if let Some(parsed) =
                                                log_content.split("\"prompt\":\"").nth(1)
                                            {
                                                if let Some(prompt) = parsed.split("\"").next() {
                                                    formatted_trace.push_str(&format!(
                                                        "📝 Prompt: {}\n\n",
                                                        prompt
                                                    ));
                                                }
                                            }
                                        }

                                        // Add steps from parsed data
                                        if let Some(steps_start) = log_content.find("\"steps\":[") {
                                            if let Some(steps_end) =
                                                log_content[steps_start + 10..].find("]")
                                            {
                                                let steps_str = &log_content[steps_start + 10
                                                    ..steps_start + 10 + steps_end];
                                                if let Ok(steps) =
                                                    serde_json::from_str::<serde_json::Value>(
                                                        format!("[{}]", steps_str).as_str(),
                                                    )
                                                {
                                                    if let Some(steps_array) = steps.as_array() {
                                                        for (i, step) in
                                                            steps_array.iter().enumerate()
                                                        {
                                                            formatted_trace.push_str(&format!(
                                                                "✓ Step {}\n",
                                                                i + 1
                                                            ));

                                                            if let Some(action) = step.get("action")
                                                            {
                                                                if let Some(action_array) =
                                                                    action.as_array()
                                                                {
                                                                    formatted_trace.push_str(
                                                                        "   ├─ ACTION:\n",
                                                                    );
                                                                    for action_item in action_array
                                                                    {
                                                                        if let Some(program_id) =
                                                                            action_item
                                                                                .get("program_id")
                                                                        {
                                                                            formatted_trace.push_str(&format!("      Program ID: {}\n", program_id));
                                                                        }
                                                                        if let Some(accounts) =
                                                                            action_item
                                                                                .get("accounts")
                                                                        {
                                                                            if let Some(
                                                                                accounts_array,
                                                                            ) =
                                                                                accounts.as_array()
                                                                            {
                                                                                formatted_trace.push_str("      Accounts:\n");
                                                                                for (
                                                                                    idx,
                                                                                    account,
                                                                                ) in
                                                                                    accounts_array
                                                                                        .iter()
                                                                                        .enumerate()
                                                                                {
                                                                                    if let Some(
                                                                                        pubkey,
                                                                                    ) = account
                                                                                        .get(
                                                                                        "pubkey",
                                                                                    ) {
                                                                                        let is_signer = account.get("is_signer").and_then(|v| v.as_bool()).unwrap_or(false);
                                                                                        let is_writable = account.get("is_writable").and_then(|v| v.as_bool()).unwrap_or(false);
                                                                                        let icon = if is_signer { "🖋️" } else { "🖍️" };
                                                                                        let arrow = if is_writable { "➕" } else { "➖" };
                                                                                        formatted_trace.push_str(&format!("      [{}] {} {} {}\n", idx, icon, arrow, pubkey));
                                                                                    }
                                                                                }
                                                                            }
                                                                        }
                                                                        if let Some(data) =
                                                                            action_item.get("data")
                                                                        {
                                                                            formatted_trace.push_str(&format!("      Data (Base58): {}\n", data));
                                                                        }
                                                                    }
                                                                }
                                                            }

                                                            if let Some(observation) =
                                                                step.get("observation")
                                                            {
                                                                formatted_trace.push_str(
                                                                    "   └─ OBSERVATION: ",
                                                                );
                                                                if let Some(status) = observation
                                                                    .get("last_transaction_status")
                                                                {
                                                                    formatted_trace.push_str(
                                                                        &format!("{}\n", status),
                                                                    );
                                                                }
                                                                if let Some(error) = observation
                                                                    .get("last_transaction_error")
                                                                {
                                                                    if !error
                                                                        .as_str()
                                                                        .unwrap_or("")
                                                                        .is_empty()
                                                                    {
                                                                        formatted_trace.push_str(
                                                                            &format!(
                                                                                "   Error: {}\n",
                                                                                error
                                                                            ),
                                                                        );
                                                                    }
                                                                }
                                                            }
                                                            formatted_trace.push('\n');
                                                        }
                                                    }
                                                }
                                            }
                                        }

                                        // Add final result if available
                                        if let Some(final_result) = flow_log.final_result {
                                            formatted_trace.push_str(&format!(
                                                "✅ Execution completed - Score: {:.1}%\n",
                                                final_result.score * 100.0
                                            ));
                                        }

                                        (
                                            StatusCode::OK,
                                            [("Content-Type", "text/plain")],
                                            formatted_trace,
                                        )
                                            .into_response()
                                    }
                                    Err(_) => {
                                        // Fallback: try to parse as simple JSON structure
                                        if log_content.trim().starts_with("{") {
                                            if let Ok(parsed) =
                                                serde_json::from_str::<serde_json::Value>(
                                                    &log_content,
                                                )
                                            {
                                                let mut formatted = String::new();

                                                if let Some(prompt) =
                                                    parsed.get("prompt").and_then(|v| v.as_str())
                                                {
                                                    formatted.push_str(&format!(
                                                        "📝 Prompt: {}\n\n",
                                                        prompt
                                                    ));
                                                }

                                                if let Some(steps) =
                                                    parsed.get("steps").and_then(|v| v.as_array())
                                                {
                                                    for (i, step) in steps.iter().enumerate() {
                                                        formatted.push_str(&format!(
                                                            "✓ Step {}\n",
                                                            i + 1
                                                        ));

                                                        if let Some(action) = step.get("action") {
                                                            formatted.push_str("   ├─ ACTION:\n");
                                                            if let Some(action_array) =
                                                                action.as_array()
                                                            {
                                                                for action_item in action_array {
                                                                    if let Some(program_id) =
                                                                        action_item
                                                                            .get("program_id")
                                                                    {
                                                                        formatted.push_str(&format!("      Program ID: {}\n", program_id));
                                                                    }
                                                                    if let Some(accounts) =
                                                                        action_item.get("accounts")
                                                                    {
                                                                        if let Some(
                                                                            accounts_array,
                                                                        ) = accounts.as_array()
                                                                        {
                                                                            formatted.push_str(
                                                                                "      Accounts:\n",
                                                                            );
                                                                            for (idx, account) in
                                                                                accounts_array
                                                                                    .iter()
                                                                                    .enumerate()
                                                                            {
                                                                                if let Some(
                                                                                    pubkey,
                                                                                ) = account
                                                                                    .get("pubkey")
                                                                                {
                                                                                    let is_signer = account.get("is_signer").and_then(|v| v.as_bool()).unwrap_or(false);
                                                                                    let is_writable = account.get("is_writable").and_then(|v| v.as_bool()).unwrap_or(false);
                                                                                    let icon =
                                                                                        if is_signer
                                                                                        {
                                                                                            "🖋️"
                                                                                        } else {
                                                                                            "🖍️"
                                                                                        };
                                                                                    let arrow = if is_writable { "➕" } else { "➖" };
                                                                                    formatted.push_str(&format!("      [{}] {} {} {}\n", idx, icon, arrow, pubkey));
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                    if let Some(data) =
                                                                        action_item.get("data")
                                                                    {
                                                                        formatted.push_str(&format!("      Data (Base58): {}\n", data));
                                                                    }
                                                                }
                                                            }
                                                        }

                                                        if let Some(observation) =
                                                            step.get("observation")
                                                        {
                                                            formatted
                                                                .push_str("   └─ OBSERVATION: ");
                                                            if let Some(status) = observation
                                                                .get("last_transaction_status")
                                                            {
                                                                formatted.push_str(&format!(
                                                                    "{}\n",
                                                                    status
                                                                ));
                                                            }
                                                            if let Some(error) = observation
                                                                .get("last_transaction_error")
                                                            {
                                                                if !error
                                                                    .as_str()
                                                                    .unwrap_or("")
                                                                    .is_empty()
                                                                {
                                                                    formatted.push_str(&format!(
                                                                        "   Error: {}\n",
                                                                        error
                                                                    ));
                                                                }
                                                            }
                                                        }
                                                        formatted.push('\n');
                                                    }
                                                }

                                                (
                                                    StatusCode::OK,
                                                    [("Content-Type", "text/plain")],
                                                    formatted,
                                                )
                                                    .into_response()
                                            } else {
                                                // If all parsing fails, return raw content
                                                (
                                                    StatusCode::OK,
                                                    [("Content-Type", "text/plain")],
                                                    format!(
                                                        "📊 Execution Trace:\n\n{}",
                                                        &log_content[..log_content.len().min(1000)]
                                                    ),
                                                )
                                                    .into_response()
                                            }
                                        } else {
                                            // Not JSON, return as-is
                                            (
                                                StatusCode::OK,
                                                [("Content-Type", "text/plain")],
                                                format!(
                                                    "📊 Execution Trace:\n\n{}",
                                                    &log_content[..log_content.len().min(1000)]
                                                ),
                                            )
                                                .into_response()
                                        }
                                    }
                                }
                            }
                            Ok(None) => (
                                StatusCode::OK,
                                [("Content-Type", "text/plain")],
                                "📝 No execution data available".to_string(),
                            )
                                .into_response(),
                            Err(e) => {
                                warn!("Failed to get session log: {}", e);
                                (
                                    StatusCode::INTERNAL_SERVER_ERROR,
                                    [("Content-Type", "text/plain")],
                                    "❌ Failed to retrieve execution data".to_string(),
                                )
                                    .into_response()
                            }
                        }
                    }
                    Some("Failed") | Some("failed") | Some("Error") | Some("error") => {
                        // Get the session log even for failed executions to show error details
                        match state.db.get_session_log(&session.session_id).await {
                            Ok(Some(log_content)) => {
                                if log_content.trim().is_empty() {
                                    return (
                                        StatusCode::OK,
                                        [("Content-Type", "text/plain")],
                                        "❌ Execution failed - No details available".to_string(),
                                    )
                                        .into_response();
                                }

                                // Parse and format even for failed executions
                                match serde_json::from_str::<serde_json::Value>(&log_content) {
                                    Ok(parsed) => {
                                        let mut formatted =
                                            String::from("❌ Execution Failed:\n\n");

                                        if let Some(prompt) =
                                            parsed.get("prompt").and_then(|v| v.as_str())
                                        {
                                            formatted
                                                .push_str(&format!("📝 Prompt: {}\n\n", prompt));
                                        }

                                        if let Some(steps) =
                                            parsed.get("steps").and_then(|v| v.as_array())
                                        {
                                            for (i, step) in steps.iter().enumerate() {
                                                formatted.push_str(&format!("✓ Step {}\n", i + 1));

                                                if let Some(action) = step.get("action") {
                                                    formatted.push_str("   ├─ ACTION:\n");
                                                    if let Some(action_array) = action.as_array() {
                                                        for action_item in action_array {
                                                            if let Some(program_id) =
                                                                action_item.get("program_id")
                                                            {
                                                                formatted.push_str(&format!(
                                                                    "      Program ID: {}\n",
                                                                    program_id
                                                                ));
                                                            }
                                                            if let Some(accounts) =
                                                                action_item.get("accounts")
                                                            {
                                                                if let Some(accounts_array) =
                                                                    accounts.as_array()
                                                                {
                                                                    formatted.push_str(
                                                                        "      Accounts:\n",
                                                                    );
                                                                    for (idx, account) in
                                                                        accounts_array
                                                                            .iter()
                                                                            .enumerate()
                                                                    {
                                                                        if let Some(pubkey) =
                                                                            account.get("pubkey")
                                                                        {
                                                                            let is_signer = account
                                                                                .get("is_signer")
                                                                                .and_then(|v| {
                                                                                    v.as_bool()
                                                                                })
                                                                                .unwrap_or(false);
                                                                            let is_writable = account.get("is_writable").and_then(|v| v.as_bool()).unwrap_or(false);
                                                                            let icon = if is_signer
                                                                            {
                                                                                "🖋️"
                                                                            } else {
                                                                                "🖍️"
                                                                            };
                                                                            let arrow =
                                                                                if is_writable {
                                                                                    "➕"
                                                                                } else {
                                                                                    "➖"
                                                                                };
                                                                            formatted.push_str(&format!("      [{}] {} {} {}\n", idx, icon, arrow, pubkey));
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                            if let Some(data) =
                                                                action_item.get("data")
                                                            {
                                                                formatted.push_str(&format!(
                                                                    "      Data (Base58): {}\n",
                                                                    data
                                                                ));
                                                            }
                                                        }
                                                    }
                                                }

                                                if let Some(observation) = step.get("observation") {
                                                    formatted.push_str("   └─ OBSERVATION: ");
                                                    if let Some(status) =
                                                        observation.get("last_transaction_status")
                                                    {
                                                        formatted
                                                            .push_str(&format!("{}\n", status));
                                                    }
                                                    if let Some(error) =
                                                        observation.get("last_transaction_error")
                                                    {
                                                        if !error.as_str().unwrap_or("").is_empty()
                                                        {
                                                            formatted.push_str(&format!(
                                                                "   Error: {}\n",
                                                                error
                                                            ));
                                                        }
                                                    }
                                                }
                                                formatted.push('\n');
                                            }
                                        }

                                        (
                                            StatusCode::OK,
                                            [("Content-Type", "text/plain")],
                                            formatted,
                                        )
                                            .into_response()
                                    }
                                    Err(_) => {
                                        // If parsing fails, return raw content
                                        (
                                            StatusCode::OK,
                                            [("Content-Type", "text/plain")],
                                            format!(
                                                "❌ Execution Failed:\n\n{}",
                                                &log_content[..log_content.len().min(1000)]
                                            ),
                                        )
                                            .into_response()
                                    }
                                }
                            }
                            Ok(None) => (
                                StatusCode::OK,
                                [("Content-Type", "text/plain")],
                                "❌ Execution failed - No details available".to_string(),
                            )
                                .into_response(),
                            Err(e) => {
                                warn!("Failed to get session log: {}", e);
                                (
                                    StatusCode::INTERNAL_SERVER_ERROR,
                                    [("Content-Type", "text/plain")],
                                    "❌ Failed to retrieve error details".to_string(),
                                )
                                    .into_response()
                            }
                        }
                    }
                    _ => {
                        info!(
                            "Unknown session status: {} for benchmark: {}",
                            session.final_status.as_deref().unwrap_or("None"),
                            benchmark_id
                        );
                        (
                            StatusCode::OK,
                            [("Content-Type", "text/plain")],
                            format!(
                                "❓ Unknown execution status: {}",
                                session.final_status.as_deref().unwrap_or("None")
                            ),
                        )
                            .into_response()
                    }
                }
            } else {
                info!(
                    "No sessions found for benchmark: {} by agent: {}",
                    benchmark_id, agent_type
                );
                (
                    StatusCode::OK,
                    [("Content-Type", "text/plain")],
                    "📝 No execution history found".to_string(),
                )
                    .into_response()
            }
        }
        Err(e) => {
            error!("Failed to list sessions: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                [("Content-Type", "text/plain")],
                "❌ Failed to retrieve execution history".to_string(),
            )
                .into_response()
        }
    }
}

/// Render ASCII tree from JSON test result
pub async fn render_ascii_tree(
    State(_state): State<ApiState>,
    Json(_payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    info!("Rendering ASCII tree from JSON payload");

    // For now, return a simple message
    // TODO: Implement proper ASCII tree rendering from JSON
    (
        StatusCode::OK,
        [("Content-Type", "text/plain")],
        "🌳 ASCII tree rendering from JSON not yet implemented".to_string(),
    )
        .into_response()
}

/// Parse YML to TestResult
pub async fn parse_yml_to_testresult(
    State(_state): State<ApiState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    info!("Parsing YML to TestResult");

    // For now, return a simple message
    // TODO: Implement proper YML parsing
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "message": "YML parsing not yet implemented",
            "received": payload
        })),
    )
        .into_response()
}

/// Get ASCII tree from execution state (temporary endpoint)
pub async fn get_ascii_tree_from_state(
    State(state): State<ApiState>,
    Path((benchmark_id, agent_type)): Path<(String, String)>,
) -> impl IntoResponse {
    info!(
        "Getting ASCII tree from execution state for benchmark: {} by agent: {}",
        benchmark_id, agent_type
    );

    // Check if there's an active execution in memory
    let executions = state.executions.lock().await;
    let execution_key = format!("{benchmark_id}:{agent_type}");

    if let Some(execution) = executions.get(&execution_key) {
        match execution.status {
            crate::types::ExecutionStatus::Running => (
                StatusCode::OK,
                [("Content-Type", "text/plain")],
                "⏳ Execution in progress...".to_string(),
            )
                .into_response(),
            crate::types::ExecutionStatus::Completed => (
                StatusCode::OK,
                [("Content-Type", "text/plain")],
                format!(
                    "✅ Execution completed\n\nTrace:\n{}",
                    &execution.trace[..execution.trace.len().min(500)]
                ),
            )
                .into_response(),
            crate::types::ExecutionStatus::Failed => (
                StatusCode::OK,
                [("Content-Type", "text/plain")],
                format!(
                    "❌ Execution failed\n\nError: {}\n\nTrace:\n{}",
                    execution.error.as_deref().unwrap_or("Unknown error"),
                    &execution.trace[..execution.trace.len().min(500)]
                ),
            )
                .into_response(),
            _ => (
                StatusCode::OK,
                [("Content-Type", "text/plain")],
                "📝 Execution status unknown".to_string(),
            )
                .into_response(),
        }
    } else {
        (
            StatusCode::OK,
            [("Content-Type", "text/plain")],
            "📝 No active execution found".to_string(),
        )
            .into_response()
    }
}
