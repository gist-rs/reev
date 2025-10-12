//! # Reev TUI Library
//!
//! This library provides the terminal user interface for the reev framework,
//! allowing users to interact with benchmarks and agents through a text-based UI.

pub mod app;
pub mod event;
pub mod tui;
pub mod ui;

// Re-export the main app types for external use
pub use app::{ActivePanel, App, BenchmarkStatus, SelectedAgent, TuiEvent};
