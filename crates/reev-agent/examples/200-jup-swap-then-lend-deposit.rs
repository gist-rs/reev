//! # Multi-Step Flow Agent Example
//!
//! This example demonstrates the FlowAgent's ability to orchestrate
//! multi-step DeFi workflows. The agent will swap SOL to USDC
//! and then deposit the USDC into Jupiter lending.
//!
//! ## Flow Overview:
//! 1. Swap 0.5 SOL to USDC using Jupiter
//! 2. Deposit received USDC into Jupiter lending
//!
//! This example showcases:
//! - RAG-based tool selection and discovery
//! - Multi-step conversation state management
//! - Dynamic tool embedding and vector search
//! - Context-aware prompt enrichment

use anyhow::Result;
use reev_agent::flow::{FlowAgent, FlowBenchmark};
use std::fs;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    println!("🚀 Multi-Step Flow Agent Example");
    println!("================================");

    // Load the multi-step benchmark
    println!("📋 Loading flow benchmark...");
    let benchmark_path = "benchmarks/200-jup-swap-then-lend-deposit.yml";

    let benchmark_content =
        fs::read_to_string(benchmark_path).expect("Failed to read benchmark file");

    let benchmark: FlowBenchmark =
        serde_yaml::from_str(&benchmark_content).expect("Failed to parse benchmark YAML");

    println!("✅ Flow benchmark loaded: {}", benchmark.id);
    println!("📊 Flow summary:\n{}", benchmark.get_summary());

    // Create the flow agent
    println!("\n🤖 Initializing Flow Agent...");
    let mut flow_agent = FlowAgent::new("local-model")
        .await
        .expect("Failed to create flow agent");

    // Load the benchmark into the agent
    println!("📥 Loading benchmark into agent...");
    flow_agent
        .load_benchmark(&benchmark)
        .await
        .expect("Failed to load benchmark into agent");

    // Execute the multi-step flow
    println!("\n🎯 Executing Multi-Step Flow...");
    println!("================================");

    let flow_results = flow_agent
        .execute_flow(&benchmark)
        .await
        .expect("Failed to execute flow");

    // Display results
    println!("\n✅ Flow Execution Complete!");
    println!("================================");
    println!("📊 Results Summary:");

    for (i, result) in flow_results.iter().enumerate() {
        println!(
            "  Step {}: {} - Status: {:?}",
            i + 1,
            result.description,
            result.status
        );
        println!("    Instructions: {}", result.instructions.len());
        println!(
            "    LLM Response Length: {} chars",
            result.llm_response.len()
        );

        if !result.metadata.is_empty() {
            println!("    Metadata: {:?}", result.metadata);
        }
        println!();
    }

    // Display final state
    println!("📈 Final Flow State:");
    println!("==================");
    println!("{}", flow_agent.get_state().get_summary());

    // Display conversation history
    if !flow_agent.get_state().conversation_history.is_empty() {
        println!("\n💬 Conversation History:");
        println!("=====================");
        for turn in &flow_agent.get_state().conversation_history {
            println!("Turn {} (Step {}):", turn.turn, turn.step);
            println!("  User: {}", turn.user_prompt);
            println!("  Tools: {:?}", turn.tools_called);
            println!("  Response Length: {} chars", turn.llm_response.len());
            println!();
        }
    }

    // Display step results in detail
    println!("🔍 Detailed Step Results:");
    println!("========================");
    for (step_id, result) in &flow_agent.get_state().step_results {
        println!("{step_id}:");
        println!("  Status: {:?}", result.status);
        println!("  Description: {}", result.description);
        println!("  Instructions: {}", result.instructions.len());

        if !result.instructions.is_empty() {
            println!("  Instructions:");
            for (i, instruction) in result.instructions.iter().enumerate() {
                println!(
                    "    {}: {} ({})",
                    i + 1,
                    instruction.program_id,
                    instruction.data
                );
                println!("       Accounts: {}", instruction.accounts.len());
            }
        }
        println!();
    }

    println!("🎉 Multi-Step Flow Example Completed Successfully!");
    println!("================================");
    println!("This demonstrates the FlowAgent's ability to:");
    println!("• Execute multi-step DeFi workflows");
    println!("• Use RAG for intelligent tool selection");
    println!("• Maintain conversation state across steps");
    println!("• Provide context-aware decision making");
    println!("• Handle complex DeFi operations end-to-end");

    Ok(())
}
