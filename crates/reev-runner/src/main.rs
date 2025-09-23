use anyhow::Result;
use reev_lib::{
    agent::{Agent, DummyAgent},
    benchmark::TestCase,
    env::GymEnv,
    metrics::calculate_quantitative_metrics,
    solana_env::SolanaEnv,
    trace::{ExecutionTrace, TraceStep},
};
use std::fs::File;
use std::path::Path;

fn main() -> Result<()> {
    println!("--- Reev Evaluation Runner ---");

    // 1. Load the benchmark file.
    let benchmark_path = Path::new("benchmarks/nft-transfer-001.yml");
    println!("[1/7] Loading benchmark from: {benchmark_path:?}");
    let f = File::open(benchmark_path)?;
    let test_case: TestCase = serde_yaml::from_reader(f)?;
    println!("      Loaded test case: {}", test_case.id);

    // 2. Instantiate the agent.
    println!("[2/7] Instantiating agent...");
    let mut agent = DummyAgent::new();
    println!("      Using DummyAgent");

    // 3. Instantiate the environment.
    println!("[3/7] Instantiating Solana environment...");
    let mut env = SolanaEnv::new()?;
    println!("      Environment created.");

    // 4. Initialize trace
    println!("[4/7] Initializing execution trace...");
    let mut trace = ExecutionTrace::new(test_case.prompt.clone());
    println!("      Trace initialized.");

    // 5. Run the evaluation loop.
    println!("[5/7] Starting evaluation loop...");
    let options = serde_json::to_value(&test_case.initial_state)?;
    let mut observation = env.reset(None, Some(options))?;

    for i in 0..10 {
        println!("\n--- Step {} ---", i + 1);
        let action = agent.get_action(&observation)?;
        let step_result = env.step(action.clone(), &test_case.ground_truth)?;

        let trace_step = TraceStep {
            thought: None, // DummyAgent doesn't produce thoughts yet.
            action,
            observation: step_result.observation.clone(),
            info: step_result.info,
        };
        trace.add_step(trace_step);
        println!("      Step recorded in trace.");

        observation = step_result.observation;

        if step_result.terminated || step_result.truncated {
            println!("\n--- Episode Finished ---");
            break;
        }
    }

    // 6. Calculate metrics.
    println!("\n[6/7] Calculating metrics...");
    match calculate_quantitative_metrics(&observation, &test_case.ground_truth) {
        Ok(scores) => println!("      Scores: {scores:?}"),
        Err(e) => println!("      Error calculating metrics: {e}"),
    }

    // 7. Finalize and report.
    println!("\n[7/7] Finalizing run...");
    println!("      --- Execution Trace ---");
    let trace_json = serde_json::to_string_pretty(&trace)?;
    println!("{trace_json}");

    env.close();
    println!("      Environment closed.");

    println!("\n--- Evaluation Runner Finished ---");

    Ok(())
}
