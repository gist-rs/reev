use super::error::{FlowError, FlowResult};
use super::types::*;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info};

/// Main flow logger interface
pub struct FlowLogger {
    session_id: String,
    benchmark_id: String,
    agent_type: String,
    start_time: SystemTime,
    events: Vec<FlowEvent>,
    output_path: PathBuf,
}

impl FlowLogger {
    /// Create a new flow logger instance
    pub fn new(benchmark_id: String, agent_type: String, output_path: PathBuf) -> Self {
        let session_id = uuid::Uuid::new_v4().to_string();
        let start_time = SystemTime::now();

        info!(
            session_id = %session_id,
            benchmark_id = %benchmark_id,
            agent_type = %agent_type,
            "Initializing flow logger"
        );

        Self {
            session_id,
            benchmark_id,
            agent_type,
            start_time,
            events: Vec::new(),
            output_path,
        }
    }

    /// Log an LLM request event
    pub fn log_llm_request(&mut self, content: LlmRequestContent, depth: u32) {
        let event = FlowEvent {
            timestamp: SystemTime::now(),
            event_type: FlowEventType::LlmRequest,
            depth,
            content: EventContent {
                data: serde_json::to_value(content).unwrap_or_default(),
                metadata: std::collections::HashMap::new(),
            },
        };
        self.events.push(event);
        debug!("Logged LLM request");
    }

    /// Log a tool call event
    pub fn log_tool_call(&mut self, content: ToolCallContent, depth: u32) {
        let event = FlowEvent {
            timestamp: SystemTime::now(),
            event_type: FlowEventType::ToolCall,
            depth,
            content: EventContent {
                data: serde_json::to_value(content).unwrap_or_default(),
                metadata: std::collections::HashMap::new(),
            },
        };
        self.events.push(event);
        debug!("Logged tool call");
    }

    /// Log a tool result event
    pub fn log_tool_result(&mut self, content: ToolCallContent, depth: u32) {
        let event = FlowEvent {
            timestamp: SystemTime::now(),
            event_type: FlowEventType::ToolResult,
            depth,
            content: EventContent {
                data: serde_json::to_value(content).unwrap_or_default(),
                metadata: std::collections::HashMap::new(),
            },
        };
        self.events.push(event);
        debug!("Logged tool result");
    }

    /// Log a transaction execution event
    pub fn log_transaction(&mut self, content: TransactionExecutionContent, depth: u32) {
        let event = FlowEvent {
            timestamp: SystemTime::now(),
            event_type: FlowEventType::TransactionExecution,
            depth,
            content: EventContent {
                data: serde_json::to_value(content).unwrap_or_default(),
                metadata: std::collections::HashMap::new(),
            },
        };
        self.events.push(event);
        debug!("Logged transaction execution");
    }

    /// Log an error event
    pub fn log_error(&mut self, content: ErrorContent, depth: u32) {
        let event = FlowEvent {
            timestamp: SystemTime::now(),
            event_type: FlowEventType::Error,
            depth,
            content: EventContent {
                data: serde_json::to_value(&content).unwrap_or_default(),
                metadata: std::collections::HashMap::new(),
            },
        };
        self.events.push(event);
        debug!("Logged error: {}", content.message);
    }

    /// Complete the flow log with final results
    pub fn complete(&mut self, result: ExecutionResult) -> FlowResult<PathBuf> {
        let end_time = SystemTime::now();

        let flow_log = FlowLog {
            session_id: self.session_id.clone(),
            benchmark_id: self.benchmark_id.clone(),
            agent_type: self.agent_type.clone(),
            start_time: self.start_time,
            end_time: Some(end_time),
            events: self.events.clone(),
            final_result: Some(result),
        };

        // Generate filename with timestamp
        let timestamp = end_time
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let filename = format!(
            "flow_{}_{}_{}.yml",
            self.benchmark_id, self.agent_type, timestamp
        );
        let file_path = self.output_path.join(filename);

        // Write YML file
        let yml_content = serde_yaml::to_string(&flow_log)
            .map_err(|e| FlowError::serialization(e.to_string()))?;

        std::fs::write(&file_path, yml_content).map_err(|e| FlowError::file(e.to_string()))?;

        info!(
            session_id = %self.session_id,
            file_path = %file_path.display(),
            "Flow log completed and saved"
        );

        Ok(file_path)
    }

    /// Get current statistics
    pub fn get_current_statistics(&self) -> ExecutionStatistics {
        let mut stats = ExecutionStatistics {
            total_llm_calls: 0,
            total_tool_calls: 0,
            total_tokens: 0,
            tool_usage: std::collections::HashMap::new(),
            max_depth: 0,
        };

        for event in &self.events {
            stats.max_depth = stats.max_depth.max(event.depth);

            match event.event_type {
                FlowEventType::LlmRequest => {
                    stats.total_llm_calls += 1;
                    if let Some(tokens) = event
                        .content
                        .data
                        .get("context_tokens")
                        .and_then(|v| v.as_u64())
                    {
                        stats.total_tokens += tokens;
                    }
                }
                FlowEventType::ToolCall => {
                    stats.total_tool_calls += 1;
                    if let Some(tool_name) =
                        event.content.data.get("tool_name").and_then(|v| v.as_str())
                    {
                        *stats.tool_usage.entry(tool_name.to_string()).or_insert(0) += 1;
                    }
                }
                _ => {}
            }
        }

        stats
    }
}

/// Website export functionality
pub struct WebsiteExporter {
    output_path: PathBuf,
}

impl WebsiteExporter {
    pub fn new(output_path: PathBuf) -> Self {
        Self { output_path }
    }

    /// Export flow data for website consumption
    pub fn export_for_website(&self, flows: &[FlowLog]) -> FlowResult<WebsiteData> {
        let website_data = WebsiteData {
            flows: flows.to_vec(),
            flow_visualization: self.build_flow_graph(flows),
            tool_usage_stats: self.calculate_tool_usage_stats(flows),
            performance_metrics: self.extract_metrics(flows),
            agent_behavior_analysis: self.analyze_behavior(flows),
        };

        // Write website data
        let json_content = serde_json::to_string_pretty(&website_data)
            .map_err(|e| FlowError::serialization(e.to_string()))?;

        let file_path = self.output_path.join("website_data.json");
        std::fs::write(&file_path, json_content).map_err(|e| FlowError::file(e.to_string()))?;

        info!("Website data exported to {}", file_path.display());
        Ok(website_data)
    }

    fn build_flow_graph(&self, _flows: &[FlowLog]) -> FlowGraph {
        // Build interactive flow visualization data
        let nodes = Vec::new();
        let edges = Vec::new();

        FlowGraph { nodes, edges }
    }

    fn calculate_tool_usage_stats(&self, flows: &[FlowLog]) -> ToolUsageStats {
        let mut total_usage = std::collections::HashMap::new();
        let mut execution_times = std::collections::HashMap::new();

        for flow in flows {
            for event in &flow.events {
                if let FlowEventType::ToolCall = event.event_type {
                    if let Some(tool_name) =
                        event.content.data.get("tool_name").and_then(|v| v.as_str())
                    {
                        *total_usage.entry(tool_name.to_string()).or_insert(0) += 1;

                        if let Some(exec_time) = event
                            .content
                            .data
                            .get("execution_time_ms")
                            .and_then(|v| v.as_u64())
                        {
                            execution_times
                                .entry(tool_name.to_string())
                                .or_insert_with(Vec::new)
                                .push(exec_time);
                        }
                    }
                }
            }
        }

        let success_rates = std::collections::HashMap::new(); // Placeholder - would need proper tool result tracking

        ToolUsageStats {
            total_usage,
            success_rates,
            average_execution_times: execution_times
                .into_iter()
                .map(|(tool, times)| (tool, times.iter().sum::<u64>() as f64 / times.len() as f64))
                .collect(),
        }
    }

    fn extract_metrics(&self, flows: &[FlowLog]) -> PerformanceMetrics {
        let mut total_execution_time = 0u64;
        let mut total_llm_calls = 0u32;
        let mut total_tool_calls = 0u32;
        let mut total_tokens = 0u64;

        for flow in flows {
            if let Some(end) = flow.end_time {
                let start = flow.start_time;
                if let Ok(duration) = end.duration_since(start) {
                    total_execution_time += duration.as_millis() as u64;
                }
            }

            if let Some(result) = &flow.final_result {
                total_llm_calls += result.statistics.total_llm_calls;
                total_tool_calls += result.statistics.total_tool_calls;
                total_tokens += result.statistics.total_tokens;
            }
        }

        PerformanceMetrics {
            total_execution_time_ms: total_execution_time,
            average_execution_time_ms: if flows.is_empty() {
                0
            } else {
                total_execution_time / flows.len() as u64
            },
            total_llm_calls,
            total_tool_calls,
            total_tokens,
            success_rate: flows
                .iter()
                .filter(|f| f.final_result.as_ref().map(|r| r.success).unwrap_or(false))
                .count() as f64
                / flows.len() as f64,
        }
    }

    fn analyze_behavior(&self, flows: &[FlowLog]) -> AgentBehaviorAnalysis {
        // Analyze agent behavior patterns
        let mut depth_patterns = std::collections::HashMap::new();
        let mut tool_sequences = Vec::new();

        for flow in flows {
            // Analyze depth patterns
            let max_depth = flow.events.iter().map(|e| e.depth).max().unwrap_or(0);
            *depth_patterns.entry(max_depth).or_insert(0) += 1;

            // Analyze tool sequences
            let mut current_sequence = Vec::new();
            for event in &flow.events {
                if let FlowEventType::ToolCall = event.event_type {
                    if let Some(tool_name) =
                        event.content.data.get("tool_name").and_then(|v| v.as_str())
                    {
                        current_sequence.push(tool_name.to_string());
                    }
                }
            }
            if !current_sequence.is_empty() {
                tool_sequences.push(current_sequence);
            }
        }

        AgentBehaviorAnalysis {
            depth_patterns,
            common_tool_sequences: tool_sequences,
            average_decision_time_ms: 1500, // Placeholder - would calculate from actual data
            error_recovery_rate: 0.85,      // Placeholder
        }
    }
}
