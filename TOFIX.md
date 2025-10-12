# Issues to Fix

---

## 🔍 Code Smells & Anti-Patterns Identified

### 1. MAGIC NUMBERS & HARDCODED VALUES

#### 📍 Location: Multiple files
**Issue**: Extensive use of magic numbers without named constants

**Files Affected**:
- `crates/reev-agent/src/agents/coding/d_100_jup_swap_sol_usdc.rs`: `100_000_000` (0.1 SOL), `800` (8% slippage)
- `crates/reev-agent/src/agents/coding/d_111_jup_lend_deposit_usdc.rs`: `10_000_000` (10 USDC)
- `crates/reev-agent/src/agents/coding/d_113_jup_lend_withdraw_usdc.rs`: `10_000_000` (10 USDC)
- `crates/reev-agent/src/agents/coding/d_200_jup_swap_then_lend_deposit.rs`: `250_000_000` (0.5 SOL), `500` (5% slippage), `9_000_000` (~9 USDC)
- `crates/reev-agent/src/lib.rs`: `50_000_000`, `49_500_000`, `40_000_000` (USDC amounts)
- `crates/reev-lib/src/solana_env/reset.rs`: `5000000000` (5 SOL for fees), `2039280` (rent exemption)

**Impact**: Hard to maintain, error-prone, unclear intent

**Solution**: Create constants module with named values

---

### 3. CODE DUPLICATION (DRY VIOLATIONS)

#### 📍 Location: Example files (14+ instances)
**Issue**: Identical health check and URL construction code repeated across examples

**Pattern Repeated**:
```rust
let health_url = "http://127.0.0.1:9090/health";
let agent_url = if agent_name == "deterministic" {
    "http://127.0.0.1:9090/gen/tx?mock=true"
} else {
    "http://127.0.0.1:9090/gen/tx"
};
```

**Files Affected**:
- `examples/001-sol-transfer.rs`
- `examples/002-spl-transfer.rs`
- `examples/100-jup-swap-sol-usdc.rs`
- `examples/110-jup-lend-deposit-sol.rs`
- `examples/111-jup-lend-deposit-usdc.rs`
- `examples/112-jup-lend-withdraw-sol.rs`
- `examples/113-jup-lend-withdraw-usdc.rs`
- `examples/114-jup-positions-and-earnings.rs`
- `examples/115-jup-lend-mint-usdc.rs`
- `examples/116-jup-lend-redeem-usdc.rs`

**Impact**: Maintenance nightmare, inconsistent updates

**Solution**: Create common example helper functions

**✅ RESOLVED**: Created common/helpers.rs with shared functionality and common/config.rs with centralized constants. Foundation established for reducing duplication across examples.

---

### 3. HARDCODED BLOCKCHAIN ADDRESSES

#### 📍 Location: Throughout codebase
**Issue**: Magic addresses scattered without centralization

**Examples**:
- `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v` (USDC mint) - 20+ occurrences
- `11111111111111111111111111111111` (System Program) - 10+ occurrences  
- `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA` (Token Program) - 5+ occurrences
- `So11111111111111111111111111111111111111112` (SOL mint) - 5+ occurrences
- `9BEcn9aPEmhSPbPQeFGjidRiEKki46fVQDyPpSQXPA2D` (jUSDC mint) - 3+ occurrences

**Impact**: Typos could cause silent failures, hard to update

**Solution**: Central address constants module

**✅ RESOLVED**: Created centralized constants module with addresses.rs, amounts.rs, and proper re-exports. All hardcoded addresses in agent code have been replaced with constants.

---


### 4. PORT NUMBERS & CONFIGURATION

#### 📍 Location: Multiple files
**Issue**: Hardcoded ports without configuration

**Examples**:
- `9090` (reev-agent port) - 15+ occurrences
- `8899` (surfpool port) - 10+ occurrences
- `127.0.0.1` (localhost) - 20+ occurrences

**Impact**: Cannot run multiple instances, inflexible deployment

**Solution**: Environment variables or config file

---

### 5. HACK COMMENTS

#### 📍 Location: Multiple files
**Issue**: Outstanding technical debt markers

**Found**:
- `crates/reev-runner/tests/common/helpers.rs`: HACK for race conditions
- `crates/reev-runner/tests/scoring_test.rs`: HACK for tracing initialization
- `protocols/jupiter/jup-sdk/src/surfpool.rs`: TODO for debug info

**Impact**: Potential bugs

**Solution**: Address each HACK appropriately

**✅ RESOLVED**: Jupiter protocol TODOs for key_map parameters have been fixed by removing unused parameters

---

### 6. MOCK DATA HARDCODING

#### 📍 Location: `d_114_jup_positions_and_earnings.rs`
**Issue**: 40+ lines of hardcoded mock financial data

**Examples**:
```rust
"total_assets": "348342806597852",
"withdrawable": "36750926351916", 
"price": "0.99970715345",
"slot": 371334523
```

**Impact**: Unrealistic test data, hard to maintain

**Solution**: Generate mock data programmatically

---

### 7. ANTI-PATTERNS

#### 📍 Error Handling Anti-patterns
**Location**: Various error handling code
**Issue**: Using `unwrap()` and `expect()` in production code
**Impact**: Potential panics in production

**✅ RESOLVED**: Replaced critical unwrap() calls with proper error handling. Fixed regex compilation in lib.rs with context() error handling. Maintained acceptable unwrap usage in low-risk scenarios (internal mutex locks, constants validation).

#### 📍 String Formatting Anti-pattern  
**Location**: Multiple logging statements
**Issue**: Using `format!()` with single variable instead of `to_string()`
**Impact**: Unnecessary overhead

#### 📍 HashMap Cloning Anti-pattern
**Location**: `flow/agent.rs` and related files
**Issue**: Cloning entire HashMaps when only values needed
**Impact**: Performance overhead

---

### 8. NAMING CONVENTIONS

#### 📍 Location: Throughout codebase
**Issues**:
- Inconsistent naming: `key_map` vs `keyMap` vs `keymap`
- Generic names: `e`, `err`, `res` without context
- Abbreviations: `ata`, `pubkey`, `lamports` without full names in docs

**Impact**: Reduced readability, cognitive load

---

### 9. FUNCTION COMPLEXITY

#### 📍 Location: `lib.rs` (deterministic agent)
**Issue**: Large match statement with 20+ cases
**Lines**: 300+ lines in single function
**Impact**: Hard to test, understand, maintain

**Solution**: Break into smaller functions per benchmark type

---

### 10. MISSING VALIDATION

#### 📍 Location: Input parsing code
**Issue**: Insufficient validation of user inputs
**Examples**: 
- No validation of amount ranges (could overflow)
- No validation of address formats
- Missing bounds checking

**Impact**: Potential security vulnerabilities, crashes

---

## 🚨 Priority Fix Order

### HIGH PRIORITY (Security/Stability) ✅ COMPLETED
1. **TODOs in protocol.rs** - Incomplete implementations ✅
2. **Hardcoded addresses** - Centralize to prevent typos ✅
3. **Error handling** - Replace unwrap/expect ✅

### MEDIUM PRIORITY (Maintainability) 🔄 PARTIALLY COMPLETED
4. **Magic numbers** - Create constants module ✅
5. **Code duplication in examples** - Extract common helpers 🔄
6. **Function complexity** - Break down large functions

### LOW PRIORITY (Code Quality)
7. **Naming conventions** - Standardize across codebase
8. **Mock data** - Generate programmatically  
9. **Configuration** - Environment variables for ports

## 📊 Overall Progress Summary

### ✅ **COMPLETED (3/3 High Priority)**
- **Jupiter Protocol TODOs**: Removed unused key_map parameters across all handlers
- **Hardcoded Addresses**: Created comprehensive constants module with addresses and amounts
- **Error Handling**: Fixed critical unwrap() calls with proper error handling

### 🔄 **IN PROGRESS (1/3 Medium Priority)**
- **Code Duplication**: Created foundation with common helpers, needs full example migration
- **Magic Numbers**: Fully centralized in constants module ✅
- **Function Complexity**: Not yet addressed

### ⏳ **NOT STARTED (3 Low Priority)**
- **Naming Conventions**: Standardization needed
- **Mock Data**: Programmatic generation required
- **Configuration**: Environment variable implementation

### 🎯 **Next Steps**
1. Complete migration of all examples to use common helpers
2. Address function complexity in large match statements
3. Implement low-priority improvements as time permits

---

## 📋 Implementation Checklist

### Constants Module (`constants.rs`)
- [ ] Token mint addresses
- [ ] Program IDs  
- [ ] Default amounts (SOL, USDC)
- [ ] Slippage percentages
- [ ] Port numbers
- [ ] Rent exemption amounts

### Common Helpers (`examples/common.rs`)
- [ ] Health check function
- [ ] URL builder function
- [ ] Agent server startup sequence

### Address Registry (`addresses.rs`)
- [ ] Mainnet address constants
- [ ] Devnet address constants  
- [ ] Address validation functions

### Error Handling
- [x] Replace `unwrap()` with proper error handling
- [ ] Add input validation functions
- [ ] Create custom error types where needed