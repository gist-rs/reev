//! Flow conversion traits and implementations
//!
//! This module provides conversion traits and implementations for converting
//! between domain-specific FlowLog types and the shared FlowLog type.
//!
//! ## Usage
//!
//! ```rust
//! use reev_db::shared::flow::converter::FlowLogConverter;
//! use reev_db::shared::flow::ConversionError;
//! use reev_db::shared::flow::{FlowLog, FlowLogUtils};
//!
//! // For domain-specific types, implement the FlowLogConverter trait
//! struct MyDomainFlowLog { /* ... */ }
//!
//! impl FlowLogConverter<MyDomainFlowLog> for MyDomainFlowLog {
//!     fn to_flow_log(&self) -> Result<FlowLog, ConversionError> {
//!         // Convert your domain type to shared FlowLog
//!         todo!("Implement conversion")
//!     }
//!
//!     fn from_flow_log(flow_log: &FlowLog) -> Result<MyDomainFlowLog, ConversionError> {
//!         // Convert shared FlowLog to your domain type
//!         todo!("Implement conversion")
//!     }
//! }
//! ```

use super::types::*;
use crate::shared::flow::ConversionError;
use serde_json;

/// Conversion trait for FlowLog types
///
/// This trait should be implemented by domain-specific FlowLog types
/// to enable conversion to/from the shared FlowLog type.
pub trait FlowLogConverter<T> {
    /// Convert from domain type to shared FlowLog
    fn to_flow_log(&self) -> Result<FlowLog, ConversionError>;

    /// Convert from shared FlowLog to domain type
    fn from_flow_log(flow_log: &FlowLog) -> Result<T, ConversionError>;
}

/// Generic conversion utilities for common patterns
pub struct FlowConverter;

impl FlowConverter {
    /// Convert any serializable events to JSON string
    pub fn events_to_json<T: serde::Serialize>(events: &[T]) -> Result<String, ConversionError> {
        serde_json::to_string(events).map_err(Into::into)
    }

    /// Convert JSON string to any deserializable events
    pub fn json_to_events<T: serde::de::DeserializeOwned>(
        json: &str,
    ) -> Result<Vec<T>, ConversionError> {
        serde_json::from_str(json).map_err(Into::into)
    }

    /// Convert any serializable result to JSON string
    pub fn result_to_json<T: serde::Serialize>(result: &T) -> Result<String, ConversionError> {
        serde_json::to_string(result).map_err(Into::into)
    }

    /// Convert JSON string to any deserializable result
    pub fn json_to_result<T: serde::de::DeserializeOwned>(
        json: &str,
    ) -> Result<T, ConversionError> {
        serde_json::from_str(json).map_err(Into::into)
    }

    /// Create a FlowLog from basic components
    pub fn create_flow_log(
        session_id: String,
        benchmark_id: String,
        agent_type: String,
        start_time: &str,
    ) -> FlowLog {
        FlowLog {
            session_id,
            benchmark_id,
            agent_type,
            start_time: start_time.to_string(),
            end_time: None,
            flow_data: serde_json::to_string(&Vec::<FlowEvent>::new()).unwrap(),
            final_result: None,
            id: None,
            created_at: Some(start_time.to_string()),
        }
    }

    /// Update FlowLog with completion data
    pub fn mark_completed(
        mut flow_log: FlowLog,
        end_time: Option<&str>,
        final_result: Option<String>,
    ) -> FlowLog {
        flow_log.end_time = end_time.map(|s| s.to_string());
        flow_log.final_result = final_result;
        flow_log
    }
}

/// Macro to help implement FlowLogConverter for structs with similar field patterns
#[macro_export]
macro_rules! impl_flow_log_converter {
    (
        $domain_type:ty,
        $session_id:expr,
        $benchmark_id:expr,
        $agent_type:expr,
        $start_time:expr,
        $end_time:expr,
        $events:expr,
        $final_result:expr
    ) => {
        impl FlowLogConverter<$domain_type> for $domain_type {
            fn to_flow_log(&self) -> Result<FlowLog, ConversionError> {
                let start_time_str = FlowLogUtils::system_time_to_rfc3339($start_time(self))?;
                let end_time_str = $end_time(self)
                    .map(|et| FlowLogUtils::system_time_to_rfc3339(et))
                    .transpose()?;

                let events_json = FlowLogUtils::serialize_events(&$events(self))?;
                let final_result_json = $final_result(self)
                    .map(|fr| FlowLogUtils::serialize_result(&fr))
                    .transpose()?;

                Ok(FlowLog {
                    session_id: $session_id(self).to_string(),
                    benchmark_id: $benchmark_id(self).to_string(),
                    agent_type: $agent_type(self).to_string(),
                    start_time: start_time_str,
                    end_time: end_time_str,
                    flow_data: events_json,
                    final_result: final_result_json,
                    id: None,
                    created_at: Some(start_time_str),
                })
            }

            fn from_flow_log(flow_log: &FlowLog) -> Result<$domain_type, ConversionError> {
                let start_time = FlowLogUtils::rfc3339_to_system_time(&flow_log.start_time)?;
                let end_time = flow_log
                    .end_time
                    .as_ref()
                    .map(|et| FlowLogUtils::rfc3339_to_system_time(et))
                    .transpose()?;

                let events = FlowLogUtils::deserialize_events(&flow_log.flow_data)?;
                let final_result = flow_log
                    .final_result
                    .as_ref()
                    .map(|fr| FlowLogUtils::deserialize_result(fr))
                    .transpose()?;

                // This part needs to be implemented manually for each type
                // as it depends on the constructor of the domain type
                todo!("Implement domain type construction from parsed data")
            }
        }
    };
}

// Example implementation for reev-lib FlowLog (when ready to migrate)
// TODO: Implement this module when reev-lib compatibility is needed
// This module will contain conversion functions between reev-lib FlowLog
// and the shared FlowLog type
/*
#[cfg(feature = "reev-lib-compat")]
pub mod reev_lib_compat {
    use super::*;

    /// Placeholder for reev-lib FlowLog conversion
    /// This would be implemented when reev-lib is ready to use shared types
    pub struct ReeveLibFlowLogConverter;

    impl ReeveLibFlowLogConverter {
        /// Convert reev-lib FlowLog to shared FlowLog
        pub fn from_reev_lib(
            _reev_lib_flow: &reev_lib::flow::types::FlowLog,
        ) -> Result<FlowLog, ConversionError> {
            todo!("Implement when reev-lib types are available")
        }

        /// Convert shared FlowLog to reev-lib FlowLog
        pub fn to_reev_lib(
            _shared_flow: &FlowLog,
        ) -> Result<reev_lib::flow::types::FlowLog, ConversionError> {
            todo!("Implement when reev-lib types are available")
        }
    }
}
*/

#[cfg(test)]
mod tests {
    use super::*;
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
        fn to_flow_log(&self) -> Result<FlowLog, ConversionError> {
            let mut flow_log = FlowConverter::create_flow_log(
                self.session_id.clone(),
                self.benchmark_id.clone(),
                self.agent_type.clone(),
                &self.start_time,
            );

            // Convert test events to shared events
            let shared_events: Result<Vec<FlowEvent>, ConversionError> = self
                .events
                .iter()
                .map(|e| {
                    Ok(FlowEvent {
                        timestamp: e.timestamp.clone(),
                        event_type: FlowEventType::LlmRequest, // Simplified for test
                        depth: 1,
                        content: EventContent {
                            data: e.data.clone(),
                            metadata: HashMap::new(),
                        },
                    })
                })
                .collect();

            flow_log.flow_data = FlowLogUtils::serialize_events(&shared_events?)?;

            if let Some(end_time) = &self.end_time {
                flow_log.end_time = Some(end_time.clone());
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
                flow_log.final_result = Some(FlowLogUtils::serialize_result(&shared_result)?);
            }

            Ok(flow_log)
        }

        fn from_flow_log(flow_log: &FlowLog) -> Result<TestFlowLog, ConversionError> {
            let events = FlowLogUtils::deserialize_events(&flow_log.flow_data)?;
            let test_events: Result<Vec<_>, ConversionError> = events
                .iter()
                .map(|e| {
                    Ok(TestEvent {
                        timestamp: e.timestamp.clone(),
                        event_type: "llm_request".to_string(), // Simplified for test
                        data: e.content.data.clone(),
                    })
                })
                .collect();

            let final_result = flow_log
                .final_result
                .as_ref()
                .map(|fr| FlowLogUtils::deserialize_result(fr))
                .transpose()?;

            let test_result = final_result.map(|r| TestResult {
                success: r.success,
                score: r.score,
            });

            Ok(TestFlowLog {
                session_id: flow_log.session_id.clone(),
                benchmark_id: flow_log.benchmark_id.clone(),
                agent_type: flow_log.agent_type.clone(),
                start_time: flow_log.start_time.clone(),
                end_time: flow_log.end_time.clone(),
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
        assert_eq!(shared_flow.session_id, test_flow.session_id);
        assert_eq!(shared_flow.benchmark_id, test_flow.benchmark_id);
        assert_eq!(shared_flow.agent_type, test_flow.agent_type);

        // Convert back to domain type
        let converted_back = TestFlowLog::from_flow_log(&shared_flow).unwrap();
        assert_eq!(converted_back.session_id, test_flow.session_id);
        assert_eq!(converted_back.benchmark_id, test_flow.benchmark_id);
        assert_eq!(converted_back.agent_type, test_flow.agent_type);
        assert_eq!(converted_back.events.len(), test_flow.events.len());
        assert!(converted_back.result.is_some());
    }
}
