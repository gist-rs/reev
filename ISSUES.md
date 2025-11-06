# Issues

## Issue #29 - USER_WALLET_PUBKEY Auto-Generation Missing - NEW 🐛
**Status**: REPORTED  
**Description**: API dynamic flow execution doesn't auto-generate keys for USER_WALLET_PUBKEY placeholder  
**Problem**: 
- Documentation and examples use USER_WALLET_PUBKEY as placeholder
- Benchmark mode has auto-generation in reev-lib/src/solana_env/reset.rs
- API dynamic flow expects real wallet address or fails silently with empty tool_calls
- No validation or auto-generation in API handlers
**Impact**: 
- Users following docs get "No tool calls found" errors
- Test scripts fail with placeholder values
- Inconsistent behavior between benchmark and dynamic modes
**Evidence**:
- `crates/reev-orchestrator/src/execution/ping_pong_executor.rs:create_key_map_with_wallet()` maps USER_WALLET_PUBKEY to provided wallet
- `crates/reev-lib/src/solana_env/reset.rs` has auto-generation but only used in benchmark mode
- API handlers in `crates/reev-api/src/handlers/dynamic_flows/` don't call reset functionality
**Next Steps**:
- Add auto-generation in API dynamic flow handlers when USER_WALLET_PUBKEY detected
- Use existing `Pubkey::new_unique()` pattern from examples
- Add validation to detect placeholder vs real pubkeys
- Update API documentation with clarification

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

## Issue #28 - Test Coverage - COMPLETED ✅
**Status**: UPDATED  
**Description**: Updated tests to work with new ToolName enum  
**Implementation**:
- Fixed integration_tests.rs to use ToolName enum
- Updated orchestrator_tests.rs for proper async handling
- Added conversion methods for backward compatibility
- Fixed test assertions to use ToolName instead of strings
