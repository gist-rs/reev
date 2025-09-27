use anyhow::{Context, Result};
use clap::Parser;
use dotenvy::dotenv;
use opentelemetry::global::{self};
use opentelemetry::trace::TracerProvider;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::trace as sdktrace;
use project_root::get_project_root;
use reev_runner::renderer;
use std::path::PathBuf;
use tracing::{info, subscriber};
use tracing_subscriber::{EnvFilter, Registry, fmt, prelude::*};

/// A command-line runner for the Reev evaluation framework.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to a specific benchmark YAML file or a directory containing multiple benchmarks.
    path: PathBuf,

    /// The agent to run the benchmarks with.
    /// Can be 'deterministic', 'local', or a specific model name like 'gemini-2.5-pro'.
    #[arg(long, default_value = "deterministic")]
    agent: String,
}

/// Initializes the OpenTelemetry pipeline for tracing.
fn init_tracing() -> Result<sdktrace::SdkTracerProvider> {
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
    info!(
        "Running benchmarks at: '{}' with agent: '{}'",
        cli.path.display(),
        cli.agent
    );

    // Run the benchmarks using the library function.
    let results = reev_runner::run_benchmarks(cli.path, &cli.agent).await?;

    // Render the results.
    for result in &results {
        let tree_output = renderer::render_result_as_tree(result);
        info!("\n{tree_output}");
    }

    // Shutdown tracing.
    tracer_provider.shutdown()?;
    Ok(())
}
