# TOFIX.md - Current Issues to Fix

## ~~Flow Diagram Tool Name Bug~~ ✅ COMPLETELY RESOLVED
- **Issue**: Flow diagram shows generic tool names (`transfer_sol`) instead of actual tool names (`sol_transfer`)
- **Current Output**: `Agent --> transfer_sol : 0.1 SOL`
- **Expected Output**: `Agent --> sol_transfer : 1 ix`
- **Root Cause**: Hardcoded tool name mapping in session parser
- **Location**: `crates/reev-api/src/handlers/flow_diagram/session_parser.rs:290`
- **Priority**: High - affects flow diagram accuracy
- **Fix Applied**: 
  1. Created `reev-tools/src/tool_names.rs` with centralized tool name constants
  2. Updated `reev-tools/src/tools/native.rs` to use shared constants (`SOL_TRANSFER`, `SPL_TRANSFER`)
  3. Added `reev-tools` dependency to `reev-api`
  4. Updated session parser to import and use `reev_tools::tool_names::tool_name_from_program_id()`
  5. Now tool names come from the source of truth in `reev-tools` instead of being hardcoded
- **Architecture**: Proper decoupling achieved - `reev-tools` owns tool definitions, `reev-api` consumes them

## Flow Diagram Tool Call Collection Issue 🔄 IN PROGRESS
- **Issue**: Flow diagrams show 0 tool calls despite tool tracking implementation
- **Current Output**: `{"tool_count":0}` and "No tool calls found in database log"
- **Expected Output**: Tool calls from `GlobalFlowTracker` should be collected and displayed
- **Root Cause**: Agent execution failing with "Agent returned no actions to execute"
- **Location**: `reev-runner` integration with `GlobalFlowTracker`
- **Priority**: High - core flow diagram functionality not working
- **Fix Applied**:
  1. Added `reev-tools` dependency to `reev-runner`
  2. Modified `run_evaluation_loop` to collect from `GlobalFlowTracker`
  3. Fixed `ToolCallInfo` conversion between `reev-lib::agent` and `reev-lib::session_logger` formats
  4. Flow logging enabled by default
- **Current Status**: GLM API working, agent executing successfully (100% score), but flow tracking not capturing tool calls
- **Next Step**: Debug why GlobalFlowTracker is not recording tool calls despite successful execution

## GLM Agent Tool Usage Issue 🔄 HIGH PRIORITY
- **Issue**: GLM agent in `reev-lib/src/llm_agent.rs` doesn't use tools, violates RULES.md
- **Problem**: GLM agent asks LLM to generate raw transaction JSON instead of using tools
- **Root Cause**: GLM agent makes raw HTTP requests without `rig` framework tool integration
- **Impact**: Violates "No LLM Generation" and "API-Only Instructions" rules from RULES.md
- **Current Behavior**: LLM generates `{"program_id": "...", "accounts": [...], "data": "..."}` 
- **Expected Behavior**: LLM should use `sol_transfer`, `jupiter_swap`, etc. tools via `rig` framework
- **Priority**: High - breaks core architecture principles and rules
- **Fix Required**: 
  1. Update GLM agent to use `rig` framework with proper tool integration
  2. Model after OpenAI agent in `reev/crates/reev-agent/src/enhanced/openai.rs`
  3. Remove all raw transaction generation prompts
  4. Ensure GLM agent has access to same tools as OpenAI agent
- **Alternative**: Remove GLM agent entirely, use enhanced agents for all models

## API Graceful Shutdown ✅ COMPLETELY RESOLVED
- **Issue**: API server didn't gracefully shutdown database connections on exit
- **Impact**: Database connections remained open, potential resource leaks
- **Fix Applied**:
  1. Added `close()` method to `ConnectionPool` in `reev-db/src/pool/mod.rs`
  2. Added `shutdown()` method to `PooledDatabaseWriter` in `reev-db/src/pool/pooled_writer.rs`
  3. Added graceful shutdown handling in `reev-api/src/main.rs` with Ctrl+C signal handling
  4. Fixed async block ownership issue with `move` keyword
- **Architecture**: Proper cleanup of database connections on server shutdown
- **Validation**: Compiles successfully, passes clippy checks

## GLM API URL Logging ✅ COMPLETELY RESOLVED
- **Issue**: GLM API URL not being logged for debugging
- **Impact**: Hard to debug API endpoint configuration issues
- **Fix Applied**:
  1. Added logging in `reev-lib/src/llm_agent.rs` before LLM request: `info!("[LlmAgent] GLM API URL: {}", self.api_url);`
  2. Added GLM API URL and key logging during agent initialization
- **Location**: `reev-lib/src/llm_agent.rs:417` and `reev-lib/src/llm_agent.rs:56-57`
- **Result**: API URL now clearly visible in logs for debugging
