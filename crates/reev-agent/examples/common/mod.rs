use clap::Parser;

/// A common CLI structure for the reev-agent examples.
#[derive(Parser, Debug)]
pub struct Cli {
    /// The agent to use for the API call.
    /// Can be 'deterministic', 'local', or a specific model name (e.g., 'glm-4.6').
    #[arg(long, default_value = "deterministic")]
    pub agent: String,
}

/// Parses the CLI arguments and returns the selected agent name.
#[allow(unused)]
pub fn get_agent_name() -> String {
    let cli = Cli::parse();
    cli.agent
}

pub mod helpers;
