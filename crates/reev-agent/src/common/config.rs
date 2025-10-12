//! Common configuration values shared across the reev-agent crate

/// Network configuration
pub mod network {
    /// Default host for reev-agent
    pub const DEFAULT_HOST: &str = "127.0.0.1";

    /// Default port for reev-agent
    pub const DEFAULT_PORT: u16 = 9090;

    /// Default port for surfpool
    pub const SURFPOOL_PORT: u16 = 8899;
}

/// API endpoints
pub mod endpoints {
    /// Health check endpoint
    pub const HEALTH: &str = "/health";

    /// Transaction generation endpoint
    pub const GENERATE_TX: &str = "/gen/tx";
}

/// Default timeout values in seconds
pub mod timeouts {
    /// Default HTTP request timeout
    pub const HTTP_REQUEST: u64 = 30;

    /// Health check timeout
    pub const HEALTH_CHECK: u64 = 5;
}

/// Agent configuration values
pub mod agents {
    /// Mock parameter for deterministic agent
    pub const MOCK_PARAM: &str = "mock=true";

    /// Default agent name
    pub const DEFAULT_AGENT: &str = "deterministic";
}
