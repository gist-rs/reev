# Issues

## Open Issues

### #12 Critical Session ID Collision - IDENTIFIED ⚠️ HIGH PRIORITY

**Date**: 2025-10-27  
**Status**: Open  
**Priority**: Critical  
**Description**: Sequential benchmark runs overwrite each other's log files due to session_id collision

**Root Cause**: FlowLogger::with_database() generates NEW UUID instead of preserving existing session_id

**Fix Status**: 🔧 IMPLEMENTED - Fix applied but requires verification testing

## Recent Resolved Issues

### #14 Deterministic Agent API Parsing Issue - RESOLVED ✅

**Date**: 2025-10-27  
**Status**: Closed  
**Priority**: High  
**Description**: Deterministic agent worked in CLI execution (100% score) but failed in API execution (0% score) due to response parsing mismatch.

**Root Cause**: Response parsing discrepancy between CLI and API paths:
- CLI used GLM-style parsing (is_glm=true) which correctly extracts transactions from result.text field
- API used standard parsing (is_glm=false) which couldn't find instructions in deterministic agent's response format

**Fix Applied**: Modified LlmAgent in `reev-lib/src/llm_agent.rs` to set `is_glm=true` for deterministic agent regardless of environment variables, ensuring consistent parsing behavior.

**Verification**: 
✅ CLI: Still works perfectly (Score: 100.0%)
✅ API: Now works perfectly (Score: 100.0%)

**Key Learning**: Deterministic agent returns responses in GLM-compatible format (result.text containing JSON instructions), requiring GLM-style parser even though it's not actually a GLM model.

### #13 Empty Log Files - RESOLVED ✅

**Date**: 2025-10-27  
**Status**: Closed  
**Priority**: Critical  
**Description**: All benchmark log files were empty across all benchmark executions. reev-runner could not open log files for any process (reev-agent, surfpool), causing immediate benchmark failures.

**Root Cause**: `OpenOptions::new().append(true).open()` fails when file doesn't exist - missing `.create(true)` flag.

**Fix Applied**: Added `.create(true)` flag to ProcessManager for stdout/stderr file creation.

**Verification**: ✅ Log files now created and contain proper output for both reev-agent and surfpool processes.

### #11 Jupiter Lend Deposit Amount Parsing Issue - RESOLVED ✅

**Date**: 2025-10-27  
**Status**: Closed  
**Priority**: High  
**Description**: Jupiter lend deposit benchmark was parsing wrong amount from context (1000 SOL instead of 0.01 SOL).

**Root Cause**: Incorrect amount extraction logic in Jupiter lending handlers.

**Fix Applied**: Fixed amount parsing to correctly extract 0.01 SOL for deposit operations.

### #10 Jupiter Earn Tool Scope Issue - RESOLVED ✅

**Date**: 2025-10-27  
**Status**: Closed  
**Priority**: Medium  
**Description**: Jupiter earn tool was being used outside of position/earnings benchmarks (114-*.yml), violating security rules.

**Fix Applied**: Restricted jupiter_earn tool to only position/earnings benchmarks (114-*.yml) as per security requirements.

### #9 SPL Transfer Uses Wrong Recipient Address - RESOLVED ✅

**Date**: 2025-10-27  
**Status**: Closed  
**Priority**: High  
**Description**: SPL transfer benchmarks were using wrong recipient address from key_map.

**Fix Applied**: Fixed recipient address resolution in SPL transfer handlers.

### #8 API vs CLI Tool Selection Issue - RESOLVED ✅

**Date**: 2025-10-27  
**Status**: Closed  
**Priority**: High  
**Description**: API and CLI were selecting different tools for the same benchmark, causing inconsistent behavior.

**Fix Applied**: Modified AgentTools::new() to respect allowed_tools parameter in both ZAIAgent and OpenAIAgent.

### #7 SOL Transfer Placeholder Resolution Issue - RESOLVED ✅

**Date**: 2025-10-27  
**Status**: Closed  
**Priority**: High  
**Description**: SOL transfer benchmarks had placeholder resolution issues affecting recipient addresses.

**Fix Applied**: Fixed placeholder resolution logic in SOL transfer handlers.

### #6 GLM SPL Transfer ATA Resolution Issue - RESOLVED ✅

**Date**: 2025-10-27  
**Status**: Closed  
**Priority**: Medium  
**Description**: GLM models had issues resolving Associated Token Account addresses in SPL transfers.

**Fix Applied**: Enhanced ATA resolution logic for GLM model compatibility.

### #5 Failed Test Color Display Issue - RESOLVED ✅

**Date**: 2025-10-27  
**Status**: Closed  
**Priority**: Low  
**Description**: Failed tests were not displaying color output properly in terminal.

**Fix Applied**: Fixed terminal color output for failed test indicators.

### #4 Jupiter Lending Deposit AI Model Interpretation Issue - RESOLVED ✅

**Date**: 2025-10-27  
**Status**: Closed  
**Priority**: Medium  
**Description**: AI models were incorrectly interpreting Jupiter lending deposit instructions.

**Fix Applied**: Clarified deposit instruction format and improved AI model prompt context.

### #3 Database Test Failure - RESOLVED ✅

**Date**: 2025-10-27  
**Status**: Closed  
**Priority**: High  
**Description**: Database connection tests were failing due to connection pool issues.

**Fix Applied**: Fixed database connection pool configuration and test isolation.

### #2 Flow Test Assertion Failure - RESOLVED ✅

**Date**: 2025-10-27  
**Status**: Closed  
**Priority**: Medium  
**Description**: Flow tests had assertion failures due to incorrect expected values.

**Fix Applied**: Updated test assertions to match correct expected flow behavior.

### #1 Web API YAML Parsing Issue - RESOLVED ✅

**Date**: 2025-10-27  
**Status**: Closed  
**Priority**: High  
**Description**: Web API benchmark execution failed with YAML parsing errors while individual CLI execution worked fine.

**Root Cause**: Different context resolution between API and CLI paths.

**Fix Applied**: Unified context resolution logic between API and CLI execution paths.