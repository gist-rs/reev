//! Flow conversion traits and implementations
//!
//! This module provides conversion traits and implementations for converting
//! between domain-specific FlowLog types and the shared FlowLog type.
//!
//! ## Usage
//!
/// ```rust
/// use reev_db::shared::flow::converter::FlowLogConverter;
/// use reev_db::shared::flow::ConversionError;
/// use reev_db::shared::flow::FlowLogUtils;
/// use reev_flow::database::DBFlowLog;
///
/// // For domain-specific types, implement the FlowLogConverter trait
/// struct MyDomainFlowLog { /* ... */ }
///
/// impl FlowLogConverter<MyDomainFlowLog> for MyDomainFlowLog {
///     fn to_flow_log(&self) -> Result<DBFlowLog, ConversionError> {
///         // Convert your domain type to shared FlowLog
///         todo!("Implement conversion")
///     }
///
///     fn from_flow_log(flow_log: &DBFlowLog) -> Result<MyDomainFlowLog, ConversionError> {
///         // Convert shared FlowLog to your domain type
///         todo!("Implement conversion")
///     }
/// }
/// ```
use crate::shared::flow::ConversionError;
use reev_flow::database::DBFlowLog;
use serde_json;

/// Conversion trait for FlowLog types
///
/// This trait should be implemented by domain-specific FlowLog types
/// to enable conversion to/from the shared FlowLog type.
pub trait FlowLogConverter<T> {
    /// Convert from domain type to shared FlowLog
    fn to_flow_log(&self) -> Result<DBFlowLog, ConversionError>;

    /// Convert from shared FlowLog to domain type
    fn from_flow_log(flow_log: &DBFlowLog) -> Result<T, ConversionError>;
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

    /// Create a basic FlowLog for testing/simple cases
    pub fn create_basic_flow_log(
        session_id: String,
        benchmark_id: String,
        agent_type: String,
        start_time: chrono::DateTime<chrono::Utc>,
    ) -> DBFlowLog {
        let flow = reev_flow::FlowUtils::create_flow_log(
            session_id.clone(),
            benchmark_id.clone(),
            agent_type.clone(),
        );

        let mut db_flow = DBFlowLog::new(flow);
        db_flow.created_at = Some(start_time.to_rfc3339());
        db_flow
    }

    /// Add end time to a FlowLog
    pub fn complete_flow_log(
        mut flow_log: DBFlowLog,
        end_time: Option<String>,
        final_result: Option<String>,
    ) -> Result<DBFlowLog, ConversionError> {
        if let Some(end_time) = end_time {
            let system_time = reev_flow::FlowUtils::rfc3339_to_system_time(&end_time)
                .map_err(|e| ConversionError::TimestampError(e.to_string()))?;
            flow_log.flow.end_time = Some(system_time);
        }

        if let Some(final_result) = final_result {
            let result = serde_json::from_str(&final_result)
                .map_err(|e| ConversionError::JsonError(e.to_string()))?;
            flow_log.flow.final_result = Some(result);
        }

        Ok(flow_log)
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
