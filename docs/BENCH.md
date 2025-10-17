# 🪸 Benchmark Development Guide

## 🎯 Overview

This guide covers creating, testing, and validating benchmarks for the `reev` framework. Benchmarks define evaluation scenarios for Solana LLM agents using declarative YAML files with initial state, prompts, and ground truth criteria.

## 📋 Benchmark Structure

### Core Components

```yaml
id: unique-benchmark-id
description: Clear description of what this benchmark tests
tags: ["protocol", "operation-type", "category"]

initial_state:
  - pubkey: "PLACEHOLDER_NAME"
    owner: "PROGRAM_PUBKEY"
    lamports: 1000000000
    data: # Optional account data

prompt: "Natural language task description for the agent"

ground_truth:
  final_state_assertions: []
  expected_instructions: []
  skip_instruction_validation: false
```

### Flow Benchmarks (Multi-Step)

```yaml
flow:
  - step: 1
    description: "First action description"
    prompt: "Specific instruction for this step"
    critical: true
    timeout: 30
    depends_on: []  # Optional dependencies

  - step: 2
    description: "Second action"
    prompt: "Follow-up instruction"
    critical: true
    timeout: 30
    depends_on: ["step_1_result"]
```

## 🏗️ Benchmark Categories

### 1. Transaction Benchmarks (100-series)
Single transaction operations with state changes:

**SOL Transfer Example:**
```yaml
id: 001-sol-transfer
description: Basic SOL transfer from one wallet to another
tags: ["system-program", "transfer", "t2"]

initial_state:
  - pubkey: "USER_WALLET_PUBKEY"
    owner: "11111111111111111111111111111111"
    lamports: 1000000000

  - pubkey: "RECIPIENT_WALLET_PUBKEY" 
    owner: "11111111111111111111111111111111"
    lamports: 0

prompt: "Please send 0.1 SOL to the recipient (RECIPIENT_WALLET_PUBKEY)."

ground_truth:
  final_state_assertions:
    - type: SolBalance
      pubkey: "RECIPIENT_WALLET_PUBKEY"
      expected: 100000000
      weight: 1.0

  expected_instructions:
    - program_id: "11111111111111111111111111111111"
      program_id_weight: 1.0
      accounts:
        - pubkey: "USER_WALLET_PUBKEY"
          is_signer: true
          is_writable: true
          weight: 1.0
        - pubkey: "RECIPIENT_WALLET_PUBKEY" 
          is_signer: false
          is_writable: true
          weight: 1.0
      data: "..."
```

### 2. Jupiter Protocol Benchmarks (100-116 series)
Real DeFi operations using Jupiter protocols:

**Jupiter Swap Example:**
```yaml
id: 100-jup-swap-sol-usdc
description: Swap SOL for USDC using Jupiter aggregator
tags: ["jupiter", "swap", "defi"]

initial_state:
  - pubkey: "USER_WALLET_PUBKEY"
    owner: "11111111111111111111111111111111"
    lamports: 1000000000

  - pubkey: "USER_USDC_ATA"
    owner: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
    lamports: 2039280
    data:
      mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
      owner: "USER_WALLET_PUBKEY"
      amount: "0"

prompt: "Swap 0.1 SOL for USDC using Jupiter."

ground_truth:
  skip_instruction_validation: true  # API-based benchmark
  
  final_state_assertions:
    - type: SolBalance
      pubkey: "USER_WALLET_PUBKEY"
      expected_approx: 899000000  # Account for slippage + fees
      weight: 0.3
    
    - type: TokenAccountBalance
      pubkey: "USER_USDC_ATA"
      mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
      expected_approx: 10000000  # ~10 USDC with 6 decimals
      weight: 0.7
```

### 3. Flow Benchmarks (200-series)
Multi-step workflows orchestrated by the FlowAgent:

**Multi-Step Flow Example:**
```yaml
id: 200-jup-swap-then-lend-deposit
description: Multi-step flow - Swap SOL to USDC then deposit into Jupiter lending
tags: ["jupiter", "swap", "lend", "multi-step", "flow", "yield"]

initial_state:
  - pubkey: "USER_WALLET_PUBKEY"
    owner: "11111111111111111111111111111111"
    lamports: 5000000000

  - pubkey: "USER_USDC_ATA_PLACEHOLDER"
    owner: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
    lamports: 2039280
    data:
      mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
      owner: "USER_WALLET_PUBKEY"
      amount: "0"

prompt: "I want to earn yield on my SOL by converting it to USDC and depositing into Jupiter lending. Can you help me swap 0.1 SOL to USDC and then deposit the USDC into Jupiter lending?"

flow:
  - step: 1
    description: "Swap 0.1 SOL to USDC using Jupiter"
    prompt: "I want to swap 0.1 SOL for USDC using Jupiter."
    critical: true
    timeout: 30

  - step: 2
    description: "Deposit received USDC into Jupiter lending"
    prompt: "I want to deposit all the USDC I received from the swap into Jupiter lending to start earning yield."
    depends_on: ["step_1_result"]
    critical: true
    timeout: 30

ground_truth:
  min_score: 0.6
  
  final_state_assertions:
    - type: SolBalance
      pubkey: "USER_WALLET_PUBKEY"
      expected_approx: 1500000000  # Account for both operations
      weight: 0.3

    - type: TokenAccountBalance
      pubkey: "USER_USDC_ATA_PLACEHOLDER"
      mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
      expected: 50000000  # Remaining USDC after deposit
      weight: 0.4

  success_criteria:
    - type: "steps_completed"
      description: "All critical steps must be completed successfully"
      required: 2
      weight: 0.5

    - type: "no_critical_errors"
      description: "No errors in critical steps"
      required: true
      weight: 0.5
```

## 🔧 Ground Truth Configuration

### Final State Assertions

#### SolBalance
```yaml
- type: SolBalance
  pubkey: "PLACEHOLDER_NAME"
  expected: EXACT_LAMPORTS
  expected_approx: APPROXIMATE_LAMPORTS  # Use for variable fees
  weight: 1.0
```

#### TokenAccountBalance
```yaml
- type: TokenAccountBalance
  pubkey: "PLACEHOLDER_NAME"
  mint: "TOKEN_MINT_PUBKEY"
  expected: EXACT_AMOUNT
  expected_approx: APPROXIMATE_AMOUNT
  weight: 1.0
```

#### JupiterLendingPosition
```yaml
- type: JupiterLendingPosition
  pubkey: "USER_WALLET_PUBKEY"
  mint: "TOKEN_MINT_PUBKEY"
  expected: DEPOSIT_AMOUNT
  expected_approx: APPROXIMATE_AMOUNT
  weight: 1.0
```

### Expected Instructions

For non-API benchmarks (system program, simple transfers):

```yaml
expected_instructions:
  - program_id: "PROGRAM_PUBKEY"
    program_id_weight: 0.5
    instruction_count: 1  # For single instruction
    instruction_count_range: [2, 8]  # For variable instruction counts
    weight: 1.0
    accounts:
      - pubkey: "PLACEHOLDER_NAME"
        is_signer: true
        is_writable: true
        weight: 1.0
    data: "BASE58_ENCODED_INSTRUCTION_DATA"
    data_weight: 0.5
```

### API-First Protocol Configuration

For Jupiter and other complex protocols:

```yaml
ground_truth:
  skip_instruction_validation: true  # Critical for API-based protocols
  
  final_state_assertions:
    # Focus on end results, not instruction structure
    - type: TokenAccountBalance
      pubkey: "USER_USDC_ATA"
      mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
      expected_approx: 10000000
      weight: 0.7
```

## 📝 Best Practices

### 1. Placeholder Naming

Use clear, descriptive placeholder names:
- `USER_WALLET_PUBKEY` - Primary user wallet
- `RECIPIENT_WALLET_PUBKEY` - Transfer recipient
- `USER_USDC_ATA` - User's USDC token account
- `USER_SOL_ATA` - User's wrapped SOL token account

### 2. Amount Calculations

Always account for fees and slippage:

```yaml
# For transfers: expected = initial - amount - fees
expected: 899995000  # 1 SOL - 0.1 SOL - 5000 lamports fee

# For swaps: use expected_approx for slippage
expected_approx: 9500000  # Account for Jupiter slippage

# For lending: account for reserve fractions
expected_approx: 9800000  # Small deduction from lending protocol
```

### 3. Real Address Usage

**Critical**: All addresses in `initial_state` must be real mainnet addresses:

```yaml
# ✅ Correct: Real USDC mint
mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"

# ❌ Wrong: Fictional addresses
mint: "FakeTokenMint11111111111111111111111111"
```

### 4. Protocol-Specific Tips

#### Jupiter Operations
- Always use `skip_instruction_validation: true`
- Focus on final state, not instruction structure
- Account for multi-transaction operations (swap = 4-8 instructions)
- Use `expected_approx` for all token amounts due to slippage

#### Lending Operations
- Test both deposit and withdraw scenarios
- Account for interest accrual in multi-step tests
- Verify both token balance and lending position

#### Multi-Step Flows
- Mark `critical: true` for essential steps
- Use `depends_on` for sequential dependencies
- Set reasonable timeouts (30-60 seconds)
- Define `success_criteria` for partial credit scenarios

## 🧪 Testing Benchmarks

### 1. Validate YAML Syntax
```bash
# Check YAML validity
python -c "import yaml; yaml.safe_load(open('benchmarks/your-benchmark.yml'))"
```

### 2. Test with Deterministic Agent
```bash
# Validate ground truth
cargo run -p reev-runner -- benchmarks/your-benchmark.yml --agent deterministic
```

### 3. Test Score Scenarios
```bash
# Test 0% score (should fail)
# Modify benchmark to create failure scenario
cargo run -p reev-runner -- benchmarks/your-benchmark-fail.yml --agent deterministic

# Test 100% score (should pass)
cargo run -p reev-runner -- benchmarks/your-benchmark.yml --agent deterministic
```

### 4. Cross-Agent Validation
```bash
# Test with different agents
cargo run -p reev-runner -- benchmarks/your-benchmark.yml --agent local
cargo run -p reev-runner -- benchmarks/your-benchmark.yml --agent gemini-2.5-flash-lite
```

## 🔍 Debugging Failed Benchmarks

### 1. Check Initial State
```bash
# Enable debug logging
RUST_LOG=debug cargo run -p reev-runner -- benchmarks/your-benchmark.yml --agent deterministic
```

### 2. Validate Ground Truth
- Ensure all placeholder names match between `initial_state` and `ground_truth`
- Check that expected amounts account for fees
- Verify real addresses are used for all mainnet dependencies

### 3. Score Breakdown
The framework provides detailed scoring information:
- Instruction Score (75%): How closely generated instructions match ground truth
- On-Chain Score (25%): Whether the transaction executed successfully

### 4. Common Issues

**Address Not Found:**
```
Error: Account not found: FakeAddress123...
```
Solution: Use real mainnet addresses

**Amount Mismatch:**
```
Expected: 100000000, Got: 99995000
```
Solution: Account for transaction fees and slippage

**Instruction Count Wrong:**
```
Expected: 1 instruction, Got: 6 instructions
```
Solution: Use `skip_instruction_validation` for complex protocols

## 📋 Benchmark Checklist

Before submitting a benchmark:

- [ ] YAML syntax is valid
- [ ] All placeholder names are consistent
- [ ] Real mainnet addresses used
- [ ] Amounts account for fees/slippage
- [ ] Description clearly explains purpose
- [ ] Tags match protocol and operation type
- [ ] `skip_instruction_validation` set correctly for API protocols
- [ ] Weights sum to reasonable values (usually 1.0)
- [ ] Tested with deterministic agent (should get 100%)
- [ ] Tested failure scenario (should get 0% or partial credit)
- [ ] Cross-agent validation completed
- [ ] Documentation added for complex scenarios

## 🚀 Advanced Features

### Custom Success Criteria
```yaml
success_criteria:
  - type: "steps_completed"
    description: "Complete all critical steps"
    required: 2
    weight: 0.5

  - type: "final_balance_positive"
    description: "End with positive token balance"
    required: true
    weight: 0.3

  - type: "transaction_count"
    description: "Execute within reasonable transaction count"
    min: 1
    max: 10
    weight: 0.2
```

### Conditional Assertions
```yaml
final_state_assertions:
  - type: TokenAccountBalance
    pubkey: "USER_USDC_ATA"
    condition: "greater_than_zero"  # Custom condition
    weight: 1.0
```

### Dynamic Discovery Patterns
For complex protocols with dynamically generated addresses:

```rust
// Development tooling for finding real addresses
async fn discover_jupiter_addresses() -> Result<HashMap<String, String>> {
    // Query mainnet for real program addresses
    // Store discovered addresses for benchmark use
}
```

This comprehensive guide provides everything needed to create robust, production-ready benchmarks for the reev framework.