//! # Reev Flow
//!
//! Shared flow types and utilities for the reev ecosystem.
//!
//! This crate provides core types for tracking and analyzing agent execution flows,
//! designed to be:
//! 1. Database-friendly when database feature is enabled
//! 2. Generic enough for different use cases
//! 3. Easily convertible to/from domain-specific types
//! 4. Serializable and deserializable for storage and API communication

pub mod types;
pub mod utils;

pub use types::*;
pub use utils::*;

#[cfg(feature = "database")]
pub mod database;
