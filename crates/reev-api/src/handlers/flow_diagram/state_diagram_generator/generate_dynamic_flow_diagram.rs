//! Generate Dynamic Flow Diagram Module
//!
//! This module provides the function for generating Mermaid stateDiagram visualizations for dynamic flows.

use crate::handlers::flow_diagram::{FlowDiagram, ParsedSession};

use super::{
    extract_amount_from_params, extract_transfer_details, generate_enhanced_step_note,
    sanitize_tool_name, summarize_params, summarize_result_data,
};

/// Generate a Mermaid stateDiagram for dynamic flow execution
pub fn generate_dynamic_flow_diagram(session: &ParsedSession, session_id: &str) -> FlowDiagram {
    let mut diagram_lines = Vec::new();

    // Start with stateDiagram declaration
    diagram_lines.push("stateDiagram".to_string());

    // Add initial transition from [*] to DynamicFlow
    diagram_lines.push("    [*] --> DynamicFlow".to_string());

    // Determine flow type from session_id
    let flow_type = if session_id.starts_with("direct-") {
        "Direct Mode (Zero File I/O)"
    } else if session_id.starts_with("bridge-") {
        "Bridge Mode (Temporary YML)"
    } else if session_id.starts_with("recovery-") {
        "Recovery Mode (Resilient Execution)"
    } else if session_id.starts_with("enhanced-300") {
        "Enhanced 300-Series Flow"
    } else {
        "Dynamic Flow"
    };

    // Add DynamicFlow to Orchestrator transition
    diagram_lines.push(format!("    DynamicFlow --> Orchestrator : {flow_type}"));

    // Add prompt information if available
    let _previous_state = if let Some(prompt) = &session.prompt {
        let escaped_prompt = prompt.replace('"', "&quot;");
        let truncated_prompt = if escaped_prompt.len() > 80 {
            format!("{}...", &escaped_prompt[..77])
        } else {
            escaped_prompt
        };
        diagram_lines.push(format!(
            "    Orchestrator --> ContextResolution : {truncated_prompt}"
        ));
        "ContextResolution".to_string()
    } else {
        diagram_lines.push(
            "    Orchestrator --> ContextResolution : Resolve wallet and price context".to_string(),
        );
        "ContextResolution".to_string()
    };

    // Add context resolution steps
    diagram_lines
        .push("    ContextResolution --> FlowPlanning : Generate dynamic flow plan".to_string());

    // Add flow planning step
    diagram_lines
        .push("    FlowPlanning --> AgentExecution : Execute with selected agent".to_string());

    // If there are tool calls, show them with enhanced details
    if !session.tool_calls.is_empty() {
        let mut tool_previous = "AgentExecution".to_string();

        for (index, tool_call) in session.tool_calls.iter().enumerate() {
            let tool_state = sanitize_tool_name(&tool_call.tool_name);

            // Enhanced transition label: try result_data first, then params
            let transition_label = if let Some(result_data) = &tool_call.result_data {
                summarize_result_data(result_data)
                    .unwrap_or_else(|| summarize_params(&tool_call.params))
            } else if tool_call.tool_name.contains("transfer") {
                extract_amount_from_params(tool_call)
                    .unwrap_or_else(|| summarize_params(&tool_call.params))
            } else {
                summarize_params(&tool_call.params)
            };

            diagram_lines.push(format!(
                "    {tool_previous} --> {tool_state} : {transition_label}"
            ));

            // Add detailed notes for enhanced 300-series flows and consolidated sessions
            if session_id.starts_with("enhanced-300") || session_id.contains("consolidated") {
                let detailed_note = generate_enhanced_step_note(tool_call, index);
                diagram_lines.push(format!("    note right of {tool_state} : {detailed_note}"));
            }

            // Add nested state for transfer operations
            if tool_call.tool_name.contains("transfer") {
                diagram_lines.push(format!("    state {tool_state} {{"));
                if let Some((from, to, amount)) = extract_transfer_details(tool_call) {
                    diagram_lines.push(format!("        {from} --> {to} : {amount}"));
                }
                diagram_lines.push("    }".to_string());
            }

            tool_previous = tool_state;
        }

        // Add completion from last tool
        diagram_lines.push(format!("    {tool_previous} --> [*]"));
    } else {
        // No tool calls, complete after AgentExecution
        diagram_lines.push("    AgentExecution --> [*]".to_string());
    }

    // Add enhanced CSS classes for dynamic flows
    diagram_lines.push("".to_string());
    diagram_lines.push("classDef dynamic fill:#e1f5fe".to_string());
    diagram_lines.push("classDef orchestration fill:#f3e5f5".to_string());
    diagram_lines.push("classDef execution fill:#e8f5e8".to_string());
    diagram_lines.push("classDef tools fill:grey".to_string());
    diagram_lines.push("classDef enhanced fill:#fff3e0".to_string());

    // Apply classes
    diagram_lines.push("class DynamicFlow,ContextResolution,FlowPlanning dynamic".to_string());
    diagram_lines.push("class Orchestrator orchestration".to_string());
    diagram_lines.push("class AgentExecution execution".to_string());

    for tool_call in &session.tool_calls {
        let tool_state = sanitize_tool_name(&tool_call.tool_name);
        let class = if session_id.starts_with("enhanced-300") || session_id.contains("consolidated")
        {
            "enhanced"
        } else {
            "tools"
        };
        diagram_lines.push(format!("class {tool_state} {class}"));
    }

    // Join all lines with newlines
    let diagram = diagram_lines.join("\n");

    // Create enhanced metadata for dynamic flows
    let execution_time_ms = session.tool_calls.iter().map(|t| t.duration_ms).sum();
    let flow_type = if session_id.starts_with("direct-") {
        "direct"
    } else if session_id.starts_with("bridge-") {
        "bridge"
    } else if session_id.starts_with("recovery-") {
        "recovery"
    } else if session_id.starts_with("enhanced-300") {
        "enhanced-300"
    } else if session_id.contains("consolidated") {
        "consolidated"
    } else {
        "unknown"
    };

    // Use standard DiagramMetadata structure
    let metadata = crate::handlers::flow_diagram::DiagramMetadata {
        state_count: 4 + session.tool_calls.len(), // DynamicFlow + Orchestrator + ContextResolution + FlowPlanning + AgentExecution + Tools + [*]
        tool_count: session.tool_calls.len(),
        execution_time_ms,
        benchmark_id: format!("dynamic-flow-{flow_type}"),
        session_id: Some(session_id.to_string()),
    };

    FlowDiagram {
        diagram,
        metadata,
        tool_calls: session.tool_calls.clone(),
    }
}
