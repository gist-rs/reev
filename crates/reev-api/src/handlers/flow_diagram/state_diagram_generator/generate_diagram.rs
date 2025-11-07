//! Generate Diagram Module
//!
//! This module provides the main function for generating Mermaid stateDiagram visualizations.

use crate::handlers::flow_diagram::{FlowDiagram, ParsedSession};
use reev_types::ToolName;

use super::{extract_tool_details, extract_transfer_details, sanitize_tool_name, summarize_params};

/// Generate a Mermaid stateDiagram from parsed session data
pub fn generate_diagram(
    session: &ParsedSession,
) -> Result<FlowDiagram, crate::handlers::flow_diagram::FlowDiagramError> {
    if session.tool_calls.is_empty() {
        return Err(crate::handlers::flow_diagram::FlowDiagramError::NoToolCalls);
    }

    let mut diagram_lines = Vec::new();

    // Start with stateDiagram declaration
    diagram_lines.push("stateDiagram".to_string());

    // Add initial transition from [*] to Prompt
    diagram_lines.push("    [*] --> Prompt".to_string());

    // Add Prompt to Agent transition with the actual prompt
    if let Some(prompt) = &session.prompt {
        // Escape quotes in prompt and limit length for readability
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

    // Add tool calls in sequence
    let mut previous_state = "Agent".to_string();

    for tool_call in session.tool_calls.iter() {
        // Use actual tool_name, sanitized for Mermaid
        let tool_state = sanitize_tool_name(&tool_call.tool_name);

        // Get tool-specific details for enhanced display
        let _tool_details = extract_tool_details(tool_call);

        // For transfer operations, show amount in transition
        let transition_label = if tool_call
            .tool_name
            .contains(ToolName::SolTransfer.to_string().as_str())
            || tool_call
                .tool_name
                .contains(ToolName::SplTransfer.to_string().as_str())
        {
            super::extract_amount_from_params(tool_call)
                .unwrap_or_else(|| summarize_params(&tool_call.params))
        } else {
            summarize_params(&tool_call.params)
        };

        // Add transition from previous state to this tool
        diagram_lines.push(format!(
            "    {previous_state} --> {tool_state} : {transition_label}"
        ));

        // Add nested state for transfer operations
        if tool_call.tool_name.contains("transfer") {
            diagram_lines.push(format!("    state {tool_state} {{"));
            if let Some((from, to, amount)) = extract_transfer_details(tool_call) {
                diagram_lines.push(format!("        {from} --> {to} : {amount}"));
            }
            diagram_lines.push("    }".to_string());
        }

        previous_state = tool_state;
    }

    // Add final transition from last tool to [*]
    diagram_lines.push(format!("    {previous_state} --> [*]"));

    // Add CSS classes for tools
    diagram_lines.push("".to_string());
    diagram_lines.push("classDef tools fill:grey".to_string());

    for tool_call in &session.tool_calls {
        let tool_state = sanitize_tool_name(&tool_call.tool_name);
        diagram_lines.push(format!("class {tool_state} tools"));
    }

    // Join all lines with newlines
    let diagram = diagram_lines.join("\n");

    // Create metadata
    let metadata =
        crate::handlers::flow_diagram::session_parser::SessionParser::create_metadata(session);

    Ok(FlowDiagram {
        diagram,
        metadata,
        tool_calls: session.tool_calls.clone(),
    })
}
