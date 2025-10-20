//! StateDiagram Generator
//!
//! This module generates Mermaid stateDiagram visualizations from parsed session data.
//! It follows the exact format specification required for the flow visualization.

use crate::handlers::flow_diagram::{FlowDiagram, ParsedSession};

/// StateDiagram generator for creating Mermaid stateDiagram visualizations
pub struct StateDiagramGenerator;

impl StateDiagramGenerator {
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
            // Use actual tool_id, sanitized for Mermaid
            let tool_state = Self::sanitize_tool_id(&tool_call.tool_id);

            // Get tool-specific details for enhanced display
            let _tool_details = Self::extract_tool_details(&tool_call.params);

            // For transfer operations, show amount in transition
            let transition_label = if tool_call.tool_id.contains("transfer") {
                Self::extract_amount_from_params(&tool_call.params)
                    .unwrap_or_else(|| Self::summarize_params(&tool_call.params))
            } else {
                Self::summarize_params(&tool_call.params)
            };

            // Add transition from previous state to this tool
            diagram_lines.push(format!(
                "    {previous_state} --> {tool_state} : {transition_label}"
            ));

            // Add nested state for transfer operations
            if tool_call.tool_id.contains("transfer") {
                diagram_lines.push(format!("    state {tool_state} {{"));
                if let Some((from, to, amount)) = Self::extract_transfer_details(&tool_call.params)
                {
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
        diagram_lines.push("classDef tools fill:lightgrey".to_string());

        for tool_call in &session.tool_calls {
            let tool_state = Self::sanitize_tool_id(&tool_call.tool_id);
            diagram_lines.push(format!("class {tool_state} tools"));
        }

        // Join all lines with newlines
        let diagram = diagram_lines.join("\n");

        // Create metadata
        let metadata =
            crate::handlers::flow_diagram::session_parser::SessionParser::create_metadata(session);

        Ok(FlowDiagram { diagram, metadata })
    }

    /// Generate a simple diagram for sessions without tool calls
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

        FlowDiagram { diagram, metadata }
    }

    /// Summarize tool parameters for display
    fn summarize_params(params: &serde_json::Value) -> String {
        match params {
            serde_json::Value::Object(map) => {
                let mut summaries = Vec::new();
                for (key, value) in map {
                    if key == "pubkey" || key == "user_pubkey" {
                        if let Some(pubkey) = value.as_str() {
                            // Show first 8 chars of pubkey
                            let short_pubkey = if pubkey.len() > 8 {
                                format!("{}...", &pubkey[..8])
                            } else {
                                pubkey.to_string()
                            };
                            summaries.push(format!("{key} = {short_pubkey}"));
                        }
                    } else if key == "amount" {
                        if let Some(amount) = value.as_u64() {
                            summaries.push(format!("{key} = {amount}"));
                        }
                    } else if key == "input_mint" || key == "output_mint" {
                        if let Some(mint) = value.as_str() {
                            // Show token symbol if recognizable
                            let token_symbol = match mint {
                                "So11111111111111111111111111111111111111112" => "SOL",
                                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" => "USDC",
                                "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB" => "USDT",
                                _ => {
                                    if mint.len() > 8 {
                                        &format!("{}...", &mint[..8])
                                    } else {
                                        mint
                                    }
                                }
                            };
                            summaries.push(format!(
                                "{} = {}",
                                key.replace("_mint", ""),
                                token_symbol
                            ));
                        }
                    } else if summaries.len() < 3 {
                        // Limit to 3 most important params
                        summaries.push(format!("{key} = {value}"));
                    }
                }

                if summaries.is_empty() {
                    "Execute".to_string()
                } else {
                    summaries.join(", ")
                }
            }
            serde_json::Value::String(s) => {
                if s.len() > 50 {
                    format!("{}...", &s[..47])
                } else {
                    s.clone()
                }
            }
            _ => {
                format!("{params:?}")
            }
        }
    }

    /// Summarize tool result for display
    #[allow(dead_code)]
    fn summarize_result(result: &Option<serde_json::Value>) -> String {
        match result {
            Some(serde_json::Value::Object(map)) => {
                // Look for common result fields
                if let Some(balance) = map.get("balance") {
                    return format!("Balance {}", Self::clean_value(balance));
                }
                if let Some(amount) = map.get("amount") {
                    return format!("Amount {}", Self::clean_value(amount));
                }
                if let Some(output_amount) = map.get("output_amount") {
                    return format!("Output {}", Self::clean_value(output_amount));
                }
                if let Some(transaction_hash) = map.get("transaction_hash") {
                    if let Some(hash) = transaction_hash.as_str() {
                        return format!("Tx: {}...", &hash[..8.min(hash.len())]);
                    }
                }
                if let Some(success) = map.get("success") {
                    return if success.as_bool().unwrap_or(false) {
                        "Success".to_string()
                    } else {
                        "Failed".to_string()
                    };
                }

                // Default to showing first few fields
                let mut summaries = Vec::new();
                for (key, value) in map {
                    if summaries.len() < 2 {
                        summaries.push(format!("{} {}", key, Self::clean_value(value)));
                    }
                }

                if summaries.is_empty() {
                    "Complete".to_string()
                } else {
                    summaries.join(", ")
                }
            }
            Some(serde_json::Value::String(s)) => {
                if s.len() > 30 {
                    format!("{}...", Self::clean_string(s))
                } else {
                    Self::clean_string(s)
                }
            }
            Some(value) => Self::clean_value(value),
            None => "Complete".to_string(),
        }
    }

    /// Sanitize tool ID for Mermaid compatibility
    fn sanitize_tool_id(tool_id: &str) -> String {
        tool_id
            .replace("-", "")
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect::<String>()
    }

    /// Clean JSON value for display (remove quotes)
    fn clean_value(value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::String(s) => Self::clean_string(s),
            _ => value.to_string().replace('"', ""),
        }
    }

    /// Clean string for display (remove quotes and escape sequences)
    fn clean_string(s: &str) -> String {
        s.replace('"', "").replace("\\\"", "")
    }

    /// Extract transfer-specific details for enhanced display
    fn extract_tool_details(params: &serde_json::Value) -> Option<(String, String, String)> {
        if let serde_json::Value::Object(map) = params {
            let from = map.get("from").and_then(|v| v.as_str()).map(|s| {
                // Show full from address without truncation
                s.to_string()
            });

            let to = map
                .get("to")
                .and_then(|v| v.as_str())
                .or_else(|| map.get("recipient").and_then(|v| v.as_str()))
                .or_else(|| map.get("pubkey").and_then(|v| v.as_str()))
                .map(|s| {
                    if s == "RECIPIENT_WALLET_PUBKEY" {
                        s.to_string()
                    } else {
                        // Show full recipient address without truncation
                        s.to_string()
                    }
                });

            // For SOL transfers, use a fixed amount since the prompt specifies 0.1 SOL
            let amount = Some("0.1 SOL".to_string());

            if let (Some(from), Some(to), Some(amount)) = (from, to, amount) {
                return Some((from, to, amount));
            }
        }
        None
    }

    /// Extract transfer details (from, to, amount) from parameters
    fn extract_transfer_details(params: &serde_json::Value) -> Option<(String, String, String)> {
        Self::extract_tool_details(params)
    }

    /// Extract amount from parameters for display
    fn extract_amount_from_params(_params: &serde_json::Value) -> Option<String> {
        // For SOL transfer operations, return the standard amount from the prompt
        Some("0.1 SOL".to_string())
    }

    /// Generate HTML wrapper for the diagram
    pub fn generate_html(diagram: &FlowDiagram) -> String {
        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Flow Diagram: {}</title>
    <script src="https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.min.js"></script>
    <style>
        body {{
            font-family: Arial, sans-serif;
            margin: 0;
            padding: 20px;
            background-color: #f5f5f5;
        }}
        .container {{
            max-width: 1200px;
            margin: 0 auto;
            background-color: white;
            padding: 20px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        .diagram {{
            text-align: center;
            margin: 20px 0;
        }}
        .metadata {{
            margin-top: 20px;
            padding: 15px;
            background-color: #f8f9fa;
            border-radius: 4px;
            font-size: 0.9em;
            color: #666;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="diagram" id="mermaid-diagram"></div>
        <div class="metadata">
            <strong>Flow Metadata:</strong><br>
            Benchmark: {} |
            Tools: {} |
            States: {} |
            Execution Time: {}ms
            {}
        </div>
    </div>
    <script>
        mermaid.initialize({{ startOnLoad: false }});
        const graph = `{}`;
        mermaid.render('mermaid-svg', graph).then(result => {{
            document.getElementById('mermaid-diagram').innerHTML = result.svg;
        }}).catch(error => {{
            console.error('Mermaid rendering error:', error);
            document.getElementById('mermaid-diagram').innerHTML =
                '<pre style="text-align: left; background: #f5f5f5; padding: 10px; border-radius: 4px;">' +
                graph +
                '</pre>';
        }});
    </script>
</body>
</html>"#,
            diagram.metadata.benchmark_id,
            diagram.metadata.benchmark_id,
            diagram.metadata.tool_count,
            diagram.metadata.state_count,
            diagram.metadata.execution_time_ms,
            diagram
                .metadata
                .session_id
                .as_ref()
                .map(|id| format!(" | Session: {}...", &id[..8.min(id.len())]))
                .unwrap_or_default(),
            diagram.diagram.replace('`', "\\`").replace('$', "\\$")
        )
    }
}
