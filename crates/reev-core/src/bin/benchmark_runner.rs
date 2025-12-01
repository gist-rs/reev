//! Benchmark Runner CLI for Protocol Interface Integration
//!
//! This CLI tool provides a command-line interface for running benchmarks
//! using the protocol interface. It supports both static and dynamic benchmarks
//! with various output formats and configuration options.

use clap::{Arg, ArgMatches, Command};
use reev_core::benchmark::runner::{DynamicBenchmarkRunner, Flow, StaticBenchmarkRunner};
use std::path::PathBuf;
use std::process;

/// Main function for benchmark runner CLI
#[tokio::main]
async fn main() {
    // Parse command line arguments
    let matches = Command::new("benchmark_runner")
        .version("0.1.0")
        .about("Run Reev benchmarks using protocol interface")
        .arg(
            Arg::new("static")
                .short('s')
                .long("static")
                .help("Run static benchmarks from YML files")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("dynamic")
                .short('d')
                .long("dynamic")
                .help("Run dynamic benchmarks from prompts")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("path")
                .short('p')
                .long("path")
                .value_name("PATH")
                .help("Path to benchmark files (for static benchmarks)")
                .value_parser(clap::value_parser!(PathBuf)),
        )
        .arg(
            Arg::new("category")
                .short('c')
                .long("category")
                .value_name("CATEGORY")
                .help("Benchmark category (swap, lend, earn, all)")
                .default_value("all"),
        )
        .arg(
            Arg::new("prompt")
                .short('P')
                .long("prompt")
                .value_name("PROMPT")
                .help("Prompt to benchmark (for dynamic benchmarks)"),
        )
        .arg(
            Arg::new("wallet")
                .short('w')
                .long("wallet")
                .value_name("PUBKEY")
                .help("Wallet public key for dynamic benchmarks"),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FORMAT")
                .help("Output format (json, csv, pretty)")
                .default_value("pretty"),
        )
        .get_matches();

    // Determine which mode to run
    let is_static = matches.get_flag("static");
    let is_dynamic = matches.get_flag("dynamic");

    if is_static && is_dynamic {
        eprintln!("Error: Cannot specify both --static and --dynamic");
        process::exit(1);
    }

    if !is_static && !is_dynamic {
        eprintln!("Error: Must specify either --static or --dynamic");
        process::exit(1);
    }

    // Get output format
    let output_format = matches.get_one::<String>("output").unwrap();

    // Run appropriate benchmark mode
    let result = if is_static {
        run_static_benchmarks(&matches).await
    } else {
        run_dynamic_benchmarks(&matches).await
    };

    // Handle result
    match result {
        Ok(reports) => {
            output_reports(&reports, output_format);
        }
        Err(e) => {
            eprintln!("Error running benchmarks: {e}");
            process::exit(1);
        }
    }
}

/// Run static benchmarks
async fn run_static_benchmarks(
    matches: &ArgMatches,
) -> Result<Vec<BenchmarkReport>, Box<dyn std::error::Error>> {
    let path = matches
        .get_one::<PathBuf>("path")
        .cloned()
        .unwrap_or_else(|| PathBuf::from("benchmarks/flows/"));

    let category = matches.get_one::<String>("category").unwrap();

    // Initialize static benchmark runner
    let runner = StaticBenchmarkRunner::new();

    // Find benchmark files
    let benchmark_files = find_benchmark_files(&path, category)?;

    if benchmark_files.is_empty() {
        eprintln!("No benchmark files found for category: {category}");
        return Ok(vec![]);
    }

    println!(
        "Found {} benchmark files for category: {}",
        benchmark_files.len(),
        category
    );

    let mut reports = Vec::new();

    // Run each benchmark
    for file in benchmark_files {
        println!("Running benchmark: {file:?}");

        // Load flow from file
        let flow = load_flow_from_file(&file)?;

        // Execute flow
        let report = runner.execute_flow(&flow).await?;

        reports.push(report);
    }

    Ok(reports)
}

/// Run dynamic benchmarks
async fn run_dynamic_benchmarks(
    matches: &ArgMatches,
) -> Result<Vec<BenchmarkReport>, Box<dyn std::error::Error>> {
    let category = matches.get_one::<String>("category").unwrap();

    // Get prompt
    let prompt = match matches.get_one::<String>("prompt") {
        Some(p) => p.clone(),
        None => {
            eprintln!("Error: --prompt is required for dynamic benchmarks");
            process::exit(1);
        }
    };

    // Get wallet pubkey
    let wallet_pubkey = match matches.get_one::<String>("wallet") {
        Some(w) => w.clone(),
        None => {
            eprintln!("Error: --wallet is required for dynamic benchmarks");
            process::exit(1);
        }
    };

    // Initialize dynamic benchmark runner
    let mut runner = DynamicBenchmarkRunner::new().await?;

    let mut reports = Vec::new();

    if category == "all" {
        // Run all categories
        let categories = ["swap", "lend", "earn"];

        for cat in categories {
            println!("Running benchmark for category: {cat}");

            let category_prompt = format!("{cat} {prompt}");

            // Generate and execute flow
            let report = runner
                .execute_prompt(&category_prompt, &wallet_pubkey)
                .await?;

            reports.push(report);
        }
    } else {
        // Run specific category
        println!("Running benchmark for category: {category}");

        let category_prompt = format!("{category} {prompt}");

        // Generate and execute flow
        let report = runner
            .execute_prompt(&category_prompt, &wallet_pubkey)
            .await?;

        reports.push(report);
    }

    Ok(reports)
}

/// Find benchmark files by category
fn find_benchmark_files(
    path: &PathBuf,
    category: &str,
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();

    if category == "all" {
        // Find all YML files
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("yml") {
                files.push(path);
            }
        }
    } else {
        // Find files for specific category
        let _pattern = format!("*{category}*.yml");

        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

                if filename.contains(category) {
                    files.push(path);
                }
            }
        }
    }

    files.sort();
    Ok(files)
}

/// Load flow from YML file
fn load_flow_from_file(path: &PathBuf) -> Result<Flow, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let flow: Flow = serde_yaml::from_str(&content)?;
    Ok(flow)
}

/// Output benchmark reports
fn output_reports(reports: &[BenchmarkReport], format: &str) {
    match format {
        "json" => {
            let json = serde_json::to_string_pretty(reports).unwrap();
            println!("{json}");
        }
        "csv" => {
            // Simple CSV output
            println!("flow_id,execution_id,overall_score,execution_time_ms");
            for report in reports {
                println!(
                    "{},{},{},{}",
                    report.flow_id,
                    report.execution_id,
                    report.overall_score,
                    report.execution_metrics.total_execution_time_ms
                );
            }
        }
        "pretty" => {
            for report in reports {
                println!("Benchmark Report");
                println!("===============");
                println!("Flow ID: {}", report.flow_id);
                println!("Execution ID: {}", report.execution_id);
                println!("Prompt: {}", report.prompt);
                println!("Overall Score: {:.2}", report.overall_score);
                println!(
                    "Execution Time: {}ms",
                    report.execution_metrics.total_execution_time_ms
                );
                println!(
                    "Steps Executed: {}",
                    report.execution_metrics.steps_executed
                );
                println!("Tool Calls: {}", report.execution_metrics.tool_calls_made);
                println!(
                    "Successful Tool Calls: {}",
                    report.execution_metrics.successful_tool_calls
                );

                if !report.category_scores.is_empty() {
                    println!("Category Scores:");
                    for (category, score) in &report.category_scores {
                        println!("  {category}: {score:.2}");
                    }
                }

                if !report.improvement_suggestions.is_empty() {
                    println!("Improvement Suggestions:");
                    for suggestion in &report.improvement_suggestions {
                        println!("  - {suggestion}");
                    }
                }

                println!();
            }
        }
        _ => {
            eprintln!("Unknown output format: {format}");
        }
    }
}

// Import required types for benchmark runner
use reev_core::benchmark::types::BenchmarkReport;
// Removed duplicate import, Flow is already imported above
