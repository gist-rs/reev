use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::info;

mod common;
mod full_simulation_deposit;
mod full_simulation_withdraw;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Runs a full simulation for Jupiter Lend Deposit. This includes wallet funding,
    /// local signing, and transaction execution.
    Deposit,
    /// Runs a full simulation for Jupiter Lend Withdraw. This includes wallet funding,
    /// local signing, and transaction execution.
    Withdraw,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Setup logging
    tracing_subscriber::fmt()
        .with_env_filter("info,lend_poc=info")
        .init();

    let cli = Cli::parse();

    match cli.command {
            Commands::Deposit => {
                info!("--- Running Jupiter Lend: Deposit Simulation ---");
                info!("Make sure you are running a local surfpool validator.");
                full_simulation_deposit::execute_lend_deposit().await?;
                info!("--- Deposit Simulation Complete ---");
            }
            Commands::Withdraw => {
                info!("--- Running Jupiter Lend: Withdraw Simulation ---");
                info!("Make sure you are running a local surfpool validator.");
                full_simulation_withdraw::execute_lend_withdraw().await?;
                info!("--- Withdraw Simulation Complete ---");
            }
        }

    Ok(())
}
