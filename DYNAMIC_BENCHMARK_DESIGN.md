# Dynamic Flow Benchmark Design (300-Series)

## 🎯 **Purpose & Philosophy**

The 300-series benchmarks demonstrate the full capabilities of the reev dynamic flow system by presenting complex, real-world scenarios that require:

- **Natural Language Understanding**: Complex prompts with percentages, risk levels, and strategic goals
- **Multi-Step Orchestration**: Automatic flow planning and execution coordination
- **Context Intelligence**: Real-time market analysis and wallet state resolution
- **Decision Making**: Strategic choices based on multiple data sources
- **Recovery Mechanisms**: Fault tolerance and fallback strategies

## 📊 **Benchmark Categories**

### **301: Yield Optimization** - *Intelligent Yield Seeking*
**Scenario**: User wants to maximize returns using 50% of SOL
**Key Capabilities Tested**:
- Percentage calculation and allocation
- Market rate analysis
- Optimal strategy selection
- Swap → Lend coordination
- Yield maximization logic

**Complexity Level**: ⭐⭐⭐ (Medium)

### **302: Portfolio Rebalancing** - *Strategic Asset Allocation*
**Scenario**: User wants to rebalance mixed holdings based on market conditions
**Key Capabilities Tested**:
- Portfolio analysis and valuation
- Market condition assessment
- Optimal allocation calculation
- Multi-direction trading (SOL↔USDC)
- Risk-aware positioning

**Complexity Level**: ⭐⭐⭐⭐ (High)

### **303: Risk-Adjusted Growth** - *Conservative Capital Management*
**Scenario**: User wants growth with capital preservation using 30% of SOL
**Key Capabilities Tested**:
- Risk tolerance assessment
- Conservative strategy implementation
- Capital preservation logic
- Liquidity buffer management
- Controlled exposure calculation

**Complexity Level**: ⭐⭐⭐ (Medium-High)

### **304: Emergency Exit Strategy** - *Crisis Management & Recovery*
**Scenario**: User needs immediate liquidation due to market stress
**Key Capabilities Tested**:
- Emergency context recognition
- Position liquidation coordination
- Asset consolidation (stable conversion)
- Speed-optimized execution
- Recovery mechanism activation

**Complexity Level**: ⭐⭐⭐⭐⭐ (Very High)

### **305: Yield Farming Optimization** - *Advanced Multi-Pool Strategy*
**Scenario**: User wants to maximize yield across multiple Jupiter pools using 70% capital
**Key Capabilities Tested**:
- Multi-pool analysis and comparison
- APY optimization algorithms
- Diversification strategies
- Auto-compounding integration
- Complex capital allocation

**Complexity Level**: ⭐⭐⭐⭐⭐⭐ (Expert)

## 🏗️ **Design Patterns**

### **1. Progressive Complexity**
- **301**: Basic optimization (single pool, clear percentage)
- **302**: Multi-variable analysis (portfolio, market conditions)
- **303**: Risk constraints (conservative parameters)
- **304**: Crisis response (time pressure, safety first)
- **305**: Advanced optimization (multi-pool, mathematical optimization)

### **3. Natural Language to Tool Call Mapping**
```yaml
# Simple (301) - Direct instructions
+"Use my 50% SOL to maximize my USDC returns through Jupiter lending."
→ Tools: [account_balance] → [jupiter_swap] → [jupiter_lend]

# Complex (302) - Portfolio analysis required
+"I want to rebalance my portfolio based on current market conditions."
→ Tools: [account_balance, jupiter_positions] → Analysis → [jupiter_swap] → [jupiter_lend]

# Emergency (304) - Crisis response
+"I need an emergency exit strategy for all my positions due to market stress."
→ Tools: [account_balance, jupiter_positions] → [jupiter_withdraw] → [jupiter_swap] → Stable assets
```

### **3. Capital Allocation Patterns**
| Benchmark | SOL Usage | USDC Usage | Strategy Type | Risk Level |
|------------|-------------|--------------|----------------|-------------|
| 301 | 50% | Existing + New | Yield Maximization | Moderate |
| 302 | Variable | Variable | Portfolio Optimization | Moderate-High |
| 303 | 30% | Conservative | Growth with Preservation | Low-Moderate |
| 304 | Minimal (fees) | Maximum | Emergency Preservation | Crisis Mode |
| 305 | 70% | 70% | Yield Farming Optimization | High |

### **4. Tool Call Evolution & Success Criteria**

**Basic Tool Sequence (301)**:
```yaml
Expected Tools:
  - account_balance (context)
  - jupiter_swap (execution) 
  - jupiter_lend (yield)
  - jupiter_positions (validation)
```

**Intermediate Tool Sequences (302-303)**:
```yaml
Expected Tools (302 - Rebalancing):
  - account_balance (initial state)
  - jupiter_positions (current holdings)
  - jupiter_swap (rebalancing trades)
  - jupiter_lend (new positions)
  - jupiter_positions (final validation)

Expected Tools (303 - Risk-Adjusted):
  - account_balance (capital assessment)
  - jupiter_lend_rates (risk analysis)
  - jupiter_swap (partial conversion)
  - jupiter_lend (conservative deposit)
```

**Advanced Tool Orchestrations (304-305)**:
```yaml
Expected Tools (304 - Emergency Exit):
  - account_balance (position analysis)
  - jupiter_positions (withdraw targets)
  - jupiter_withdraw (rapid liquidation)
  - jupiter_swap (stable conversion)
  - account_balance (final validation)

Expected Tools (305 - Multi-Pool):
  - account_balance (total capital)
  - jupiter_pools (pool analysis)
  - jupiter_lend_rates (apy comparison)
  - jupiter_swap (pool conversions)
  - jupiter_lend (multi-pool deposits)
```

## 🧪 **Testing Strategy**

### **OpenTelemetry Integration Testing**
```rust
// Tool Call Tracking
POST /api/v1/benchmarks/execute-direct
- Triggers: account_balance, jupiter_swap, jupiter_lend tools
- Validates: OpenTelemetry captures tool_name, parameters, execution_time
- Confirms: Sequential tool execution with proper dependencies

// Recovery Mode Testing  
POST /api/v1/benchmarks/execute-recovery
- Tests: Tool failure scenarios and recovery mechanisms
- Validates: Automatic retry with exponential backoff
- Confirms: Alternative tool sequences when primary fails

// Flow Visualization from OTEL
GET /api/v1/flows/{session_id}
- Tests: Mermaid diagram generation from tool call traces
- Validates: State transitions (account_balance → swap → lend → validation)
- Confirms: Dynamic flow detection by execution_id prefixes
```

### **Progressive Validation**
1. **Unit Tests**: Individual component validation
2. **Integration Tests**: Multi-component coordination
3. **Flow Tests**: End-to-end execution
4. **Performance Tests**: < 50ms overhead verification
5. **Recovery Tests**: Failure scenario handling

### **Expected Tool Call Success Rates**
| Benchmark | Min Score | Target Score | Tool Call Success | Expected Pass Rate |
|-----------|-------------|---------------|------------------|-------------------|
| 301 | 0.7 | 0.8+ | 3/4 tools | 90%+ |
| 302 | 0.7 | 0.75+ | 4/5 tools | 85%+ |
| 303 | 0.75 | 0.8+ | 4/5 tools | 85%+ |
| 304 | 0.8 | 0.85+ | 5/6 tools | 80%+ |
| 305 | 0.75 | 0.8+ | 5/7 tools | 75%+ |

**Tool Call Validation:**
- **account_balance**: State discovery, non-critical
- **jupiter_swap**: Execution step, critical
- **jupiter_lend**: Yield generation, critical  
- **jupiter_positions**: Validation step, non-critical
- **jupiter_withdraw**: Emergency response, critical
- **jupiter_pools**: Analysis step, non-critical

## 📈 **Performance Metrics**

### **Expected Execution Characteristics**
| Metric | Target | Measurement Method |
|--------|---------|-------------------|
| **Flow Generation Time** | < 200ms | Orchestrator timing |
| **Context Resolution** | < 500ms | API call aggregation |
| **Total Execution Time** | < 5s | End-to-end timing |
| **Memory Overhead** | < 2KB | Flow object sizing |
| **API Call Efficiency** | Minimal | Count of external calls |
| **Recovery Overhead** | < 100ms | Recovery mechanism timing |

### **OpenTelemetry-Based Quality Metrics**
- **Tool Call Success Rate**: > 95% (agent executes correct tools)
- **Parameter Accuracy**: > 90% (correct params passed to tools)
- **Tool Sequence Logic**: > 85% (logical flow between tools)
- **Recovery Mechanism Success**: > 80% (fallback strategies work)
- **Flow Visualization Generation**: 100% (Mermaid from OTEL traces)
- **Performance Overhead**: < 50ms (tool call execution time)

## 🎮 **Usage Examples**

### **Development Testing**
```bash
# Direct mode testing
reev-runner --direct \
  --prompt "Use my 50% SOL to maximize my USDC returns through Jupiter lending" \
  --wallet TestWallet123 \
  --agent glm-4.6-coding

# Recovery mode testing (for 304)
reev-runner --recovery \
  --prompt "Emergency exit strategy for all positions" \
  --wallet EmergencyWallet456 \
  --agent glm-4.6-coding \
  --max-recovery-time-ms 20000
```

### **API Testing**
```bash
# Execute benchmark 301
curl -X POST http://localhost:3001/api/v1/benchmarks/execute-direct \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "Use my 50% SOL to maximize my USDC returns through Jupiter lending",
    "wallet": "TestWallet123",
    "agent": "glm-4.6-coding",
    "shared_surfpool": false
  }'

# Get flow visualization
curl -H "Accept: application/json" \
  http://localhost:3001/api/v1/flows/direct-session123
```

## 🔄 **Continuous Improvement**

### **Benchmark Evolution Path**
1. **Phase 1**: Core scenarios (301-303) - Establish baseline
2. **Phase 2**: Advanced scenarios (304-305) - Push complexity limits
3. **Phase 3**: Specialized scenarios (306-310) - Domain-specific strategies
4. **Phase 4**: Adaptive scenarios (311-315) - Machine learning integration

### **Success Metrics Tracking**
- **Execution Success Rates**: Monitor over time
- **Natural Language Accuracy**: Improve parsing capabilities
- **Flow Optimization**: Reduce execution overhead
- **Recovery Effectiveness**: Increase success rate of fallbacks
- **User Satisfaction**: Gather qualitative feedback

### **Expected Demonstrations - Tool Call Intelligence**

### **Dynamic Flow Capabilities via Tool Calls**
1. **Natural Language → Tool Sequence**: Complex prompts to logical tool orchestration
2. **Context-Aware Tool Selection**: Tools selected based on wallet state and goals
3. **Intelligent Parameter Passing**: Correct tool parameters from prompt analysis
4. **Multi-Step Tool Orchestration**: Sequential tool execution with dependencies
5. **Fault-Tolerant Tool Execution**: Recovery mechanisms when tools fail
6. **OTEL-Based Monitoring**: Complete tool call tracking and visualization
7. **Performance Optimization**: < 50ms overhead per tool call

### **Tool Call Examples by Scenario:**

**301 Yield Optimization:**
```
account_balance → jupiter_swap → jupiter_lend → jupiter_positions
```

**304 Emergency Exit:**
```
account_balance → jupiter_positions → jupiter_withdraw → jupiter_swap → account_balance
```

**305 Multi-Pool:**
```
account_balance → jupiter_pools → jupiter_lend_rates → jupiter_swap → jupiter_lend → jupiter_positions
```

### **Production Readiness Indicators**
- **All benchmarks passing** with target scores
- **Zero compilation warnings** and clean code
- **Comprehensive test coverage** for all scenarios
- **API integration working** with all endpoints
- **Documentation complete** with examples and guidelines
- **Performance targets met** across all metrics

---

**The 300-series benchmarks provide a comprehensive demonstration of the reev dynamic flow system's ability to handle complex, real-world DeFi scenarios with intelligence, reliability, and enterprise-grade robustness.**