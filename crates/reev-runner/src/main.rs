use anyhow::{Context, Result};
use clap::Parser;
use dotenvy::dotenv;
use opentelemetry::global::{self};
use opentelemetry::trace::TracerProvider;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::trace as sdktrace;
use project_root::get_project_root;
use reev_orchestrator::OrchestratorGateway;
use reev_runner::renderer;
use std::path::PathBuf;
use tracing::{info, subscriber};
use tracing_subscriber::{EnvFilter, Registry, fmt, prelude::*};

/// A command-line runner for the Reev evaluation framework.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to a specific benchmark YAML file, a directory containing multiple benchmarks, or a flow log file.
    /// Not used when --dynamic or --direct is specified.
    #[arg(required_unless_present_any = ["dynamic", "direct"])]
    path: Option<PathBuf>,

    /// The agent to run the benchmarks with.
    /// Can be 'deterministic', 'local', or a specific model name like 'glm-4.6'.
    #[arg(long, default_value = "deterministic")]
    agent: String,

    /// Render flow log as ASCII tree (only works with .yml flow files)
    #[arg(long)]
    render_flow: bool,

    /// Use shared surfpool instances instead of creating fresh ones
    #[arg(long)]
    shared_surfpool: bool,

    /// Execution ID to use for this run (for API coordination)
    #[arg(long)]
    execution_id: Option<String>,

    /// Use dynamic flow generation from natural language prompt
    #[arg(long)]
    dynamic: bool,

    /// Use direct in-memory flow execution (Phase 2 - no temporary files)
    #[arg(long)]
    direct: bool,

    /// Wallet pubkey for dynamic flow context resolution
    #[arg(long)]
    wallet: Option<String>,

    /// Natural language prompt for dynamic flow generation
    #[arg(long)]
    prompt: Option<String>,

    /// Enable Phase 3 recovery mechanisms
    #[arg(long)]
    recovery: bool,

    /// Atomic mode for flow execution: strict, lenient, or conditional
    #[arg(long, value_parser = ["strict", "lenient", "conditional"])]
    atomic_mode: Option<String>,

    /// Maximum recovery time per step in milliseconds
    #[arg(long, default_value = "30000")]
    max_recovery_time_ms: u64,

    /// Enable alternative flow recovery strategies
    #[arg(long)]
    enable_alternative_flows: bool,

    /// Enable user fulfillment recovery (interactive mode)
    #[arg(long)]
    enable_user_fulfillment: bool,

    /// Number of retry attempts for recovery
    #[arg(long, default_value = "3")]
    retry_attempts: usize,
}

/// Initializes OpenTelemetry pipeline for tracing with console output.
fn init_tracing() -> Result<sdktrace::SdkTracerProvider> {
    // Use regular tracing instead of enhanced otel logging to avoid file conflicts
    // Agent will handle enhanced otel logging for tool calls
    info!("Initializing runner tracing (agent will handle tool call logging)");

    let provider = sdktrace::SdkTracerProvider::builder()
        .with_resource(Resource::builder().with_service_name("reev-runner").build())
        .build();
    let tracer = provider.tracer("reev-runner");
    global::set_tracer_provider(provider.clone());

    let subscriber = Registry::default()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,reev_lib=debug,reev_runner=debug")),
        )
        .with(fmt::layer())
        .with(tracing_opentelemetry::layer().with_tracer(tracer));

    subscriber::set_global_default(subscriber)
        .context("Failed to set global default tracing subscriber")?;

    Ok(provider)
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from a .env file.
    dotenv().ok();

    // Initialize tracing.
    let tracer_provider = init_tracing()?;

    // Set the current directory to the workspace root for consistent path resolution.
    let workspace_root = get_project_root().context("Failed to find workspace root")?;
    std::env::set_current_dir(&workspace_root)
        .with_context(|| format!("Failed to set current directory to {workspace_root:?}"))?;

    let cli = Cli::parse();

    info!("--- Reev Evaluation Runner ---");

    // Handle Phase 3 recovery flow execution
    if cli.recovery {
        return handle_recovery_flow(cli).await;
    }

    // Handle Phase 2 direct flow execution
    if cli.direct {
        return handle_direct_flow(cli).await;
    }

    // Handle dynamic flow generation (Phase 1 - backward compatibility)
    if cli.dynamic {
        return handle_dynamic_flow(cli).await;
    }

    // Check if this is a flow log file that should be rendered
    if cli.render_flow {
        let path = cli.path.expect("--render-flow requires a path");
        if path.extension().and_then(|s| s.to_str()) == Some("yml") {
            info!("Rendering flow log as ASCII tree: '{}'", path.display());
            match reev_lib::flow::render_flow_file_as_ascii_tree(&path) {
                Ok(tree_output) => {
                    println!("\n{tree_output}");
                }
                Err(e) => {
                    eprintln!("Error rendering flow log: {e}");
                    std::process::exit(1);
                }
            }
        } else {
            eprintln!("Error: --render-flow requires a .yml flow log file");
            std::process::exit(1);
        }
    } else {
        let path = cli
            .path
            .expect("PATH should be present when not using --dynamic");
        info!(
            "Running benchmarks at: '{}' with agent: '{}'",
            path.display(),
            cli.agent
        );

        // Run benchmarks using the library function.
        let results = reev_runner::run_benchmarks(
            path,
            &cli.agent,
            cli.shared_surfpool,
            false,
            cli.execution_id,
        )
        .await?;

        // Render the results.
        for result in &results {
            let tree_output = renderer::render_result_as_tree(result);
            info!("\n{tree_output}");
        }
    }

    // Shutdown tracing.
    tracer_provider.shutdown()?;
    Ok(())
}

/// Handle dynamic flow generation from natural language prompt
async fn handle_dynamic_flow(cli: Cli) -> Result<()> {
    info!("--- Dynamic Flow Generation ---");

    // Validate required parameters
    let prompt = cli
        .prompt
        .ok_or_else(|| anyhow::anyhow!("--prompt is required when using --dynamic"))?;
    let wallet = cli
        .wallet
        .ok_or_else(|| anyhow::anyhow!("--wallet is required when using --dynamic"))?;

    info!("Generating dynamic flow for prompt: '{}'", prompt);
    info!("Using wallet: {}", wallet);

    // Initialize orchestrator gateway
    let gateway = OrchestratorGateway::new();

    // Process user request and generate dynamic flow
    let (flow_plan, yml_path) = gateway
        .process_user_request(&prompt, &wallet)
        .await
        .context("Failed to process dynamic flow request")?;

    info!(
        "Generated flow plan '{}' with {} steps",
        flow_plan.flow_id,
        flow_plan.steps.len()
    );
    info!("Temporary YML file: {}", yml_path);

    // Run the generated flow using existing runner functionality
    let yml_path = PathBuf::from(yml_path);
    let results = reev_runner::run_benchmarks(
        yml_path.clone(),
        &cli.agent,
        cli.shared_surfpool,
        false,
        cli.execution_id,
    )
    .await
    .context("Failed to execute generated dynamic flow")?;

    // Render the results
    for result in &results {
        let tree_output = renderer::render_result_as_tree(result);
        info!("\n{tree_output}");
    }

    Ok(())
}

/// Handle Phase 3 recovery flow execution from natural language prompt
async fn handle_recovery_flow(cli: Cli) -> Result<()> {
    info!("--- Phase 3: Recovery Flow Execution ---");

    // Validate required parameters
    let prompt = cli
        .prompt
        .ok_or_else(|| anyhow::anyhow!("--prompt is required when using --recovery"))?;
    let wallet = cli
        .wallet
        .ok_or_else(|| anyhow::anyhow!("--wallet is required when using --recovery"))?;

    info!(
        "Executing recovery flow for prompt: '{}' with wallet: {}",
        prompt, wallet
    );

    // Parse atomic mode
    let atomic_mode = match cli.atomic_mode.as_deref() {
        Some("lenient") => Some(reev_types::flow::AtomicMode::Lenient),
        Some("conditional") => Some(reev_types::flow::AtomicMode::Conditional),
        _ => Some(reev_types::flow::AtomicMode::Strict), // default
    };

    info!(
        "Atomic mode: {:?}, Max recovery time: {}ms, Alternative flows: {}, User fulfillment: {}",
        atomic_mode,
        cli.max_recovery_time_ms,
        cli.enable_alternative_flows,
        cli.enable_user_fulfillment
    );

    // Create recovery configuration
    let recovery_config = reev_orchestrator::RecoveryConfig {
        max_recovery_time_ms: cli.max_recovery_time_ms,
        enable_alternative_flows: cli.enable_alternative_flows,
        enable_user_fulfillment: cli.enable_user_fulfillment,
        base_retry_delay_ms: 1000,
        max_retry_delay_ms: 10000,
        backoff_multiplier: 2.0,
    };

    // Use new recovery execution function
    let results = reev_runner::run_recovery_flow(
        &prompt,
        &wallet,
        &cli.agent,
        cli.shared_surfpool,
        cli.execution_id,
        recovery_config,
        atomic_mode,
    )
    .await
    .context("Failed to execute recovery flow")?;

    // Render the results
    for result in &results {
        let tree_output = renderer::render_result_as_tree(result);
        info!("\n{tree_output}");
    }

    Ok(())
}

/// Handle Phase 2 direct flow execution from natural language prompt
async fn handle_direct_flow(cli: Cli) -> Result<()> {
    info!("--- Phase 2: Direct Dynamic Flow Execution ---");

    // Validate required parameters
    let prompt = cli
        .prompt
        .ok_or_else(|| anyhow::anyhow!("--prompt is required when using --direct"))?;
    let wallet = cli
        .wallet
        .ok_or_else(|| anyhow::anyhow!("--wallet is required when using --direct"))?;

    info!(
        "Executing direct flow for prompt: '{}' with wallet: {}",
        prompt, wallet
    );

    // Use new direct execution function
    let results = reev_runner::run_dynamic_flow(
        &prompt,
        &wallet,
        &cli.agent,
        cli.shared_surfpool,
        cli.execution_id,
    )
    .await
    .context("Failed to execute direct dynamic flow")?;

    // Render the results
    for result in &results {
        let tree_output = renderer::render_result_as_tree(result);
        info!("\n{tree_output}");
    }

    Ok(())
}
