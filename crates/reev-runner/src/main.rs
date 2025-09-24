use anyhow::{Context, Result};
use clap::Parser;
use opentelemetry::global;
use opentelemetry::trace::TracerProvider;
use opentelemetry_sdk::trace as sdktrace;
use reev_lib::{
    agent::{Agent, DummyAgent},
    benchmark::TestCase,
    env::GymEnv,
    results::{FinalStatus, TestResult},
    solana_env::SolanaEnv,
    trace::{ExecutionTrace, TraceStep},
};
use std::fs::File;
use std::path::PathBuf;
use tracing::subscriber;
use tracing_subscriber::{EnvFilter, Registry, prelude::*};

mod renderer;

/// A command-line runner for the Reev evaluation framework.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to the benchmark YAML file to execute.
    #[arg(short, long)]
    benchmark: PathBuf,
}

/// Initializes the OpenTelemetry pipeline for tracing.
///
/// Sets up a pipeline that exports traces to stdout in a machine-readable
/// JSON format. It integrates `tracing` with `opentelemetry`.
fn init_tracing() -> Result<()> {
    let exporter = opentelemetry_stdout::SpanExporter::default();
    let provider = sdktrace::TracerProvider::builder()
        .with_simple_exporter(exporter)
        .build();
    let tracer = provider.tracer("reev-runner");
    global::set_tracer_provider(provider);

    let subscriber = Registry::default()
        // Filter logs based on the RUST_LOG env var, or a default.
        .with(EnvFilter::new("info,reev_lib=debug,reev_runner=debug"))
        .with(tracing_opentelemetry::layer().with_tracer(tracer));

    subscriber::set_global_default(subscriber)
        .context("Failed to set global default tracing subscriber")?;

    Ok(())
}

fn main() -> Result<()> {
    // This must be the first call in main.
    init_tracing()?;

    // When running with `cargo run -p`, the CWD is the crate root.
    // We change it to the workspace root to resolve benchmark paths correctly.
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        if let Some(workspace_root) = std::path::Path::new(&manifest_dir)
            .parent()
            .and_then(|p| p.parent())
        {
            std::env::set_current_dir(workspace_root)?;
        }
    }

    let cli = Cli::parse();
    println!("--- Reev Evaluation Runner ---");

    // 1. Load the benchmark file.
    let benchmark_path = &cli.benchmark;

    println!("[1/6] Loading benchmark from: {benchmark_path:?}");
    let f = File::open(benchmark_path)
        .with_context(|| format!("Failed to open benchmark file at: {benchmark_path:?}"))?;
    let test_case: TestCase = serde_yaml::from_reader(f)
        .with_context(|| format!("Failed to parse benchmark file: {benchmark_path:?}"))?;
    println!("      Loaded test case: '{}'", test_case.id);

    // 2. Instantiate the agent.
    println!("[2/6] Instantiating agent...");
    let mut agent = DummyAgent::new(test_case.ground_truth.expected_tool_calls.clone());
    println!("      Using DummyAgent");

    // 3. Instantiate the environment.
    println!("[3/6] Instantiating Solana environment...");
    let mut env = SolanaEnv::new()?;
    println!("      Environment created.");

    // 4. Run the evaluation loop and get the final state.
    let (final_observation, trace) = run_evaluation_loop(&mut env, &mut agent, &test_case)?;

    // 5. Calculate metrics and determine final status.
    println!("\n[5/6] Calculating metrics...");
    let scores = reev_lib::metrics::calculate_quantitative_metrics(
        &final_observation,
        &trace,
        &test_case.ground_truth,
    )?;
    let final_status = if scores.task_success_rate == 1.0 {
        FinalStatus::Succeeded
    } else {
        FinalStatus::Failed
    };
    println!("      --- Final Scores ---");
    println!("      Task Success Rate: {}", scores.task_success_rate);
    println!(
        "      Tool Selection Accuracy: {}",
        scores.tool_selection_accuracy
    );
    println!(
        "      Parameterization Accuracy: {}",
        scores.parameterization_accuracy
    );
    match final_status {
        FinalStatus::Succeeded => println!("      ✅ TASK SUCCEEDED"),
        FinalStatus::Failed => println!("      ❌ TASK FAILED"),
    }
    println!("      --------------------");

    // 6. Construct the final result and render it.
    println!("\n[6/6] Finalizing run...");
    let result = TestResult::new(&test_case, final_status, scores, trace);

    // The YAML output is generated but not printed by default in this step.
    let _yaml_output = serde_yaml::to_string(&result)?;

    // Render the result as a tree for immediate, human-readable feedback.
    let tree_output = renderer::render_result_as_tree(&result);
    println!("{}", tree_output);

    env.close()?;
    println!("      Environment closed.");

    println!("\n--- Evaluation Runner Finished ---");

    // Shutdown the tracer provider. This must be the last call.
    global::shutdown_tracer_provider();

    Ok(())
}

#[tracing::instrument(skip_all, fields(benchmark_id = %test_case.id))]
fn run_evaluation_loop(
    env: &mut SolanaEnv,
    agent: &mut dyn Agent,
    test_case: &TestCase,
) -> Result<(reev_lib::agent::AgentObservation, ExecutionTrace)> {
    println!(
        "[4/6] Starting evaluation loop for prompt: '{}'",
        test_case.prompt
    );
    let options = serde_json::to_value(&test_case.initial_state)?;
    let mut observation = env.reset(None, Some(options))?;
    env.render();

    let mut trace = ExecutionTrace::new(test_case.prompt.clone());
    let mut final_observation = observation.clone();

    for i in 0..10 {
        // Max 10 steps
        println!("\n--- Step {} ---", i + 1);
        let action = agent.get_action(&observation)?;
        let step_result = env.step(action.clone(), &test_case.ground_truth)?;
        env.render();

        let trace_step = TraceStep {
            thought: None, // DummyAgent doesn't have "thoughts"
            action,
            observation: step_result.observation.clone(),
            info: step_result.info,
        };
        trace.add_step(trace_step);

        observation = step_result.observation;

        if step_result.terminated || step_result.truncated {
            final_observation = observation.clone();
            println!(
                "\n--- Episode Finished (Terminated: {}, Truncated: {}) ---",
                step_result.terminated, step_result.truncated
            );
            break;
        }

        // If the loop finishes without termination, the last observation is the final one.
        if i == 9 {
            final_observation = observation.clone();
        }
    }

    Ok((final_observation, trace))
}
