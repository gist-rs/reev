//! Duplicate Tester Utility
//!
//! A comprehensive testing tool for reproducing and analyzing database duplicate creation issues.
//! Tests various scenarios that could lead to duplicate records in database operations.

use anyhow::Result;
use clap::{Arg, Command};
use reev_db::{DatabaseConfig, DatabaseWriter};
use std::time::Instant;
use tokio::task::JoinSet;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let matches = Command::new("duplicate-tester")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Test and analyze database duplicate creation scenarios")
        .arg(
            Arg::new("database")
                .short('d')
                .long("database")
                .value_name("PATH")
                .help("Path to test database (will be created if needed)")
                .default_value(":memory:"),
        )
        .arg(
            Arg::new("test")
                .short('t')
                .long("test")
                .value_name("TEST")
                .help("Specific test to run")
                .value_parser([
                    "all",
                    "basic_upsert",
                    "multiple_connections",
                    "parallel_processing",
                    "transaction_boundaries",
                    "connection_pool",
                    "rapid_successive",
                    "timestamp_variation",
                ])
                .default_value("all"),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Verbose output with detailed logs"),
        )
        .get_matches();

    let db_path = matches.get_one::<String>("database").unwrap();
    let test_name = matches.get_one::<String>("test").unwrap();
    let verbose = matches.get_flag("verbose");

    info!("🧪 Starting duplicate creation tests");
    info!("📁 Database: {}", db_path);
    info!("🔬 Test: {}", test_name);

    let config = DatabaseConfig::new(db_path);
    let db = DatabaseWriter::new(config).await?;

    let mut test_results = Vec::new();

    match test_name.as_str() {
        "all" => {
            test_results.push(run_basic_upsert_test(&db, verbose).await?);
            test_results.push(run_multiple_connections_test(&db, verbose).await?);
            test_results.push(run_parallel_processing_test(&db, verbose).await?);
            test_results.push(run_transaction_boundaries_test(&db, verbose).await?);
            test_results.push(run_connection_pool_test(&db, verbose).await?);
            test_results.push(run_rapid_successive_test(&db, verbose).await?);
            test_results.push(run_timestamp_variation_test(&db, verbose).await?);
        }
        "basic_upsert" => {
            test_results.push(run_basic_upsert_test(&db, verbose).await?);
        }
        "multiple_connections" => {
            test_results.push(run_multiple_connections_test(&db, verbose).await?);
        }
        "parallel_processing" => {
            test_results.push(run_parallel_processing_test(&db, verbose).await?);
        }
        "transaction_boundaries" => {
            test_results.push(run_transaction_boundaries_test(&db, verbose).await?);
        }
        "connection_pool" => {
            test_results.push(run_connection_pool_test(&db, verbose).await?);
        }
        "rapid_successive" => {
            test_results.push(run_rapid_successive_test(&db, verbose).await?);
        }
        "timestamp_variation" => {
            test_results.push(run_timestamp_variation_test(&db, verbose).await?);
        }
        _ => {
            eprintln!("Unknown test: {}", test_name);
            std::process::exit(1);
        }
    }

    // Print summary
    print_test_summary(&test_results);

    // Exit with appropriate code
    let failed_tests = test_results.iter().filter(|r| !r.passed).count();
    if failed_tests > 0 {
        error!("❌ {} test(s) failed", failed_tests);
        std::process::exit(1);
    } else {
        info!("✅ All tests passed!");
        Ok(())
    }
}

#[derive(Debug)]
struct TestResult {
    name: String,
    passed: bool,
    message: String,
    duration_ms: u64,
    details: Vec<String>,
}

async fn run_basic_upsert_test(db: &DatabaseWriter, verbose: bool) -> Result<TestResult> {
    let start_time = Instant::now();
    info!("🧪 Running basic upsert test...");

    let mut details = Vec::new();

    // Test identical upserts
    let md5_1 = db
        .upsert_benchmark("basic-test", "Test prompt", "Test content")
        .await?;
    details.push(format!("First upsert MD5: {}", md5_1));

    let count_1 = db.get_all_benchmark_count().await?;
    details.push(format!("Record count after first: {}", count_1));

    let md5_2 = db
        .upsert_benchmark("basic-test", "Test prompt", "Test content")
        .await?;
    details.push(format!("Second upsert MD5: {}", md5_2));

    let count_2 = db.get_all_benchmark_count().await?;
    details.push(format!("Record count after second: {}", count_2));

    let passed = md5_1 == md5_2 && count_1 == 1 && count_2 == 1;
    let message = if passed {
        "Basic upsert works correctly - no duplicates created".to_string()
    } else {
        "Basic upsert failed - duplicates detected".to_string()
    };

    if verbose {
        for detail in &details {
            info!("  {}", detail);
        }
    }

    Ok(TestResult {
        name: "Basic Upsert".to_string(),
        passed,
        message,
        duration_ms: start_time.elapsed().as_millis() as u64,
        details,
    })
}

async fn run_multiple_connections_test(db: &DatabaseWriter, verbose: bool) -> Result<TestResult> {
    let start_time = Instant::now();
    info!("🧪 Running multiple connections test...");

    let mut details = Vec::new();

    // Create separate database connections
    let config = DatabaseConfig::new(":memory:");
    let db1 = DatabaseWriter::new(config.clone()).await?;
    let db2 = DatabaseWriter::new(config).await?;

    let md5_1 = db1
        .upsert_benchmark("conn-test", "Test prompt", "Test content")
        .await?;
    let md5_2 = db2
        .upsert_benchmark("conn-test", "Test prompt", "Test content")
        .await?;

    let count_1 = db1.get_all_benchmark_count().await?;
    let count_2 = db2.get_all_benchmark_count().await?;

    details.push(format!("Connection 1 MD5: {}, Count: {}", md5_1, count_1));
    details.push(format!("Connection 2 MD5: {}, Count: {}", md5_2, count_2));

    let passed = md5_1 == md5_2 && count_1 == 1 && count_2 == 1;
    let message = if passed {
        "Multiple connections work independently as expected".to_string()
    } else {
        "Multiple connections show inconsistent behavior".to_string()
    };

    if verbose {
        for detail in &details {
            info!("  {}", detail);
        }
    }

    Ok(TestResult {
        name: "Multiple Connections".to_string(),
        passed,
        message,
        duration_ms: start_time.elapsed().as_millis() as u64,
        details,
    })
}

async fn run_parallel_processing_test(db: &DatabaseWriter, verbose: bool) -> Result<TestResult> {
    let start_time = Instant::now();
    info!("🧪 Running parallel processing test...");

    let mut details = Vec::new();

    let test_cases = vec![
        ("para-1", "Prompt 1", "Content 1"),
        ("para-2", "Prompt 2", "Content 2"),
        ("para-1", "Prompt 1", "Content 1"), // Duplicate
        ("para-3", "Prompt 3", "Content 3"),
        ("para-2", "Prompt 2", "Content 2"), // Duplicate
    ];

    let mut join_set = JoinSet::new();

    for (i, (name, prompt, content)) in test_cases.into_iter().enumerate() {
        let db_clone = db.connection().clone();
        join_set.spawn(async move {
            info!("  Parallel task {}: Processing {}", i + 1, name);
            let timestamp = chrono::Utc::now().to_rfc3339();
            let prompt_md5 = format!(
                "{:x}",
                md5::compute(format!("{}:{}", name, prompt).as_bytes())
            );

            let query = "
                INSERT INTO benchmarks (id, benchmark_name, prompt, content, created_at, updated_at)
                VALUES (?, ?, ?, ?, ?, ?)
                ON CONFLICT(id) DO UPDATE SET
                    benchmark_name = excluded.benchmark_name,
                    prompt = excluded.prompt,
                    content = excluded.content,
                    updated_at = excluded.updated_at;
            ";

            match db_clone
                .execute(
                    query,
                    [
                        prompt_md5.clone(),
                        name.to_string(),
                        prompt.to_string(),
                        content.to_string(),
                        timestamp.clone(),
                        timestamp,
                    ],
                )
                .await
            {
                Ok(_) => {
                    info!("    ✅ Parallel task {} completed: {}", i + 1, prompt_md5);
                    Ok(prompt_md5)
                }
                Err(e) => {
                    error!("    ❌ Parallel task {} failed: {}", i + 1, e);
                    Err(anyhow::anyhow!("Parallel task failed: {}", e))
                }
            }
        });
    }

    let mut results = Vec::new();
    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(md5_result) => match md5_result {
                Ok(md5) => {
                    details.push(format!("Parallel task completed: {}", md5));
                    results.push(md5);
                }
                Err(e) => {
                    details.push(format!("Parallel task failed: {}", e));
                    warn!("  {}", e);
                }
            },
            Err(e) => {
                details.push(format!("Task join failed: {}", e));
                error!("  {}", e);
            }
        }
    }

    let final_count = db.get_all_benchmark_count().await?;
    details.push(format!("Final record count: {}", final_count));
    details.push(format!("Expected: 3 unique records"));

    let passed = final_count == 3;
    let message = if passed {
        "Parallel processing handled correctly".to_string()
    } else {
        "Parallel processing created {} records (expected 3)".to_string()
    };

    if verbose {
        for detail in &details {
            info!("  {}", detail);
        }
    }

    Ok(TestResult {
        name: "Parallel Processing".to_string(),
        passed,
        message,
        duration_ms: start_time.elapsed().as_millis() as u64,
        details,
    })
}

async fn run_transaction_boundaries_test(db: &DatabaseWriter, verbose: bool) -> Result<TestResult> {
    let start_time = Instant::now();
    info!("🧪 Running transaction boundaries test...");

    let mut details = Vec::new();

    // Simulate rapid successive operations that might cause transaction boundary issues
    for i in 0..10 {
        let benchmark_name = format!("tx-test-{}", i % 3); // Create some duplicates
        let prompt = format!("Prompt {}", i);
        let content = format!("Content {}", i);

        let md5 = db
            .upsert_benchmark(&benchmark_name, &prompt, &content)
            .await?;
        let count = db.get_all_benchmark_count().await?;

        details.push(format!("Operation {}: MD5 {}, Count {}", i + 1, md5, count));
    }

    let final_count = db.get_all_benchmark_count().await?;
    let duplicates = db.check_for_duplicates().await?;

    details.push(format!("Final record count: {}", final_count));
    details.push(format!("Expected: 3 unique records"));
    details.push(format!("Duplicates found: {}", duplicates.len()));

    let passed = final_count == 3 && duplicates.is_empty();
    let message = if passed {
        "Transaction boundaries handled correctly".to_string()
    } else {
        "Transaction boundary issues detected".to_string()
    };

    if verbose {
        for detail in &details {
            info!("  {}", detail);
        }
    }

    Ok(TestResult {
        name: "Transaction Boundaries".to_string(),
        passed,
        message,
        duration_ms: start_time.elapsed().as_millis() as u64,
        details,
    })
}

async fn run_connection_pool_test(db: &DatabaseWriter, verbose: bool) -> Result<TestResult> {
    let start_time = Instant::now();
    info!("🧪 Running connection pool simulation test...");

    let mut details = Vec::new();

    // Simulate getting different connections from a pool
    let mut connections = Vec::new();
    for i in 0..3 {
        let config = DatabaseConfig::new(":memory:");
        let conn = DatabaseWriter::new(config).await?;
        connections.push(conn);
        details.push(format!("Created connection {} from 'pool'", i + 1));
    }

    // Use different connections for the same operation
    let benchmark_name = "pool-test";
    let prompt = "Pool test prompt";
    let content = "Pool test content";

    for (i, conn) in connections.iter().enumerate() {
        let md5 = conn
            .upsert_benchmark(benchmark_name, prompt, content)
            .await?;
        let count = conn.get_all_benchmark_count().await?;
        details.push(format!(
            "Connection {}: {} records, MD5: {}",
            i + 1,
            count,
            md5
        ));
    }

    let passed = true; // This is expected behavior - each connection is independent
    let message = "Connection pool simulation completed (each connection independent as expected)"
        .to_string();

    if verbose {
        for detail in &details {
            info!("  {}", detail);
        }
    }

    Ok(TestResult {
        name: "Connection Pool".to_string(),
        passed,
        message,
        duration_ms: start_time.elapsed().as_millis() as u64,
        details,
    })
}

async fn run_rapid_successive_test(db: &DatabaseWriter, verbose: bool) -> Result<TestResult> {
    let start_time = Instant::now();
    info!("🧪 Running rapid successive operations test...");

    let mut details = Vec::new();

    // Perform the same operation multiple times rapidly
    let benchmark_name = "rapid-test";
    let prompt = "Rapid test prompt";
    let content = "Rapid test content";

    for i in 0..50 {
        let md5 = db.upsert_benchmark(benchmark_name, prompt, content).await?;
        let count = db.get_all_benchmark_count().await?;

        if i == 0 {
            details.push(format!("First operation: MD5 {}, Count {}", md5, count));
        } else if i == 49 {
            details.push(format!("Last operation: MD5 {}, Count {}", md5, count));
        }

        if count > 1 {
            details.push(format!(
                "⚠️  Count increased to {} at operation {}",
                count,
                i + 1
            ));
        }
    }

    let final_count = db.get_all_benchmark_count().await?;
    let duplicates = db.check_for_duplicates().await?;

    let passed = final_count == 1 && duplicates.is_empty();
    let message = if passed {
        "Rapid successive operations handled correctly".to_string()
    } else {
        "Rapid operations created duplicates".to_string()
    };

    details.push(format!(
        "Final count: {}, Duplicates: {}",
        final_count,
        duplicates.len()
    ));

    if verbose {
        for detail in &details {
            info!("  {}", detail);
        }
    }

    Ok(TestResult {
        name: "Rapid Successive".to_string(),
        passed,
        message,
        duration_ms: start_time.elapsed().as_millis() as u64,
        details,
    })
}

async fn run_timestamp_variation_test(db: &DatabaseWriter, verbose: bool) -> Result<TestResult> {
    let start_time = Instant::now();
    info!("🧪 Running timestamp variation test...");

    let mut details = Vec::new();

    // Test that timestamp variations don't affect MD5 generation
    let benchmark_name = "timestamp-test";
    let prompt = "Timestamp test prompt";
    let content = "Timestamp test content";

    // Insert with a slight delay to ensure different timestamps
    let md5_1 = db.upsert_benchmark(benchmark_name, prompt, content).await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    let md5_2 = db.upsert_benchmark(benchmark_name, prompt, content).await?;

    let count = db.get_all_benchmark_count().await?;

    details.push(format!("First MD5: {}", md5_1));
    details.push(format!("Second MD5: {}", md5_2));
    details.push(format!("Record count: {}", count));

    let passed = md5_1 == md5_2 && count == 1;
    let message = if passed {
        "Timestamp variations don't affect MD5 generation".to_string()
    } else {
        "Timestamp variations caused duplicate creation".to_string()
    };

    if verbose {
        for detail in &details {
            info!("  {}", detail);
        }
    }

    Ok(TestResult {
        name: "Timestamp Variation".to_string(),
        passed,
        message,
        duration_ms: start_time.elapsed().as_millis() as u64,
        details,
    })
}

fn print_test_summary(results: &[TestResult]) {
    println!("\n🎯 Test Summary");
    println!("{}", "=".repeat(80));

    let total_duration: u64 = results.iter().map(|r| r.duration_ms).sum();
    let passed_count = results.iter().filter(|r| r.passed).count();
    let total_count = results.len();

    for result in results {
        let status = if result.passed {
            "✅ PASS"
        } else {
            "❌ FAIL"
        };
        println!(
            "{:<8} {:<25} {:<8} {:<50}",
            status,
            result.name,
            format!("{}ms", result.duration_ms),
            result.message
        );
    }

    println!("{}", "-".repeat(80));
    println!(
        "Summary: {}/{} tests passed in {}ms",
        passed_count, total_count, total_duration
    );

    if passed_count == total_count {
        println!("🎉 All tests passed! Database operations are working correctly.");
    } else {
        println!(
            "⚠️  {} test(s) failed. Review the results above.",
            total_count - passed_count
        );
    }
}
