//! Generate HTML Module
//!
//! This module provides the function for generating HTML output with Mermaid diagram visualizations.

use crate::handlers::flow_diagram::FlowDiagram;

/// Generate HTML output with embedded Mermaid diagram
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
