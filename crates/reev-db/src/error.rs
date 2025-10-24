//! Database error handling module for reev-db
//!
//! Provides comprehensive error types and handling for database operations
//! with detailed context and helpful error messages.

use thiserror::Error;

/// Result type for database operations
pub type Result<T> = std::result::Result<T, DatabaseError>;

/// Comprehensive database error types
#[derive(Error, Debug)]
pub enum DatabaseError {
    /// Configuration-related errors
    #[error("Database configuration error: {message}")]
    Configuration {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Connection-related errors
    #[error("Database connection failed: {message}")]
    ConnectionError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Query execution errors
    #[error("Query execution failed: {query}")]
    QueryError {
        query: String,
        #[source]
        source: turso::Error,
    },

    /// Schema-related errors
    #[error("Schema error: {message}")]
    SchemaError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Data validation errors
    #[error("Data validation error: {field} - {message}")]
    ValidationError { field: String, message: String },

    /// Duplicate record detected
    #[error("Duplicate record detected: {id} appears {count} times")]
    DuplicateDetected { id: String, count: i64 },

    /// Record not found
    #[error("Record not found: {id} in table {table}")]
    RecordNotFound { id: String, table: String },

    /// Integrity constraint violation
    #[error("Integrity constraint violation: {constraint}")]
    IntegrityViolation {
        constraint: String,
        #[source]
        source: Option<turso::Error>,
    },

    /// Transaction-related errors
    #[error("Transaction error: {message}")]
    TransactionError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// General operation errors
    #[error("Operation failed: {message}")]
    OperationError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Filesystem I/O errors
    #[error("Filesystem error: {path}")]
    FilesystemError {
        path: String,
        #[source]
        source: std::io::Error,
    },

    /// Serialization/deserialization errors
    #[error("Serialization error: {message}")]
    SerializationError {
        message: String,
        #[source]
        source: serde_json::Error,
    },

    /// YAML parsing errors
    #[error("YAML parsing error: {message}")]
    YamlError {
        message: String,
        #[source]
        source: serde_yaml::Error,
    },

    /// MD5 calculation errors
    #[error("Hash calculation error: {message}")]
    HashError { message: String },

    /// Timeout errors
    #[error("Operation timed out after {seconds}s")]
    Timeout { seconds: u64 },

    /// Retry limit exceeded
    #[error("Retry limit exceeded after {attempts} attempts")]
    RetryLimitExceeded { attempts: u32 },

    /// Generic database errors
    #[error("Database error: {message}")]
    Generic {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl DatabaseError {
    /// Create a new configuration error
    pub fn configuration<S: Into<String>>(message: S) -> Self {
        Self::Configuration {
            message: message.into(),
            source: None,
        }
    }

    /// Create a new configuration error with source
    pub fn configuration_with_source<
        S: Into<String>,
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    >(
        message: S,
        source: E,
    ) -> Self {
        Self::Configuration {
            message: message.into(),
            source: Some(source.into()),
        }
    }

    /// Create a new connection error
    pub fn connection<S: Into<String>>(message: S) -> Self {
        Self::ConnectionError {
            message: message.into(),
            source: None,
        }
    }

    /// Create a new connection error with source
    pub fn connection_with_source<
        S: Into<String>,
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    >(
        message: S,
        source: E,
    ) -> Self {
        Self::ConnectionError {
            message: message.into(),
            source: Some(source.into()),
        }
    }

    /// Create a new query error
    pub fn query<S: Into<String>>(query: S, source: turso::Error) -> Self {
        Self::QueryError {
            query: query.into(),
            source,
        }
    }

    /// Create a new schema error
    pub fn schema<S: Into<String>>(message: S) -> Self {
        Self::SchemaError {
            message: message.into(),
            source: None,
        }
    }

    /// Create a new schema error with source
    pub fn schema_with_source<
        S: Into<String>,
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    >(
        message: S,
        source: E,
    ) -> Self {
        Self::SchemaError {
            message: message.into(),
            source: Some(source.into()),
        }
    }

    /// Create a new validation error
    pub fn validation<F: Into<String>, M: Into<String>>(field: F, message: M) -> Self {
        Self::ValidationError {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Create a new duplicate detected error
    pub fn duplicate_detected<S: Into<String>>(id: S, count: i64) -> Self {
        Self::DuplicateDetected {
            id: id.into(),
            count,
        }
    }

    /// Create a new record not found error
    pub fn record_not_found<I: Into<String>, T: Into<String>>(id: I, table: T) -> Self {
        Self::RecordNotFound {
            id: id.into(),
            table: table.into(),
        }
    }

    /// Create a new integrity violation error
    pub fn integrity_violation<S: Into<String>>(constraint: S) -> Self {
        Self::IntegrityViolation {
            constraint: constraint.into(),
            source: None,
        }
    }

    /// Create a new transaction error
    pub fn transaction<S: Into<String>>(message: S) -> Self {
        Self::TransactionError {
            message: message.into(),
            source: None,
        }
    }

    /// Create a new filesystem error
    pub fn filesystem<P: Into<String>>(path: P, source: std::io::Error) -> Self {
        Self::FilesystemError {
            path: path.into(),
            source,
        }
    }

    /// Create a new serialization error
    pub fn serialization<S: Into<String>>(message: S, source: serde_json::Error) -> Self {
        Self::SerializationError {
            message: message.into(),
            source,
        }
    }

    /// Create a new YAML error
    pub fn yaml<S: Into<String>>(message: S, source: serde_yaml::Error) -> Self {
        Self::YamlError {
            message: message.into(),
            source,
        }
    }

    /// Create a new hash error
    pub fn hash<S: Into<String>>(message: S) -> Self {
        Self::HashError {
            message: message.into(),
        }
    }

    /// Create a new timeout error
    pub fn timeout(seconds: u64) -> Self {
        Self::Timeout { seconds }
    }

    /// Create a new retry limit exceeded error
    pub fn retry_limit_exceeded(attempts: u32) -> Self {
        Self::RetryLimitExceeded { attempts }
    }

    /// Create a new generic error
    pub fn generic<S: Into<String>>(message: S) -> Self {
        Self::Generic {
            message: message.into(),
            source: None,
        }
    }

    /// Create a new generic error with source
    pub fn generic_with_source<
        S: Into<String>,
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    >(
        message: S,
        source: E,
    ) -> Self {
        Self::Generic {
            message: message.into(),
            source: Some(source.into()),
        }
    }

    /// Create a new filesystem error with source
    pub fn filesystem_with_source<
        S: Into<String>,
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    >(
        message: S,
        _source: E,
    ) -> Self {
        Self::FilesystemError {
            path: "unknown".to_string(),
            source: std::io::Error::other(message.into()),
        }
    }

    /// Create a new YAML error with source
    pub fn yaml_with_source<S: Into<String>, E: Into<serde_yaml::Error>>(
        message: S,
        source: E,
    ) -> Self {
        Self::YamlError {
            message: message.into(),
            source: source.into(),
        }
    }

    /// Create a new operation error
    pub fn operation<S: Into<String>>(message: S) -> Self {
        Self::OperationError {
            message: message.into(),
            source: None,
        }
    }

    /// Create a new operation error with source
    pub fn operation_with_source<
        S: Into<String>,
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    >(
        message: S,
        source: E,
    ) -> Self {
        Self::OperationError {
            message: message.into(),
            source: Some(source.into()),
        }
    }

    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::ConnectionError { .. } => true,
            Self::Timeout { .. } => true,
            Self::RetryLimitExceeded { .. } => false,
            Self::ValidationError { .. } => false,
            Self::DuplicateDetected { .. } => false,
            Self::IntegrityViolation { .. } => false,
            _ => false,
        }
    }

    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::ValidationError { .. } => ErrorSeverity::Warning,
            Self::DuplicateDetected { .. } => ErrorSeverity::Warning,
            Self::RecordNotFound { .. } => ErrorSeverity::Info,
            Self::ConnectionError { .. } => ErrorSeverity::Error,
            Self::Timeout { .. } => ErrorSeverity::Error,
            Self::IntegrityViolation { .. } => ErrorSeverity::Critical,
            Self::TransactionError { .. } => ErrorSeverity::Critical,
            _ => ErrorSeverity::Error,
        }
    }

    /// Get user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            Self::ValidationError { field, message } => {
                format!("Validation failed for {field}: {message}")
            }
            Self::DuplicateDetected { id, count } => {
                format!("Duplicate entry found: '{id}' appears {count} times")
            }
            Self::RecordNotFound { id, table } => {
                format!("Could not find '{id}' in {table}")
            }
            Self::ConnectionError { message, .. } => {
                format!("Database connection failed: {message}")
            }
            Self::Timeout { seconds } => {
                format!("Operation timed out after {seconds} seconds")
            }
            Self::RetryLimitExceeded { attempts } => {
                format!("Operation failed after {attempts} retry attempts")
            }
            _ => self.to_string(),
        }
    }
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    /// Informational message
    Info,
    /// Warning message
    Warning,
    /// Error message
    Error,
    /// Critical error
    Critical,
}

impl ErrorSeverity {
    /// Get the log level for this severity
    pub fn log_level(&self) -> tracing::Level {
        match self {
            Self::Info => tracing::Level::INFO,
            Self::Warning => tracing::Level::WARN,
            Self::Error => tracing::Level::ERROR,
            Self::Critical => tracing::Level::ERROR,
        }
    }
}

/// Conversion from turso::Error
impl From<turso::Error> for DatabaseError {
    fn from(err: turso::Error) -> Self {
        Self::Generic {
            message: "Turso database error".to_string(),
            source: Some(Box::new(err)),
        }
    }
}

/// Conversion from std::io::Error
impl From<std::io::Error> for DatabaseError {
    fn from(err: std::io::Error) -> Self {
        Self::FilesystemError {
            path: "unknown".to_string(),
            source: err,
        }
    }
}

/// Conversion from serde_json::Error
impl From<serde_json::Error> for DatabaseError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerializationError {
            message: "JSON serialization failed".to_string(),
            source: err,
        }
    }
}

/// Conversion from serde_yaml::Error
impl From<serde_yaml::Error> for DatabaseError {
    fn from(err: serde_yaml::Error) -> Self {
        Self::YamlError {
            message: "YAML parsing failed".to_string(),
            source: err,
        }
    }
}
