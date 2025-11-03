# Issues

## Issue #7: Template Token Price Helper Not Working

**Priority**: 🟡 **MEDIUM**
**Status**: ✅ **FIXED**
**Assigned**: reev-orchestrator
**Component**: Template System

**Problem**: 
The `get_token_price` helper in Handlebars templates shows `$0.0` for all token prices, even when the `WalletContext.token_prices` map contains correct prices. The helper is not finding prices in the correct location within the nested template rendering structure.

**Root Cause**: 
- Template rendering nests `WalletContext` under a `"wallet"` key for template access
- `get_token_price` helper was using incorrect data path: `render_context.context().data().get("wallet")` 
- Handlebars helper functions should access root data via `ctx.data()` instead of nested render context
- Helper was not finding the nested context data structure properly

**Fix Applied**:
- Updated helper functions to access context data via `ctx.data()` instead of `render_context.context()`
- Implemented direct JSON path traversal for better performance: `root_data.get("wallet")` → `token_prices` → token mint
- Added fallback to full deserialization if direct JSON access fails
- Applied same fix to both `get_token_price` and `get_token_balance` helpers

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
- [x] `get_token_price` helper returns correct prices from nested context
- [x] Templates display actual token prices instead of $0.0
- [x] All template integration tests pass with real price values
- [x] Price helper works with both mint addresses and token symbols (if supported)

**Investigation Steps Completed**:
1. ✅ Debugged the data path in `get_token_price` helper - found `render_context.context()` was returning None
2. ✅ Verified `WalletContext` deserialization from nested JSON structure - switched to direct JSON access
3. ✅ Tested with various token mint addresses and formats - all working correctly
4. ✅ Ensured helper handles missing prices gracefully - returns 0.0 for unknown tokens

**Test Results**:
- Created comprehensive test suite in `crates/reev-orchestrator/tests/token_price_helper_test.rs`
- All tests pass, including real template integration tests
- Templates now correctly display: `$150.420000` for SOL, `$1.000000` for USDC
- No more `$0.0` values for tokens with known prices

**Dependencies**: None
**Timeline**: ✅ Completed (1 day)
**Risk**: ✅ Low - No breaking changes, backward compatible fix

**Files Modified**:
- `crates/reev-orchestrator/src/templates/mod.rs` - Fixed helper functions
- `crates/reev-orchestrator/tests/token_price_helper_test.rs` - Added test suite
- `crates/reev-orchestrator/tests/real_template_test.rs` - Added integration tests

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