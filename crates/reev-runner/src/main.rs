use anyhow::Result;
use reev_lib::{
    agent::{Agent, DummyAgent},
    benchmark::TestCase,
    env::GymEnv,
    solana_env::SolanaEnv,
};
use std::fs::File;
use std::path::Path;

fn main() -> Result<()> {
    println!("--- Reev Evaluation Runner ---");

    // 1. Load the benchmark file.
    let benchmark_path = Path::new("benchmarks/transfer-simple-001.yml");
    println!("[1/5] Loading benchmark from: {:?}", benchmark_path);
    if !benchmark_path.exists() {
        anyhow::bail!(
            "Benchmark file not found. Make sure you are running from the workspace root."
        );
    }
    let f = File::open(benchmark_path)?;
    let test_case: TestCase = serde_yaml::from_reader(f)
        .map_err(|e| anyhow::anyhow!("Failed to parse benchmark file: {}", e))?;
    println!("      Loaded test case: {}", test_case.id);

    // 2. Instantiate the agent.
    println!("[2/5] Instantiating agent...");
    let mut agent = DummyAgent;
    println!("      Using DummyAgent");

    // 3. Instantiate the environment.
    println!("[3/5] Instantiating Solana environment...");
    let mut env = SolanaEnv::new()?;
    println!("      Environment created.");

    // 4. Reset the environment and get the initial observation.
    println!("[4/5] Resetting environment...");
    let initial_observation = env.reset(None, None)?;
    println!(
        "      Environment reset. Initial observation: {:?}",
        initial_observation
    );

    let action = agent.get_action(&initial_observation)?;
    println!("      Agent decided first action: {:?}", action);

    // 5. Clean up.
    println!("[5/5] Closing environment...");
    env.close();
    println!("      Environment closed.");

    println!("--- Evaluation Runner Finished ---");

    Ok(())
}
