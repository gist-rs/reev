//! Demo program to demonstrate GLM tab behavior in the TUI
//!
//! This program shows how the GLM tab appears and behaves
//! based on environment variable configuration.

use reev_tui::SelectedAgent;
use std::env;

fn main() {
    println!("🪸 Reev TUI GLM Tab Demo\n");

    // Show all available agents
    println!("Available agents in TUI:");
    for (index, agent) in SelectedAgent::iter().enumerate() {
        let agent_name = agent.to_agent_name();
        let display_name = agent.to_string();
        let is_disabled = agent.is_disabled(false);

        let status = if is_disabled {
            "❌ DISABLED"
        } else {
            "✅ ENABLED"
        };
        println!(
            "  {}. {} ({}) - {}",
            index + 1,
            display_name.trim(),
            agent_name,
            status
        );
    }

    println!("\n" + "=".repeat(60).as_str());
    println!("GLM Environment Variable Status:");

    // Check GLM environment variables
    let has_glm_key = env::var("GLM_API_KEY").is_ok();
    let has_glm_url = env::var("GLM_API_URL").is_ok();

    println!(
        "  GLM_API_KEY: {}",
        if has_glm_key {
            "✅ Set"
        } else {
            "❌ Not set"
        }
    );
    println!(
        "  GLM_API_URL: {}",
        if has_glm_url {
            "✅ Set"
        } else {
            "❌ Not set"
        }
    );

    println!("\nGLM Tab Status:");
    let glm_agent = SelectedAgent::Glm46;
    let is_disabled = glm_agent.is_disabled(false);

    if is_disabled {
        println!("  ❌ GLM tab is DISABLED");
        if !has_glm_key && !has_glm_url {
            println!("     → Both GLM_API_KEY and GLM_API_URL are missing");
            println!("     → Set both environment variables to enable GLM 4.6");
        } else if has_glm_key && !has_glm_url {
            println!("     → GLM_API_KEY is set but GLM_API_URL is missing");
            println!("     → Set GLM_API_URL to enable GLM 4.6");
        } else if !has_glm_key && has_glm_url {
            println!("     → GLM_API_URL is set but GLM_API_KEY is missing");
            println!("     → Set GLM_API_KEY to enable GLM 4.6");
        }
    } else {
        println!("  ✅ GLM tab is ENABLED");
        println!("     → GLM 4.6 is ready to use with OpenAI-compatible API");
    }

    println!("\n" + "=".repeat(60).as_str());
    println!("How to use GLM 4.6 in TUI:");
    println!("  1. Set environment variables:");
    println!("     export GLM_API_KEY='your-glm-api-key'");
    println!("     export GLM_API_URL='https://api.example.com/v1/chat/completions'");
    println!("  2. Run the TUI: cargo run --package reev-tui");
    println!("  3. Navigate to GLM 4.6 tab using arrow keys (◄ ►)");
    println!("  4. Select and run benchmarks with GLM 4.6");

    println!("\n" + "=".repeat(60).as_str());
    println!("TUI Navigation:");
    println!("  ◄ ► : Switch between agents (including GLM 4.6)");
    println!("  ↑ ↓ : Navigate benchmarks");
    println!("  Tab : Switch between panels");
    println!("  Enter/R : Run selected benchmark");
    println!("  A : Run all benchmarks");
    println!("  Q : Quit");

    // Demonstrate current GLM agent state
    println!("\nCurrent GLM Agent State:");
    println!("  Enum value: {:?}", glm_agent);
    println!("  Agent name: {}", glm_agent.to_agent_name());
    println!("  Display name: {}", glm_agent.to_string().trim());
    println!("  Previous agent: {:?}", glm_agent.previous());
    println!("  Next agent: {:?}", glm_agent.next());
}
