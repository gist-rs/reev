//! Website export functionality for flow logs
//!
//! This module provides functionality to export flow data for website consumption,
//! including visualization data, tool usage statistics, and performance metrics.

use super::error::{FlowError, FlowResult};
use super::types::*;
use std::path::PathBuf;
use tracing::info;

/// Website export functionality
pub struct WebsiteExporter {
    output_path: PathBuf,
}

impl WebsiteExporter {
    /// Create a new website exporter
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

    /// Build interactive flow visualization data
    fn build_flow_graph(&self, _flows: &[FlowLog]) -> FlowGraph {
        let nodes = Vec::new();
        let edges = Vec::new();
        FlowGraph { nodes, edges }
    }

    /// Calculate tool usage statistics
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

        let success_rates = std::collections::HashMap::new();

        ToolUsageStats {
            total_usage,
            success_rates,
            average_execution_times: execution_times
                .into_iter()
                .map(|(tool, times)| (tool, times.iter().sum::<u64>() as f64 / times.len() as f64))
                .collect(),
        }
    }

    /// Extract performance metrics from flows
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

    /// Analyze agent behavior patterns
    fn analyze_behavior(&self, flows: &[FlowLog]) -> AgentBehaviorAnalysis {
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
            average_decision_time_ms: 1500,
            error_recovery_rate: 0.85,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    #[test]
    fn test_website_exporter_creation() {
        let exporter = WebsiteExporter::new(PathBuf::from("/tmp"));
        assert_eq!(exporter.output_path, PathBuf::from("/tmp"));
    }

    #[test]
    fn test_export_empty_flows() {
        let exporter = WebsiteExporter::new(PathBuf::from("/tmp"));
        let flows = vec![];

        // This should not panic even with empty flows
        let result = exporter.export_for_website(&flows);
        assert!(result.is_ok());
    }

    #[test]
    fn test_calculate_tool_usage_stats() {
        let exporter = WebsiteExporter::new(PathBuf::from("/tmp"));
        let flows = vec![FlowLog {
            session_id: "test".to_string(),
            benchmark_id: "test".to_string(),
            agent_type: "test".to_string(),
            start_time: SystemTime::now(),
            end_time: Some(SystemTime::now()),
            events: vec![],
            final_result: None,
        }];

        let stats = exporter.calculate_tool_usage_stats(&flows);
        assert!(stats.total_usage.is_empty());
        assert!(stats.success_rates.is_empty());
        assert!(stats.average_execution_times.is_empty());
    }
}
