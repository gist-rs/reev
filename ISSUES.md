# Reev Project Issues

## Current Issues 300

### Issue 301: E2E Test Failures Need Resolution

**Description:**
While the protocol interface implementation is solid, several e2e tests are failing:

1. **e2e_swap test**: One case passes but the "all sol for usdc" case fails with Jupiter transaction error:
   ```
   Program TessVdML9pBGgG9yGks7o4HewRaXVAMuoVj4x83GLQH failed: custom program error: 0xffff
   ```

2. **e2e_transfer test**: One case passes but the "all sol" case fails with insufficient funds error:
   ```
   Transfer: insufficient lamports 4999995000, need 5000000000
   ```

**Priority:** High
**Status:** Open
**Assigned:** Unassigned

### Issue 302: JupiterProtocol Implementation Is Placeholder

**Description:**
The JupiterProtocol wrapper currently returns hardcoded values rather than executing real operations. It needs to be integrated with actual Jupiter handlers for Swap, Lend, and Earn operations.

Current implementation just returns placeholder signatures and calculations:
```rust
let swap_result = super::JupiterSwapResult {
    success: true,
    signature: Some("jupiter_swap_placeholder".to_string()),
    input_amount: parse_amount(amount)?,
    output_amount: parse_amount(amount)? * 2, // Placeholder calculation
    price_impact: Some(0.05),
};
```

**Priority:** High
**Status:** Open
**Assigned:** Unassigned

### Issue 303: Build Warning in Workspace

**Description:**
There's a build warning that should be addressed:
```
warning: /Users/katopz/git/gist/reev/Cargo.toml: unused manifest key: workspace.dev-dependencies
```

**Priority:** Low
**Status:** Open
**Assigned:** Unassigned