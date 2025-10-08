use anyhow::{Context, Result, anyhow};
use reev_lib::{
    agent::{Agent, AgentObservation},
    benchmark::TestCase,
    env::GymEnv,
    llm_agent::LlmAgent,
    results::{FinalStatus, TestResult},
    score::calculate_final_score,
    solana_env::environment::SolanaEnv,
    test_scenarios,
    trace::ExecutionTrace,
};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tracing::{info, instrument};

pub mod db;
pub mod dependency;
pub mod renderer;

#[allow(dead_code)]
const AGENT_PORT: u16 = 9090;

/// RAII guard for dependency management
struct DependencyManagerGuard {
    #[allow(dead_code)]
    manager: dependency::DependencyManager,
}

impl Drop for DependencyManagerGuard {
    fn drop(&mut self) {
        info!("Cleaning up dependency manager...");
        // Note: Cleanup is handled asynchronously in the main runtime
        // Avoiding runtime-in-runtime issues in Drop implementation
    }
}

/// Initialize dependency management and ensure services are running
async fn init_dependencies() -> Result<(DependencyManagerGuard, dependency::DependencyUrls)> {
    info!("Initializing dependency management...");

    let config = dependency::DependencyConfig::from_env();
    let mut manager = dependency::DependencyManager::new(config)
        .context("Failed to create dependency manager")?;

    // Set up signal handlers for graceful shutdown
    manager
        .setup_signal_handlers()
        .context("Failed to setup signal handlers")?;

    // Ensure all dependencies are running
    let urls = manager
        .ensure_dependencies()
        .await
        .context("Failed to ensure dependencies are running")?;

    info!("Dependencies initialized successfully");
    info!("reev-agent: {}", urls.reev_agent);
    info!("surfpool: {}", urls.surfpool_rpc);

    let guard = DependencyManagerGuard { manager };
    Ok((guard, urls))
}

/// Runs all benchmarks found at the given path and returns the results.
pub async fn run_benchmarks(path: PathBuf, agent_name: &str) -> Result<Vec<TestResult>> {
    let benchmark_paths = discover_benchmarks(&path)?;
    if benchmark_paths.is_empty() {
        return Ok(vec![]);
    }

    // Initialize dependency management system
    let _dependency_guard = init_dependencies()
        .await
        .context("Failed to initialize dependencies")?;

    let db = db::Db::new("db/reev_results.db").await?;
    let mut results = vec![];

    for path in benchmark_paths {
        info!(path = %path.display(), "Running benchmark");
        let f = fs::File::open(&path)?;
        let test_case: TestCase = serde_yaml::from_reader(f)?;
        info!(id = %test_case.id, "Loaded test case");

        let mut agent = LlmAgent::new(agent_name)?;
        let mut env = SolanaEnv::new().context("Failed to create Solana environment")?;

        let options = serde_json::to_value(&test_case)
            .context("Failed to serialize test case for env options")?;
        let mut initial_observation = env.reset(None, Some(options)).await?;
        test_scenarios::setup_spl_scenario(&mut env, &test_case, &mut initial_observation)
            .await
            .context("Failed to set up SPL scenario")?;

        let (final_observation, trace, actions) =
            run_evaluation_loop(&mut env, &mut agent, &test_case, &initial_observation).await?;

        // Use the new comprehensive scoring function from reev-lib.
        // Use the new comprehensive scoring function.
        let score = calculate_final_score(
            &test_case,
            &actions,
            &initial_observation,
            &final_observation,
        );
        // A score >= 0.75 means the instruction was perfect, even if it failed on-chain.
        // This is the primary signal for agent success.
        let final_status = if score >= 0.75 {
            FinalStatus::Succeeded
        } else {
            FinalStatus::Failed
        };

        if let Some(step) = trace.steps.first() {
            // Serialize the Vec<AgentAction> to a JSON string for database storage.
            let action_json = serde_json::to_string(&step.action)
                .context("Failed to serialize action vector to JSON for DB insertion")?;
            db.insert_result(
                &test_case.id,
                &test_case.prompt,
                &action_json,
                &final_observation,
                final_status,
                score,
            )
            .await?;
        }

        let result = TestResult::new(&test_case, final_status, score, trace);
        results.push(result);

        env.close()?;
    }
    info!("All benchmarks finished.");
    Ok(results)
}

/// Discovers benchmark files from a given path.
fn discover_benchmarks(path: &Path) -> Result<Vec<PathBuf>> {
    let mut benchmark_paths = vec![];
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file()
                && (path.extension() == Some("yml".as_ref())
                    || path.extension() == Some("yaml".as_ref()))
            {
                benchmark_paths.push(path);
            }
        }
    } else if path.is_file() {
        benchmark_paths.push(path.to_path_buf());
    } else {
        return Err(anyhow!("Provided path is not a valid file or directory"));
    }

    if benchmark_paths.is_empty() {
        info!("No benchmark files found to run.");
    }

    benchmark_paths.sort();
    Ok(benchmark_paths)
}

#[instrument(skip_all, fields(benchmark_id = %test_case.id))]
async fn run_evaluation_loop(
    env: &mut SolanaEnv,
    agent: &mut (dyn Agent + Send),
    test_case: &TestCase,
    initial_observation: &AgentObservation,
) -> Result<(
    AgentObservation,
    ExecutionTrace,
    Vec<reev_lib::agent::AgentAction>,
)> {
    let mut trace = ExecutionTrace::new(test_case.prompt.clone());

    let fee_payer = env.fee_payer_placeholder();
    // The agent now returns a vector of actions.
    let actions = agent
        .get_action(
            &test_case.id,
            &test_case.prompt,
            initial_observation,
            Some(&fee_payer.to_owned()),
            Some(test_case.ground_truth.skip_instruction_validation),
        )
        .await?;

    // The environment's step function now takes a vector of actions to be bundled
    // into a single transaction.
    let step_result = env.step(actions.clone(), &test_case.ground_truth)?;

    let trace_step = reev_lib::trace::TraceStep {
        thought: None,
        action: actions.clone(),
        observation: step_result.observation.clone(),
        info: step_result.info,
    };
    trace.add_step(trace_step);
    info!("Episode finished.");
    Ok((step_result.observation, trace, actions))
}
