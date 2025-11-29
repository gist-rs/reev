//! Enhanced RigAgent Features for Issue #105
//!
//! This module implements enhanced features for RigAgent to improve
//! context passing between operations, enhance prompt engineering
//! for complex scenarios, and add comprehensive tool execution validation.

pub mod constraints;
pub mod dynamic_context;
pub mod operation_history;

// Re-export commonly used types
pub use constraints::{
    ConstraintBuilder, ConstraintType, ParameterValidator, StepConstraint, ValidationReport,
};

pub use dynamic_context::{ContextPromptBuilder, ContextUpdateResult, DynamicContextUpdater};

pub use operation_history::{BalanceCalculator, OperationHistory, OperationHistoryBuilder};
