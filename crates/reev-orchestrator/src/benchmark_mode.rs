//! Benchmark Mode Execution
//!
//! This module handles static benchmark YML file management and execution routing.
//! It provides clean separation between benchmark and dynamic execution modes.

use crate::Result;
use reev_types::execution::ExecutionResponse;
use std::path::PathBuf;
use tracing::{debug, info, instrument};

/// Execute a static benchmark using its YML file
///
/// This function:
/// 1. Validates benchmark exists
/// 2. Returns path for execution by caller
/// 3. Provides metadata for routing
///
/// # Arguments
/// * `benchmark_id` - The ID of the benchmark (e.g., "300-jup-swap-then-lend-deposit-dyn")
/// * `agent` - Optional agent type to use
///
/// # Returns
/// * `Result<BenchmarkExecutionPlan>` - Execution plan with path and metadata
///
/// # Errors
/// * If benchmark file doesn't exist
/// * If validation fails
#[instrument(skip_all)]
pub async fn prepare_static_benchmark(
    benchmark_id: &str,
    agent: Option<&str>,
) -> Result<BenchmarkExecutionPlan> {
    info!("Preparing static benchmark: {}", benchmark_id);

    let yml_path = get_static_benchmark_path(benchmark_id)?;
    let metadata = get_benchmark_metadata(benchmark_id)?;

    debug!("Using benchmark file: {}", yml_path.display());

    let plan = BenchmarkExecutionPlan {
        yml_path,
        benchmark_id: benchmark_id.to_string(),
        agent: agent.map(|a| a.to_string()),
        execution_mode: ExecutionMode::Benchmark,
        metadata: ExecutionMetadata {
            source: "static_file".to_string(),
            created_at: chrono::Utc::now(),
            benchmark_type: "300_series".to_string(),
            description: metadata.description,
            tags: metadata.tags,
        },
    };

    info!("Benchmark preparation completed: {}", benchmark_id);
    Ok(plan)
}

/// Execute a static benchmark using its YML file
///
/// This is a convenience function that calls prepare_static_benchmark
/// and delegates actual execution to the caller
///
/// # Arguments
/// * `benchmark_id` - The ID of the benchmark
/// * `agent` - Optional agent type
/// * `executor` - Function to execute the YML file
///
/// # Returns
/// * `Result<ExecutionResult>` - The execution result
pub async fn execute_static_benchmark<F, Fut>(
    benchmark_id: &str,
    agent: Option<&str>,
    executor: F,
) -> Result<ExecutionResponse>
where
    F: FnOnce(PathBuf, Option<String>) -> Fut,
    Fut: std::future::Future<Output = Result<ExecutionResponse>>,
{
    let plan = prepare_static_benchmark(benchmark_id, agent).await?;
    executor(plan.yml_path, plan.agent).await
}

/// Get the path to a static benchmark YML file
///
/// # Arguments
/// * `id` - The benchmark ID
///
/// # Returns
/// * `Result<PathBuf>` - Path to the YML file
///
/// # Errors
/// * If benchmark file doesn't exist
pub fn get_static_benchmark_path(id: &str) -> Result<PathBuf> {
    let path = PathBuf::from("../../benchmarks").join(format!("{id}.yml"));

    if !path.exists() {
        return Err(anyhow::anyhow!(
            "Benchmark file not found: {}",
            path.display()
        ));
    }

    Ok(path)
}

/// List all available static benchmarks
///
/// # Returns
/// * `Result<Vec<String>>` - List of benchmark IDs
pub fn list_static_benchmarks() -> Result<Vec<String>> {
    let benchmarks_dir = PathBuf::from("benchmarks");

    if !benchmarks_dir.exists() {
        return Ok(vec![]);
    }

    let mut benchmarks = Vec::new();

    for entry in std::fs::read_dir(benchmarks_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("yml") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                benchmarks.push(stem.to_string());
            }
        }
    }

    benchmarks.sort();
    Ok(benchmarks)
}

/// Check if a benchmark exists
///
/// # Arguments
/// * `benchmark_id` - The benchmark ID to check
///
/// # Returns
/// * `bool` - True if benchmark exists
pub fn benchmark_exists(benchmark_id: &str) -> bool {
    let path = PathBuf::from("benchmarks").join(format!("{benchmark_id}.yml"));
    path.exists()
}

/// Get benchmark metadata without executing
///
/// # Arguments
/// * `benchmark_id` - The benchmark ID
///
/// # Returns
/// * `Result<BenchmarkMetadata>` - Benchmark metadata
pub fn get_benchmark_metadata(benchmark_id: &str) -> Result<BenchmarkMetadata> {
    let yml_path = get_static_benchmark_path(benchmark_id)?;

    let content = std::fs::read_to_string(&yml_path)?;
    let yaml: serde_yaml::Value = serde_yaml::from_str(&content)?;

    let metadata = BenchmarkMetadata {
        id: benchmark_id.to_string(),
        description: yaml["description"]
            .as_str()
            .unwrap_or("No description")
            .to_string(),
        tags: yaml["tags"]
            .as_sequence()
            .map(|seq| {
                seq.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default(),
        file_path: yml_path,
    };

    Ok(metadata)
}

/// Benchmark execution plan
#[derive(Debug, Clone)]
pub struct BenchmarkExecutionPlan {
    /// Path to YML file
    pub yml_path: PathBuf,
    /// Benchmark ID
    pub benchmark_id: String,
    /// Agent type to use
    pub agent: Option<String>,
    /// Execution mode
    pub execution_mode: ExecutionMode,
    /// Execution metadata
    pub metadata: ExecutionMetadata,
}

/// Execution mode
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionMode {
    Benchmark,
    Dynamic,
}

/// Execution metadata
#[derive(Debug, Clone)]
pub struct ExecutionMetadata {
    /// Source of execution (static_file, dynamic, etc.)
    pub source: String,
    /// When plan was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Type of benchmark
    pub benchmark_type: String,
    /// Description
    pub description: String,
    /// Tags
    pub tags: Vec<String>,
}

/// Benchmark metadata
#[derive(Debug, Clone)]
pub struct BenchmarkMetadata {
    /// Benchmark ID
    pub id: String,
    /// Benchmark description
    pub description: String,
    /// Benchmark tags
    pub tags: Vec<String>,
    /// Path to YML file
    pub file_path: PathBuf,
}

// Tests moved to tests/benchmark_mode_tests.rs
