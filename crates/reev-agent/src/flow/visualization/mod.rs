//! # Flow Visualization Module
//!
//! This module provides utilities to convert flow execution logs into Mermaid state diagrams
//! for visualizing agent decision flows and tool execution patterns.

pub mod log_parser;
pub mod mermaid_generator;

pub use log_parser::FlowLogParser;
pub use mermaid_generator::MermaidStateDiagramGenerator;

/// Re-export main functionality
pub fn generate_mermaid_diagram(log_content: &str) -> Result<String, Box<dyn std::error::Error>> {
    let parser = FlowLogParser::new();
    let generator = MermaidStateDiagramGenerator::new();

    let flow_data = parser.parse_log(log_content)?;
    generator.generate_diagram(&flow_data)
}
