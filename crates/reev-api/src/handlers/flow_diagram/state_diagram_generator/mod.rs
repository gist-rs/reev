//! StateDiagram Generator Modules
//!
//! This module contains all the sub-modules for generating Mermaid stateDiagram visualizations.
//! The main StateDiagramGenerator struct implementation is split into focused modules.

// Import all sub-modules
pub mod amount_utils;
pub mod extract_amount_from_params;
pub mod extract_tool_details;
pub mod extract_transfer_details;
pub mod generate_diagram;
pub mod generate_dynamic_flow_diagram;
pub mod generate_enhanced_step_note;
pub mod generate_html;
pub mod generate_simple_diagram;
pub mod string_utils;
pub mod summarize_params;
pub mod summarize_result_data;
pub mod token_utils;

// Re-export all functions for external access
pub use generate_diagram::generate_diagram;
pub use generate_dynamic_flow_diagram::generate_dynamic_flow_diagram;
pub use generate_enhanced_step_note::generate_enhanced_step_note;
pub use generate_html::generate_html;
pub use generate_simple_diagram::generate_simple_diagram;

// Re-export utility functions for internal use
pub use amount_utils::{extract_amount_from_tool_args, lamports_to_sol};
pub use extract_amount_from_params::extract_amount_from_params;
pub use extract_tool_details::extract_tool_details;
pub use extract_transfer_details::extract_transfer_details;
pub use string_utils::sanitize_tool_name;
pub use summarize_params::summarize_params;
pub use summarize_result_data::summarize_result_data;
pub use token_utils::{lamports_to_token_amount, mint_to_symbol};

/// StateDiagram generator for creating Mermaid stateDiagram visualizations
pub struct StateDiagramGenerator;

impl StateDiagramGenerator {
    /// Generate a Mermaid stateDiagram from parsed session data
    pub fn generate_diagram(
        session: &crate::handlers::flow_diagram::ParsedSession,
    ) -> Result<
        crate::handlers::flow_diagram::FlowDiagram,
        crate::handlers::flow_diagram::FlowDiagramError,
    > {
        generate_diagram(session)
    }

    /// Generate a Mermaid stateDiagram for dynamic flow execution
    pub fn generate_dynamic_flow_diagram(
        session: &crate::handlers::flow_diagram::ParsedSession,
        session_id: &str,
    ) -> crate::handlers::flow_diagram::FlowDiagram {
        generate_dynamic_flow_diagram(session, session_id)
    }

    /// Generate a simple Mermaid stateDiagram from parsed session data
    pub fn generate_simple_diagram(
        session: &crate::handlers::flow_diagram::ParsedSession,
    ) -> crate::handlers::flow_diagram::FlowDiagram {
        generate_simple_diagram(session)
    }

    /// Generate HTML output with embedded Mermaid diagram
    pub fn generate_html(diagram: &crate::handlers::flow_diagram::FlowDiagram) -> String {
        generate_html(diagram)
    }
}
