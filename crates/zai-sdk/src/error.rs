//! Error types for ZAI SDK

use thiserror::Error;

/// Result type alias for ZAI SDK operations
pub type ZaiResult<T> = Result<T, ZaiError>;

/// Errors that can occur when using the ZAI SDK
#[derive(Debug, Error)]
pub enum ZaiError {
    /// Authentication error
    #[error("Authentication failed: {message}")]
    Authentication { message: String },

    /// API request error
    #[error("API request failed: {message}")]
    ApiRequest { message: String },

    /// Network error
    #[error("Network error: {source}")]
    Network {
        #[from]
        source: reqwest::Error,
    },

    /// Serialization/deserialization error
    #[error("JSON error: {source}")]
    Json {
        #[from]
        source: serde_json::Error,
    },

    /// Invalid response from the API
    #[error("Invalid API response: {message}")]
    InvalidResponse { message: String },

    /// Request timeout
    #[error("Request timed out after {seconds} seconds")]
    Timeout { seconds: u64 },

    /// Model not found or not available
    #[error("Model '{model}' is not available")]
    ModelUnavailable { model: String },

    /// Rate limit exceeded
    #[error("Rate limit exceeded, retry after {seconds} seconds")]
    RateLimit { seconds: u64 },

    /// Invalid request parameters
    #[error("Invalid request parameters: {message}")]
    InvalidParameters { message: String },

    /// Configuration error
    #[error("Configuration error: {message}")]
    Configuration { message: String },

    /// Other error
    #[error("Error: {message}")]
    Other { message: String },
}

impl ZaiError {
    /// Create a new authentication error
    pub fn authentication(message: impl Into<String>) -> Self {
        Self::Authentication {
            message: message.into(),
        }
    }

    /// Create a new API request error
    pub fn api_request(message: impl Into<String>) -> Self {
        Self::ApiRequest {
            message: message.into(),
        }
    }

    /// Create a new invalid response error
    pub fn invalid_response(message: impl Into<String>) -> Self {
        Self::InvalidResponse {
            message: message.into(),
        }
    }

    /// Create a new timeout error
    pub fn timeout(seconds: u64) -> Self {
        Self::Timeout { seconds }
    }

    /// Create a new model unavailable error
    pub fn model_unavailable(model: impl Into<String>) -> Self {
        Self::ModelUnavailable {
            model: model.into(),
        }
    }

    /// Create a new rate limit error
    pub fn rate_limit(seconds: u64) -> Self {
        Self::RateLimit { seconds }
    }

    /// Create a new invalid parameters error
    pub fn invalid_parameters(message: impl Into<String>) -> Self {
        Self::InvalidParameters {
            message: message.into(),
        }
    }

    /// Create a new configuration error
    pub fn configuration(message: impl Into<String>) -> Self {
        Self::Configuration {
            message: message.into(),
        }
    }

    /// Create a new other error
    pub fn other(message: impl Into<String>) -> Self {
        Self::Other {
            message: message.into(),
        }
    }
}

/// Convert HTTP status codes to ZaiError
impl From<reqwest::Response> for ZaiError {
    fn from(response: reqwest::Response) -> Self {
        match response.status().as_u16() {
            401 => Self::authentication("Invalid API key"),
            403 => Self::authentication("Access forbidden"),
            404 => Self::model_unavailable("Model not found"),
            429 => Self::rate_limit(60), // Default to 60 seconds
            500..=599 => Self::api_request("Server error"),
            _ => Self::api_request(format!("HTTP error: {}", response.status())),
        }
    }
}

/// Convert HTTP errors with status codes to ZaiError
pub fn http_error_to_zai(error: reqwest::Error) -> ZaiError {
    if error.is_timeout() {
        ZaiError::timeout(30)
    } else if error.is_connect() {
        ZaiError::Network { source: error }
    } else if error.is_request() {
        ZaiError::api_request(format!("Request error: {error}"))
    } else {
        ZaiError::Network { source: error }
    }
}

/// Error that can occur when parsing a response
#[derive(Debug, Error)]
pub enum ResponseError {
    /// Missing expected field in response
    #[error("Missing field '{field}' in response")]
    MissingField { field: String },

    /// Invalid field value
    #[error("Invalid value for field '{field}': {reason}")]
    InvalidFieldValue { field: String, reason: String },

    /// Unexpected response format
    #[error("Unexpected response format: {reason}")]
    UnexpectedFormat { reason: String },
}

impl ResponseError {
    /// Create a new missing field error
    pub fn missing_field(field: impl Into<String>) -> Self {
        Self::MissingField {
            field: field.into(),
        }
    }

    /// Create a new invalid field value error
    pub fn invalid_field_value(field: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidFieldValue {
            field: field.into(),
            reason: reason.into(),
        }
    }

    /// Create a new unexpected format error
    pub fn unexpected_format(reason: impl Into<String>) -> Self {
        Self::UnexpectedFormat {
            reason: reason.into(),
        }
    }
}

/// Convert ResponseError to ZaiError
impl From<ResponseError> for ZaiError {
    fn from(error: ResponseError) -> Self {
        Self::InvalidResponse {
            message: error.to_string(),
        }
    }
}
