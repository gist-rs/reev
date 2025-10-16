//! Test for flow log insertion to identify database issues

#[cfg(test)]
mod flow_log_tests {
    use reev_db::{DatabaseConfig, DatabaseWriter};
    use reev_flow::database::DBFlowLog;
    use reev_flow::types::{
        EventContent, ExecutionResult, ExecutionStatistics, FlowEvent, FlowEventType, FlowLog,
    };
    use std::collections::HashMap;
    use std::error::Error;
    use std::time::SystemTime;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_flow_log_insertion() -> Result<(), Box<dyn std::error::Error>> {
        // Setup test database
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test_flow.db");
        let config = DatabaseConfig::new(db_path.to_string_lossy());

        println!("🧪 Creating database writer...");
        let writer = DatabaseWriter::new(config).await?;

        // Create a test flow log
        println!("🧪 Creating test flow log...");
        let flow_log = FlowLog {
            session_id: "test-session-123".to_string(),
            benchmark_id: "test-benchmark".to_string(),
            agent_type: "test-agent".to_string(),
            start_time: SystemTime::now(),
            end_time: Some(SystemTime::now()),
            events: vec![FlowEvent {
                timestamp: SystemTime::now(),
                event_type: FlowEventType::LlmRequest,
                depth: 0,
                content: EventContent {
                    data: serde_json::json!({"test": "data"}),
                    metadata: HashMap::new(),
                },
            }],
            final_result: Some(ExecutionResult {
                success: true,
                score: 1.0,
                total_time_ms: 1000,
                statistics: ExecutionStatistics {
                    total_llm_calls: 1,
                    total_tool_calls: 0,
                    total_tokens: 100,
                    tool_usage: HashMap::new(),
                    max_depth: 1,
                },
                scoring_breakdown: None,
            }),
        };

        // Wrap in DBFlowLog
        let db_flow_log = DBFlowLog::new(flow_log);

        println!("🧪 Testing DBFlowLog methods...");

        // Test each method individually
        match db_flow_log.start_time() {
            Ok(start_time) => println!("✅ start_time: {start_time}"),
            Err(e) => println!("❌ start_time failed: {e}"),
        }

        match db_flow_log.end_time() {
            Ok(end_time) => println!("✅ end_time: {end_time:?}"),
            Err(e) => println!("❌ end_time failed: {e}"),
        }

        match db_flow_log.events_json() {
            Ok(events) => println!("✅ events_json: {events}"),
            Err(e) => println!("❌ events_json failed: {e}"),
        }

        match db_flow_log.final_result_json() {
            Ok(result) => println!("✅ final_result_json: {result:?}"),
            Err(e) => println!("❌ final_result_json failed: {e}"),
        }

        // Test the actual insertion
        println!("🧪 Attempting database insertion...");
        match writer.insert_flow_log(&db_flow_log).await {
            Ok(id) => {
                println!("✅ Flow log inserted successfully with ID: {id}");
            }
            Err(e) => {
                println!("❌ Flow log insertion failed: {e}");
                println!(
                    "🔍 Root cause: {}",
                    e.source()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "Unknown".to_string())
                );
                return Err(e.into());
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_flow_log_minimal() -> Result<(), Box<dyn std::error::Error>> {
        // Test with minimal data to isolate the issue
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test_minimal.db");
        let config = DatabaseConfig::new(db_path.to_string_lossy());

        let writer = DatabaseWriter::new(config).await?;

        // Create minimal flow log
        let flow_log = FlowLog {
            session_id: "minimal-session".to_string(),
            benchmark_id: "minimal-benchmark".to_string(),
            agent_type: "minimal-agent".to_string(),
            start_time: SystemTime::now(),
            end_time: None,     // No end time
            events: vec![],     // No events
            final_result: None, // No final result
        };

        let db_flow_log = DBFlowLog::new(flow_log);

        println!("🧪 Testing minimal flow log insertion...");
        match writer.insert_flow_log(&db_flow_log).await {
            Ok(id) => {
                println!("✅ Minimal flow log inserted with ID: {id}");
            }
            Err(e) => {
                println!("❌ Minimal flow log insertion failed: {e}");
                return Err(e.into());
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_flow_database_writer_error_conversion() -> Result<(), Box<dyn std::error::Error>>
    {
        // Test to identify the root cause of the database insertion error
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test_error_conversion.db");
        let config = DatabaseConfig::new(db_path.to_string_lossy());

        println!("🧪 Creating database writer to test error conversion...");
        let writer = DatabaseWriter::new(config).await?;

        // Create a simple flow log that should work
        let flow_log = FlowLog {
            session_id: "simple-test".to_string(),
            benchmark_id: "simple".to_string(),
            agent_type: "test".to_string(),
            start_time: SystemTime::now(),
            end_time: None,
            events: vec![],
            final_result: None,
        };

        let db_flow_log = DBFlowLog::new(flow_log);

        println!("🧪 Testing simple flow log insertion...");
        match writer.insert_flow_log(&db_flow_log).await {
            Ok(id) => {
                println!("✅ Simple flow log inserted with ID: {id}");
                println!("🔍 This means the database insertion itself works!");
                println!("🔍 The issue might be in the data from the actual runner");
            }
            Err(e) => {
                println!("❌ Even simple flow log insertion failed!");
                println!(
                    "🔍 DatabaseError type: {:?}",
                    std::any::type_name::<reev_db::error::DatabaseError>()
                );
                println!("🔍 Error details: {e:?}");
                println!("🔍 Error display: {e}");
                println!("🔍 Error source: {:?}", e.source());

                // Check if it's a connection or schema issue
                if e.to_string().contains("no such table") {
                    println!("🔍 LIKELY ISSUE: Database schema not initialized properly");
                } else if e.to_string().contains("UNIQUE constraint") {
                    println!("🔍 LIKELY ISSUE: Constraint violation");
                } else if e.to_string().contains("datatype mismatch") {
                    println!("🔍 LIKELY ISSUE: Data type mismatch");
                }

                return Err(e.into());
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_flow_log_with_problematic_data() -> Result<(), Box<dyn std::error::Error>> {
        // Test with potentially problematic data that might cause database issues
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test_problematic.db");
        let config = DatabaseConfig::new(db_path.to_string_lossy());

        let writer = DatabaseWriter::new(config).await?;

        // Create flow log with potentially problematic data
        let flow_log = FlowLog {
            session_id:
                "test-session-with-very-long-id-that-might-cause-issues-12345678901234567890"
                    .to_string(),
            benchmark_id: "test-benchmark-with-special-chars-!@#$%^&*()".to_string(),
            agent_type: "test-agent-with-unicode-ñáéíóú".to_string(),
            start_time: SystemTime::UNIX_EPOCH, // Edge case timestamp
            end_time: Some(SystemTime::UNIX_EPOCH),
            events: vec![FlowEvent {
                timestamp: SystemTime::UNIX_EPOCH,
                event_type: FlowEventType::LlmRequest,
                depth: 0,
                content: EventContent {
                    data: serde_json::json!({
                        "large_string": "x".repeat(10000),
                        "null_value": null,
                        "nested": {"deep": {"deeper": {"deepest": "value"}}}
                    }),
                    metadata: {
                        let mut metadata = HashMap::new();
                        metadata.insert("key".to_string(), "value with special chars".to_string());
                        metadata.insert("unicode".to_string(), "test".to_string());
                        metadata
                    },
                },
            }],
            final_result: Some(ExecutionResult {
                success: true,
                score: f64::INFINITY,    // Edge case value
                total_time_ms: u64::MAX, // Edge case value
                statistics: ExecutionStatistics {
                    total_llm_calls: u32::MAX,
                    total_tool_calls: u32::MAX,
                    total_tokens: u64::MAX,
                    tool_usage: HashMap::new(),
                    max_depth: u32::MAX,
                },
                scoring_breakdown: None,
            }),
        };

        let db_flow_log = DBFlowLog::new(flow_log);

        println!("🧪 Testing problematic data insertion...");
        match writer.insert_flow_log(&db_flow_log).await {
            Ok(id) => {
                println!("✅ Problematic data flow log inserted with ID: {id}");
            }
            Err(e) => {
                println!("❌ Problematic data insertion failed: {e}");
                println!("🔍 This might be the source of the issue in production");
                return Err(e.into());
            }
        }

        Ok(())
    }
}
