# Handover: Architecture Cleanup & Flow Visualization

## Current State (2025-10-21)

### ✅ What's Working
1. **Flow API**: `/flows/{session_id}` endpoint working correctly
2. **Session logs**: Generated with proper structure in `logs/sessions/`
3. **Flow diagrams**: Basic stateDiagram generation working
4. **Architecture**: Decoupled design implemented (reev-api layer tracking)
5. **Database Cleanup**: Fixed database lock conflicts with proper process killing
6. **Jupiter Rules Compliance**: Removed explicit LLM transaction generation instruction

### ❌ What's Broken
1. **Tool Call Tracking**: `tools: []` arrays are empty in session logs
2. **OTEL Integration**: Created spans but not extracting data correctly
3. **Agent Architecture**: LlmAgent violates Jupiter Integration Rules (preserved for history)
4. **Process Management**: API server not killed before runner execution (FIXED)

## 🎯 Real Challenge

**We must extract tool calls from OpenTelemetry traces via reev-api layer**

### Current Misconceptions
- ❌ Relying on agent code changes (we can't control 3rd party agents)
- ❌ Modifying LlmAgent logging (wrong layer)
- ❌ Binary OTEL format parsing (not needed)

### Correct Approach
- ✅ **OTEL spans are being created** in reev-api layer around HTTP requests
- ✅ **Flow visualization is now handled by reev-api** web interface
- ✅ **Session-based flow diagrams** generated via API endpoints
- ✅ **Process cleanup working**: kills API(3001), reev-agent(9090), surfpool(8899)

### Recent Fixes Applied
- **Oct 21, 2025**: Removed explicit LLM transaction generation instruction
- **Oct 21, 2025**: Fixed database lock conflicts by killing API processes
- **Oct 21, 2025**: Preserved git history while fixing Jupiter rule violations

## 📋 Next Steps (CRITICAL)

### Step 1: Replace LlmAgent Architecture
```rust
// Current: Broken JSON parsing agent
let agent = LlmAgent::new("glm-4.6")?; // ❌ Violates Jupiter rules

// Target: Tool-based agent from reev-agent
let agent = GlmAgent::run("glm-4.6", payload, key_map).await?; // ✅ Uses tools
```

### Step 2: Implement Tool-Based Runner
- Use `crates/reev-agent/src/enhanced/glm_agent.rs` pattern
- Replace LlmAgent imports with proper tool-based agents
- Update runner to work with tool execution instead of JSON parsing

### Step 3: OTEL Integration (Secondary)
- Once tool-based agent works, integrate OTEL extraction
- Use existing `reev-lib/src/otel_extraction.rs` module
- Extract tool calls from rig's OpenTelemetry spans

## 📁 Key Files Status

### ✅ Fixed Files
- `crates/reev-lib/src/llm_agent.rs` - Removed explicit violation (preserved for history)
- `crates/reev-runner/src/lib.rs` - Added API process killing
- `TOFIX.md` - Updated with current status
- `REFLECT.md` - Added learnings from cleanup

### 🔄 Files Needing Work
- `crates/reev-runner/src/lib.rs` - Replace LlmAgent with tool-based agent
- Integration with `reev-agent` pattern for proper tool usage

## 🎯 Success Criteria

### IMMEDIATE (Required for working system)
- ✅ Database lock conflicts resolved
- ✅ Jupiter Integration Rules no longer violated
- 🎯 Tool-based agent execution working
- 🎯 GLM-4.6 benchmarks run without rule violations

### FUTURE (Flow visualization)
- 🎯 Session logs contain non-empty `tools: []` arrays from real tool calls
- 🎯 Flow diagrams show actual tool execution paths
- 🎯 OTEL extraction working with rig framework

## 🚨 CURRENT BLOCKERS

### Primary Blocker
- **Architecture**: LlmAgent violates Jupiter Integration Rules
- **Solution**: Replace with tool-based agents from `reev-agent`

### Secondary Blockers
- **OTEL Integration**: Need to implement with tool-based agents
- **Testing**: Verify tool calls appear in session logs

## 🔄 Next Developer Priority

1. **HIGH**: Replace LlmAgent with tool-based agent implementation
2. **MEDIUM**: Integrate OTEL extraction with new tool-based agents  
3. **LOW**: Flow visualization improvements once tool tracking works

## 📚 Architecture Context

### Current State
- ✅ **Violations Fixed**: No longer explicitly telling LLM to generate raw JSON
- ✅ **Process Management**: All processes properly killed before runs
- ⚠️ **Architecture**: Still using broken agent pattern (preserved for history)

### Target Architecture
```
Tool-based Agent → Tool Calls → OTEL Spans → Session Logs → Flow Diagrams
```

### Files to Reference
- `crates/reev-agent/src/enhanced/glm_agent.rs` - Working tool-based pattern
- `crates/reev-lib/src/otel_extraction.rs` - OTEL extraction module
- `TOFIX.md` - Detailed status and remaining work

The immediate priority is replacing the broken LlmAgent with proper tool-based execution!
