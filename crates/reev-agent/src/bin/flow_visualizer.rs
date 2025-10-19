//! # Flow Visualizer CLI Tool
//!
//! This tool converts flow execution logs into Mermaid state diagrams
//! for visualizing agent decision flows and tool execution patterns.
//!
//! Usage:
//! ```bash
//! cargo run --bin flow_visualizer -- --input logs/tool_calls.log --output diagram.mmd
//! ```
//!
//! Examples:
//! ```bash
//! # Generate diagram from log file
//! cargo run --bin flow_visualizer -- --input logs/tool_calls.log
//!
//! # Generate diagram with custom output
//! cargo run --bin flow_visualizer -- --input logs/tool_calls.log --output my_diagram.mmd
//!
//! # Generate diagram without timing info
//! cargo run --bin flow_visualizer -- --input logs/tool_calls.log --no-timing
//!
//! # Include tool parameters in diagram
//! cargo run --bin flow_visualizer -- --input logs/tool_calls.log --include-params
//! ```

use anyhow::{Context, Result};
use clap::Parser;
use reev_agent::flow::visualization::generate_mermaid_diagram;
use reev_agent::flow::visualization::mermaid_generator::DiagramConfig;
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "flow_visualizer",
    about = "Generate Mermaid state diagrams from flow execution logs",
    long_about = "Convert structured flow logs into visual state diagrams showing agent decision flows and tool execution patterns."
)]
struct Args {
    /// Input log file to process
    #[arg(short, long, help = "Path to the log file to process")]
    input: PathBuf,

    /// Output file for the diagram (default: stdout)
    #[arg(short, long, help = "Output file for the generated diagram")]
    output: Option<PathBuf>,

    /// Exclude timing information from the diagram
    #[arg(long, help = "Exclude timing information from the diagram")]
    no_timing: bool,

    /// Include tool parameters in the diagram
    #[arg(long, help = "Include tool parameters in the diagram")]
    include_params: bool,

    /// Maximum depth of nested states to show
    #[arg(
        long,
        default_value = "3",
        help = "Maximum depth of nested states to show"
    )]
    max_depth: usize,

    /// Hide error states from the diagram
    #[arg(long, help = "Hide error states from the diagram")]
    hide_errors: bool,

    /// Don't group tools by category
    #[arg(long, help = "Don't group tools by category")]
    no_grouping: bool,

    /// Generate HTML preview with embedded Mermaid
    #[arg(long, help = "Generate HTML preview with embedded Mermaid")]
    html: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Validate input file exists
    if !args.input.exists() {
        anyhow::bail!("Input file does not exist: {:?}", args.input);
    }

    println!("🔍 Reading flow log from: {:?}", args.input);
    let log_content = fs::read_to_string(&args.input)
        .with_context(|| format!("Failed to read log file: {:?}", args.input))?;

    if log_content.trim().is_empty() {
        anyhow::bail!("Input log file is empty");
    }

    println!("📊 Parsing flow execution data...");
    let _config = DiagramConfig {
        include_timing: !args.no_timing,
        include_parameters: args.include_params,
        max_depth: args.max_depth,
        show_errors: !args.hide_errors,
        group_tools: !args.no_grouping,
    };

    let diagram = generate_mermaid_diagram(&log_content)
        .map_err(|e| anyhow::anyhow!("Failed to generate Mermaid diagram from log content: {e}"))?;

    let content = if args.html {
        generate_html_preview(&diagram)?
    } else {
        diagram
    };

    // Extract base name for consistent file naming
    let base_name = extract_base_name_from_log(&args.input);

    // Output the result
    match args.output {
        Some(output_path) => {
            println!("💾 Writing diagram to: {output_path:?}");
            fs::write(&output_path, content.clone())
                .with_context(|| format!("Failed to write to output file: {output_path:?}"))?;

            println!("✅ Diagram generated successfully!");
            println!("📄 Open the file in a Mermaid-compatible viewer to see the visualization.");
            if args.html {
                println!("🌐 You can open the HTML file directly in a web browser.");
            }

            // Generate additional files with correct naming convention
            generate_additional_files(&base_name, &content)?;
        }
        None => {
            println!("📄 Generated Mermaid Diagram:");
            println!("{}", "=".repeat(60));
            println!("{content}");
            println!("{}", "=".repeat(60));
            println!("💡 Copy this diagram into a Mermaid-compatible editor to visualize.");

            // Even without explicit output, generate additional files
            generate_additional_files(&base_name, &content)?;
        }
    }

    Ok(())
}

/// Generate HTML preview with embedded Mermaid
fn generate_html_preview(diagram: &str) -> Result<String> {
    let html = format!(
        r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Flow Execution Diagram</title>
    <script src="https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.min.js"></script>
    <style>
        body {{
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            margin: 0;
            padding: 20px;
            background-color: #f5f5f5;
        }}
        .container {{
            max-width: 1200px;
            margin: 0 auto;
            background-color: white;
            padding: 30px;
            border-radius: 10px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }}
        .header {{
            text-align: center;
            margin-bottom: 30px;
            color: #333;
        }}
        .diagram-container {{
            background-color: #fafafa;
            padding: 20px;
            border-radius: 8px;
            border: 1px solid #e0e0e0;
            overflow-x: auto;
        }}
        .info {{
            margin-top: 20px;
            padding: 15px;
            background-color: #e3f2fd;
            border-radius: 5px;
            border-left: 4px solid #2196f3;
        }}
        pre {{
            margin: 0;
            white-space: pre-wrap;
            word-wrap: break-word;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🔀 Flow Execution Diagram</h1>
            <p>Generated from flow execution logs using OpenTelemetry tracing data</p>
        </div>

        <div class="diagram-container">
            <pre class="mermaid">
{diagram}
            </pre>
        </div>

        <div class="info">
            <h3>ℹ️ About this Diagram</h3>
            <p>This state diagram visualizes the agent's decision flow and tool execution sequence. Each state represents a step in the flow, and transitions show the order of execution. Tool states are color-coded by category (Swap, Transfer, Discovery, Lending, etc.).</p>
        </div>
    </div>

    <script>
        mermaid.initialize({{
            startOnLoad: true,
            theme: 'default',
            themeVariables: {{
                primaryColor: '#4ecdc4',
                primaryTextColor: '#333',
                primaryBorderColor: '#2c3e50',
                lineColor: '#666',
                secondaryColor: '#f8f9fa',
                tertiaryColor: '#e9ecef'
            }}
        }});
    </script>
</body>
</html>
"#
    );

    Ok(html)
}

/// Extract base name from input file path
fn extract_base_name_from_log(input_path: &PathBuf) -> String {
    let file_stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("flow_diagram");

    // Remove common prefixes/suffixes
    let binding = file_stem
        .replace("tool_calls", "")
        .replace("sample_opentelemetry", "")
        .replace("sample", "")
        .replace("test", "");
    let base_name = binding.trim_end_matches(['.', '-']);

    if base_name.is_empty() {
        "flow_diagram".to_string()
    } else {
        base_name.to_string()
    }
}

/// Generate additional files with correct naming convention
fn generate_additional_files(base_name: &str, content: &str) -> Result<()> {
    // Create viz directory if it doesn't exist
    let viz_dir = PathBuf::from("viz");
    if !viz_dir.exists() {
        fs::create_dir_all(&viz_dir)?;
        println!("📁 Created viz directory");
    }

    // Generate MMD file with correct naming
    let mmd_path = viz_dir.join(format!("{base_name}.mmd"));
    fs::write(&mmd_path, content)?;
    println!("📄 Generated: {}", mmd_path.display());

    // Generate HTML file with correct naming
    let html_path = viz_dir.join(format!("{base_name}.html"));
    let html_content = generate_html_preview(content)?;
    fs::write(&html_path, html_content)?;
    println!("🌐 Generated: {}", html_path.display());

    Ok(())
}
