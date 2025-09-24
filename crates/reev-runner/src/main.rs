use anyhow::{Context, Result};
use clap::Parser;
use reev_lib::{
    agent::{Agent, DummyAgent},
    benchmark::TestCase,
    env::GymEnv,
    solana_env::SolanaEnv,
    trace::{ExecutionTrace, TraceStep},
};
use std::fs::File;
use std::path::PathBuf;

/// A command-line runner for the Reev evaluation framework.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to the benchmark YAML file to execute.
    #[arg(short, long)]
    benchmark: PathBuf,
}

fn main() -> Result<()> {
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

    // 5. Calculate metrics from the final state.
    println!("\n[5/6] Calculating metrics...");
    match reev_lib::metrics::calculate_quantitative_metrics(
        &final_observation,
        &test_case.ground_truth,
    ) {
        Ok(scores) => {
            println!("      --- Final Scores ---");
            println!("      Task Success Rate: {}", scores.task_success_rate);
            if scores.task_success_rate == 1.0 {
                println!("      ✅ TASK SUCCEEDED");
            } else {
                println!("      ❌ TASK FAILED");
            }
            println!("      --------------------");
        }
        Err(e) => println!("      Error calculating metrics: {e}"),
    }

    // 6. Finalize and report.
    println!("\n[6/6] Finalizing run...");
    println!("      --- Execution Trace ---");
    let trace_json = serde_json::to_string_pretty(&trace)?;
    println!("{trace_json}");

    let _ = env.close();
    println!("      Environment closed.");

    println!("\n--- Evaluation Runner Finished ---");
    Ok(())
}

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
