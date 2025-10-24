use reev_agent::run_server;

/// The main entry point for the mock agent server.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the logging subscriber only if not already set.
    // This prevents the "global default trace dispatcher has already been set" error
    // when the agent is started from the API server which already initialized tracing.
    // Use try_init() which returns an error if already set, but we ignore it.
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,reev_lib=info,reev_agent=info".into()),
        )
        .try_init();

    // Enhanced OpenTelemetry logging will be initialized in run_agent() with session_id

    // Run the server.
    run_server().await
}
