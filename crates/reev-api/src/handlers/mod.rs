//! Handlers module - organized by functionality
//! Handler modules

pub mod agents;
pub mod benchmarks;
pub mod consolidation;
pub mod dynamic_flows;
pub mod execution_logs;
pub mod flow_diagram;
pub mod flow_logs;
pub mod flows;
pub mod health;
pub mod parsers;
pub mod transaction_logs;
pub mod yml;

// Re-export all handlers for easier importing
pub use agents::*;
pub use benchmarks::*;
pub use consolidation::*;
pub use dynamic_flows::*;
pub use execution_logs::*;
pub use flow_logs::*;
pub use flows::*;
pub use health::*;
// pub use parsers::*;  // Parser is only used internally by execution_logs
pub use transaction_logs::*;
pub use yml::*;
