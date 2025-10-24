//! Comprehensive Benchmark YAML Validation Tests
//!
//! Tests ALL benchmark YAML files using only ground_truth data (no surfpool)
//! Validates context preparation, ground_truth fulfillment, and LLM prompt preparation
//! Ensures benchmarks are self-contained and testable without external dependencies

use anyhow::{Context, Result};
use reev_context::{AgentContext, ContextResolver, InitialState};
use serde_json::json;
use std::fs;
use std::path::Path;
use std::{collections::HashMap, str::FromStr};

/// Test ALL benchmark YAML files using only YAML ground_truth data
#[tokio::test]
async fn test_all_benchmarks_yaml_ground_truth_only() -> Result<()> {
    println!("🚀 Testing ALL benchmarks using only YAML ground_truth (no surfpool)");
    println!("{}", "=".repeat(80));

    let benchmarks_dir = Path::new("../../benchmarks");
    let mut all_passed = true;
    let mut test_results = Vec::new();
    let mut total_benchmarks = 0;
    let mut passed_benchmarks = 0;

    // Test each benchmark YAML file
    for entry in fs::read_dir(benchmarks_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("yml") {
            total_benchmarks += 1;
            let filename = path.file_name().unwrap().to_str().unwrap();

            println!("\n📋 Testing Benchmark: {filename}");
            println!("{}", "-".repeat(60));

            match test_single_benchmark_yaml_only(&path).await {
                Ok(_) => {
                    passed_benchmarks += 1;
                    println!("✅ {filename} - YAML ground_truth validation PASSED");
                    test_results.push(format!("✅ {filename} - PASSED"));
                }
                Err(e) => {
                    all_passed = false;
                    println!("❌ {filename} - YAML ground_truth validation FAILED: {e}");
                    test_results.push(format!("❌ {filename} - FAILED: {e}"));
                }
            }
        }
    }

    println!("\n{}", "=".repeat(80));
    println!("📊 SUMMARY - YAML GROUND TRUTH VALIDATION");
    println!("{}", "=".repeat(80));
    println!("Total Benchmarks: {total_benchmarks}");
    println!("Passed: {passed_benchmarks}");
    println!("Failed: {}", total_benchmarks - passed_benchmarks);
    println!(
        "Success Rate: {:.1}%",
        (passed_benchmarks as f64 / total_benchmarks as f64) * 100.0
    );

    println!("\nDetailed Results:");
    for result in &test_results {
        println!("  {result}");
    }

    if !all_passed {
        return Err(anyhow::anyhow!(
            "{} out of {} benchmark YAML ground_truth validations failed",
            total_benchmarks - passed_benchmarks,
            total_benchmarks
        ));
    }

    println!("\n🎉 ALL benchmark YAML files have valid ground_truth data!");
    Ok(())
}

/// Test a single benchmark YAML file using only ground_truth (no surfpool)
async fn test_single_benchmark_yaml_only(benchmark_path: &Path) -> Result<()> {
    let filename = benchmark_path.file_name().unwrap().to_str().unwrap();

    // Load and parse benchmark YAML
    let benchmark_content = fs::read_to_string(benchmark_path)
        .with_context(|| format!("Failed to read benchmark file: {filename}"))?;

    let benchmark_yaml: serde_yaml::Value = serde_yaml::from_str(&benchmark_content)
        .with_context(|| format!("Failed to parse YAML: {filename}"))?;

    // Extract and validate required fields
    let benchmark_id = benchmark_yaml
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing required 'id' field"))?;

    let description = benchmark_yaml
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("No description");

    println!("🔍 ID: {benchmark_id}");
    println!("📝 Description: {description}");

    // Validate initial_state
    let initial_state = extract_and_validate_initial_state(&benchmark_yaml, filename)?;
    println!(
        "🏗️  Initial State: {} accounts defined",
        initial_state.len()
    );

    // Validate ground_truth
    let ground_truth = extract_and_validate_ground_truth(&benchmark_yaml, filename)?;
    println!("✅ Ground Truth: Validated");

    // Create mock context using only YAML data (no surfpool)
    let mock_context = create_context_from_yaml_only(&initial_state, &ground_truth)?;
    println!("🔧 Mock Context: Created from YAML only");

    // Validate the mock context
    validate_mock_context(&mock_context, &initial_state)?;
    println!("✅ Context Validation: PASSED");

    // Test LLM prompt preparation using only context data
    let llm_prompt = prepare_llm_prompt_from_yaml(&benchmark_yaml, &mock_context)?;
    println!(
        "📋 LLM Prompt: Prepared ({len} chars)",
        len = llm_prompt.len()
    );

    // Validate prompt contains essential elements
    validate_llm_prompt(&llm_prompt, &mock_context, filename)?;
    println!("✅ LLM Prompt Validation: PASSED");

    // For multi-step benchmarks, validate step dependencies
    if let Some(flow) = benchmark_yaml.get("flow") {
        validate_flow_dependencies(flow, filename)?;
        println!("✅ Flow Dependencies: PASSED");
    }

    println!("🎯 {filename} - All YAML-only validations completed successfully");
    Ok(())
}

/// Extract and validate initial_state from YAML
fn extract_and_validate_initial_state(
    benchmark_yaml: &serde_yaml::Value,
    filename: &str,
) -> Result<Vec<InitialState>> {
    let initial_state_yaml = benchmark_yaml
        .get("initial_state")
        .ok_or_else(|| anyhow::anyhow!("Missing 'initial_state' field in {filename}"))?
        .as_sequence()
        .ok_or_else(|| anyhow::anyhow!("'initial_state' must be an array in {filename}"))?;

    let mut initial_state = Vec::new();

    for (i, state_item) in initial_state_yaml.iter().enumerate() {
        let state_map = state_item
            .as_mapping()
            .ok_or_else(|| anyhow::anyhow!("initial_state[{i}] must be an object in {filename}"))?;

        // Required fields
        let pubkey = state_map
            .get("pubkey")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("initial_state[{i}] missing 'pubkey' in {filename}"))?
            .to_string();

        let owner = state_map
            .get("owner")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("initial_state[{i}] missing 'owner' in {filename}"))?
            .to_string();

        let lamports = state_map
            .get("lamports")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        // Optional data field
        let data = state_map
            .get("data")
            .map(|v| serde_json::to_string(v).unwrap_or_default());

        initial_state.push(InitialState {
            pubkey,
            owner,
            lamports,
            data,
        });
    }

    if initial_state.is_empty() {
        return Err(anyhow::anyhow!(
            "initial_state cannot be empty in {filename}"
        ));
    }

    Ok(initial_state)
}

/// Extract and validate ground_truth from YAML
fn extract_and_validate_ground_truth(
    benchmark_yaml: &serde_yaml::Value,
    filename: &str,
) -> Result<serde_json::Value> {
    let ground_truth = benchmark_yaml
        .get("ground_truth")
        .ok_or_else(|| anyhow::anyhow!("Missing 'ground_truth' field in {filename}"))?;

    // Convert to JSON for easier validation
    let ground_truth_json = serde_json::to_value(ground_truth)
        .with_context(|| format!("Failed to convert ground_truth to JSON in {filename}"))?;

    // Validate ground_truth structure
    if let Some(obj) = ground_truth_json.as_object() {
        // Check for expected fields
        if obj.contains_key("final_state_assertions") {
            let assertions = obj
                .get("final_state_assertions")
                .and_then(|v| v.as_array())
                .ok_or_else(|| {
                    anyhow::anyhow!("final_state_assertions must be an array in {filename}")
                })?;

            println!("   📊 Final State Assertions: {} defined", assertions.len());
        }

        if obj.contains_key("expected_instructions") {
            let instructions = obj
                .get("expected_instructions")
                .and_then(|v| v.as_array())
                .ok_or_else(|| {
                    anyhow::anyhow!("expected_instructions must be an array in {filename}")
                })?;

            println!(
                "   🔧 Expected Instructions: {} defined",
                instructions.len()
            );
        }

        if let Some(min_score) = obj.get("min_score").and_then(|v| v.as_f64()) {
            println!("   🎯 Min Score: {min_score:.1}");
        }
    }

    Ok(ground_truth_json)
}

/// Create context using only YAML data (no external dependencies)
fn create_context_from_yaml_only(
    initial_state: &[InitialState],
    ground_truth: &serde_json::Value,
) -> Result<AgentContext> {
    let mut key_map = HashMap::new();
    let mut account_states = HashMap::new();

    // Process initial state to create key_map and account_states
    for state in initial_state {
        let pubkey_str = &state.pubkey;

        // Check if it's a placeholder or a literal pubkey
        if solana_sdk::pubkey::Pubkey::from_str(pubkey_str).is_err() {
            // It's a placeholder, generate a mock address
            if !key_map.contains_key(pubkey_str) {
                let mock_pubkey = generate_mock_pubkey_for_placeholder(pubkey_str);
                key_map.insert(pubkey_str.clone(), mock_pubkey.clone());

                println!("   🔄 Resolved '{pubkey_str}' -> '{mock_pubkey}'");
            }
        } else {
            // It's a literal pubkey, use as-is
            if !key_map.contains_key(pubkey_str) {
                key_map.insert(pubkey_str.clone(), pubkey_str.clone());
            }
        }

        // Create account state from YAML data
        let account_state = json!({
            "lamports": state.lamports,
            "owner": state.owner,
            "exists": true,
            "data": state.data.as_deref().unwrap_or("")
        });

        let key_for_state = key_map.get(pubkey_str).unwrap_or(pubkey_str);
        account_states.insert(key_for_state.clone(), account_state);
    }

    // Apply ground_truth to create final expected state
    if let Some(assertions) = ground_truth
        .get("final_state_assertions")
        .and_then(|v| v.as_array())
    {
        println!(
            "   📋 Applying {} ground truth assertions",
            assertions.len()
        );

        for assertion in assertions {
            if let Some(obj) = assertion.as_object() {
                apply_ground_truth_assertion(&mut account_states, obj, &key_map)?;
            }
        }
    }

    Ok(AgentContext {
        key_map,
        account_states,
        fee_payer_placeholder: Some("USER_WALLET_PUBKEY".to_string()),
        current_step: Some(0),
        step_results: HashMap::new(),
    })
}

/// Generate mock pubkey for placeholder
fn generate_mock_pubkey_for_placeholder(placeholder: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // Generate deterministic mock address based on placeholder name
    let mut hasher = DefaultHasher::new();
    placeholder.hash(&mut hasher);
    let hash = hasher.finish();

    // Create a mock pubkey (not a real Solana address, but consistent)
    format!("MockAddr_{}_{}", placeholder, hash % 1000000)
}

/// Apply ground truth assertion to account states
fn apply_ground_truth_assertion(
    account_states: &mut HashMap<String, serde_json::Value>,
    assertion: &serde_json::Map<String, serde_json::Value>,
    key_map: &HashMap<String, String>,
) -> Result<()> {
    if let Some(pubkey) = assertion.get("pubkey").and_then(|v| v.as_str()) {
        let resolved_pubkey = key_map
            .get(pubkey)
            .cloned()
            .unwrap_or_else(|| pubkey.to_string());

        if let Some(account_state) = account_states.get_mut(&resolved_pubkey) {
            if let Some(obj) = account_state.as_object_mut() {
                // Apply expected lamports if specified
                if let Some(expected_lamports) = assertion.get("expected_lamportas") {
                    obj.insert("lamports".to_string(), expected_lamports.clone());
                    println!("     💰 Set lamports: {expected_lamports}");
                }

                // Apply expected amount if specified (for token accounts)
                if let Some(expected_amount) = assertion.get("expected_amount") {
                    obj.insert("amount".to_string(), expected_amount.clone());
                    println!("     🪙 Set amount: {expected_amount}");
                }

                // Apply exists flag if specified
                if let Some(exists) = assertion.get("exists") {
                    obj.insert("exists".to_string(), exists.clone());
                }
            }
        }
    }

    Ok(())
}

/// Validate mock context
fn validate_mock_context(context: &AgentContext, initial_state: &[InitialState]) -> Result<()> {
    // Ensure all initial placeholders are resolved
    for state in initial_state {
        let resolved_key = context
            .key_map
            .get(&state.pubkey)
            .ok_or_else(|| anyhow::anyhow!("Failed to resolve placeholder: {}", state.pubkey))?;

        // Ensure account state exists for resolved key
        if !context.account_states.contains_key(resolved_key) {
            return Err(anyhow::anyhow!(
                "Missing account state for resolved key: {resolved_key}"
            ));
        }
    }

    // Ensure fee payer is specified
    if context.fee_payer_placeholder.is_none() {
        return Err(anyhow::anyhow!("Missing fee_payer_placeholder in context"));
    }

    // Validate all resolved addresses are consistent format
    for (placeholder, address) in &context.key_map {
        if address.is_empty() {
            return Err(anyhow::anyhow!(
                "Empty address for placeholder: {placeholder}"
            ));
        }
    }

    Ok(())
}

/// Prepare LLM prompt using only YAML context data
fn prepare_llm_prompt_from_yaml(
    benchmark_yaml: &serde_yaml::Value,
    context: &AgentContext,
) -> Result<String> {
    let prompt = benchmark_yaml
        .get("prompt")
        .and_then(|v| v.as_str())
        .unwrap_or("No prompt specified");

    let resolver = ContextResolver::new(solana_client::rpc_client::RpcClient::new(
        "http://mock:8899",
    ));

    // Generate YAML context
    let context_yaml = resolver.context_to_yaml_with_comments(context)?;

    // Format as LLM prompt (same format as production)
    let llm_prompt = format!(
        "---\n\n# On-Chain Context for Transaction Processing\n{context_yaml}\n\n---\n\n# Request\n{prompt}"
    );

    Ok(llm_prompt)
}

/// Validate LLM prompt contains essential elements
fn validate_llm_prompt(llm_prompt: &str, context: &AgentContext, filename: &str) -> Result<()> {
    // Check prompt is not empty
    if llm_prompt.trim().is_empty() {
        return Err(anyhow::anyhow!("LLM prompt is empty in {filename}"));
    }

    // Check context section is present
    if !llm_prompt.contains("On-Chain Context") {
        return Err(anyhow::anyhow!(
            "Missing context section in LLM prompt for {filename}"
        ));
    }

    // Check key_map is included
    if !llm_prompt.contains("key_map:") {
        return Err(anyhow::anyhow!(
            "Missing key_map in LLM prompt for {filename}"
        ));
    }

    // Check account states are included
    if !llm_prompt.contains("account_states:") {
        return Err(anyhow::anyhow!(
            "Missing account_states in LLM prompt for {filename}"
        ));
    }

    // Check request section is present
    if !llm_prompt.contains("# Request") {
        return Err(anyhow::anyhow!(
            "Missing request section in LLM prompt for {filename}"
        ));
    }

    // Validate essential placeholders are resolved
    for placeholder in &["USER_WALLET_PUBKEY"] {
        if !llm_prompt.contains(placeholder) && context.key_map.contains_key(*placeholder) {
            return Err(anyhow::anyhow!(
                "Placeholder {placeholder} not properly resolved in LLM prompt for {filename}"
            ));
        }
    }

    Ok(())
}

/// Validate flow dependencies for multi-step benchmarks
fn validate_flow_dependencies(flow: &serde_yaml::Value, filename: &str) -> Result<()> {
    let flow_steps = flow
        .as_sequence()
        .ok_or_else(|| anyhow::anyhow!("flow must be an array in {filename}"))?;

    println!("🔄 Multi-step Flow: {} steps", flow_steps.len());

    for (i, step) in flow_steps.iter().enumerate() {
        let step_map = step
            .as_mapping()
            .ok_or_else(|| anyhow::anyhow!("flow[{i}] must be an object in {filename}"))?;

        // Check required fields
        if !step_map.contains_key("step") {
            return Err(anyhow::anyhow!(
                "flow[{i}] missing 'step' field in {filename}"
            ));
        }

        if !step_map.contains_key("description") {
            return Err(anyhow::anyhow!(
                "flow[{i}] missing 'description' field in {filename}"
            ));
        }

        if !step_map.contains_key("prompt") {
            return Err(anyhow::anyhow!(
                "flow[{i}] missing 'prompt' field in {filename}"
            ));
        }

        // Validate dependencies if present
        if let Some(depends_on) = step_map.get("depends_on") {
            let deps = depends_on.as_sequence().ok_or_else(|| {
                anyhow::anyhow!("flow[{i}] depends_on must be an array in {filename}")
            })?;

            for dep in deps {
                let dep_str = dep.as_str().ok_or_else(|| {
                    anyhow::anyhow!("flow[{i}] dependency must be a string in {filename}")
                })?;

                // Check if dependency exists (basic validation)
                if dep_str.is_empty() {
                    return Err(anyhow::anyhow!(
                        "flow[{i}] has empty dependency in {filename}"
                    ));
                }
            }
        }

        println!("   ✅ Step {}: Validated", i + 1);
    }

    Ok(())
}
