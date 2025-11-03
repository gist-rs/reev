# Issues

## Issue #7: Template Token Price Helper Not Working

**Priority**: 🟡 **MEDIUM**
**Status**: 🔴 **OPEN**
**Assigned**: reev-orchestrator
**Component**: Template System

**Problem**: 
The `get_token_price` helper in Handlebars templates shows `$0.0` for all token prices, even when the `WalletContext.token_prices` map contains correct prices. The helper is not finding prices in the correct location within the nested template rendering structure.

**Root Cause**: 
- Template rendering nests `WalletContext` under a `"wallet"` key for template access
- `get_token_price` helper expects direct access to `WalletContext.token_prices`
- Current helper path: `render_context.context().data().get("wallet")` → deserialize to `WalletContext`
- Issue may be in the data path traversal or deserialization step

**Current Behavior**:
```handlebars
{{#if (get_token_price "So11111111111111111111111111111111111112")}}
Current SOL price: ${{get_token_price "So11111111111111111111111111111111111112"}}
{{/if}}
```
Renders as: `Current SOL price: $0.0` (incorrect)

**Expected Behavior**:
Should render as: `Current SOL price: $150.00` (with actual price from context)

**Debug Information Found**:
- `WalletContext` JSON correctly contains token prices:
  ```json
  "token_prices": {
    "So11111111111111111111111111111111111112": 150.0,
    "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v": 1.0
  }
  ```
- Helper function is being called correctly
- Issue is in the price lookup within the helper

**Files Affected**:
- `crates/reev-orchestrator/src/templates/mod.rs` (helper implementation)
- All template files using `get_token_price` helper

**Impact**:
- Templates cannot display real-time price information
- Reduces user experience in generated prompts
- Affects context-awareness of dynamic flows

**Acceptance Criteria**:
- [ ] `get_token_price` helper returns correct prices from nested context
- [ ] Templates display actual token prices instead of $0.0
- [ ] All template integration tests pass with real price values
- [ ] Price helper works with both mint addresses and token symbols (if supported)

**Investigation Steps**:
1. Debug the data path in `get_token_price` helper
2. Verify `WalletContext` deserialization from nested JSON structure
3. Test with various token mint addresses and formats
4. Ensure helper handles missing prices gracefully

**Dependencies**: None
**Timeline**: 1-2 days (investigation and fix)
**Risk**: Low - Helper functionality issue, no breaking changes expected

---

## 🎯 **Issues Status Summary**

### 🟡 **CURRENT WORK**
- **Issue #7**: Template Token Price Helper Not Working (Medium Priority)
- **Issue #1**: ZAI Agent Agent Builder Pattern Migration (Low Priority Enhancement)

### 📊 **System Status**

**Dynamic Flow Implementation**: ✅ COMPLETE (Phases 1-3)
**Template System**: ✅ IMPLEMENTED (8 templates, caching, validation)
**Recovery Framework**: ✅ COMPLETE (Enterprise-grade with 3 strategies)
**Production Readiness**: ✅ PRODUCTION READY

All major phases completed. Remaining work consists of bug fixes and enhancements.