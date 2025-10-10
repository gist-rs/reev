use super::types::*;
use ascii_tree::Tree;
use std::path::Path;

/// ASCII tree rendering for flow logs
impl FlowLog {
    /// Render the flow log as an ASCII tree
    pub fn render_as_ascii_tree(&self) -> String {
        let duration = if let Some(end) = self.end_time {
            match end.duration_since(self.start_time) {
                Ok(d) => {
                    let total_ms = d.as_millis();
                    if total_ms >= 1000 {
                        format!("{:.2}s", total_ms as f64 / 1000.0)
                    } else {
                        format!("{total_ms}ms")
                    }
                }
                Err(_) => "Unknown".to_string(),
            }
        } else {
            "In Progress".to_string()
        };

        let status = if let Some(result) = &self.final_result {
            if result.success {
                "✅ SUCCESS"
            } else {
                "❌ FAILED"
            }
        } else {
            "⏳ RUNNING"
        };

        let root_label = format!(
            "🌊 {} [{}] - {} (Duration: {})",
            self.benchmark_id, self.agent_type, status, duration
        );

        let mut children = Vec::new();

        // Add detailed score breakdown if available
        if let Some(result) = &self.final_result {
            let score_percent = result.score * 100.0;
            let score_grade = match score_percent {
                s if s >= 95.0 => "🏆 PERFECT",
                s if s >= 85.0 => "🥇 EXCELLENT",
                s if s >= 75.0 => "🥈 GOOD",
                s if s >= 60.0 => "🥉 FAIR",
                s if s >= 40.0 => "⚠️  POOR",
                _ => "❌ VERY POOR",
            };

            let score_summary = format!(
                "📊 Score: {:.1}% {} | LLM: {} | Tools: {} | Tokens: {}",
                score_percent,
                score_grade,
                result.statistics.total_llm_calls,
                result.statistics.total_tool_calls,
                result.statistics.total_tokens
            );
            children.push(Tree::Leaf(vec![score_summary]));

            // Add detailed scoring breakdown if available
            if let Some(scoring) = &result.scoring_breakdown {
                let instruction_percent = scoring.instruction_score * 100.0;
                let onchain_percent = scoring.onchain_score * 100.0;

                let breakdown = format!(
                    "🔍 Breakdown: Instructions {:.1}% (×75%) + On-chain {:.1}% (×25%) = {:.1}%",
                    instruction_percent,
                    onchain_percent,
                    scoring.final_score * 100.0
                );
                children.push(Tree::Leaf(vec![breakdown]));

                // Add specific issues if not perfect
                if scoring.final_score < 1.0 && !scoring.issues.is_empty() {
                    let issues_text = format!("⚠️  Issues: {}", scoring.issues.join(" | "));
                    children.push(Tree::Leaf(vec![issues_text]));
                }

                // Add specific mismatches if available
                if !scoring.mismatches.is_empty() {
                    let mismatches_text = format!("🔧 Details: {}", scoring.mismatches.join(" | "));
                    children.push(Tree::Leaf(vec![mismatches_text]));
                }
            }
        }

        // Add events as tree nodes
        for (i, event) in self.events.iter().enumerate() {
            // Calculate event duration
            let duration_str = if let Some(exec_time) = event
                .content
                .data
                .get("execution_time_ms")
                .and_then(|v| v.as_u64())
            {
                if exec_time >= 1000 {
                    format!("{:.2}s", exec_time as f64 / 1000.0)
                } else {
                    format!("{exec_time}ms")
                }
            } else {
                // For LLM requests and other events without explicit timing, use cumulative time
                if let Ok(duration_since_start) = event.timestamp.duration_since(self.start_time) {
                    let ms = duration_since_start.as_millis();
                    if ms >= 1000 {
                        format!("{:.2}s", ms as f64 / 1000.0)
                    } else {
                        format!("{ms}ms")
                    }
                } else {
                    "Unknown".to_string()
                }
            };

            children.push(self.render_event_as_tree_node(i + 1, event, &duration_str));
        }

        let tree = Tree::Node(root_label, children);
        let mut buffer = String::new();
        ascii_tree::write_tree(&mut buffer, &tree).unwrap();
        buffer
    }

    fn render_event_as_tree_node(
        &self,
        event_num: usize,
        event: &FlowEvent,
        event_duration: &str,
    ) -> Tree {
        let event_label = match &event.event_type {
            FlowEventType::LlmRequest => {
                let model = event
                    .content
                    .data
                    .get("model")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let tokens = event
                    .content
                    .data
                    .get("context_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                format!(
                    "🤖 Event {} ({}): LLM Request (Depth: {}) - {} ({} tokens)",
                    event_num, event_duration, event.depth, model, tokens
                )
            }
            FlowEventType::ToolCall => {
                let tool_name = event
                    .content
                    .data
                    .get("tool_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                format!(
                    "🔧 Event {} ({}): Tool Call (Depth: {}) - {}",
                    event_num, event_duration, event.depth, tool_name
                )
            }
            FlowEventType::ToolResult => {
                let tool_name = event
                    .content
                    .data
                    .get("tool_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let result_status = event
                    .content
                    .data
                    .get("result_status")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                format!(
                    "📋 Event {} ({}): Tool Result (Depth: {}) - {} - {}",
                    event_num, event_duration, event.depth, tool_name, result_status
                )
            }
            FlowEventType::TransactionExecution => {
                let signature = event
                    .content
                    .data
                    .get("signature")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let success = event
                    .content
                    .data
                    .get("success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let status = if success { "✅" } else { "❌" };
                format!(
                    "💰 Event {} ({}): Transaction {} - {}",
                    event_num,
                    event_duration,
                    status,
                    &signature[..8.min(signature.len())]
                )
            }
            FlowEventType::Error => {
                let error_type = event
                    .content
                    .data
                    .get("error_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                format!(
                    "🚨 Event {} ({}): Error (Depth: {}) - {}",
                    event_num, event_duration, event.depth, error_type
                )
            }
            FlowEventType::BenchmarkStateChange => {
                format!(
                    "🔄 Event {} ({}): State Change (Depth: {})",
                    event_num, event_duration, event.depth
                )
            }
        };

        let mut children = Vec::new();

        // Add event-specific details
        match &event.event_type {
            FlowEventType::LlmRequest => {
                if let Some(prompt) = event.content.data.get("prompt").and_then(|v| v.as_str()) {
                    let preview = if prompt.len() > 100 {
                        format!("{}...", &prompt[..100])
                    } else {
                        prompt.to_string()
                    };
                    children.push(Tree::Leaf(vec![format!("💬 Prompt: {}", preview)]));
                }
            }
            FlowEventType::ToolCall => {
                if let Some(args) = event.content.data.get("tool_args").and_then(|v| v.as_str()) {
                    let preview = if args.len() > 80 {
                        format!("{}...", &args[..80])
                    } else {
                        args.to_string()
                    };
                    children.push(Tree::Leaf(vec![format!("📝 Args: {}", preview)]));
                }
            }
            FlowEventType::ToolResult => {
                if let Some(error) = event
                    .content
                    .data
                    .get("error_message")
                    .and_then(|v| v.as_str())
                {
                    children.push(Tree::Leaf(vec![format!("❌ Error: {}", error)]));
                } else if let Some(result) = event.content.data.get("result_data") {
                    let result_str = serde_json::to_string_pretty(result).unwrap_or_default();
                    let preview = if result_str.len() > 100 {
                        format!("{}...", &result_str[..100])
                    } else {
                        result_str
                    };
                    children.push(Tree::Leaf(vec![format!("✅ Result: {}", preview)]));
                }
            }
            FlowEventType::Error => {
                if let Some(message) = event.content.data.get("message").and_then(|v| v.as_str()) {
                    children.push(Tree::Leaf(vec![format!("💥 Message: {}", message)]));
                }
            }
            _ => {}
        }

        Tree::Node(event_label, children)
    }
}

/// Load and render a flow log from file as ASCII tree
pub fn render_flow_file_as_ascii_tree(file_path: &Path) -> super::error::FlowResult<String> {
    let content = std::fs::read_to_string(file_path)?;
    let flow: FlowLog = serde_yaml::from_str(&content)?;
    Ok(flow.render_as_ascii_tree())
}
