use anyhow::{Context, Result};
use clap::Parser;
use dotenvy::dotenv;
use opentelemetry::global::{self};
use opentelemetry::trace::TracerProvider;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::trace as sdktrace;
use reev_runner::renderer;
use std::path::PathBuf;
use tracing::{info, subscriber};
use tracing_subscriber::{EnvFilter, Registry, fmt, prelude::*};

/// A command-line runner for the Reev evaluation framework.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to a specific benchmark YAML file or a directory containing multiple benchmarks.
    #[arg(default_value = "benchmarks/")]
    path: PathBuf,

    /// The agent to run the benchmarks with ('deterministic' for ground truth, 'ai' for the model).
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
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        if let Some(workspace_root) = std::path::Path::new(&manifest_dir)
            .parent()
            .and_then(|p| p.parent())
        {
            std::env::set_current_dir(workspace_root)?;
        }
    }

    let cli = Cli::parse();
    info!("--- Reev Evaluation Runner ---");

    // Run the benchmarks using the library function.
    let results = reev_runner::run_benchmarks(cli.path, &cli.agent).await?;

    // Render the results.
    for result in &results {
        let tree_output = renderer::render_result_as_tree(result);
        info!("{tree_output}");
    }

    // Shutdown tracing.
    tracer_provider.shutdown()?;
    Ok(())
}
