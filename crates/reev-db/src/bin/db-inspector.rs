//! Database Inspector Utility
//!
//! A command-line tool for inspecting and analyzing reev database contents.
//! Provides detailed information about database structure, statistics, and health.

use anyhow::Result;
use clap::{Arg, Command};
use reev_db::{DatabaseConfig, DatabaseWriter};
use serde_json;
use std::path::PathBuf;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let matches = Command::new("db-inspector")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Inspect and analyze reev database contents")
        .arg(
            Arg::new("database")
                .short('d')
                .long("database")
                .value_name("PATH")
                .help("Path to the database file")
                .required(true),
        )
        .arg(
            Arg::new("format")
                .short('f')
                .long("format")
                .value_name("FORMAT")
                .help("Output format (table, json)")
                .value_parser(["table", "json"])
                .default_value("table"),
        )
        .arg(
            Arg::new("check-duplicates")
                .short('c')
                .long("check-duplicates")
                .help("Check for duplicate records"),
        )
        .arg(
            Arg::new("stats")
                .short('s')
                .long("stats")
                .help("Show comprehensive database statistics"),
        )
        .arg(
            Arg::new("benchmarks")
                .short('b')
                .long("benchmarks")
                .help("List all benchmarks"),
        )
        .arg(
            Arg::new("cleanup")
                .long("cleanup")
                .help("Cleanup duplicate records (keep most recent)"),
        )
        .get_matches();

    let db_path = matches.get_one::<String>("database").unwrap();
    let format = matches.get_one::<String>("format").unwrap();
    let config = DatabaseConfig::new(db_path);

    info!("Connecting to database: {}", db_path);
    let db = DatabaseWriter::new(config).await?;

    // Check for duplicates if requested
    if matches.get_flag("check-duplicates") {
        check_duplicates(&db, format).await?;
    }

    // Show statistics if requested
    if matches.get_flag("stats") {
        show_statistics(&db, format).await?;
    }

    // List benchmarks if requested
    if matches.get_flag("benchmarks") {
        list_benchmarks(&db, format).await?;
    }

    // Cleanup duplicates if requested
    if matches.get_flag("cleanup") {
        cleanup_duplicates(&db).await?;
    }

    // If no specific operation requested, show overview
    if !matches.get_flag("check-duplicates")
        && !matches.get_flag("stats")
        && !matches.get_flag("benchmarks")
        && !matches.get_flag("cleanup")
    {
        show_overview(&db, format).await?;
    }

    Ok(())
}

async fn check_duplicates(db: &DatabaseWriter, format: &str) -> Result<()> {
    println!("🔍 Checking for duplicate records...\n");

    let duplicates = db.check_for_duplicates().await?;

    if duplicates.is_empty() {
        println!("✅ No duplicate records found");
    } else {
        println!("❌ Found {} duplicate records:\n", duplicates.len());

        if format == "json" {
            println!("{}", serde_json::to_string_pretty(&duplicates)?);
        } else {
            println!(
                "{:<40} {:<20} {:<10} {:<20} {:<20}",
                "ID", "Benchmark Name", "Count", "First Created", "Last Updated"
            );
            println!("{}", "-".repeat(120));

            for duplicate in duplicates {
                println!(
                    "{:<40} {:<20} {:<10} {:<20} {:<20}",
                    duplicate.id,
                    duplicate.benchmark_name,
                    duplicate.count,
                    duplicate.first_created_at,
                    duplicate.last_updated_at
                );
            }
        }
    }

    Ok(())
}

async fn show_statistics(db: &DatabaseWriter, format: &str) -> Result<()> {
    println!("📊 Database Statistics\n");

    let stats = db.get_database_stats().await?;

    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&stats)?);
    } else {
        println!("Benchmarks:");
        println!("  Total: {}", stats.total_benchmarks);
        println!("  Duplicates: {}", stats.duplicate_count);
        if !stats.duplicate_details.is_empty() {
            println!("  Duplicate Details:");
            for (id, name, count) in &stats.duplicate_details {
                println!("    {} ({}): {} records", id, name, count);
            }
        }

        println!("\nOther Tables:");
        println!("  Test Results: {}", stats.total_results);
        println!("  Flow Logs: {}", stats.total_flow_logs);
        println!("  Performance Records: {}", stats.total_performance_records);

        if let Some(size) = stats.database_size_bytes {
            println!(
                "\nDatabase Size: {} bytes ({:.2} MB)",
                size,
                size as f64 / 1024.0 / 1024.0
            );
        }

        println!("\nLast Updated: {}", stats.last_updated);
    }

    Ok(())
}

async fn list_benchmarks(db: &DatabaseWriter, format: &str) -> Result<()> {
    println!("📋 All Benchmarks\n");

    let benchmarks = db.get_all_benchmarks().await?;

    if benchmarks.is_empty() {
        println!("No benchmarks found");
        return Ok(());
    }

    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&benchmarks)?);
    } else {
        println!(
            "{:<32} {:<20} {:<50} {:<20}",
            "ID", "Name", "Prompt Preview", "Updated"
        );
        println!("{}", "-".repeat(130));

        for benchmark in benchmarks {
            let prompt_preview = if benchmark.prompt.len() > 47 {
                format!("{}...", &benchmark.prompt[..47])
            } else {
                benchmark.prompt.clone()
            };

            println!(
                "{:<32} {:<20} {:<50} {:<20}",
                benchmark.id, benchmark.benchmark_name, prompt_preview, benchmark.updated_at
            );
        }

        println!("\nTotal: {} benchmarks", benchmarks.len());
    }

    Ok(())
}

async fn cleanup_duplicates(db: &DatabaseWriter) -> Result<()> {
    println!("🧹 Cleaning up duplicate records...\n");

    let duplicates_before = db.check_for_duplicates().await?;

    if duplicates_before.is_empty() {
        println!("✅ No duplicates to cleanup");
        return Ok(());
    }

    println!("Found {} duplicate records", duplicates_before.len());

    // Ask for confirmation
    println!("WARNING: This will delete duplicate records, keeping only the most recent version.");
    print!("Do you want to continue? (y/N): ");
    std::io::stdout().flush()?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if !input.trim().to_lowercase().starts_with('y') {
        println!("Cleanup cancelled");
        return Ok(());
    }

    let cleaned_count = db.cleanup_duplicates().await?;
    let duplicates_after = db.check_for_duplicates().await?;

    println!("\n✅ Cleanup completed:");
    println!("  Records cleaned: {}", cleaned_count);
    println!("  Duplicates remaining: {}", duplicates_after.len());

    if duplicates_after.is_empty() {
        println!("  🎉 All duplicates successfully removed!");
    } else {
        println!("  ⚠️  Some duplicates remain - manual intervention may be required");
    }

    Ok(())
}

async fn show_overview(db: &DatabaseWriter, format: &str) -> Result<()> {
    println!("🪸 Reev Database Overview\n");

    let stats = db.get_database_stats().await?;
    let duplicates = db.check_for_duplicates().await?;

    if format == "json" {
        let overview = serde_json::json!({
            "database_stats": stats,
            "duplicates": duplicates
        });
        println!("{}", serde_json::to_string_pretty(&overview)?);
    } else {
        // Database Health
        println!("🏥 Database Health:");
        if duplicates.is_empty() {
            println!("  ✅ No duplicate records");
        } else {
            println!("  ❌ {} duplicate records found", duplicates.len());
        }

        // Summary Statistics
        println!("\n📈 Summary:");
        println!("  Benchmarks: {} total", stats.total_benchmarks);
        println!("  Test Results: {}", stats.total_results);
        println!("  Flow Logs: {}", stats.total_flow_logs);
        println!("  Performance Records: {}", stats.total_performance_records);

        // Database Size
        if let Some(size) = stats.database_size_bytes {
            println!(
                "\n💾 Database Size: {:.2} MB",
                size as f64 / 1024.0 / 1024.0
            );
        }

        // Recent Activity
        println!("\n🕒 Last Updated: {}", stats.last_updated);

        // Quick Actions
        println!("\n🔧 Quick Actions:");
        println!(
            "  Check duplicates:   db-inspector -d {} --check-duplicates",
            db.config.path
        );
        println!(
            "  Show statistics:    db-inspector -d {} --stats",
            db.config.path
        );
        println!(
            "  List benchmarks:    db-inspector -d {} --benchmarks",
            db.config.path
        );
        println!(
            "  Cleanup duplicates: db-inspector -d {} --cleanup",
            db.config.path
        );
    }

    Ok(())
}
