use reev_agent::run_server;

/// The main entry point for the mock agent server.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the logging subscriber.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,reev_lib=info,reev_agent=info".into()),
        )
        .init();

    // Run the server.
    run_server().await
}
