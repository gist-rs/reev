use thiserror::Error;

/// Errors that can occur during flow logging operations
#[derive(Debug, Error)]
pub enum FlowError {
    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// File system error
    #[error("File error: {0}")]
    FileError(String),

    /// Invalid flow data
    #[error("Invalid flow data: {0}")]
    InvalidData(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// I/O error
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// YAML error
    #[error("YAML error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    /// Database error
    #[error("Database error: {0}")]
    DatabaseError(String),
}

impl FlowError {
    /// Create a new serialization error
    pub fn serialization(msg: impl Into<String>) -> Self {
        Self::SerializationError(msg.into())
    }

    /// Create a new file error
    pub fn file(msg: impl Into<String>) -> Self {
        Self::FileError(msg.into())
    }

    /// Create a new invalid data error
    pub fn invalid_data(msg: impl Into<String>) -> Self {
        Self::InvalidData(msg.into())
    }

    /// Create a new configuration error
    pub fn config(msg: impl Into<String>) -> Self {
        Self::ConfigError(msg.into())
    }

    /// Create a new database error
    pub fn database(msg: impl Into<String>) -> Self {
        Self::DatabaseError(msg.into())
    }
}

/// Result type for flow operations
pub type FlowResult<T> = Result<T, FlowError>;
