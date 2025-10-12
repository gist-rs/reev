//! Test to debug GLM tab highlighting issue
//!
//! This test will help identify why the GLM tab isn't being highlighted
//! when navigating with left/right arrow keys.

use reev_tui::{App, SelectedAgent};

fn main() {
    println!("🔍 Debugging GLM Tab Highlighting Issue\n");

    // Create a new app instance
    let mut app = App::new();

    println!("Initial state:");
    println!("  Selected agent: {:?}", app.selected_agent);
    println!("  Is running: {}", app.is_running_benchmark);

    // Test 1: Check if GLM agent is disabled initially
    let glm_agent = SelectedAgent::Glm46;
    let is_disabled = glm_agent.is_disabled(false);
    println!("\nGLM Agent Status:");
    println!("  GLM agent disabled (not running): {}", is_disabled);

    // Test 2: Try to navigate through agents
    println!("\nTesting agent navigation:");

    // Start from deterministic
    println!("  Current: {:?}", app.selected_agent);

    // Navigate right to gemini
    app.on_right();
    println!("  After right: {:?}", app.selected_agent);

    // Navigate right to local
    app.on_right();
    println!("  After right: {:?}", app.selected_agent);

    // Navigate right to GLM
    app.on_right();
    println!("  After right: {:?}", app.selected_agent);

    // Test 3: Check if GLM is actually selected
    println!("\nAfter navigation to GLM:");
    println!("  Selected agent: {:?}", app.selected_agent);
    println!(
        "  Is GLM selected: {}",
        app.selected_agent == SelectedAgent::Glm46
    );

    // Test 4: Test direct selection
    println!("\nTesting direct GLM selection:");
    app.select_glm46();
    println!("  After direct selection: {:?}", app.selected_agent);
    println!(
        "  Is GLM selected: {}",
        app.selected_agent == SelectedAgent::Glm46
    );

    // Test 5: Check environment variables
    println!("\nEnvironment variables:");
    let has_glm_key = std::env::var("GLM_API_KEY").is_ok();
    let has_glm_url = std::env::var("GLM_API_URL").is_ok();
    println!("  GLM_API_KEY set: {}", has_glm_key);
    println!("  GLM_API_URL set: {}", has_glm_url);

    // Test 6: Check is_disabled logic with current environment
    let glm_disabled_when_running = glm_agent.is_disabled(true);
    let glm_disabled_when_not_running = glm_agent.is_disabled(false);
    println!("\nGLM disabled status:");
    println!("  When running: {}", glm_disabled_when_running);
    println!("  When not running: {}", glm_disabled_when_not_running);

    // Test 7: Test navigation when GLM should be disabled
    if !has_glm_key || !has_glm_url {
        println!("\nTesting navigation when GLM is disabled:");

        // Go back to deterministic
        app.selected_agent = SelectedAgent::Deterministic;

        // Try to navigate right through to GLM
        for i in 0..4 {
            println!("  Step {}: {:?}", i, app.selected_agent);
            app.on_right();

            // Check if we actually moved
            if i == 3 && app.selected_agent != SelectedAgent::Glm46 {
                println!("  ⚠️  GLM tab was skipped due to being disabled!");
                println!("  This might be the issue - disabled tabs are being skipped during navigation.");
            }
        }
    }

    println!("\n🔧 Potential fixes:");
    println!("1. Check if on_right/on_left methods are skipping disabled agents");
    println!("2. Verify that disabled agents can still be selected for highlighting");
    println!("3. Ensure the custom tab rendering logic is working correctly");

    // Test 8: Manually set GLM and check if it would be highlighted
    println!("\nTesting manual GLM selection for highlighting:");
    app.selected_agent = SelectedAgent::Glm46;
    println!("  Manually set to GLM: {:?}", app.selected_agent);

    // Test highlighting logic directly
    let is_selected = app.selected_agent == SelectedAgent::Glm46;
    let is_disabled = app.selected_agent.is_disabled(app.is_running_benchmark);

    println!("  Would be highlighted: {}", is_selected && !is_disabled);
    println!("  Is selected: {}", is_selected);
    println!("  Is disabled: {}", is_disabled);
}
