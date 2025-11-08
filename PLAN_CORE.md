# Reev Core Architecture Plan

## 🎯 **Core Principles**

- **Start from ground up** - no migrations, fresh database schema
- **All prompts in YML** - versionable, parseable, debuggable
- **No JSON in database** - structured and typed fields only
- **Complete audit trail** - every state, prompt, and result stored
- **Real verification** - transaction hashes and on-chain verification
- **UUIDv7 for request tracking** - time-sortable IDs

## 📋 **Database Schema (Ground Up)**

```sql
-- Core request tracking
CREATE TABLE requests (
    request_id TEXT PRIMARY KEY,           -- UUIDv7 (time-sortable)
    user_prompt TEXT NOT NULL,
    user_wallet_pubkey TEXT NOT NULL,       -- Original placeholder
    resolved_wallet_pubkey TEXT NOT NULL,    -- Generated/filled wallet
    status TEXT NOT NULL DEFAULT 'running', -- running/completed/failed
    created_at INTEGER DEFAULT CURRENT_TIMESTAMP,
    updated_at INTEGER DEFAULT CURRENT_TIMESTAMP
);

-- Prompt management (all prompts stored for audit)
CREATE TABLE prompts (
    prompt_id INTEGER PRIMARY KEY AUTOINCREMENT,
    request_id TEXT NOT NULL,
    step_number INTEGER NOT NULL,
    prompt_type TEXT NOT NULL,              -- refinement/tool_execution/context_building
    prompt_content TEXT NOT NULL,          -- YML format
    template_used TEXT,                     -- Reference to prompt template file
    created_at INTEGER DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (request_id) REFERENCES requests(request_id)
);







-- Tool execution tracking (ONE ROW PER TOOL CALL)
CREATE TABLE tool_executions (
    execution_id INTEGER PRIMARY KEY AUTOINCREMENT,
    request_id TEXT NOT NULL,
    step_number INTEGER NOT NULL,
    tool_name TEXT NOT NULL,
    tool_params TEXT NOT NULL,              -- YML format
    llm_response TEXT NOT NULL,             -- Raw LLM response
    refined_prompt_id INTEGER,              -- Reference to prompt used
    execution_status TEXT NOT NULL,         -- pending/executing/verified/failed
    jupiter_tx_hash TEXT,                   -- Transaction hash from Jupiter protocol
    surfpool_tx_hash TEXT,                  -- Transaction hash from SurfPool executor
    execution_result TEXT,                  -- YML format
    verification_status TEXT,               -- verified/unverified/failed
    verification_details TEXT,             -- YML format with verification data
    wallet_context TEXT,                    -- YML format - wallet state before execution
    updated_wallet_context TEXT,            -- YML format - wallet state after execution
    execution_time_ms INTEGER,
    error_message TEXT,
    created_at INTEGER DEFAULT CURRENT_TIMESTAMP,
    updated_at INTEGER DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (request_id) REFERENCES requests(request_id),
    FOREIGN KEY (refined_prompt_id) REFERENCES prompts(prompt_id)
);



-- Error tracking for debugging
CREATE TABLE execution_errors (
    error_id INTEGER PRIMARY KEY AUTOINCREMENT,
    request_id TEXT NOT NULL,
    step_number INTEGER,
    execution_id INTEGER,
    error_type TEXT NOT NULL,               -- prompt_refinement/tool_execution/verification/context_building
    error_code TEXT NOT NULL,               -- Specific error code
    error_message TEXT NOT NULL,
    error_details TEXT,                     -- YML format with additional context
    recovery_attempted BOOLEAN DEFAULT FALSE,
    created_at INTEGER DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (request_id) REFERENCES requests(request_id),
    FOREIGN KEY (execution_id) REFERENCES tool_executions(execution_id)
);

-- Indexes for performance
CREATE INDEX idx_requests_created_at ON requests(created_at);

CREATE INDEX idx_tool_executions_request_id ON tool_executions(request_id);
CREATE INDEX idx_tool_executions_status ON tool_executions(execution_status);
CREATE INDEX idx_prompts_request_id ON prompts(request_id);

CREATE INDEX idx_execution_errors_request_id ON execution_errors(request_id);
```

## 🔄 **18-Step Core Flow (Pseudo-Code)**

### **Step 1: User Prompt Input & Request Initialization**
```rust
// Input: user_prompt = "use my 50% sol to multiply usdc 1.5x on jup"
// Output: request_id (UUIDv7)

async fn initialize_request(user_prompt: &str, user_wallet_pubkey: &str) -> Result<String> {
    let request_id = generate_uuidv7(); // Time-sortable UUID
    let status = "running";
    
    // Store initial request
    db.execute("INSERT INTO requests (request_id, user_prompt, user_wallet_pubkey, status) 
                VALUES (?, ?, ?, ?)", 
               [request_id, user_prompt, user_wallet_pubkey, status])?;
    
    // Store user prompt in prompts table for audit
    db.execute("INSERT INTO prompts (request_id, step_number, prompt_type, prompt_content) 
                VALUES (?, 0, 'user_input', ?)",
               [request_id, user_prompt])?;
    
    Ok(request_id)
}
```

### **Step 2: Wallet Detection & Resolution**
```rust
// Input: user_wallet_pubkey = "USER_WALLET_PUBKEY"
// Output: resolved_wallet_pubkey (generated/filled)

async fn resolve_wallet_address(request_id: &str, user_wallet_pubkey: &str) -> Result<String> {
    let resolved_wallet = if user_wallet_pubkey.contains("USER_WALLET_PUBKEY") {
        generate_filled_test_wallet().await? // Generate wallet with pre-filled SOL
    } else {
        user_wallet_pubkey.to_string() // Use provided wallet
    };
    
    // Update request with resolved wallet
    db.execute("UPDATE requests SET resolved_wallet_pubkey = ? WHERE request_id = ?",
               [resolved_wallet, request_id])?;
    
    Ok(resolved_wallet)
}
```

### **Step 3: Entry Wallet State Recording**
```rust
// Input: resolved_wallet_pubkey, request_id
// Output: wallet_state record with token pricing

async fn record_entry_wallet_state(request_id: &str, wallet_pubkey: &str) -> Result<WalletState> {
    // Get current token prices (fresh data, no caching)
        let sol_price = fetch_token_price("So11111111111111111111111111111111111111112").await?;
        let usdc_price = 1.0; // USDC is stablecoin
    
    // Get wallet balances
    let sol_balance = get_token_balance(wallet_pubkey, SOL_MINT).await?;
    let usdc_balance = get_token_balance(wallet_pubkey, USDC_MINT).await?;
    
    let sol_usd_value = sol_balance * sol_price;
    let usdc_usd_value = usdc_balance * usdc_price;
    let total_usd_value = sol_usd_value + usdc_usd_value;
    
    // No separate wallet_states table - context stored in tool_executions
    
    // No token price caching - fetch fresh data each time
    
    Ok(WalletState {
        sol_amount: sol_balance,
        usdc_amount: usdc_balance,
        sol_usd_value,
        usdc_usd_value,
        total_usd_value
    })
}
```

### **Step 4: Tool Context Collection**
```rust
// Input: None (system-wide)
// Output: tool_context (YML format)

async fn get_tool_context() -> Result<String> {
    // Tool definitions loaded from code, not database
    let tools = get_tool_definitions_from_code(); // Load from Rust code
    
    let mut tool_context = String::new();
    tool_context.push_str("available_tools:\n");
    
    for tool in tools {
        tool_context.push_str(&format!("  {}:\n", tool.tool_name));
        tool_context.push_str(&format!("    description: \"{}\"\n", tool.description));
        tool_context.push_str(&format!("    parameters: {}\n", tool.parameters));
    }
    
    Ok(tool_context) // YML format
}
```

### **Step 5: Prompt Refinement Preparation**
```rust
// Input: tool_context
// Output: refined_instruct (YML format)

async fn prepare_refinement_instructions(tool_context: &str) -> Result<String> {
    // Load refinement prompt template from YML file
    let template = load_yml_file("prompts/refine_user_prompt.yml")?;
    
    // Prepare complete refinement instructions
    let refined_instruct = format!(
        "refinement_instructions:\n  task: \"refine user prompt to match tools description\"\n  tool_context: |\n    {}\n  requirements:\n    - generate sequence of executable tool calls\n    - use current wallet amounts and prices\n    - target goal must be achievable with available tools\n    - each call must match tool description exactly\n",
        tool_context
    );
    
    Ok(refined_instruct) // YML format
}
```

### **Step 6: LLM Prompt Refinement**
```rust
// Input: token_context, refined_instruct, user_prompt
// Output: refined_prompt_series (YML format)

async fn refine_user_prompt_with_llm(
    request_id: &str,
    token_context: &str,
    refined_instruct: &str,
    user_prompt: &str
) -> Result<Vec<RefinedPrompt>> {
    // Build complete LLM request
    let full_prompt = format!(
        "prompt_refinement_request:\n  user_prompt: \"{}\"\n  token_context: |\n    {}\n  {}\n\nGenerate refined prompt series:",
        user_prompt, token_context, refined_instruct
    );
    
    // Store refinement prompt for audit
    db.execute("INSERT INTO prompts (request_id, step_number, prompt_type, prompt_content, template_used)
                VALUES (?, 1, 'refinement', ?, 'prompts/refine_user_prompt.yml')",
               [request_id, full_prompt])?;
    
    // Call LLM
    let llm_response = call_llm_with_timeout("glm-4.6-coding", &full_prompt, 30000).await?;
    
    // Store LLM response
    db.execute("INSERT INTO prompts (request_id, step_number, prompt_type, prompt_content)
                VALUES (?, 2, 'refinement_response', ?)",
               [request_id, llm_response])?;
    
    // Parse LLM response into structured prompts
    let refined_prompts = parse_refined_prompt_series(&llm_response)?;
    
    Ok(refined_prompts)
}
```

### **Step 7: LLM Response Parsing**
```rust
// Input: LLM response from Step 6
// Output: prompt_series (structured data)

async fn parse_refined_prompt_series(llm_response: &str) -> Result<Vec<RefinedPrompt>> {
    // Expected format from LLM:
    // refined_prompt_series:
    //   - step: 1
    //     prompt: "swap 0.5 SOL to USDC using jupiter_swap"
    //     reasoning: "50% of 1 SOL = 0.5 SOL × $161.50 = $80.75 USDC"
    //     expected_tool: "jupiter_swap"
    //   - step: 2
    //     prompt: "lend 90.75 USDC to jupiter using jupiter_lend"
    //     reasoning: "Current 10 USDC + swapped 80.75 = 90.75 USDC to lend"
    //     expected_tool: "jupiter_lend"
    
    let parsed_response: PromptSeriesResponse = serde_yaml::from_str(llm_response)?;
    Ok(parsed_response.refined_prompt_series)
}
```

### **Step 8: Tool Execution Manager Initialization**
```rust
// Input: token_context, prompt_series
// Output: execution_manager initialized

struct ExecutionManager {
    request_id: String,
    current_context: WalletContext,
    prompt_series: Vec<RefinedPrompt>,
    step_number: usize,
}

async fn initialize_execution_manager(
    request_id: &str,
    token_context: &str,
    prompt_series: Vec<RefinedPrompt>
) -> Result<ExecutionManager> {
    Ok(ExecutionManager {
        request_id: request_id.to_string(),
        current_context: parse_wallet_context(token_context)?,
        prompt_series,
        step_number: 0,
    })
}
```

### **Step 9: LLM Tool Calling Preparation**
```rust
// Input: execution_manager, step_index
// Output: tool_name, tool_params

async fn prepare_tool_execution(
    manager: &mut ExecutionManager,
    step_index: usize
) -> Result<(String, String)> {
    let refined_prompt = &manager.prompt_series[step_index];
    
    // Build tool calling prompt with current context
    let tool_calling_prompt = format!(
        "tool_execution_request:\n  current_wallet_context: |\n    {}\n  task: \"{}\"\n  expected_tool: \"{}\"\n\nExecute this tool call with proper parameters:",
        serialize_wallet_context(&manager.current_context),
        refined_prompt.prompt,
        refined_prompt.expected_tool
    );
    
    // Store tool execution prompt
    db.execute("INSERT INTO prompts (request_id, step_number, prompt_type, prompt_content)
                VALUES (?, ?, 'tool_execution', ?)",
               [manager.request_id, step_index + 3, tool_calling_prompt])?;
    
    // Call LLM for tool calling
    let llm_response = call_llm_with_timeout("glm-4.6-coding", &tool_calling_prompt, 30000).await?;
    
    // Parse tool calling response
    let (tool_name, tool_params) = parse_tool_calling_response(&llm_response)?;
    
    Ok((tool_name, tool_params))
}
```

### **Step 10: Tool Parameter Recording**
```rust
// Input: request_id, step_number, tool_name, tool_params, wallet_context
// Output: execution_id

async fn record_tool_execution_request(
    request_id: &str,
    step_number: usize,
    tool_name: &str,
    tool_params: &str,
    wallet_context: &str
) -> Result<i64> {
    // Get refined prompt ID
    let refined_prompt_id = db.query_one("SELECT prompt_id FROM prompts 
                                         WHERE request_id = ? AND step_number = ? 
                                         AND prompt_type = 'tool_execution'",
                                        [request_id, step_number + 3])?.prompt_id;
    
    // Create tool execution record with wallet context
    db.execute("INSERT INTO tool_executions 
                (request_id, step_number, tool_name, tool_params, refined_prompt_id, 
                 execution_status, wallet_context)
                VALUES (?, ?, ?, ?, ?, 'pending', ?)",
               [request_id, step_number, tool_name, tool_params, refined_prompt_id, wallet_context])?;
    
    let execution_id = db.last_insert_rowid();
    Ok(execution_id)
}
```

### **Step 11: Tool Execution with Token Context**
```rust
// Input: tool_name, tool_params, token_context
// Output: jupiter_tx_hash

async fn execute_tool_with_context(
    execution_id: i64,
    tool_name: &str,
    tool_params: &str,
    token_context: &str
) -> Result<String> {
    // Parse tool parameters from YML
    let params: ToolParameters = serde_yaml::from_str(tool_params)?;
    
    // Enrich parameters with token context
    let enriched_params = enrich_parameters_with_token_context(params, token_context)?;
    
    // Execute tool via Jupiter protocol
        let jupiter_response = call_jupiter_protocol(tool_name, &enriched_params).await?;
    
        let tx_hash = jupiter_response.transaction_hash;
    
    // Update execution record
    db.execute("UPDATE tool_executions 
                SET jupiter_tx_hash = ?, execution_status = 'executing'
                WHERE execution_id = ?",
               [tx_hash, execution_id])?;
    
    Ok(tx_hash)
}
```

### **Step 12: Jupiter Transaction Recording**
```rust
// Input: execution_id, jupiter_tx_hash
// Output: tx_details stored

async fn record_jupiter_transaction(
    execution_id: i64,
    jupiter_tx_hash: &str
) -> Result<()> {
    // Get transaction details from Jupiter
    let tx_details = get_jupiter_transaction_details(jupiter_tx_hash).await?;
    
    // Store transaction details in YML format
    let tx_details_yml = serde_yaml::to_string(&tx_details)?;
    
    // Update execution record with transaction details
        db.execute("UPDATE tool_executions 
                    SET execution_result = ? 
                    WHERE execution_id = ?",
                   [tx_details_yml, execution_id])?;
    
    Ok(())
}
```

### **Step 13: SurfPool Transaction Processing**
```rust
// Input: jupiter_tx_hash
// Output: surfpool_tx_hash

async fn process_with_surfpool(jupiter_tx_hash: &str) -> Result<String> {
    // Submit Jupiter transaction to SurfPool executor
        let surfpool_response = execute_with_surfpool(jupiter_tx_hash).await?;
    
    let surfpool_tx_hash = surfpool_response.transaction_hash;
    
    // Update execution record
    db.execute("UPDATE tool_executions 
                SET surfpool_tx_hash = ? 
                WHERE jupiter_tx_hash = ?",
               [surfpool_tx_hash, jupiter_tx_hash])?;
    
    Ok(surfpool_tx_hash)
}
```

### **Step 14: Execution Result Collection**
```rust
// Input: surfpool_tx_hash
// Output: execution results (success/failure details)

async fn collect_execution_results(
    execution_id: i64,
    surfpool_tx_hash: &str
) -> Result<ExecutionResult> {
    // Get transaction status from SurfPool executor
        let surfpool_status = get_surfpool_execution_status(surfpool_tx_hash).await?;
    
    // Verify transaction on-chain
    let verification_result = verify_transaction_on_chain(surfpool_tx_hash).await?;
    
    // Build execution result
    let execution_result = ExecutionResult {
        success: surfpool_status.success && verification_result.verified,
        transaction_details: surfpool_status.details,
        verification_details: verification_result,
        execution_time_ms: surfpool_status.execution_time_ms,
    };
    
    // Store execution results in YML format
    let result_yml = serde_yaml::to_string(&execution_result)?;
    let verification_status = if verification_result.verified { "verified" } else { "unverified" };
    
    // Update execution record with updated wallet context
    let updated_wallet_context = build_wallet_context_yml(&current_wallet_state.wallet_pubkey).await?;
    
    db.execute("UPDATE tool_executions 
                SET execution_status = ?, verification_status = ?, 
                    execution_result = ?, verification_details = ?, execution_time_ms = ?,
                    updated_wallet_context = ?
                WHERE execution_id = ?",
               ["completed", verification_status, result_yml, 
                serde_yaml::to_string(&verification_result)?, 
                surfpool_status.execution_time_ms, updated_wallet_context, execution_id])?;
    
    Ok(execution_result)
}
```

### **Step 15: Context Building for Next Step**
```rust
// Input: previous_wallet_state, current_tool_result, next_prompt
// Output: new_token_context (YML format)

async fn build_next_context(
    request_id: &str,
    previous_wallet_state: &WalletState,
    current_tool_result: &ExecutionResult,
    next_prompt: &str
) -> Result<String> {
    // Get updated wallet state after tool execution
    let updated_wallet_state = get_current_wallet_state(&current_wallet_state.wallet_pubkey).await?;
    
    // Build detailed context with before/after comparison
    let next_context = format!(
        "wallet_context_update:\n  step_number: {}\n  tool_executed: \"{}\"\n  execution_success: {}\n  \n  previous_wallet_state:\n    sol_amount: {}\n    usdc_amount: {}\n    total_usd_value: {}\n  \n  current_wallet_state:\n    sol_amount: {}\n    usdc_amount: {}\n    total_usd_value: {}\n  \n  changes:\n    sol_delta: {}\n    usdc_delta: {}\n    value_delta: {}\n  \n  next_task: \"{}\"\n  comment: \"{}\"",
        previous_wallet_state.step_number + 1,
        current_tool_result.tool_name,
        current_tool_result.success,
        previous_wallet_state.sol_amount,
        previous_wallet_state.usdc_amount,
        previous_wallet_state.total_usd_value,
        updated_wallet_state.sol_amount,
        updated_wallet_state.usdc_amount,
        updated_wallet_state.total_usd_value,
        updated_wallet_state.sol_amount - previous_wallet_state.sol_amount,
        updated_wallet_state.usdc_amount - previous_wallet_state.usdc_amount,
        updated_wallet_state.total_usd_value - previous_wallet_state.total_usd_value,
        next_prompt,
        generate_context_comment(previous_wallet_state, &updated_wallet_state, current_tool_result)
    );
    
    // No separate execution_contexts table - context stored in tool_executions
    
    Ok(next_context) // YML format
}

// Helper function to generate meaningful comments for LLM understanding
fn generate_context_comment(
    previous: &WalletState,
    current: &WalletState,
    tool_result: &ExecutionResult
) -> String {
    match tool_result.tool_name.as_str() {
        "jupiter_swap" => {
            if tool_result.success {
                format!("Successfully swapped {:.6} SOL for {:.2} USDC. Total USDC available: {:.2}",
                       previous.sol_amount - current.sol_amount,
                       current.usdc_amount - previous.usdc_amount,
                       current.usdc_amount)
            } else {
                "Swap failed, wallet state unchanged".to_string()
            }
        }
        "jupiter_lend" => {
            if tool_result.success {
                format!("Successfully lent {:.2} USDC to Jupiter lending protocol at current APY",
                       tool_result.parameters.usdc_amount)
            } else {
                "Lending failed, check available USDC balance".to_string()
            }
        }
        _ => format!("Executed {} with result: {}", tool_result.tool_name, tool_result.success)
    }
}
```

### **Step 16: Step-by-Step Repetition Loop**
```rust
// Input: execution_manager, prompt_series length
// Output: all steps executed or failed

async fn execute_prompt_series(
    mut manager: ExecutionManager
) -> Result<Vec<ExecutionResult>> {
    let mut all_results = Vec::new();
    
    while manager.step_number < manager.prompt_series.len() {
        match execute_single_step(&mut manager).await {
            Ok(result) => {
                all_results.push(result);
                manager.step_number += 1;
                
                // Update manager's context for next step
                let next_context = build_next_context(
                    &manager.request_id,
                    &manager.current_context,
                    &result,
                    &manager.prompt_series[manager.step_number].prompt
                ).await?;
                
                manager.current_context = parse_wallet_context(&next_context)?;
            }
            Err(e) => {
                // Record error and break
                record_execution_error(&manager.request_id, manager.step_number, e).await?;
                break;
            }
        }
    }
    
    Ok(all_results)
}

async fn execute_single_step(manager: &mut ExecutionManager) -> Result<ExecutionResult> {
    let step_index = manager.step_number;
    let refined_prompt = &manager.prompt_series[step_index];
    
    // Step 9: Prepare tool execution
    let (tool_name, tool_params) = prepare_tool_execution(
        manager, step_index
    ).await?;
    
    // Step 10: Record tool execution request
    let execution_id = record_tool_execution_request(
        &manager.request_id,
        step_index,
        &tool_name,
        &tool_params
    ).await?;
    
    // Step 11: Execute tool with context
    let jupiter_tx_hash = execute_tool_with_context(
        execution_id,
        &tool_name,
        &tool_params,
        &serialize_wallet_context(&manager.current_context)
    ).await?;
    
    // Step 12: Record Jupiter transaction
    record_jupiter_transaction(execution_id, &jupiter_tx_hash).await?;
    
    // Step 13: Process with SurfPool
    let surfpool_tx_hash = process_with_surfpool(&jupiter_tx_hash).await?;
    
    // Step 14: Collect results
    let execution_result = collect_execution_results(
        execution_id,
        &surfpool_tx_hash
    ).await?;
    
    Ok(execution_result)
}
```

### **Step 17: Error Handling & Recovery**
```rust
// Input: request_id, step_number, error
// Output: error recorded, recovery attempted if possible

async fn record_execution_error(
    request_id: &str,
    step_number: usize,
    error: anyhow::Error
) -> Result<()> {
    // Determine error type and code
    let (error_type, error_code, recovery_possible) = classify_error(&error);
    
    // Store error details in YML format
    let error_details = yaml!({
        "step_number": step_number,
        "error_chain": format!("{:?}", error.chain()),
        "root_cause": format!("{:?}", error.root_cause()),
        "context": "tool_execution_phase"
    });
    
    // Record error
    db.execute("INSERT INTO execution_errors 
                (request_id, step_number, error_type, error_code, 
                 error_message, error_details, recovery_attempted)
                VALUES (?, ?, ?, ?, ?, ?, ?)",
               [request_id, step_number, error_type, error_code,
                error.to_string(), serde_yaml::to_string(&error_details)?,
                recovery_possible])?;
    
    // Attempt recovery if possible
    if recovery_possible {
        attempt_error_recovery(request_id, step_number, &error).await?;
    }
    
    Ok(())
}

fn classify_error(error: &anyhow::Error) -> (String, String, bool) {
    let error_str = error.to_string().to_lowercase();
    
    if error_str.contains("insufficient") && error_str.contains("balance") {
        ("insufficient_balance", "BALANCE_ERROR", true).to_strings()
    } else if error_str.contains("timeout") {
        ("timeout", "TIMEOUT_ERROR", true).to_strings()
    } else if error_str.contains("network") || error_str.contains("rpc") {
        ("network", "NETWORK_ERROR", true).to_strings()
    } else if error_str.contains("invalid") && error_str.contains("parameter") {
        ("parameter", "INVALID_PARAMS", true).to_strings()
    } else {
        ("unknown", "UNKNOWN_ERROR", false).to_strings()
    }
}

async fn attempt_error_recovery(
    request_id: &str,
    step_number: usize,
    error: &anyhow::Error
) -> Result<()> {
    let error_str = error.to_string().to_lowercase();
    
    // Recovery strategies based on error type
    if error_str.contains("insufficient") && error_str.contains("balance") {
        // Try with reduced amount
        attempt_retry_with_reduced_amount(request_id, step_number).await?;
    } else if error_str.contains("timeout") {
        // Retry with longer timeout
        attempt_retry_with_longer_timeout(request_id, step_number).await?;
    }
    
    Ok(())
}
```

### **Step 18: Exit Wallet State Recording**
```rust
// Input: request_id, resolved_wallet_pubkey
// Output: final_wallet_state recorded

async fn record_exit_wallet_state(
    request_id: &str,
    wallet_pubkey: &str
) -> Result<WalletState> {
    // Get current token prices (fresh data)
        let sol_price = fetch_token_price(SOL_MINT).await?;
        let usdc_price = 1.0;
    
    // Get final wallet balances
    let sol_balance = get_token_balance(wallet_pubkey, SOL_MINT).await?;
    let usdc_balance = get_token_balance(wallet_pubkey, USDC_MINT).await?;
    
    let sol_usd_value = sol_balance * sol_price;
    let usdc_usd_value = usdc_balance * usdc_price;
    let total_usd_value = sol_usd_value + usdc_usd_value;
    
    // No separate wallet_states table - exit state in tool_executions
    
    // Update request status
    db.execute("UPDATE requests SET status = 'completed' WHERE request_id = ?", [request_id])?;
    
    Ok(WalletState {
        sol_amount: sol_balance,
        usdc_amount: usdc_balance,
        sol_usd_value,
        usdc_usd_value,
        total_usd_value
    })
}
```



## 📝 **YML Prompt Templates**

### `prompts/templates/refine_user_prompt.yml`
```yaml
task: "refine user prompt to match available tools"
context:
  user_wallet_state: !include "../current_wallet_state.yml"
  available_tools: !include "../tool_definitions.yml"
  current_prices: !include "../token_prices.yml"

requirements:
  - generate sequence of executable tool calls
  - use current wallet amounts and prices
  - each call must match tool description exactly
  - include reasoning for each step
  - ensure goal is achievable with available tools

output_format:
  refined_prompt_series:
    - step: <integer>
      prompt: "<exact prompt for LLM>"
      reasoning: "<detailed reasoning>"
      expected_tool: "<tool_name>"
      expected_outcome: "<expected result>"
```

### `prompts/templates/tool_execution.yml`
```yaml
task: "execute specific tool with given parameters"
context:
  current_wallet_state: !include "../current_wallet_state.yml"
  task_description: "<from refined prompt>"
  expected_tool: "<tool_name>"
  tool_parameters: "<tool parameters>"

requirements:
  - parse tool call from LLM response
  - validate parameters against tool definition
  - include all required parameters
  - use current wallet context for values

output_format:
  tool_call:
    tool_name: "<tool_name>"
    parameters:
      <parameter_name>: <parameter_value>
    reasoning: "<execution reasoning>"
```

## 🎯 **Key Benefits of This Architecture**

1. **Complete Audit Trail**: Every prompt, execution, and state change is stored
2. **Real Verification**: On-chain transaction verification with tx hashes
3. **Structured Data**: No JSON blobs, all YML format
4. **Time-Sortable**: UUIDv7 for natural chronological ordering
5. **Error Recovery**: Intelligent error classification and recovery attempts
6. **Debugging Ready**: All state information queryable from database
7. **Flow Visualization**: Easy to build flow diagrams from stored data
8. **Scoring Ready**: Complete execution data for performance scoring

## 🚀 **Optimized Schema Benefits**

**Removed Unnecessary Tables:**
- ❌ `token_prices` - Fresh data fetched on-demand, no caching complexity
- ❌ `wallet_states` - Context stored directly in `tool_executions` for scalability
- ❌ `tool_definitions` - Tool definitions managed in code, not database
- ❌ `execution_contexts` - Context data stored with tool execution records

**Enhanced tool_executions Table:**
- ✅ `wallet_context` - Wallet state before execution (YML)
- ✅ `updated_wallet_context` - Wallet state after execution (YML)
- ✅ Self-contained state tracking - No joins needed for context
- ✅ Scalable design - Context travels with execution data

**Performance & Simplicity:**
- ✅ Fewer tables = faster queries and simpler schema
- ✅ Fresh token prices = always current market data
- ✅ Code-based tools = easier versioning and deployment
- ✅ Context bundling = reduced JOIN complexity
- ✅ YML throughout = consistent data format across all fields

## 🏗️ **Simplified Component Understanding**

Based on existing crate structure and requirements:

### **Use Existing Crates:**
- **reev-db**: Database operations (SQLite with optimized schema)
- **reev-tools**: Tool implementations (13 tools with full OTEL coverage)
- **reev-context**: Context resolution and token information
- **reev-types**: Shared type definitions
- **reev-agent**: LLM service integration
- **surfpool**: Mainnet fork executor (NOT a tool)
- **uuid**: UUIDv7 generation (existing crate, no custom implementation needed)
- **serde_yaml**: YML processing (already used throughout project)
- **anyhow**: Error handling (existing pattern in project)

### **Reev-Core Focus Areas:**
- **Core orchestration**: 18-step flow implementation
- **Prompt management**: YML template system
- **Step execution**: Tool coordination using reev-tools
- **SurfPool integration**: Transaction execution on forked mainnet
- **Database operations**: Using reev-db with optimized schema

### **SurfPool Role:**
- **NOT a tool** - It's a Solana testnet executor
- **Mainnet fork** with on-demand account fetching
- **Cheat codes** for state manipulation (`surfnet_setTokenAccount`)
- **Transaction execution** environment, not a protocol tool

### **Execution Flow:**
1. Reev-core orchestrates 18-step flow
2. Uses reev-tools for tool calls (jupiter_swap, etc.)
3. Jupiter protocol returns transaction data
4. SurfPool executes transaction on forked mainnet
5. Results stored in reev-db with optimized schema

## 🏗️ **Minimal reev-core Structure**

```
reev-core/
├── Cargo.toml
├── src/
│   ├── lib.rs                    # Core library entry point
│   ├── main.rs                   # CLI interface (optional)
│   ├── orchestrator.rs           # 18-step flow implementation
│   ├── prompts/
│   │   ├── mod.rs
│   │   └── templates/           # YML prompt templates
│   │       ├── refine_user_prompt.yml
│   │       ├── tool_execution.yml
│   │       └── context_building.yml
│   └── executor/
│       ├── mod.rs
│       ├── surfpool.rs          # SurfPool integration
│       └── manager.rs          # Transaction execution manager
├── tests/
│   ├── integration/
│   └── unit/
└── examples/
    ├── simple_swap.rs
    └── lending_flow.rs
```

## 🚀 **Next Steps for Implementation**

1. **Create reev-core crate** with minimal structure above
2. **Implement orchestrator.rs** with 18-step flow using existing crates
3. **Add prompt template system** with YML files using serde_yaml
4. **Integrate SurfPool executor** using existing surfpool crate
5. **Build testing** with integration tests
6. **Create debugging interface** for state inspection
7. **Add flow visualization** using existing reev-flow

### **Dependencies Use Existing Crates:**
- `reev-db` - Database operations with optimized schema
- `reev-tools` - Tool implementations (jupiter_swap, etc.)
- `reev-context` - Token context and wallet resolution  
- `reev-agent` - LLM service integration
- `reev-types` - Shared type definitions
- `reev-flow` - Session management and OTEL integration
- `surfpool` - Mainnet fork executor
- `serde_yaml` - Already used throughout project
- `uuid` - For UUIDv7 generation (existing dependency)
- `anyhow` - Error handling (existing pattern)

### **Implementation Priority:**
- **Week 1**: Create reev-core structure + orchestrator.rs 18-step flow
- **Week 2**: Integrate existing crates (reev-tools, reev-context, reev-agent)
- **Week 3**: Add prompt system + LLM integration
- **Week 4**: Integrate SurfPool executor
- **Week 5**: Testing + debugging interface + flow visualization

This architecture provides a solid foundation for reliable, verifiable, and debuggable automated DeFi operations.