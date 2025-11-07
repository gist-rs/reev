use anyhow::{Context, Result};
use reev_lib::agent::Agent;
use reev_lib::llm_agent::LlmAgent;
use std::process::Command;
use tracing::{info, instrument};

// Module declarations
pub mod benchmark;
pub mod dependency;
pub mod flow;
pub mod renderer;
pub mod utils;
pub mod version;

// Re-export main public API functions
pub use benchmark::{
    run_benchmarks, run_benchmarks_with_source, run_dynamic_flow, run_recovery_flow,
};

#[allow(dead_code)]
const AGENT_PORT: u16 = 9090;

/// RAII guard for dependency management
struct DependencyManagerGuard {
    pub manager: dependency::DependencyManager,
}

impl Drop for DependencyManagerGuard {
    fn drop(&mut self) {
        info!("Dependency manager dropped - processes will be cleaned up on next startup");
        // Note: Actual cleanup is handled at startup to avoid runtime-in-runtime issues
        // This ensures clean state for next run
    }
}

/// Initialize dependencies with custom configuration
#[instrument(skip_all)]
pub async fn init_dependencies_with_config(
    config: dependency::DependencyConfig,
) -> Result<DependencyManagerGuard> {
    info!("Initializing dependencies with config: {:?}", config);

    let manager = dependency::DependencyManager::new(config)
        .await
        .context("Failed to create dependency manager")?;

    info!("Dependencies initialized successfully");
    Ok(DependencyManagerGuard { manager })
}

/// Determines the appropriate agent type based on flow type
#[instrument(skip_all)]
pub fn determine_agent_from_flow_type(flow_type: &str) -> &'static str {
    match flow_type.to_lowercase().as_str() {
        "llm" | "openai" | "anthropic" | "claude" | "gpt" => "llm",
        "deterministic" | "simple" => "deterministic",
        _ => {
            info!("Unknown flow type '{}', defaulting to LLM agent", flow_type);
            "llm"
        }
    }
}

/// Initializes and returns an agent based on the specified type
#[instrument(skip_all)]
pub async fn create_agent(agent_type: &str) -> Result<Box<dyn Agent>> {
    match determine_agent_from_flow_type(agent_type) {
        "llm" => {
            let agent = LlmAgent::new(agent_type).await?;
            Ok(Box::new(agent))
        }
        "deterministic" => {
            // Implementation for deterministic agent would go here
            Err(anyhow::anyhow!("Deterministic agent not yet implemented"))
        }
        _ => Err(anyhow::anyhow!("Unsupported agent type: {}", agent_type)),
    }
}

/// Utility function to check if a port is in use
pub fn is_port_in_use(port: u16) -> bool {
    match Command::new("lsof")
        .arg("-i")
        .arg(format!(":{}", port))
        .output()
    {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

/// Utility function to get available port
pub fn get_available_port(starting_port: u16) -> u16 {
    let mut port = starting_port;
    while port < 65535 {
        if !is_port_in_use(port) {
            return port;
        }
        port += 1;
    }
    starting_port // Fallback to starting port if no available port found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_agent_from_flow_type() {
        assert_eq!(determine_agent_from_flow_type("llm"), "llm");
        assert_eq!(determine_agent_from_flow_type("openai"), "llm");
        assert_eq!(determine_agent_from_flow_type("anthropic"), "llm");
        assert_eq!(
            determine_agent_from_flow_type("deterministic"),
            "deterministic"
        );
        assert_eq!(determine_agent_from_flow_type("unknown"), "llm");
        assert_eq!(determine_agent_from_flow_type("LLM"), "llm"); // case insensitive
    }

    #[test]
    fn test_port_utilities() {
        // Test that port checking doesn't panic
        let result = is_port_in_use(80);
        println!("Port 80 is in use: {}", result);

        // Test available port function
        let port = get_available_port(9090);
        println!("Available port starting from 9090: {}", port);
        assert!(port >= 9090);
        assert!(port <= 65535);
    }
}
