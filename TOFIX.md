# TOFIX.md

## SPL Transfer Address Resolution Race Condition - CRITICAL 🚨

### Problem Description
**002-spl-transfer.yml regression from 100% to 56% after context enrichment**

Root cause: **Address generation race condition** between environment reset and test scenario setup

### Current Architecture Flow
```
1. env.reset() → Generates random addresses for ALL placeholders
2. setup_spl_scenario() → Attempts to overwrite with correct derived addresses  
3. run_evaluation_loop() → LLM receives mixed/incorrect addresses
```

### Specific Issue
```rust
// RESET: Creates random addresses
USER_WALLET_PUBKEY → address_A  
RECIPIENT_WALLET_PUBKEY → address_B

// SETUP: Derives ATAs from random addresses  
USER_USDC_ATA → derived_from(address_A)  ✅
RECIPIENT_USDC_ATA → derived_from(address_B)  ❌

// RACE: If reset runs again after setup
USER_WALLET_PUBKEY → address_C (overwrites address_A!)
RECIPIENT_WALLET_PUBKEY → address_D (overwrites address_B!)

// LLM gets inconsistent context and creates wrong instructions
```

### Evidence from Logs
```
INFO [reset] Generated new address for placeholder 'USER_WALLET_PUBKEY': DBGZHPxVD4hds2LjXw46keEuRpJjM5Gva3ciQMChmL7
INFO [setup] Set state for 8Yvk3sMeu615qH4FKmn2Ye35z3Kxo7S5yh2BkPQaRru6 with owner DBGZHPxVD4hds2LjXw46keEuRpJjM5Gva3ciQMChmL7 and amount 50000000
```

### Root Cause
Environment reset generates addresses for placeholders that test scenarios should control. But current logic allows generating base wallet addresses for SPL benchmarks, creating race conditions.

### Current Fix Status
✅ **Context Resolver**: Fixed to skip SPL placeholder generation  
✅ **Environment Reset**: Partially fixed - still generates base wallet addresses  
❌ **Integration**: Still has race condition between reset and setup

### Required Fix
**Split responsibility cleanly**:
- **Environment Reset**: Only generate SYSTEM accounts (fee payer), not benchmark-specific accounts
- **Test Scenarios**: Handle ALL benchmark-specific address generation (wallets + derived ATAs)

This eliminates the race condition by ensuring clear ownership of address generation.

### Files to Modify
1. `crates/reev-lib/src/solana_env/reset.rs` - Line ~55
2. `crates/reev-lib/src/test_scenarios.rs` - Review setup ordering
3. Consider if additional coordination needed in `crates/reev-runner/src/lib.rs`

### Success Criteria
- `002-spl-transfer.yml` returns to 100% success rate
- All other SPL benchmarks work correctly
- SOL transfer benchmarks remain unaffected
