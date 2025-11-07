//! Generate Simple Diagram Module
//!
//! This module provides the function for generating simple Mermaid stateDiagram visualizations.

use crate::handlers::flow_diagram::{FlowDiagram, ParsedSession};

/// Generate a simple Mermaid stateDiagram from parsed session data
pub fn generate_simple_diagram(session: &ParsedSession) -> FlowDiagram {
    let mut diagram_lines = Vec::new();

    diagram_lines.push("stateDiagram".to_string());
    diagram_lines.push("    [*] --> Prompt".to_string());

    if let Some(prompt) = &session.prompt {
        let escaped_prompt = prompt.replace('"', "&quot;");
        let truncated_prompt = if escaped_prompt.len() > 100 {
            format!("{}...", &escaped_prompt[..97])
        } else {
            escaped_prompt
        };
        diagram_lines.push(format!("    Prompt --> Agent : {truncated_prompt}"));
    } else {
        diagram_lines.push("    Prompt --> Agent : Execute task".to_string());
    }

    diagram_lines.push("    Agent --> [*]".to_string());

    let diagram = diagram_lines.join("\n");

    let metadata =
        crate::handlers::flow_diagram::session_parser::SessionParser::create_metadata(session);

    FlowDiagram {
        diagram,
        metadata,
        tool_calls: session.tool_calls.clone(),
    }
}
