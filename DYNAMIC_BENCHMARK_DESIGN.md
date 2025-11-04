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

### **2. Natural Language Complexity**
```yaml
# Simple (301)
"Use my 50% SOL to maximize my USDC returns through Jupiter lending."

# Complex (302)
"I want to rebalance my portfolio based on current market conditions. Please analyze my current holdings (SOL and USDC), check current market prices and Jupiter lending rates, then execute optimal rebalancing to maximize returns while maintaining some liquidity."

# Emergency (304)
"I need an emergency exit strategy for all my positions due to market stress. Please immediately analyze my current holdings, withdraw all lending positions, convert risky assets to stable ones, and preserve capital."
```

### **3. Capital Allocation Patterns**
| Benchmark | SOL Usage | USDC Usage | Strategy Type | Risk Level |
|------------|-------------|--------------|----------------|-------------|
| 301 | 50% | Existing + New | Yield Maximization | Moderate |
| 302 | Variable | Variable | Portfolio Optimization | Moderate-High |
| 303 | 30% | Conservative | Growth with Preservation | Low-Moderate |
| 304 | Minimal (fees) | Maximum | Emergency Preservation | Crisis Mode |
| 305 | 70% | 70% | Yield Farming Optimization | High |

### **4. Success Criteria Evolution**

**Basic Validation (301)**:
- Context resolution ✓
- Percentage calculation ✓
- Yield optimization ✓

**Intermediate Validation (302-303)**:
- Portfolio analysis ✓
- Market assessment ✓
- Risk management ✓

**Advanced Validation (304-305)**:
- Emergency response ✓
- Multi-pool coordination ✓
- Recovery mechanisms ✓
- Complex optimization ✓

## 🧪 **Testing Strategy**

### **API Integration Testing**
```rust
// Direct Mode Testing
POST /api/v1/benchmarks/execute-direct
- Tests natural language processing
- Validates flow generation
- Confirms execution coordination

// Recovery Mode Testing  
POST /api/v1/benchmarks/execute-recovery
- Tests emergency scenarios (304)
- Validates recovery config
- Confirms fallback mechanisms

// Visualization Testing
GET /api/v1/flows/{session_id}
- Tests dynamic flow detection
- Validates enhanced Mermaid diagrams
- Confirms HTTP caching headers
```

### **Progressive Validation**
1. **Unit Tests**: Individual component validation
2. **Integration Tests**: Multi-component coordination
3. **Flow Tests**: End-to-end execution
4. **Performance Tests**: < 50ms overhead verification
5. **Recovery Tests**: Failure scenario handling

### **Expected Success Rates**
| Benchmark | Min Score | Target Score | Expected Pass Rate |
|-----------|-------------|---------------|-------------------|
| 301 | 0.7 | 0.8+ | 90%+ |
| 302 | 0.7 | 0.75+ | 85%+ |
| 303 | 0.75 | 0.8+ | 85%+ |
| 304 | 0.8 | 0.85+ | 80%+ |
| 305 | 0.75 | 0.8+ | 75%+ |

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

### **Quality Assurance Metrics**
- **Natural Language Success Rate**: > 95%
- **Step Completion Rate**: > 90%
- **Error Recovery Rate**: > 85%
- **Flow Visualization Generation**: 100%
- **HTTP Caching Efficiency**: > 80% hit rate

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

## 🎯 **Expected Demonstrations**

### **Dynamic Flow Capabilities**
1. **Natural Language → Action**: Complex prompts to executable steps
2. **Context Awareness**: Real-time wallet and market integration
3. **Intelligent Decision Making**: Optimal strategy selection
4. **Multi-Step Coordination**: Seamless step orchestration
5. **Fault Tolerance**: Recovery mechanisms and fallbacks
6. **Performance Optimization**: Minimal overhead with maximum efficiency
7. **Real-Time Monitoring**: Live flow visualization and status tracking

### **Production Readiness Indicators**
- **All benchmarks passing** with target scores
- **Zero compilation warnings** and clean code
- **Comprehensive test coverage** for all scenarios
- **API integration working** with all endpoints
- **Documentation complete** with examples and guidelines
- **Performance targets met** across all metrics

---

**The 300-series benchmarks provide a comprehensive demonstration of the reev dynamic flow system's ability to handle complex, real-world DeFi scenarios with intelligence, reliability, and enterprise-grade robustness.**