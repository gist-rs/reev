use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Basic benchmark information for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkInfo {
    /// Benchmark identifier (filename without extension)
    pub id: String,
    /// Human-readable title
    pub title: String,
    /// Benchmark description
    pub description: String,
    /// Benchmark category (e.g., "sol-transfer", "spl-token", "jupiter-swap")
    pub category: String,
    /// Difficulty level (1-5)
    pub difficulty: u8,
    /// Estimated execution time in seconds
    pub estimated_time_seconds: u64,
    /// Required tool capabilities
    pub required_tools: Vec<String>,
    /// Benchmark tags
    pub tags: Vec<String>,
    /// Whether this is a deterministic test
    pub deterministic: bool,
    /// File path relative to benchmarks directory
    pub file_path: String,
}

/// Agent configuration information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    /// Agent identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Agent type (deterministic, llm, hybrid)
    pub agent_type: AgentType,
    /// Model name if applicable
    pub model_name: Option<String>,
    /// Supported benchmark categories
    pub supported_categories: Vec<String>,
    /// Agent capabilities
    pub capabilities: Vec<String>,
    /// Default configuration parameters
    pub default_config: HashMap<String, serde_json::Value>,
}

/// Agent type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentType {
    /// Deterministic agent for testing with ground truth
    Deterministic,
    /// LLM-based agent for evaluation
    LLM,
    /// Hybrid agent combining both approaches
    Hybrid,
}

impl AgentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AgentType::Deterministic => "deterministic",
            AgentType::LLM => "llm",
            AgentType::Hybrid => "hybrid",
        }
    }
}

impl std::str::FromStr for AgentType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "deterministic" => Ok(AgentType::Deterministic),
            "llm" => Ok(AgentType::LLM),
            "hybrid" => Ok(AgentType::Hybrid),
            _ => Err(format!("Invalid agent type: {s}")),
        }
    }
}

/// Benchmark execution result summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Benchmark identifier
    pub benchmark_id: String,
    /// Agent used
    pub agent: String,
    /// Execution timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Success status
    pub success: bool,
    /// Score (0.0 to 1.0)
    pub score: f64,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
    /// Number of tool calls made
    pub tool_calls_count: u32,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Additional metrics
    pub metrics: HashMap<String, f64>,
}

/// Benchmark validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkValidation {
    /// Benchmark identifier
    pub benchmark_id: String,
    /// Whether validation passed
    pub valid: bool,
    /// Validation errors
    pub errors: Vec<String>,
    /// Validation warnings
    pub warnings: Vec<String>,
    /// Missing required tools
    pub missing_tools: Vec<String>,
    /// Invalid configuration sections
    pub invalid_sections: Vec<String>,
}

/// List of available benchmarks with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkList {
    /// Total number of benchmarks
    pub total_count: usize,
    /// Benchmarks by category
    pub by_category: HashMap<String, Vec<BenchmarkInfo>>,
    /// Benchmarks by difficulty
    pub by_difficulty: HashMap<u8, Vec<BenchmarkInfo>>,
    /// All benchmarks
    pub benchmarks: Vec<BenchmarkInfo>,
}

/// List of available agents with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentList {
    /// Total number of agents
    pub total_count: usize,
    /// Agents by type
    pub by_type: HashMap<String, Vec<AgentInfo>>,
    /// All agents
    pub agents: Vec<AgentInfo>,
}

/// Benchmark execution parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkExecutionParams {
    /// Benchmark identifier or file path
    pub benchmark: String,
    /// Agent to use
    pub agent: String,
    /// Optional execution timeout
    pub timeout_seconds: Option<u64>,
    /// Whether to use shared surfpool
    pub shared_surfpool: Option<bool>,
    /// Additional configuration
    pub config: HashMap<String, serde_json::Value>,
}

impl BenchmarkInfo {
    /// Create a new benchmark info
    pub fn new(id: String, title: String, category: String) -> Self {
        Self {
            id,
            title,
            description: String::new(),
            category,
            difficulty: 1,
            estimated_time_seconds: 60,
            required_tools: Vec::new(),
            tags: Vec::new(),
            deterministic: false,
            file_path: String::new(),
        }
    }

    /// Check if agent type is supported
    pub fn supports_agent(&self, agent_type: &AgentType) -> bool {
        match agent_type {
            AgentType::Deterministic => self.deterministic,
            AgentType::LLM => true,    // LLM agents can run any benchmark
            AgentType::Hybrid => true, // Hybrid agents can run any benchmark
        }
    }

    /// Get estimated execution time in milliseconds
    pub fn estimated_duration_ms(&self) -> u64 {
        self.estimated_time_seconds * 1000
    }
}

impl AgentInfo {
    /// Create a new agent info
    pub fn new(id: String, name: String, agent_type: AgentType) -> Self {
        Self {
            id,
            name,
            agent_type,
            model_name: None,
            supported_categories: Vec::new(),
            capabilities: Vec::new(),
            default_config: HashMap::new(),
        }
    }

    /// Check if agent can run benchmark
    pub fn can_run_benchmark(&self, benchmark: &BenchmarkInfo) -> bool {
        // Check if category is supported
        if !self.supported_categories.is_empty()
            && !self.supported_categories.contains(&benchmark.category)
        {
            return false;
        }

        // Check if agent type is compatible
        if !benchmark.supports_agent(&self.agent_type) {
            return false;
        }

        // Check required tools
        for tool in &benchmark.required_tools {
            if !self.capabilities.contains(tool) {
                return false;
            }
        }

        true
    }
}
