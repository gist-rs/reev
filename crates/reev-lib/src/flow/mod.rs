use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info};

pub mod otel;

/// Main flow log structure for complete benchmark execution tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowLog {
    /// Unique session identifier
    pub session_id: String,
    /// Benchmark identifier
    pub benchmark_id: String,
    /// Agent type (deterministic, local, gemini, etc.)
    pub agent_type: String,
    /// Start timestamp
    pub start_time: SystemTime,
    /// End timestamp
    pub end_time: Option<SystemTime>,
    /// All events in chronological order
    pub events: Vec<FlowEvent>,
    /// Final execution result
    pub final_result: Option<ExecutionResult>,
}

/// Individual event within a flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowEvent {
    /// Event timestamp
    pub timestamp: SystemTime,
    /// Type of event
    pub event_type: FlowEventType,
    /// Conversation depth when event occurred
    pub depth: u32,
    /// Event-specific content
    pub content: EventContent,
}

/// Types of events that can occur during flow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum FlowEventType {
    /// LLM request/response cycle
    LlmRequest,
    /// Tool invocation
    ToolCall,
    /// Tool result/response
    ToolResult,
    /// Transaction execution
    TransactionExecution,
    /// Error occurrence
    Error,
    /// Benchmark state change
    BenchmarkStateChange,
}

/// Event-specific content based on event type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventContent {
    /// Event-specific data
    pub data: serde_json::Value,
    /// Optional metadata
    pub metadata: HashMap<String, String>,
}

/// LLM request content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequestContent {
    /// The prompt sent to LLM
    pub prompt: String,
    /// Number of context tokens
    pub context_tokens: u32,
    /// Model name
    pub model: String,
    /// Request ID
    pub request_id: String,
}

/// LLM response content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponseContent {
    /// Response text
    pub response: String,
    /// Number of response tokens
    pub response_tokens: u32,
    /// Total tokens used
    pub total_tokens: u32,
    /// Time to first token
    pub time_to_first_token_ms: u32,
    /// Total response time
    pub total_time_ms: u32,
}

/// Tool call content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallContent {
    /// Name of tool called
    pub tool_name: String,
    /// Arguments passed to tool
    pub tool_args: String,
    /// Execution time in milliseconds
    pub execution_time_ms: u32,
    /// Tool result status
    pub result_status: ToolResultStatus,
    /// Result data if successful
    pub result_data: Option<serde_json::Value>,
    /// Error message if failed
    pub error_message: Option<String>,
}

/// Tool execution result status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolResultStatus {
    Success,
    Error,
    Timeout,
}

/// Transaction execution content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionExecutionContent {
    /// Transaction signature
    pub signature: String,
    /// Number of instructions
    pub instruction_count: u32,
    /// Execution time
    pub execution_time_ms: u32,
    /// Success status
    pub success: bool,
    /// Error if any
    pub error: Option<String>,
}

/// Error event content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContent {
    /// Error type
    pub error_type: String,
    /// Error message
    pub message: String,
    /// Stack trace if available
    pub stack_trace: Option<String>,
    /// Context when error occurred
    pub context: HashMap<String, String>,
}

/// Final execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Overall success status
    pub success: bool,
    /// Final score
    pub score: f64,
    /// Total execution time
    pub total_time_ms: u64,
    /// Summary statistics
    pub statistics: ExecutionStatistics,
    /// Detailed scoring breakdown
    pub scoring_breakdown: Option<ScoringBreakdown>,
}

/// Execution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStatistics {
    /// Total LLM calls
    pub total_llm_calls: u32,
    /// Total tool calls
    pub total_tool_calls: u32,
    /// Total tokens used
    pub total_tokens: u64,
    /// Tool usage breakdown
    pub tool_usage: HashMap<String, u32>,
    /// Conversation depth reached
    pub max_depth: u32,
}

/// Detailed scoring breakdown for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringBreakdown {
    /// Instruction matching score (0-1)
    pub instruction_score: f64,
    /// On-chain execution score (0-1)
    pub onchain_score: f64,
    /// Weighted final score (0-1)
    pub final_score: f64,
    /// Issues that affected the score
    pub issues: Vec<String>,
    /// Specific mismatches found
    pub mismatches: Vec<String>,
}

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
                metadata: HashMap::new(),
            },
        };
        self.events.push(event);
        debug!("Logged LLM request at depth {}", depth);
    }

    /// Log a tool call event
    pub fn log_tool_call(&mut self, content: ToolCallContent, depth: u32) {
        let event = FlowEvent {
            timestamp: SystemTime::now(),
            event_type: FlowEventType::ToolCall,
            depth,
            content: EventContent {
                data: serde_json::to_value(content).unwrap_or_default(),
                metadata: HashMap::new(),
            },
        };
        self.events.push(event);
        debug!(
            "Logged tool call '{}' at depth {}",
            self.events
                .last()
                .unwrap()
                .content
                .data
                .get("tool_name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown"),
            depth
        );
    }

    /// Log a tool result event
    pub fn log_tool_result(&mut self, content: ToolCallContent, depth: u32) {
        let event = FlowEvent {
            timestamp: SystemTime::now(),
            event_type: FlowEventType::ToolResult,
            depth,
            content: EventContent {
                data: serde_json::to_value(content).unwrap_or_default(),
                metadata: HashMap::new(),
            },
        };
        self.events.push(event);
        debug!("Logged tool result at depth {}", depth);
    }

    /// Log a transaction execution event
    pub fn log_transaction_execution(&mut self, content: TransactionExecutionContent) {
        let event = FlowEvent {
            timestamp: SystemTime::now(),
            event_type: FlowEventType::TransactionExecution,
            depth: 0, // Transactions are depth-independent
            content: EventContent {
                data: serde_json::to_value(content).unwrap_or_default(),
                metadata: HashMap::new(),
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
                metadata: HashMap::new(),
            },
        };
        self.events.push(event);
        debug!("Logged error: {}", content.message);
    }

    /// Complete the flow log with final results
    pub fn complete(&mut self, result: ExecutionResult) -> Result<(), FlowError> {
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
            .map_err(|e| FlowError::SerializationError(e.to_string()))?;

        std::fs::write(&file_path, yml_content).map_err(|e| FlowError::FileError(e.to_string()))?;

        info!(
            session_id = %self.session_id,
            file_path = %file_path.display(),
            "Flow log completed and saved"
        );

        Ok(())
    }

    /// Get current statistics
    pub fn get_current_statistics(&self) -> ExecutionStatistics {
        let mut stats = ExecutionStatistics {
            total_llm_calls: 0,
            total_tool_calls: 0,
            total_tokens: 0,
            tool_usage: HashMap::new(),
            max_depth: 0,
        };

        for event in &self.events {
            stats.max_depth = stats.max_depth.max(event.depth);

            match event.event_type {
                FlowEventType::LlmRequest => {
                    stats.total_llm_calls += 1;
                    // Extract token count from LLM request
                    if let Some(tokens) = event.content.data.get("context_tokens") {
                        if let Some(token_count) = tokens.as_u64() {
                            stats.total_tokens += token_count;
                        }
                    }
                }
                FlowEventType::ToolCall => {
                    stats.total_tool_calls += 1;
                    // Extract tool name for usage tracking
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

/// Flow logger errors
#[derive(Debug, thiserror::Error)]
pub enum FlowError {
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("File error: {0}")]
    FileError(String),
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
    pub fn export_for_website(&self, flows: &[FlowLog]) -> Result<WebsiteData, FlowError> {
        let website_data = WebsiteData {
            flows: flows.to_vec(),
            flow_visualization: self.build_flow_graph(flows),
            tool_usage_stats: self.calculate_tool_usage_stats(flows),
            performance_metrics: self.extract_metrics(flows),
            agent_behavior_analysis: self.analyze_behavior(flows),
        };

        // Write website data
        let json_content = serde_json::to_string_pretty(&website_data)
            .map_err(|e| FlowError::SerializationError(e.to_string()))?;

        let file_path = self.output_path.join("website_data.json");
        std::fs::write(&file_path, json_content)
            .map_err(|e| FlowError::FileError(e.to_string()))?;

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
        let mut total_usage = HashMap::new();
        let mut success_rates = HashMap::new();
        let mut execution_times = HashMap::new();

        for flow in flows {
            for event in &flow.events {
                if let FlowEventType::ToolCall = event.event_type {
                    if let Ok(tool_content) =
                        serde_json::from_value::<ToolCallContent>(event.content.data.clone())
                    {
                        *total_usage
                            .entry(tool_content.tool_name.clone())
                            .or_insert(0) += 1;

                        // Track execution times
                        let times = execution_times
                            .entry(tool_content.tool_name.clone())
                            .or_insert_with(Vec::new);
                        times.push(tool_content.execution_time_ms);

                        // Track success rates
                        let entry = success_rates
                            .entry(tool_content.tool_name.clone())
                            .or_insert((0, 0));
                        entry.0 += 1; // total calls
                        if matches!(tool_content.result_status, ToolResultStatus::Success) {
                            entry.1 += 1; // successful calls
                        }
                    }
                }
            }
        }

        ToolUsageStats {
            total_usage,
            success_rates: success_rates
                .into_iter()
                .map(|(tool, (total, success))| (tool, success as f64 / total as f64))
                .collect(),
            average_execution_times: execution_times
                .into_iter()
                .map(|(tool, times)| (tool, times.iter().sum::<u32>() as f64 / times.len() as f64))
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
        let mut depth_patterns = HashMap::new();
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

/// Website data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebsiteData {
    pub flows: Vec<FlowLog>,
    pub flow_visualization: FlowGraph,
    pub tool_usage_stats: ToolUsageStats,
    pub performance_metrics: PerformanceMetrics,
    pub agent_behavior_analysis: AgentBehaviorAnalysis,
}

/// Flow graph for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowGraph {
    pub nodes: Vec<FlowNode>,
    pub edges: Vec<FlowEdge>,
}

/// Flow node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowNode {
    pub id: String,
    pub label: String,
    pub node_type: String,
    pub position: (f64, f64),
}

/// Flow edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowEdge {
    pub from: String,
    pub to: String,
    pub label: String,
}

/// Tool usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUsageStats {
    pub total_usage: HashMap<String, u32>,
    pub success_rates: HashMap<String, f64>,
    pub average_execution_times: HashMap<String, f64>,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub total_execution_time_ms: u64,
    pub average_execution_time_ms: u64,
    pub total_llm_calls: u32,
    pub total_tool_calls: u32,
    pub total_tokens: u64,
    pub success_rate: f64,
}

/// Agent behavior analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentBehaviorAnalysis {
    pub depth_patterns: HashMap<u32, u32>,
    pub common_tool_sequences: Vec<Vec<String>>,
    pub average_decision_time_ms: u64,
    pub error_recovery_rate: f64,
}

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
            children.push(ascii_tree::Tree::Leaf(vec![score_summary]));

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
                children.push(ascii_tree::Tree::Leaf(vec![breakdown]));

                // Add specific issues if not perfect
                if scoring.final_score < 1.0 && !scoring.issues.is_empty() {
                    let issues_text = format!("⚠️  Issues: {}", scoring.issues.join(" | "));
                    children.push(ascii_tree::Tree::Leaf(vec![issues_text]));
                }

                // Add specific mismatches if available
                if !scoring.mismatches.is_empty() {
                    let mismatches_text = format!("🔧 Details: {}", scoring.mismatches.join(" | "));
                    children.push(ascii_tree::Tree::Leaf(vec![mismatches_text]));
                }
            }
        }

        // Add events as tree nodes
        for (i, event) in self.events.iter().enumerate() {
            children.push(self.render_event_as_tree_node(i + 1, event));
        }

        let tree = ascii_tree::Tree::Node(root_label, children);
        let mut buffer = String::new();
        ascii_tree::write_tree(&mut buffer, &tree).unwrap();
        buffer
    }

    fn render_event_as_tree_node(&self, event_num: usize, event: &FlowEvent) -> ascii_tree::Tree {
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
                    "🤖 Event {}: LLM Request (Depth: {}) - {} ({} tokens)",
                    event_num, event.depth, model, tokens
                )
            }
            FlowEventType::ToolCall => {
                let tool_name = event
                    .content
                    .data
                    .get("tool_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let exec_time = event
                    .content
                    .data
                    .get("execution_time_ms")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let duration_str = if exec_time >= 1000 {
                    format!("{:.2}s", exec_time as f64 / 1000.0)
                } else {
                    format!("{exec_time}ms")
                };
                format!(
                    "🔧 Event {}: Tool Call (Depth: {}) - {} ({})",
                    event_num, event.depth, tool_name, duration_str
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
                    "📋 Event {}: Tool Result (Depth: {}) - {} - {}",
                    event_num, event.depth, tool_name, result_status
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
                    "💰 Event {}: Transaction {} - {}",
                    event_num,
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
                    "🚨 Event {}: Error (Depth: {}) - {}",
                    event_num, event.depth, error_type
                )
            }
            FlowEventType::BenchmarkStateChange => {
                format!(
                    "🔄 Event {}: State Change (Depth: {})",
                    event_num, event.depth
                )
            }
        };

        let mut children = Vec::new();

        // Add event duration (simplified)
        let event_duration =
            if let Ok(duration_since_start) = event.timestamp.duration_since(self.start_time) {
                let ms = duration_since_start.as_millis();
                if ms >= 1000 {
                    format!("⏰ +{:.2}s", ms as f64 / 1000.0)
                } else {
                    format!("⏰ +{ms}ms")
                }
            } else {
                "⏰ Unknown time".to_string()
            };
        children.push(ascii_tree::Tree::Leaf(vec![event_duration]));

        // Add event-specific details
        match &event.event_type {
            FlowEventType::LlmRequest => {
                if let Some(prompt) = event.content.data.get("prompt").and_then(|v| v.as_str()) {
                    let preview = if prompt.len() > 100 {
                        format!("{}...", &prompt[..100])
                    } else {
                        prompt.to_string()
                    };
                    children.push(ascii_tree::Tree::Leaf(vec![format!(
                        "💬 Prompt: {}",
                        preview
                    )]));
                }
            }
            FlowEventType::ToolCall => {
                if let Some(args) = event.content.data.get("tool_args").and_then(|v| v.as_str()) {
                    let preview = if args.len() > 80 {
                        format!("{}...", &args[..80])
                    } else {
                        args.to_string()
                    };
                    children.push(ascii_tree::Tree::Leaf(vec![format!(
                        "📝 Args: {}",
                        preview
                    )]));
                }
            }
            FlowEventType::ToolResult => {
                if let Some(error) = event
                    .content
                    .data
                    .get("error_message")
                    .and_then(|v| v.as_str())
                {
                    children.push(ascii_tree::Tree::Leaf(vec![format!("❌ Error: {}", error)]));
                } else if let Some(result) = event.content.data.get("result_data") {
                    let result_str = serde_json::to_string_pretty(result).unwrap_or_default();
                    let preview = if result_str.len() > 100 {
                        format!("{}...", &result_str[..100])
                    } else {
                        result_str
                    };
                    children.push(ascii_tree::Tree::Leaf(vec![format!(
                        "✅ Result: {}",
                        preview
                    )]));
                }
            }
            FlowEventType::Error => {
                if let Some(message) = event.content.data.get("message").and_then(|v| v.as_str()) {
                    children.push(ascii_tree::Tree::Leaf(vec![format!(
                        "💥 Message: {}",
                        message
                    )]));
                }
            }
            _ => {}
        }

        ascii_tree::Tree::Node(event_label, children)
    }
}

/// Load and render a flow log from file as ASCII tree
pub fn render_flow_file_as_ascii_tree(
    file_path: &std::path::Path,
) -> Result<String, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(file_path)?;
    let flow: FlowLog = serde_yaml::from_str(&content)?;
    Ok(flow.render_as_ascii_tree())
}
