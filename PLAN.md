## Architecture Analysis

### Current Flow:
```
FlowAgent (orchestrates multi-step flows)
    ↓ calls
run_agent (dispatches to model-specific agents)
    ↓ calls
ZAIAgent/OpenAIAgent (creates tools with resolved key_map)
```

### The Real Issue:

1. **FlowAgent creates tools** with `key_map.clone()` containing placeholder names (like `USER_WALLET_PUBKEY`)
2. **FlowAgent calls run_agent** passing these placeholder keys in `context_prompt`
3. **run_agent parses context_prompt** and extracts the same placeholder key_map
4. **ZAIAgent/OpenAIAgent create tools again** with the same placeholder key_map
5. **Tools try to parse placeholder names** like `RECIPIENT_WALLET_PUBKEY` as base58 addresses → FAILS

## Revised Plan: Fix Context Handling in Multi-Step Flows

### Phase 1: Centralize Context Resolution (Critical)
**Location**: `FlowAgent` level before calling `run_agent`

1. **Create Context Resolver** in `FlowAgent`:
   - Take the initial key_map with placeholder names
   - Query surfpool for real account states and balances
   - Replace ALL placeholders with real addresses
   - Handle multi-step context consolidation (step results, updated balances)

2. **Fix FlowAgent Context Building**:
   - Resolve all placeholders to real addresses before tool creation
   - Consolidate account states after each step for multi-step flows
   - Use the same resolved context for all steps in the flow

### Phase 2: Fix Tool Creation (Secondary)
**Location**: Centralize in model agents (ZAIAgent/OpenAIAgent)

1. **Remove duplicate tool creation** from `FlowAgent`
2. **Create tools once** in model agents with properly resolved addresses
3. **Pass resolved context** instead of placeholder key_map

### Phase 3: Add Multi-Step Context Management
**Location**: `FlowAgent` state management

1. **Step 1 Context**: Initial state from YAML + surfpool
2. **Step 2+ Context**: Previous step results + updated on-chain state
3. **Context Consolidation**:
   - Merge initial key_map with derived addresses
   - Update balances after each transaction
   - Handle dependencies between steps (`depends_on: ["step_1_result"]`)

### Phase 4: Fix Error Types
1. **Create `SplTransferError`** separate from `NativeTransferError`
2. **Fix base58 parsing** to use resolved addresses, not placeholder names

### Phase 5: Add Context Validation Tests
1. **Test each flow step** context resolution without LLM calls
2. **Validate YAML schema** for context consistency
3. **Ensure placeholders are fully resolved** before tool execution

### Key Insight:

The issue isn't "dual tool creation" per se - it's that **FlowAgent creates placeholder contexts that never get resolved**. The `run_agent` correctly uses whatever context it receives, but FlowAgent is passing placeholder names instead of resolved addresses.

**Multi-step flows need FlowAgent-level context management** because:
1. Each step depends on previous step results
2. Account balances change after each transaction
3. Derived addresses (ATAs) need to be tracked across steps
4. Context needs to be consolidated before each LLM call
