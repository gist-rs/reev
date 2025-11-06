# Implementation Tasks

## Issue #35: Jupiter Static Benchmarks Broken - CRITICAL 🔴

### 🎯 **Objective**
Fix static Jupiter benchmarks (200-series) that fail with deterministic agent while dynamic benchmarks (300-series) work perfectly with LLM agents.

### 📋 **Current Problems**
1. 200-jup-swap-then-lend-deposit fails with "Transaction simulation failed: Error processing Instruction 0: custom program error: 0x1"
2. Deterministic agent generates invalid Jupiter instructions
3. Flow diagram shows 0 tool calls for failed static benchmarks
4. Only affects Jupiter operations with deterministic agent (simple operations like 001 work fine)

### 🔍 **Root Cause Analysis**
**Testing Results Summary**:
- ✅ 001-sol-transfer: Score 100%, deterministic agent works fine
- ❌ 200-jup-swap-then-lend-deposit: Score 0, transaction simulation error  
- ✅ 300-jup-swap-then-lend-deposit-dyn: Score 100%, LLM agent works perfectly

**Evidence from Logs**:
```
200 benchmark failure: "Transaction simulation failed: Error processing Instruction 0: custom program error: 0x1"
300 benchmark success: "jupiter_swap execution_time_ms=795 status=success" with 6 Jupiter instructions
```

**Problem**: Deterministic agent has hardcoded Jupiter instruction generation that produces invalid transactions for current Jupiter program state.

### 🏗️ **Proposed Solutions**

#### Option 1: Fix Deterministic Agent Jupiter Instructions (Recommended)
- Update deterministic agent to generate valid Jupiter instructions
- Ensure compatibility with current Jupiter program interfaces
- Maintain backward compatibility for existing static benchmarks

#### Option 2: Migrate Static Jupiter Benchmarks to Dynamic Flow
- Update 200-series benchmarks to use `flow_type: "dynamic"`
- Route all Jupiter benchmarks to LLM agents with real Jupiter tools
- Keep deterministic agent only for simple operations (like SOL transfers)

#### Option 3: Hybrid Approach
- Keep static benchmarks for simple operations (001-series)
- Convert complex Jupiter benchmarks to dynamic flows
- Clear documentation of which benchmarks use which execution mode

### 📝 **Implementation Steps** (Option 1 - Fix Deterministic Agent)

#### Step 1: Analyze Current Jupiter Instruction Generation
```bash
# Find deterministic agent Jupiter instruction code
find crates -name "*.rs" -exec grep -l "jupiter\|JUP" {} \;
# Focus on reev-agent deterministic handler
```

#### Step 2: Compare Working vs Broken Jupiter Instructions
```bash
# Extract working Jupiter instructions from 300 benchmark logs
grep -A 10 -B 5 "jupiter_swap" api_debug.log

# Compare with deterministic agent instruction generation
# Check reev-agent/src/deterministic_agent.rs or similar
```

#### Step 3: Update Deterministic Agent Jupiter Logic
```rust
// In deterministic agent Jupiter handler
// Fix instruction generation to match current Jupiter program interface
// Update account structures, program IDs, and instruction data
// Ensure proper slippage handling and route selection
```

#### Step 4: Add Robust Error Handling
```rust
// Add transaction simulation before actual execution
// Provide fallback routes for failed transactions
// Improve error messages for debugging
```

#### Step 5: Test and Validate
```bash
# Test fixed 200 benchmark
curl -X POST "http://localhost:3001/api/v1/benchmarks/200-jup-swap-then-lend-deposit/run" \
  -H "Content-Type: application/json" \
  -d '{"agent":"deterministic"}'

# Verify tool calls captured in flow diagram
curl -s "http://localhost:3001/api/v1/flows/{execution_id}"
```

### 🧪 **Testing Strategy**

#### Test 1: Static Jupiter Benchmark (200)
```bash
# Should work with deterministic agent after fix
curl -X POST "http://localhost:3001/api/v1/benchmarks/200-jup-swap-then-lend-deposit/run" \
  -H "Content-Type: application/json" \
  -d '{"agent":"deterministic"}'
# Expected: Success with valid Jupiter transactions, tool calls captured
```

#### Test 2: Dynamic Jupiter Benchmark (300) - Regression Test
```bash
# Should continue working with LLM agent
curl -X POST "http://localhost:3001/api/v1/benchmarks/300-jup-swap-then-lend-deposit-dyn/run" \
  -H "Content-Type: application/json" \
  -d '{"agent":"glm-4.6-coding"}'
# Expected: Still works exactly as before
```

#### Test 3: Simple Benchmark (001) - Regression Test
```bash
# Should continue working with deterministic agent
curl -X POST "http://localhost:3001/api/v1/benchmarks/001-sol-transfer/run" \
  -H "Content-Type: application/json" \
  -d '{"agent":"deterministic"}'
# Expected: Still works exactly as before
```

### 📊 **Expected Results**

#### Before Fix (Current State)
- 200 benchmark: ❌ Score 0, transaction simulation error, no tool calls
- Static Jupiter flows: ❌ Completely broken
- Deterministic agent: ❌ Cannot handle Jupiter operations

#### After Fix (Expected State)
- 200 benchmark: ✅ Score 80-100%, valid Jupiter transactions, tool calls captured
- Static Jupiter flows: ✅ Working with deterministic agent
- Dynamic Jupiter flows: ✅ Still working with LLM agents (unchanged)
- Simple flows: ✅ Still working with deterministic agent (unchanged)

### 🔧 **Files to Modify**
1. `crates/reev-agent/src/deterministic_agent.rs` - Fix Jupiter instruction generation
2. `crates/reev-agent/src/handlers/jupiter.rs` - Update Jupiter-specific handlers
3. `benchmarks/200-jup-swap-then-lend-deposit.yml` - Possibly adjust expectations
4. `tests/integration_tests.rs` - Add regression tests for Jupiter flows

### ⚠️ **Breaking Changes**
- None expected - this is a bug fix for broken functionality
- Should restore deterministic Jupiter benchmark capabilities

### 🎉 **Benefits**
1. **Complete Coverage**: All benchmark series working correctly
2. **Backward Compatibility**: Static benchmarks restored to working state  
3. **Production Ready**: Full system ready for all use cases
4. **Debugging**: Clear error messages and proper transaction simulation

---

## ✅ COMPLETED Tasks

### Dynamic Benchmark Architecture - COMPLETED
- Mode-based routing (static vs dynamic) ✅
- Flow type field implementation ✅ 
- API integration with dynamic execution ✅
- Tool name enum system ✅
- Enhanced OTEL logging ✅

### Jupiter Tool Call Capture - COMPLETED
- Dynamic Jupiter tool calls captured perfectly ✅
- Flow visualization working for dynamic flows ✅
- Real Jupiter transaction execution ✅
- Mermaid diagram generation ✅

### Test Coverage - COMPLETED
- Updated tests for new architecture ✅
- ToolName enum integration ✅
- Async handling improvements ✅

---

## 🚨 Critical Path to Production

**Only One Blocker Remaining**: Fix deterministic agent Jupiter instruction generation

**Time Estimate**: 2-4 hours for analysis and fix
**Risk Level**: Medium (requires careful Jupiter program interface knowledge)
**Priority**: CRITICAL (blocks static Jupiter benchmarks)

**Next Action**: Analyze deterministic agent Jupiter code vs working LLM agent Jupiter instructions