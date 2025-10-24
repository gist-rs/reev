## Architecture Analysis

### Current Flow:
```
FlowAgent (orchestrates multi-step flows)
    ↓ calls
run_agent (dispatches to model-specific agents)
    ↓ calls
ZAIAgent/OpenAIAgent (creates tools with resolved key_map)
```

### 🚨 Critical Issue: Ground Truth Data Leakage

**Problem**: FlowAgent passes `benchmark.ground_truth` into `resolve_initial_context()`, breaking real-time multi-step decision making.

**Solution Implemented**: Clean ground truth separation with mode detection
```rust
// In FlowAgent - Proper ground truth separation ✅ FIXED
let ground_truth_for_context =
    if is_deterministic_mode(&self.model_name, &benchmark.id, &benchmark.tags) {
        info!("[FlowAgent] Using ground truth for deterministic mode");
        Some(&benchmark.ground_truth)
    } else {
        info!("[FlowAgent] Using real blockchain state for LLM mode");
        None // LLM gets actual chain state, no future info leakage
    };

// Validate no ground truth leakage in LLM mode
if !is_deterministic_mode(&self.model_name, &benchmark.id, &benchmark.tags)
    && !benchmark.ground_truth.final_state_assertions.is_empty() {
    return Err(anyhow!(
        "Ground truth not allowed in LLM mode - would leak future information"
    ));
}
```

### 🎯 Solution: Ground Truth Separation ✅ IMPLEMENTED

**Option A: Clean Separation** ✅ IMPLEMENTED
- Test files: Use ground_truth for fast validation and scoring
- Production agents: Use real blockchain state only
- Clear architectural boundary between test data and execution data

**Option B: Conditional Ground Truth** ✅ IMPLEMENTED
- Deterministic mode: Use ground_truth for reproducible tests
- LLM mode: Use blockchain state for real evaluation

**Status**: 🟢 COMPLETED - Ground truth leakage eliminated, all compilation errors fixed
