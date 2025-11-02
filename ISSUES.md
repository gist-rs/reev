# Issues

## Issue #1: Deterministic Agent Completely Bypasses OTEL Logging

**Priority**: 🔴 CRITICAL
**Status**: 🔄 Open
**Assigned**: Unassigned

**Description**: The deterministic agent (used by default in all examples) completely bypasses the tool system and calls protocol handlers directly. This means NO OTEL logs are generated when using the default agent.

**Impact**:
- Default usage (`--agent deterministic` or no agent specified) generates zero OTEL logs
- Users don't see any tool execution traces
- Mermaid diagrams show no tools
- Debugging information is missing

**Files Affected**:
- `reev-agent/src/agents/coding/d_001_sol_transfer.rs` - SOL transfer
- `reev-agent/src/agents/coding/d_100_jup_swap_sol_usdc.rs` - Jupiter swap
- `reev-agent/src/agents/coding/d_114_jup_positions_and_earnings.rs` - Positions/earnings
- All other `d_*.rs` files in coding directory

**Root Cause**: Direct protocol handler calls bypass tool system:
```rust
// ❌ NO OTEL - Direct protocol handler call
let instructions = protocol_handle_sol_transfer(from, to, lamports, key_map).await?;
```

**Proposed Solution**: Add enhanced logging macros to deterministic agent handlers:
```rust
use reev_flow::{log_tool_call, log_tool_completion};
use std::time::Instant;

pub(crate) async fn handle_sol_transfer(
    key_map: &HashMap<String, String>,
) -> Result<Vec<RawInstruction>> {
    let start_time = Instant::now();
    
    // 🎯 Add enhanced logging for deterministic agents
    let args = json!({
        "user_pubkey": key_map.get("USER_WALLET_PUBKEY"),
        "recipient_pubkey": key_map.get("RECIPIENT_WALLET_PUBKEY"),
        "lamports": 100_000_000
    });
    log_tool_call!("deterministic_sol_transfer", &args);
    
    // ... existing logic ...
    
    let instructions = protocol_handle_sol_transfer(from, to, lamports, key_map).await?;
    
    let execution_time = start_time.elapsed().as_millis() as u64;
    let result = json!({
        "instruction_count": instructions.len(),
        "lamports": lamports
    });
    
    log_tool_completion!("deterministic_sol_transfer", execution_time, &result, true);
    
    Ok(instructions)
}
```

**Acceptance Criteria**:
- [ ] Enhanced logging added to all deterministic agent functions
- [ ] OTEL logs generated when using `--agent deterministic`
- [ ] Mermaid diagrams show deterministic tool calls
- [ ] Test verification with example benchmarks

**Verification Steps**:
```bash
# Before fix: Should show NO OTEL logs
REEV_TRACE_FILE=traces_before.log RUST_LOG=info cargo run -p reev-agent -- examples/001-sol-transfer.yml --agent deterministic
grep -i "tool_call\|tool_completion" traces_before.log | wc -l  # Should be 0

# After fix: Should show full OTEL logs
REEV_TRACE_FILE=traces_after.log RUST_LOG=info cargo run -p reev-agent -- examples/001-sol-transfer.yml --agent deterministic
grep -i "tool_call\|tool_completion" traces_after.log | wc -l  # Should be >0
```

---

## Issue #2: Missing Enhanced Logging in Discovery Tools

**Priority**: 🟡 HIGH
**Status**: 🔄 Open
**Assigned**: Unassigned

**Description**: Discovery tools have `#[instrument]` attributes but are missing the enhanced logging macros (`log_tool_call!` and `log_tool_completion!`), reducing debugging visibility.

**Files Affected**:
- `reev-tools/src/tools/discovery/balance_tool.rs` - `get_account_balance`
- `reev-tools/src/tools/discovery/lend_earn_tokens.rs` - `get_lend_earn_tokens`
- `reev-tools/src/tools/discovery/position_tool.rs` - `get_position_info`

**Proposed Solution**: Add enhanced logging macros to each tool:
```rust
use reev_flow::{log_tool_call, log_tool_completion};
use std::time::Instant;

impl Tool for AccountBalanceTool {
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let start_time = Instant::now();
        
        // 🎯 Add enhanced logging
        log_tool_call!(Self::NAME, &args);
        
        info!("[AccountBalanceTool] Starting tool execution with OpenTelemetry tracing");
        
        // ... existing tool logic ...
        
        match result {
            Ok(output) => {
                let execution_time = start_time.elapsed().as_millis() as u64;
                log_tool_completion!(Self::NAME, execution_time, &output, true);
                info!("[AccountBalanceTool] Tool execution completed in {}ms", execution_time);
                Ok(output)
            }
            Err(e) => {
                let execution_time = start_time.elapsed().as_millis() as u64;
                let error_data = json!({"error": e.to_string()});
                log_tool_completion!(Self::NAME, execution_time, &error_data, false);
                error!("[AccountBalanceTool] Tool execution failed in {}ms: {}", execution_time, e);
                Err(e)
            }
        }
    }
}
```

**Acceptance Criteria**:
- [ ] Enhanced logging added to `get_account_balance` tool
- [ ] Enhanced logging added to `get_lend_earn_tokens` tool
- [ ] Enhanced logging added to `get_position_info` tool
- [ ] Test verification with local agent

---

## Issue #3: Missing Enhanced Logging in Flow Tools

**Priority**: 🟡 HIGH
**Status**: 🔄 Open
**Assigned**: Unassigned

**Description**: Flow tools are missing enhanced logging macros, reducing debugging visibility for multi-step flows.

**Files Affected**:
- `reev-tools/src/tools/flow/jupiter_swap_flow.rs` - `jupiter_swap_flow`

**Proposed Solution**: Add enhanced logging macros to flow tool with same pattern as Issue #2.

**Acceptance Criteria**:
- [ ] Enhanced logging added to `jupiter_swap_flow` tool
- [ ] Test verification with flow benchmarks

---

## Issue #4: Missing Enhanced Logging in Jupiter Lend/Earn Tools

**Priority**: 🟡 HIGH
**Status**: 🔄 Open
**Assigned**: Unassigned

**Description**: Jupiter lend/earn tools are missing enhanced logging macros, reducing debugging visibility for lending operations.

**Files Affected**:
- `reev-tools/src/tools/jupiter_lend_earn_deposit.rs` - `jupiter_lend_earn_deposit`
- `reev-tools/src/tools/jupiter_lend_earn_withdraw.rs` - `jupiter_lend_earn_withdraw`
- `reev-tools/src/tools/jupiter_lend_earn_mint_redeem.rs` - `jupiter_lend_earn_mint`
- `reev-tools/src/tools/jupiter_lend_earn_mint_redeem.rs` - `jupiter_lend_earn_redeem`

**Proposed Solution**: Add enhanced logging macros to all Jupiter lend/earn tools with same pattern as Issue #2.

**Acceptance Criteria**:
- [ ] Enhanced logging added to all 4 Jupiter lend/earn tools
- [ ] Test verification with lend/earn benchmarks

---

## Issue #5: Missing Enhanced Logging in SPL Transfer Tool

**Priority**: 🟡 HIGH
**Status**: 🔄 Open
**Assigned**: Unassigned

**Description**: SPL transfer tool is missing enhanced logging macros, while SOL transfer tool already has it.

**Files Affected**:
- `reev-tools/src/tools/native.rs` - `spl_transfer` (in same file as `sol_transfer`)

**Proposed Solution**: Add enhanced logging macros to `spl_transfer` tool with same pattern as existing `sol_transfer` tool.

**Acceptance Criteria**:
- [ ] Enhanced logging added to `spl_transfer` tool
- [ ] Test verification with SPL transfer benchmarks

---

## Issue #6: Update Trace Extraction Patterns for New Tools

**Priority**: 🟢 MEDIUM
**Status**: 🔄 Open
**Assigned**: Unassigned

**Description**: Trace extraction in `reev-lib/src/otel_extraction/mod.rs` needs updated patterns to detect all new tool names and deterministic agent calls.

**Files Affected**:
- `reev-lib/src/otel_extraction/mod.rs` - `extract_tool_name_from_span()` function

**Proposed Solution**: Add detection patterns for all new tools:
```rust
fn extract_tool_name_from_span(span: &OtelSpanData) -> Option<String> {
    // ... existing patterns ...
    
    // 🎯 Add deterministic agent patterns
    if span_name.contains("deterministic_sol_transfer") {
        return Some("deterministic_sol_transfer".to_string());
    }
    if span_name.contains("deterministic_jupiter_swap") {
        return Some("deterministic_jupiter_swap".to_string());
    }
    
    // 🎯 Add discovery tool patterns
    if span_name.contains("account_balance") || span_name.contains("get_account_balance") {
        return Some("get_account_balance".to_string());
    }
    if span_name.contains("lend_earn_tokens") || span_name.contains("get_lend_earn_tokens") {
        return Some("get_lend_earn_tokens".to_string());
    }
    if span_name.contains("position_info") || span_name.contains("get_position_info") {
        return Some("get_position_info".to_string());
    }
    
    // 🎯 Add flow tool patterns
    if span_name.contains("jupiter_swap_flow") {
        return Some("jupiter_swap_flow".to_string());
    }
    
    // 🎯 Add Jupiter lend/earn patterns
    if span_name.contains("jupiter_lend_earn_deposit") {
        return Some("jupiter_lend_earn_deposit".to_string());
    }
    if span_name.contains("jupiter_lend_earn_withdraw") {
        return Some("jupiter_lend_earn_withdraw".to_string());
    }
    if span_name.contains("jupiter_lend_earn_mint") {
        return Some("jupiter_lend_earn_mint".to_string());
    }
    if span_name.contains("jupiter_lend_earn_redeem") {
        return Some("jupiter_lend_earn_redeem".to_string());
    }
    
    // 🎯 Add SPL transfer pattern
    if span_name.contains("spl_transfer") {
        return Some("spl_transfer".to_string());
    }
    
    // ... existing fallback logic ...
}
```

**Acceptance Criteria**:
- [ ] Detection patterns added for all new tools
- [ ] Detection patterns added for deterministic agents
- [ ] Trace extraction correctly identifies all tool calls
- [ ] Mermaid diagrams show complete tool flow

---

## Issue #7: Verify Complete OTEL Integration

**Priority**: 🟢 MEDIUM
**Status**: 🔄 Open
**Assigned**: Unassigned

**Description**: Comprehensive testing and verification of OTEL integration across all agents and tools.

**Files Affected**: All OTEL-related files

**Test Plan**:
1. **Deterministic Agent Testing**:
   ```bash
   REEV_TRACE_FILE=traces_deterministic.log RUST_LOG=info cargo run -p reev-agent -- examples/001-sol-transfer.yml --agent deterministic
   ```

2. **Local Agent Testing**:
   ```bash
   REEV_TRACE_FILE=traces_local.log RUST_LOG=info cargo run -p reev-runner -- benchmarks/100-jup-swap-sol-usdc.yml --agent local
   ```

3. **Tool Coverage Testing**:
   ```bash
   # Test all benchmarks to ensure tool coverage
   for benchmark in benchmarks/*.yml; do
     REEV_TRACE_FILE=trace_$(basename $benchmark .yml).log RUST_LOG=info cargo run -p reev-runner -- $benchmark --agent local
   done
   ```

4. **Mermaid Diagram Verification**:
   - Check that all tools appear in generated diagrams
   - Verify tool execution order and timing

**Acceptance Criteria**:
- [ ] All deterministic agents generate OTEL logs
- [ ] All tools generate enhanced logging
- [ ] Trace extraction detects all tool calls
- [ ] Mermaid diagrams show complete tool flows
- [ ] Performance impact is minimal (<1ms per tool call)
