# Issues

## Issue #29 - USER_WALLET_PUBKEY Auto-Generation - RESOLVED ✅
**Status**: COMPLETED  
**Description**: ContextResolver already provides auto-generation for USER_WALLET_PUBKEY placeholder  
**Resolution**: 
- Auto-generation already exists in `ContextResolver::resolve_placeholder()`
- Called from `PingPongExecutor.execute_agent_step()` before agent execution
- Uses same logic as benchmark mode (`Keypair::new()` and storage in SolanaEnv)
- Works transparently for both API and CLI dynamic flows
- No changes needed at API level (incorrectly attempted initially)
**Evidence**:
- `reev-orchestrator/src/context_resolver.rs:resolve_placeholder()` handles placeholder detection and generation
- `reev-orchestrator/src/execution/ping_pong_executor.rs:execute_agent_step()` calls resolver for wallet pubkey
- Auto-generation consistent with `reev-lib/src/solana_env/reset.rs` benchmark implementation
- Documented in DYNAMIC_BENCHMARK_DESIGN.md under "USER_WALLET_PUBKEY Auto-Generation"
**Status**: RESOLVED - Architecture was already correct, auto-generation works at orchestrator level

## Issue #23 - RESOLVED ✅
**Status**: COMPLETED  
**Description**: Implemented type-safe tool name system with strum to eliminate string-based tool errors  
**Resolution**: 
- Created ToolName enum in reev-types/src/tools.rs with serde and strum support
- Updated DynamicStep to use Vec<ToolName> instead of Vec<String>
- Fixed all string-based tool references throughout codebase
- Added conversion helpers for backward compatibility
- Updated test files to use ToolName enum

## Issue #24 - Mode-Based Separation - IN PROGRESS 🚧
**Status**: IMPLEMENTED  
**Description**: Clean separation between benchmark and dynamic execution modes  
**Implementation**:
- Created benchmark_mode.rs for static YML file management
- Created dynamic_mode.rs for user request execution
- Implemented mode router in lib.rs for top-level separation
- Added feature flags (benchmark, production) for mode control
- Same core execution interface used by both modes

## Issue #25 - Mock Data Elimination - PENDING ⏳
**Status**: NOT STARTED  
**Description**: Remove mock data and implement real execution only  
**Next Steps**: 
- Identify remaining mock responses in codebase
- Replace with actual agent execution
- Implement real scoring for failures
- Add proper error handling without fallbacks

## Issue #26 - YML Generation Simplification - COMPLETED ✅
**Status**: IMPLEMENTED  
**Description**: Simple dynamic YML generation without over-engineering  
**Implementation**:
- Basic intent analysis using keyword detection
- Simple amount extraction with regex
- Temporary file generation and cleanup
- Integration with existing runner infrastructure

## Issue #27 - API Integration - PENDING ⏳
**Status**: BLOCKED  
**Description**: Update API endpoints to use new mode routing  
**Blocker**: reev-runner compilation errors preventing API server startup  
**Next Steps**:
- Fix circular dependency between reev-orchestrator and reev-runner
- Update API handlers to use route_execution function
- Test dynamic flow execution via API

## Issue #30 - Jupiter Tool Calls Not Captured in OTEL - NEW 🐛
**Status**: RESOLVED
**Description**: Jupiter benchmarks (200, 300) execute successfully but tool calls aren't captured in database for flow visualization  
**Problem**: 
- 200-jup-swap-then-lend-deposit completes with score 1.0 but shows 0 tool calls in flow diagram
- Agent logs show successful Jupiter swap and lend instruction generation
- Simple benchmarks (001) capture tool calls correctly via OTEL logging
- Only affects Jupiter-related tool calls (jupiter_swap, jupiter_lend)
**Evidence**:
- reev-agent logs show successful tool execution: "Successfully generated 6 Jupiter swap instructions"
- Flow diagram returns simple state: "Prompt --> Agent --> [*]" (no tool states)
- Database query for session returns 0 tool calls for Jupiter benchmarks
- 001-sol-transfer works: captures 1 tool call (deterministic_sol_transfer) correctly
**Impact**: 
- Flow visualization broken for complex DeFi operations
- No detailed mermaid diagrams for Jupiter strategies
- Users can't see actual tool execution sequence for yield farming
**Next Steps**:
- Debug OTEL logging pipeline for Jupiter tool calls
- Fix tool call capture in session_parser for Jupiter protocols
- Ensure jupiter_swap and jupiter_lend are logged with proper metadata
- Test with both benchmark (200) and dynamic (300) flows

## Issue #31 - Surfpool Service Integration Failures - NEW 🐛
**Status**: REPORTED  
**Description**: 300 benchmark fails due to surfpool service startup issues during execution  
**Problem**: 
- 300-jup-swap-then-lend-deposit-dyn fails with surfpool initialization errors
- Error logs show surfpool service startup timeout or connection issues
- 200 benchmark works (possibly using different surfpool configuration)
- Simple benchmarks (001) don't require surfpool so work fine
**Evidence**:
- 300 benchmark error: "CLI execution failed: Starting surfpool service..." gets stuck
- Error occurs during dependency manager initialization phase
- 200 benchmark completes successfully (surfpool works for static benchmarks)
- surfpool process checks pass: "No existing surfpool processes found on port 8899"
**Impact**: 
- Dynamic flow execution (300-series) completely broken
- Users cannot test yield optimization strategies
- Multiplication benchmarks unavailable via API
**Next Steps**:
- Debug surfpool service startup process in runner dependency manager
- Check surfpool configuration differences between static (200) and dynamic (300) modes
- Ensure proper surfpool process lifecycle management
- Test surfpool RPC connectivity during benchmark execution

## Issue #28 - Test Coverage - COMPLETED ✅
**Status**: UPDATED  
**Description**: Updated tests to work with new ToolName enum  
**Implementation**:
- Fixed integration_tests.rs to use ToolName enum
- Updated orchestrator_tests.rs for proper async handling
- Added conversion methods for backward compatibility
- Fixed test assertions to use ToolName instead of strings


## Issue #32 - Tool Call Transfer to Session Database - NEW 🐛
**Status**: IDENTIFIED
**Description**: Tool calls captured in OTEL logs but not transferred to session JSON database for flow visualization
**Problem**:
- 200-jup-swap-then-lend-deposit completes with score 1.0 but shows 0 tool calls in flow diagram
- Agent logs show successful Jupiter swap and lend instruction generation  
- Session database has empty events array for Jupiter benchmarks
- Simple benchmarks (001) capture tool calls correctly via OTEL logging
- Flow visualization returns "Prompt --> Agent --> [*]" (no tool states) for complex operations
**Root Cause**:
- Session creation code properly reads OTEL logs with tool calls
- Session storage/transfer logic fails to copy tool calls from OTEL events to session JSON
- API mermaid generator reads from session JSON (empty events) → no tool calls displayed
**Impact**:
- Users cannot see actual tool execution sequence for Jupiter protocols
- No detailed mermaid flow diagrams for yield farming, multiplication, or portfolio rebalancing
- Flow visualization broken for all dynamic flows and complex DeFi operations
**Next Steps**:
- Fix session storage to properly transfer tool calls from OTEL events to session JSON
- Update API flow diagram generator to read from both session JSON and OTEL logs
- Test with both static benchmarks (200) and dynamic flows (300) to ensure consistent tool call capture
