//! Tests for shared flow converter utilities

use reev_db::shared::flow::converter::{FlowConverter, FlowLogConverter};
use reev_db::shared::flow::types::ConversionError;
use reev_flow::{EventContent, ExecutionResult, ExecutionStatistics, FlowEvent, FlowEventType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestFlowLog {
    session_id: String,
    benchmark_id: String,
    agent_type: String,
    start_time: String,
    end_time: Option<String>,
    events: Vec<TestEvent>,
    result: Option<TestResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestEvent {
    timestamp: String,
    event_type: String,
    data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestResult {
    success: bool,
    score: f64,
}

impl FlowLogConverter<TestFlowLog> for TestFlowLog {
    fn to_flow_log(&self) -> Result<reev_flow::database::DBFlowLog, ConversionError> {
        let start_time = chrono::DateTime::parse_from_rfc3339(&self.start_time)
            .map_err(|e| ConversionError::TimestampError(e.to_string()))?
            .with_timezone(&chrono::Utc);

        let mut flow_log = FlowConverter::create_basic_flow_log(
            self.session_id.clone(),
            self.benchmark_id.clone(),
            self.agent_type.clone(),
            start_time,
        );

        // Convert test events to shared events
        let shared_events: Result<Vec<FlowEvent>, ConversionError> = self
            .events
            .iter()
            .map(|e| {
                let system_time = std::time::SystemTime::UNIX_EPOCH
                    + std::time::Duration::from_secs(e.timestamp.parse::<u64>().unwrap_or(0));
                Ok(FlowEvent {
                    timestamp: system_time,
                    event_type: FlowEventType::LlmRequest, // Simplified for test
                    depth: 0,
                    content: EventContent {
                        data: e.data.clone(),
                    },
                })
            })
            .collect();

        // Update the flow log with events and completion data
        flow_log.flow.events = shared_events?;

        if let Some(end_time) = &self.end_time {
            let system_time = std::time::SystemTime::UNIX_EPOCH
                + std::time::Duration::from_secs(end_time.parse::<u64>().unwrap_or(0));
            flow_log.flow.end_time = Some(system_time);
        }

        if let Some(result) = &self.result {
            let shared_result = ExecutionResult {
                success: result.success,
                score: result.score,
                total_time_ms: 1000,
                statistics: ExecutionStatistics {
                    total_llm_calls: 0,
                    total_tool_calls: 0,
                    total_tokens: 0,
                    tool_usage: HashMap::new(),
                    max_depth: 0,
                },
                scoring_breakdown: None,
            };
            flow_log.flow.final_result = Some(shared_result);
        }

        Ok(flow_log)
    }

    fn from_flow_log(
        flow_log: &reev_flow::database::DBFlowLog,
    ) -> Result<TestFlowLog, ConversionError> {
        let events = flow_log.flow.events.clone();
        let test_events: Result<Vec<_>, ConversionError> = events
            .iter()
            .map(|e| {
                Ok(TestEvent {
                    timestamp: reev_flow::FlowUtils::system_time_to_rfc3339(e.timestamp)
                        .unwrap_or_else(|_| chrono::Utc::now().to_rfc3339()),
                    event_type: "llm_request".to_string(), // Simplified for test
                    data: e.content.data.clone(),
                })
            })
            .collect();

        let final_result = flow_log.flow.final_result.clone();

        let test_result = final_result.map(|r| TestResult {
            success: r.success,
            score: r.score,
        });

        Ok(TestFlowLog {
            session_id: flow_log.session_id().to_string(),
            benchmark_id: flow_log.benchmark_id().to_string(),
            agent_type: flow_log.agent_type().to_string(),
            start_time: flow_log.start_time().unwrap_or_default(),
            end_time: flow_log.end_time().unwrap_or(None),
            events: test_events?,
            result: test_result,
        })
    }
}

#[test]
fn test_flow_log_conversion() {
    let test_flow = TestFlowLog {
        session_id: "test-session".to_string(),
        benchmark_id: "test-benchmark".to_string(),
        agent_type: "test-agent".to_string(),
        start_time: "2023-01-01T00:00:00Z".to_string(),
        end_time: Some("2023-01-01T00:01:00Z".to_string()),
        events: vec![TestEvent {
            timestamp: "2023-01-01T00:00:30Z".to_string(),
            event_type: "test".to_string(),
            data: serde_json::json!({"test": "data"}),
        }],
        result: Some(TestResult {
            success: true,
            score: 0.95,
        }),
    };

    // Convert to shared FlowLog
    let shared_flow = test_flow.to_flow_log().unwrap();
    assert_eq!(shared_flow.session_id(), test_flow.session_id);
    assert_eq!(shared_flow.benchmark_id(), test_flow.benchmark_id);
    assert_eq!(shared_flow.agent_type(), test_flow.agent_type);

    // Convert back to domain type
    let converted_back = TestFlowLog::from_flow_log(&shared_flow).unwrap();
    assert_eq!(converted_back.session_id, test_flow.session_id);
    assert_eq!(converted_back.benchmark_id, test_flow.benchmark_id);
    assert_eq!(converted_back.agent_type, test_flow.agent_type);
    assert_eq!(converted_back.events.len(), test_flow.events.len());
    assert!(converted_back.result.is_some());
}
