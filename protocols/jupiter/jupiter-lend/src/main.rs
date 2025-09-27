use anyhow::Result;
use jupiter_lend::run_server;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the tracing subscriber for structured logging.
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "jupiter_lend=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // The server logic is defined in the library part of the crate (`src/lib.rs`).
    // This architecture allows the server to be run as a standalone binary from this `main.rs` file,
    // and also to be imported and run from other crates, which is necessary for the examples you requested.
    if let Err(e) = run_server().await {
        // Using eprintln and a non-zero exit code for fatal errors is a standard practice for command-line applications.
        eprintln!("[jupiter-lend] Server failed to run: {e}");
        std::process::exit(1);
    }

    Ok(())
}
