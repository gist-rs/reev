//! Tests for database error module

use reev_db::{error::ErrorSeverity, DatabaseError};

#[test]
fn test_error_creation() {
    let err = DatabaseError::validation("field_name", "Invalid value");
    assert!(matches!(err, DatabaseError::ValidationError { .. }));

    let err = DatabaseError::duplicate_detected("test-id", 3);
    assert!(matches!(err, DatabaseError::DuplicateDetected { .. }));
}

#[test]
fn test_error_severity() {
    let validation_err = DatabaseError::validation("test", "message");
    assert_eq!(validation_err.severity(), ErrorSeverity::Warning);

    let connection_err = DatabaseError::connection("Failed to connect");
    assert_eq!(connection_err.severity(), ErrorSeverity::Error);

    let integrity_err = DatabaseError::integrity_violation("unique constraint");
    assert_eq!(integrity_err.severity(), ErrorSeverity::Critical);
}

#[test]
fn test_retryable_errors() {
    let connection_err = DatabaseError::connection("Failed to connect");
    assert!(connection_err.is_retryable());

    let timeout_err = DatabaseError::timeout(30);
    assert!(timeout_err.is_retryable());

    let validation_err = DatabaseError::validation("test", "message");
    assert!(!validation_err.is_retryable());
}

#[test]
fn test_user_message() {
    let err = DatabaseError::duplicate_detected("test-id", 5);
    let message = err.user_message();
    assert!(message.contains("test-id"));
    assert!(message.contains("5"));
}

#[test]
fn test_error_conversions() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    let db_err = DatabaseError::from(io_err);
    assert!(matches!(db_err, DatabaseError::FilesystemError { .. }));
}
