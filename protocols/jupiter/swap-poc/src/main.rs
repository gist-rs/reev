use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::info;

mod common;
mod fast_check;
mod full_simulation;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Runs a fast pre-flight check against the Jupiter API. It fetches a swap
    /// transaction but does not sign or send it, stopping at what would be a
    /// signature verification failure.
    FastCheck,
    /// Runs a full end-to-end swap on a local surfpool (mainnet fork) validator.
    /// This includes wallet funding, local signing, and transaction execution.
    FullSimulation,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Setup logging
    tracing_subscriber::fmt()
        .with_env_filter("info,swap_poc=info")
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::FastCheck => {
            info!("--- Running Jupiter Swap: Fast Pre-flight Check ---");
            fast_check::run_fast_check().await?;
            info!("--- Fast Pre-flight Check Complete ---");
        }
        Commands::FullSimulation => {
            info!("--- Running Jupiter Swap: Full End-to-End Simulation ---");
            info!("Make sure you are running a local surfpool validator.");
            full_simulation::execute_jupiter_swap().await?;
            info!("--- Full End-to-End Simulation Complete ---");
        }
    }

    Ok(())
}
