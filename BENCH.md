# 🏆 Benchmark Golden Rules Guide

## 📋 Purpose
Essential principles for developing reliable benchmarks that test AI agents on Solana protocols.

---

## 🎯 Golden Rules

### 1. **Placeholder Resolution is CRITICAL**
```rust
// ALWAYS resolve placeholders from key_map first
let resolved_value = self.key_map.get(&placeholder)
    .unwrap_or(&fallback_value);
```
- **Never** use simulated pubkeys (`111111111...`) when real ones are available
- **Always** check key_map before falling back to placeholders
- **Test** both resolved and placeholder scenarios

### 2. **Real Transaction Execution over Simulation**
- Generate actual on-chain transactions, not mock responses
- Validate transactions execute with real signatures
- Use proper Solana account relationships and ownership

### 3. **Tool Integration Pattern**
```rust
// Standard tool signature for Solana operations
pub struct ExampleTool {
    pub key_map: HashMap<String, String>,
}

impl Tool for ExampleTool {
    const NAME: &'static str = "example_tool";
    type Args = ExampleArgs;
    type Output = String;
    
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // 1. Validate business logic
        if args.amount == 0 {
            return Err(ExampleError::InvalidAmount);
        }
        
        // 2. Resolve placeholders from key_map
        let user_pubkey = self.resolve_pubkey(&args.user_pubkey)?;
        
        // 3. Call protocol handler
        let instructions = handle_protocol(user_pubkey, args.amount, &self.key_map).await?;
        
        // 4. Return serialized result
        Ok(serde_json::to_string(&instructions)?)
    }
}
```

### 4. **Benchmark Structure**
```yaml
id: unique-benchmark-id
description: Clear, unambiguous description
tags: ["protocol", "operation", "token"]

initial_state:
  - pubkey: "USER_WALLET_PUBKEY"
    owner: "11111111111111111111111111111111"
    lamports: 5000000000
  - pubkey: "USER_TOKEN_ATA_PLACEHOLDER"
    owner: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
    lamports: 2039280
    data:
      mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
      owner: "USER_WALLET_PUBKEY"
      amount: "100000000"

prompt: "Clear, specific action description. My wallet is USER_WALLET_PUBKEY."

ground_truth:
  final_state_assertions:
    - type: SolBalanceChange
      pubkey: "USER_WALLET_PUBKEY"
      expected_change_gte: -105000000
      weight: 1.0
```

### 5. **Prompt Engineering Principles**
- **Be Specific**: Use exact protocol terminology (deposit vs mint)
- **Avoid Ambiguity**: Don't mix conflicting concepts in one prompt
- **Clear Intent**: State exactly what should happen
- **Context Aware**: Reference wallet/pubkey placeholders consistently

---

## 🚨 Common Pitfalls to Avoid

### ❌ **Bad Placeholder Handling**
```rust
// WRONG: Always uses simulated pubkey
if args.user_pubkey.starts_with("USER_") {
    return Pubkey::from_str("11111111111111111111111111111111")?;
}
```

### ✅ **Correct Placeholder Resolution**
```rust
// RIGHT: Resolve from key_map first
if args.user_pubkey.starts_with("USER_") {
    if let Some(resolved) = self.key_map.get(&args.user_pubkey) {
        return Pubkey::from_str(resolved)?;
    }
    return Pubkey::from_str("11111111111111111111111111111111")?;
}
```

### ❌ **Vague Benchmarks**
```yaml
prompt: "Do something with tokens"
```

### ✅ **Specific Benchmarks**
```yaml
prompt: "Deposit 50 USDC into Jupiter lending. My wallet is USER_WALLET_PUBKEY."
```

---

## 📊 Success Metrics

### **Expected Results**
- **Deterministic Agent**: 100% success rate (baseline)
- **Enhanced Agent**: 85%+ success rate (target)
- **Real Transactions**: All benchmarks should execute on-chain
- **Proper Signatures**: Valid transaction signatures for successful operations

### **Validation Criteria**
1. **Instruction Score**: 1.0 (correct protocol calls)
2. **On-Chain Score**: 1.0 (transaction executes successfully)  
3. **Final Score**: 1.0 (weighted average)

---

## 🔧 Implementation Checklist

- [ ] Placeholder resolution from key_map
- [ ] Real transaction generation
- [ ] Proper Solana account relationships
- [ ] Clear, unambiguous prompts
- [ ] Comprehensive final state assertions
- [ ] Both resolved and placeholder test scenarios
- [ ] Error handling for edge cases
- [ ] Integration with protocol SDKs

---

## 🎯 Key Insight

**The difference between a working benchmark and a failing one often comes down to one thing: proper placeholder resolution.** Always resolve placeholders from the test environment key_map before generating transactions.

This ensures that AI agents can bridge the gap between their abstract understanding and concrete blockchain execution, enabling real-world functionality rather than just simulation.