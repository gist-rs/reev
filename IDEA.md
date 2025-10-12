# 🌊 Reev Framework: Core Architecture & Scoring System

## 🎯 Executive Summary

The Reev framework implements a sophisticated two-tiered scoring system with comprehensive flow benchmark support for evaluating Solana LLM agents across the full spectrum of performance outcomes.

## 🔒 API-Only Instruction Generation Principle

**Core Rule**: All DeFi protocol instructions must come from official APIs, never from LLM generation.

### Jupiter Operations: API-Exclusive
- ✅ **Required**: Use official Jupiter SDK methods (`get_swap_instructions`, `get_deposit_instructions`, `get_withdraw_instructions`)
- ❌ **Forbidden**: LLM-generated instructions, custom data encoding, API response modification

### Implementation Example
```rust
// ✅ Correct: API-only approach
let (instructions, _) = jupiter_client
    .swap(swap_params)
    .prepare_transaction_components()
    .await?;

// ❌ Never: LLM-generated instructions
// let instruction = Instruction { program_id: "...", data: llm_generated_data };
```

## 📊 Two-Tiered Scoring System

### Formula
```
Final Score = (Instruction Score × 75%) + (On-Chain Score × 25%)
```

### Component Breakdown
- **Instruction Score (75%)**: 
  - Program ID matching (50% weight)
  - Instruction data validation (50% weight)
  - Account metadata verification (25% per account)
- **On-Chain Score (25%)**: Binary execution success/failure

### 🌊 Flow Benchmark Scoring
Multi-step workflows evaluated step-by-step:

```
Flow Score = (Σ(Step Scores) / Number of Steps) × Flow Success Factor
```

**Flow Success Factors:**
- Complete Success: 1.0 (all critical steps succeed)
- Partial Success: 0.8 (non-critical steps may fail)
- Critical Failure: 0.5 (critical steps fail)
- Complete Failure: 0.0 (no steps succeed)

## 🧪 Validated Score Scenarios

| Benchmark | Expected | Actual | Purpose |
|-----------|----------|---------|---------|
| `001-sol-transfer` | 100% | ✅ 100% | Perfect execution |
| `003-spl-transfer-fail` | 0% | ✅ 0% | Complete failure |
| `004-partial-score-spl-transfer` | ~50% | ✅ 53.6% | Partial credit |
| `200-jup-swap-then-lend-deposit` | 100% | ✅ 100% | Multi-step flow |

## 🛡️ Anti-False-Positive Protection

Differentiated failure modes:
- **No Attempt** (0%): Agent doesn't generate instructions
- **Partial Attempt** (25-75%): Agent tries but makes mistakes
- **Good Attempt, Bad Execution** (~75%): Correct reasoning, execution failure
- **Perfect Execution** (100%): Everything works correctly

## 🔧 Implementation Architecture

### Regular Benchmark Structure
```yaml
ground_truth:
  expected_instructions:
    - program_id: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
      program_id_weight: 0.5
      data: "3Bxs4vKJW"
      data_weight: 0.5
```

### Flow Benchmark Structure
```yaml
flow:
  - step: 1
    description: "Swap SOL to USDC"
    prompt: "Swap 0.5 SOL to USDC using Jupiter."
    critical: true
    timeout: 30
  - step: 2
    description: "Deposit USDC to lending"
    prompt: "Deposit received USDC into Jupiter lending."
    depends_on: ["step_1_result"]
```

## 📈 Testing Strategy

### Validation Coverage
- ✅ 0% score scenario (complete failure)
- ✅ ~50% score scenario (partial credit)
- ✅ ~75% score scenario (reasoning success, execution failure)
- ✅ 100% score scenario (perfect execution)
- ✅ Flow benchmark support (200-series)
- ✅ Step-by-step execution with transaction isolation

### Agent Consistency
Both deterministic and AI agents handle flows identically through the same interface, ensuring consistent evaluation across all agent types.

## 🎯 Production Status

The Reev framework's scoring system is production-ready and thoroughly validated across multiple benchmark categories, flow scenarios, and failure modes. The two-tiered approach provides fair, comprehensive assessment of agent capabilities with robust anti-false-positive protection.

**Key Features:**
- API-only instruction generation for security
- Two-tiered scoring with granular component validation
- Multi-step flow support with transaction isolation
- Comprehensive test coverage across failure modes
- Consistent evaluation across deterministic and AI agents