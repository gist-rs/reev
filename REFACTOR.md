# Reev Agent Refactoring Plan

## 🎯 Objectives

1. **Modular Architecture**: Separate protocol handlers from AI tools
2. **Extensibility**: Easy addition of new protocols (Drift, Kamino, etc.)
3. **Feature Flags**: Compile-time protocol selection
4. **Consistency**: Unified error handling and configuration

## 🏗️ Current Directory Structure (After Phase 1)

```
crates/reev-agent/src/
├── protocols/              # ✅ Protocol-specific API handlers
│   ├── mod.rs
│   ├── jupiter/            # ✅ Complete Jupiter protocol implementation
│   │   ├── mod.rs          # ✅ Configuration and utilities
│   │   ├── earnings.rs     # ✅ Jupiter earn API (positions + earnings)
│   │   ├── lend_deposit.rs # ✅ Jupiter lend deposit API
│   │   ├── lend_withdraw.rs# ✅ Jupiter lend withdraw API
│   │   ├── positions.rs    # ✅ Jupiter positions API
│   │   └── swap.rs         # ✅ Jupiter swap API (uses jup-sdk)
│   ├── drift/              # 🔄 Future: Drift protocol
│   ├── kamino/             # 🔄 Future: Kamino protocol
│   └── native/             # 🔄 Future: Native Solana operations
├── tools/                  # ✅ AI tool wrappers (thin layer on top of protocols)
│   ├── mod.rs
│   ├── jupiter_earn.rs     # ✅ Wraps protocols::jupiter::earnings
│   ├── jupiter_lend_deposit.rs # ✅ Wraps protocols::jupiter::lend_deposit
│   ├── jupiter_lend_withdraw.rs# ✅ Wraps protocols::jupiter::lend_withdraw
│   ├── jupiter_swap.rs     # ✅ Wraps protocols::jupiter::swap
│   ├── native.rs           # ✅ Native SOL/SPL transfer tools
│   └── flow/               # ✅ Flow orchestration tools
├── agents/                 # ✅ Agent implementations
│   ├── coding/             # ✅ Deterministic/coding agents
│   │   ├── d_001_sol_transfer.rs
│   │   ├── d_002_spl_transfer.rs
│   │   ├── d_100_jup_swap_sol_usdc.rs
│   │   ├── d_110_jup_lend_deposit_sol.rs
│   │   ├── d_111_jup_lend_deposit_usdc.rs
│   │   ├── d_112_jup_lend_withdraw_sol.rs
│   │   ├── d_113_jup_lend_withdraw_usdc.rs
│   │   └── d_114_jup_positions_and_earnings.rs
│   └── flow/               # ✅ Flow orchestration agents
├── config/                 # 🔄 Future: Configuration management
└── lib.rs
```

## ✅ Phase 1 Complete: Jupiter Protocol Refactoring

### What Was Accomplished:

1. **✅ Separated Protocol Logic from Tools:**
   - Moved real Jupiter API logic from `tools/jupiter_swap.rs` to `protocols/jupiter/swap.rs`
   - Created dedicated protocol handlers for lend operations: `protocols/jupiter/lend_deposit.rs` and `protocols/jupiter/lend_withdraw.rs`
   - All protocol handlers now use the custom `jup-sdk` implementation

2. **✅ Refactored Tools to Thin Wrappers:**
   - `tools/jupiter_swap.rs` → thin wrapper around `protocols::jupiter::swap::handle_jupiter_swap`
   - `tools/jupiter_lend_deposit.rs` → thin wrapper around `protocols::jupiter::lend_deposit::handle_jupiter_lend_deposit`
   - `tools/jupiter_lend_withdraw.rs` → thin wrapper around `protocols::jupiter::lend_withdraw::handle_jupiter_lend_withdraw`

3. **✅ Updated All References:**
   - Fixed all coding agents to use new protocol function names
   - Updated flow agent imports
   - Cleaned up module declarations
   - Removed duplicate implementations

4. **✅ Established Working Architecture:**
   - **Protocols Layer**: Centralized Jupiter API integration using jup-sdk
   - **Tools Layer**: AI argument parsing and protocol delegation
   - **Agents Layer**: Direct protocol usage for deterministic flows

### Current Architecture Pattern:

```rust
// Protocol Handler (uses jup-sdk)
protocols/jupiter/swap.rs → handle_jupiter_swap() → jup_sdk::Jupiter::surfpool()

// AI Tool (thin wrapper)
tools/jupiter_swap.rs → JupiterSwapTool::call() → handle_jupiter_swap()

// Coding Agent (direct protocol usage)
agents/coding/d_100_jup_swap_sol_usdc.rs → handle_jupiter_swap()
```

## 🔄 Remaining Implementation Plan

### Phase 2: Native Protocol Implementation
**Status**: ✅ **COMPLETED**
- ✅ Moved native SOL/SPL transfer logic from `tools/native.rs` to `protocols/native/`
- ✅ Created `protocols/native/sol_transfer.rs` and `protocols/native/spl_transfer.rs`
- ✅ Refactored `tools/native.rs` to use protocol handlers
- ✅ Updated coding agents to use protocol handlers directly
- ✅ Fixed all module declarations and imports

### Phase 3: Jupiter Configuration Enhancement
**Status**: ✅ **COMPLETED**
- ✅ Enhanced existing `protocols/jupiter/mod.rs` configuration with more options
- ✅ Added environment variable support with dotenvy
- ✅ Integrated config with jup_sdk initialization
- ✅ Added configuration validation and debug logging
- ✅ Enhanced tools to use configuration defaults
- ✅ Added global configuration initialization on server startup

### Phase 4: Protocol Abstraction Layer
**Status**: ✅ **COMPLETED**
-- ✅ Created common protocol traits for consistent interfaces (Protocol, SwapProtocol, LendProtocol, TransferProtocol)
-- ✅ Standardized error handling across all protocols (ProtocolError enum with comprehensive error types)
-- ✅ Added protocol health checks and metrics (HealthChecker, MetricsCollector with comprehensive monitoring)
-- ✅ Created Jupiter protocol implementation using traits (JupiterProtocol with full trait implementations)
-- ✅ Established protocol abstraction foundation for future protocols
-- ✅ Added comprehensive metrics collection (request counts, response times, error tracking, volume monitoring)
-- ✅ Implemented health monitoring system (degraded/unhealthy states, auto-recovery, multi-protocol monitoring)

### Phase 5: Feature Flags Implementation
**Status**: 🔄 Not Started
```toml
# Cargo.toml
[features]
default = ["jupiter", "native"]
jupiter = []          # Jupiter protocol support
drift = []            # Future: Drift protocol support  
kamino = []           # Future: Kamino protocol support
native = []           # Native Solana operations
all-protocols = ["jupiter", "drift", "kamino", "native"]
```

```rust
// protocols/mod.rs
#[cfg(feature = "jupiter")]
pub mod jupiter;
#[cfg(feature = "native")]
pub mod native;
#[cfg(feature = "drift")]  
pub mod drift;
#[cfg(feature = "kamino")]
pub mod kamino;
```

### Phase 6: Future Protocol Support
**Status**: 🔄 Not Started
- Add Drift protocol structure
- Add Kamino protocol structure
- Follow established pattern from Jupiter + Native implementations

## 🔧 Implementation Details (Current State)

### 1. Protocol Handlers Layer ✅

**Purpose**: Real API integration using jup-sdk and Solana instructions
**Returns**: `Vec<RawInstruction>` for instruction-based operations
**Error Handling**: `anyhow::Result<T>` propagated to tools

```rust
// protocols/jupiter/swap.rs (IMPLEMENTED)
pub async fn handle_jupiter_swap(
    user_pubkey: Pubkey,
    input_mint: Pubkey,
    output_mint: Pubkey,
    amount: u64,
    slippage_bps: u16,
    _key_map: &HashMap<String, String>,
) -> Result<Vec<RawInstruction>> {
    let config = super::get_jupiter_config();
    config.log_config();
    
    // Validate slippage against configuration limits
    let validated_slippage = config.validate_slippage(slippage_bps)?;
    
    let jupiter_client = Jupiter::surfpool().with_user_pubkey(user_pubkey);
    // ... jup_sdk integration with configuration
}

// protocols/native/sol_transfer.rs (IMPLEMENTED)
pub async fn handle_sol_transfer(
    from_pubkey: Pubkey,
    to_pubkey: Pubkey,
    lamports: u64,
    _key_map: &HashMap<String, String>,
) -> Result<Vec<RawInstruction>> {
    let instruction = solana_system_interface::instruction::transfer(&from_pubkey, &to_pubkey, lamports);
    // Convert to RawInstruction format
}
```

### 4. Protocol Abstraction Layer ✅

**Purpose**: Common traits and utilities for consistent protocol interfaces
**Returns**: Standardized protocol interfaces with health monitoring and metrics

```rust
// protocols/common/traits.rs (IMPLEMENTED)
#[async_trait]
pub trait Protocol: Send + Sync {
    fn name(&self) -> &'static str;
    async fn health_check(&self) -> Result<HealthStatus, ProtocolError>;
    fn metrics(&self) -> &ProtocolMetrics;
    fn supported_operations(&self) -> Vec<ProtocolOperation>;
}

#[async_trait]
pub trait SwapProtocol: Protocol {
    async fn swap(&self, user_pubkey: &str, input_mint: &str, output_mint: &str, amount: u64, slippage_bps: u16) -> Result<Vec<RawInstruction>, ProtocolError>;
    async fn get_quote(&self, input_mint: &str, output_mint: &str, amount: u64) -> Result<SwapQuote, ProtocolError>;
}

// protocols/jupiter/protocol.rs (IMPLEMENTED)
#[async_trait]
impl Protocol for JupiterProtocol {
    fn name(&self) -> &'static str { "jupiter" }
    async fn health_check(&self) -> Result<HealthStatus, ProtocolError> { /* health check implementation */ }
    fn metrics(&self) -> &ProtocolMetrics { /* metrics access */ }
}

#[async_trait]
impl SwapProtocol for JupiterProtocol {
    async fn swap(&self, user_pubkey: &str, input_mint: &str, output_mint: &str, amount: u64, slippage_bps: u16) -> Result<Vec<RawInstruction>, ProtocolError> {
        // Jupiter swap implementation with metrics and error handling
    }
}
```

### 2. AI Tools Layer ✅

**Purpose**: Thin wrappers for AI agent usage
**Responsibility**: Argument parsing, validation, protocol delegation

```rust
// tools/jupiter_swap.rs (IMPLEMENTED)
impl Tool for JupiterSwapTool {
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Validate arguments
        let user_pubkey = Pubkey::from_str(&args.user_pubkey)?;
        
        // Call protocol handler
        let raw_instructions = handle_jupiter_swap(
            user_pubkey, input_mint, output_mint, amount, slippage_bps, &self.key_map
        ).await?;
        
        // Serialize response
        Ok(serde_json::to_string(&raw_instructions)?)
    }
}

// tools/jupiter_swap.rs (ENHANCED)
impl Tool for JupiterSwapTool {
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Use default slippage from configuration if not provided
        let config = get_jupiter_config();
        let slippage_bps = match args.slippage_bps {
            Some(slippage) => config.validate_slippage(slippage)?,
            None => config.default_slippage(),
        };
        
        // Call protocol handler with validated slippage
        let instructions = handle_jupiter_swap(
            user_pubkey, input_mint, output_mint, args.amount, slippage_bps, &self.key_map
        ).await?;
        
        Ok(serde_json::to_string(&instructions)?)
    }
}
```

### 3. Coding Agents Layer ✅

**Purpose**: Deterministic agents using protocols directly
**Responsibility**: Direct protocol handler usage

```rust
// agents/coding/d_100_jup_swap_sol_usdc.rs (IMPLEMENTED)
pub async fn handle_jup_swap_sol_usdc(
    key_map: &HashMap<String, String>,
) -> Result<Vec<RawInstruction>> {
    let instructions = handle_jupiter_swap(user_pubkey, sol_mint, usdc_mint, amount, slippage, key_map).await?;
    Ok(instructions)
}

// agents/coding/d_001_sol_transfer.rs (IMPLEMENTED)
pub async fn handle_sol_transfer(
    key_map: &HashMap<String, String>,
) -> Result<Vec<RawInstruction>> {
    let instructions = protocol_handle_sol_transfer(from, to, lamports, key_map).await?;
    Ok(instructions)
}
```

## 🧪 Testing Strategy

### Completed ✅:
- **Compilation Tests**: All refactored code compiles successfully
- **Import Tests**: All module imports resolve correctly
- **Integration Tests**: Tools → Protocols → jup-sdk flow works
- **Native Protocol Tests**: SOL/SPL transfer protocols working correctly
- **Agent Integration**: Coding agents using protocols directly

### Remaining 🔄:
- **Unit Tests**: Individual protocol handler testing
- **E2E Tests**: Complete transaction flow testing
- **Feature Flag Tests**: Compile with different feature combinations

## 🎯 Success Criteria

### Completed ✅:
1. ✅ All existing functionality preserved
2. ✅ Clear separation of concerns achieved
3. ✅ Protocol logic centralized
4. ✅ Tools act as thin wrappers
5. ✅ Coding agents use protocols directly
6. ✅ Module structure is clean and extensible
7. ✅ Native protocol moved to protocols layer
8. ✅ Two complete protocol examples (Jupiter + Native)

### Remaining 🔄:
1. 🔄 Feature flags implemented
3. ✅ Configuration enhanced with environment variables
4. 🔄 Future protocols (Drift, Kamino) structure ready
5. 🔄 All tests passing with comprehensive coverage

## 🚀 Benefits Achieved

### ✅ Current Benefits:
1. **Modularity**: Clear separation between protocols, tools, and agents
2. **Maintainability**: Jupiter logic centralized in protocols layer
3. **Reusability**: Same protocol handlers used by both tools and agents
4. **Testability**: Each layer can be tested independently
5. **Consistency**: Established pattern for future protocol additions
6. **Performance**: Optimized through protocol centralization
7. **Standardized Interfaces**: Common traits ensure consistent protocol behavior
8. **Comprehensive Monitoring**: Health checks and metrics for all protocols
9. **Error Handling**: Standardized error types across all protocol operations
10. **Extensibility**: Trait-based architecture makes adding new protocols straightforward

### 🔄 Future Benefits:
1. **Extensibility**: Easy protocol addition following established pattern
2. **Flexibility**: Feature flag configuration for compile-time selection
3. **Scalability**: Architecture supports many protocols without bloat
4. **Protocol Composition**: Multiple protocols can be combined in complex operations
5. **Runtime Monitoring**: Real-time health and performance metrics for all protocols

## 🔧 Environment Configuration

### Jupiter Configuration Options:
```bash
# .env file
JUPITER_API_BASE_URL=https://lite-api.jup.ag
JUPITER_TIMEOUT_SECONDS=30
JUPITER_MAX_RETRIES=3
JUPITER_USER_AGENT=reev-agent/0.1.0
JUPITER_DEFAULT_SLIPPAGE_BPS=50      # 0.5%
JUPITER_MAX_SLIPPAGE_BPS=1000        # 10%
JUPITER_DEBUG=false
JUPITER_SURFPOOL_RPC_URL=           # Optional custom RPC URL
```

### Native Configuration Options:
```bash
# .env file
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
SOLANA_WS_URL=wss://api.mainnet-beta.solana.com
SOLANA_TIMEOUT_SECONDS=30
SOLANA_MAX_RETRIES=3
SOLANA_CONFIRMATIONS=1
SOLANA_COMPUTE_UNITS=200000
SOLANA_PRIORITY_FEE_LAMPORTS=10000
SOLANA_USER_AGENT=reev-agent/0.1.0
```

### Configuration Features:
- **Environment Variable Support**: All settings can be overridden via environment variables
- **Validation**: Configuration values are validated on startup
- **Default Values**: Sensible defaults provided for all settings
- **Debug Logging**: Optional debug logging for troubleshooting
- **Global State**: Configuration initialized once and shared across the application

## 📊 Progress Summary

- **Phase 1 (Jupiter Refactoring)**: ✅ **COMPLETED**
- **Phase 2 (Native Protocol)**: ✅ **COMPLETED**  
- **Phase 3 (Configuration)**: ✅ **COMPLETED**
- **Phase 4 (Abstraction)**: ✅ **COMPLETED**

**Overall Progress**: 57% Complete (4 of 7 phases)

The foundation is now solid for the complete modular architecture. Both Jupiter and Native protocols serve as templates for all future protocol implementations, demonstrating the complete pattern from protocol handlers → AI tools → coding agents. The configuration system provides robust environment-based customization with validation and debugging capabilities. The protocol abstraction layer establishes consistent interfaces, standardized error handling, and comprehensive health monitoring for all protocols.

## 🔄 Phase 5: Response Parsing Unification [IN PROGRESS]

### 🎯 Problem Statement
**Current Issues:**
- **Fragile parsing**: `extract_execution_results` breaks when LLM response formats change
- **Complex nested conditions**: Multiple format-specific code paths are hard to maintain
- **MaxDepthError**: LLM makes too many tool calls due to inefficient response handling
- **Token format confusion**: Different Jupiter protocol token representations cause failures
- **Edge cases remain**: Malformed or partially structured responses still fail

**Root Cause:**
The current approach tries to predict LLM response formats instead of extracting whatever format is provided. This creates brittleness when models change behavior.

### 🚀 Solution: Unified Response Parsing Architecture

#### **Phase 5.1: Response Normalizer Core**
```rust
// crates/reev-agent/src/response_normalizer.rs
pub struct ResponseNormalizer;

impl ResponseNormalizer {
    /// 🎯 Normalize ANY LLM response format to standard TransactionResult
    /// Handles: Mixed markdown+JSON, partial structures, natural language
    /// Never fails - gracefully degrades to extract what's available
    pub fn normalize_llm_response(response_str: &str) -> Result<TransactionResult>
}
```

#### **Phase 5.2: Defensive Parsing Strategy**
```rust
/// 🛡️ Three-tier extraction: Perfect → Pattern → Heuristic fallback
impl ResponseNormalizer {
    fn normalize_llm_response(response_str: &str) -> Result<TransactionResult> {
        // 🥇 Attempt 1: Perfect structured JSON parse
        if let Ok(result) = Self::try_perfect_parse(response_str) {
            return Ok(result);
        }
        
        // 🥈 Attempt 2: Pattern-based extraction (works on malformed responses)
        if let Ok(instructions) = Self::extract_instruction_patterns(response_str) {
            return Ok(Self::create_from_patterns(instructions));
        }
        
        // 🥉 Attempt 3: Heuristic fallback (never fails)
        let result = Self::heuristic_extraction(response_str);
        return Ok(result);
    }
}
```

#### **Phase 5.3: Pattern-Based Instruction Extraction**
```rust
/// 🔍 Extract instruction patterns using regex (works on ANY malformed response)
impl ResponseNormalizer {
    fn extract_instruction_patterns(response_str: &str) -> Result<Vec<InstructionCandidate>> {
        let patterns = vec![
            // Jupiter format: {"program_id": "...", "accounts": [...], "data": "..."}
            r#""program_id"\s*:\s*"[^"]*"[^}]*"accounts"\s*:\s*\[[^\]]*\][^}]*"data"\s*:\s*"[^"]*""#,
            
            // Direct format: {program_id: "...", accounts: [...], data: "..."}
            r#""program_id"\s*:\s*"[^"]*"[^,]*"accounts"\s*:\s*\[[^\]]*\][^,]*"data"\s*:\s*"[^"]*""#,
            
            // Account arrays: [{"pubkey": "...", "is_signer": ..., "is_writable": ...}]
            r#""pubkey"\s*:\s*"[^"]*"[^}]*"is_signer"\s*:\s*(true|false)[^}]*"is_writable"\s*:\s*(true|false)"#,
        ];
        
        // Find all matches and normalize to standard InstructionCandidate format
        Self::normalize_pattern_matches(response_str, &patterns)
    }
}
```

#### **Phase 5.4: Integration**
```rust
// Replace complex extract_execution_results logic:
// crates/reev-agent/src/enhanced/openai.rs
let execution_result = ResponseNormalizer::normalize_llm_response(&response_str)?;
```

### 🎯 Phase 5.5: Implementation Timeline
+- **Day 1**: Core ResponseNormalizer implementation
+- **Day 2**: Pattern extraction and normalization logic
+- **Day 3**: Integration with OpenAI agent, replace extract_execution_results
+- **Day 4**: Comprehensive testing against all failure cases
+- **Day 5**: Documentation and validation

### 🎯 Expected Benefits
- **✅ Never fails**: Handles any LLM response format gracefully
- **✅ No MaxDepthError**: Single extraction eliminates tool call loops
- **✅ Token confusion resolved**: Extracts actual instructions regardless of format
- **✅ Future-proof**: Adapts to new LLM response formats automatically
- **✅ Simplified maintenance**: Single parsing function, no nested conditions

### 🎯 Target Benchmarks to Fix
- ✅ **113-jup-lend-withdraw-usdc.yml**: Token format confusion → 100% score
- ✅ **115-jup-lend-mint-usdc.yml**: MaxDepthError → 100% score  
- ✅ **114-jup-positions-and-earnings.yml**: Unknown status → Working
- ✅ **116-jup-lend-redeem-usdc.yml**: Unknown status → Working

---

## 📊 Phase 6: OpenTelemetry Integration & Observability [NEW]

### 🎯 **Objective**
Implement comprehensive OpenTelemetry observability for type-safe response architecture, providing real-time insights into agent behavior, API compliance, and performance metrics.

### 🏗️ **Core OTEL Integration Architecture**

#### **Component 1: Type-Aware Tracing**
```rust
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::trace::SdkTracerProvider;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

// 🎯 Type-safe instrumented agent execution
#[tracing::instrument(
    name = "agent_execution",
    fields(
        response_type = std::any::type_name::<T>(),
        operation = T::operation_type(),
        instruction_count = tracing::field::Empty,
        api_source = tracing::field::Empty,
    )
)]
pub async fn execute_typed_request<T: AgentResponse>(request: T::Request) -> Result<T, AgentError> {
    let start = std::time::Instant::now();
    
    // 🎯 OpenTelemetry tracks exact types
    tracing::info!(
        agent_type = std::any::type_name::<T>(),
        request_id = uuid::Uuid::new_v4().to_string(),
        operation = T::operation_type(),
        user_pubkey = request.user_pubkey(),
    );
    
    // Execute with automatic tracing
    let response = typed_agent.call_typed(request).await?;
    
    // 🎯 Record metrics
    let execution_time = start.elapsed();
    tracing::info!(
        execution_time_ms = execution_time.as_millis(),
        instruction_count = response.to_execution_result().transactions.len(),
        validation_result = response.validate_instructions().is_ok(),
        api_source = detect_api_source(&response),
    );
    
    Ok(response)
}
```

#### **Component 2: Structured Metrics Collection**
```rust
use opentelemetry::metrics::{Counter, Histogram, Gauge};

// 🎯 Type-aware metrics collector
pub struct TypeMetricsCollector {
    request_counter: Counter<u64>,
    execution_histogram: Histogram<f64>,
    validation_gauge: Gauge<u64>,
    api_source_counter: Counter<u64>,
}

impl TypeMetricsCollector {
    pub fn new() -> Self {
        let meter = opentelemetry::global::meter("reev_agent_metrics");
        
        Self {
            request_counter: meter.u64_counter("agent_requests_total")
                .with_description("Total number of agent requests"),
            execution_histogram: meter.f64_histogram("agent_execution_time")
                .with_description("Agent execution time in milliseconds"),
            validation_gauge: meter.u64_gauge("agent_validation_status")
                .with_description("Agent response validation status (1=valid, 0=invalid)"),
            api_source_counter: meter.u64_counter("api_source_counts")
                .with_description("Counts of API vs LLM generated responses"),
        }
    }
    
    pub fn record_request<T: AgentResponse>(&self, response: &T) {
        self.request_counter.add(
            1,
            [
                KeyValue::new("response_type", T::operation_type()),
                KeyValue::new("operation_id", uuid::Uuid::new_v4().to_string()),
            ],
        );
        
        self.execution_histogram.record(
            response.execution_time_ms() as f64,
            [
                KeyValue::new("response_type", T::operation_type()),
                KeyValue::new("instruction_count", response.instruction_count() as u64),
            ],
        );
        
        self.validation_gauge.set(
            if response.validate_instructions().is_ok() { 1 } else { 0 },
            [
                KeyValue::new("response_type", T::operation_type()),
            ],
        );
        
        self.api_source_counter.add(
            1,
            [
                KeyValue::new("api_source", response.detect_api_source()),
                KeyValue::new("response_type", T::operation_type()),
            ],
        );
    }
}
```

#### **Component 3: Custom Span Attributes**
```rust
use opentelemetry::trace::{Span, SpanKind, Status};

// 🎯 Rich span attributes for compliance tracking
impl<T: AgentResponse> AgentResponse for T {
    fn create_span(&self, operation: &str) -> Span {
        let span = tracing::span!(Level::INFO, operation, kind = SpanKind::Client);
        
        span.set_attribute("response_type", T::operation_type());
        span.set_attribute("instruction_count", self.instruction_count() as u64);
        span.set_attribute("api_compliant", self.validate_instructions().is_ok());
        span.set_attribute("execution_time_ms", self.execution_time_ms());
        span.set_attribute("api_source", self.detect_api_source());
        
        // Add protocol-specific attributes
        if let Some(jupiter_data) = self.jupiter_metadata() {
            span.set_attribute("jupiter_operation", jupiter_data.operation_type);
            span.set_attribute("jupiter_tokens", jupiter_data.token_mints);
        }
        
        span
    }
}
```

#### **Component 4: Distributed Tracing**
```rust
// 🎯 Distributed tracing for multi-step flows
#[tracing::instrument(
    name = "jupiter_swap_flow",
    skip_if = true
)]
pub async fn execute_jupiter_swap_flow<T: AgentResponse>(
    agent: &TypedAgent<T>,
    request: JupiterSwapRequest,
) -> Result<T, AgentError> {
    // Step 1: Get quote
    let quote_span = tracing::info_span!("jupiter_get_quote").entered();
    let quote = agent.get_quote(&request).instrument(quote_span).await?;
    quote_span.exit();
    
    // Step 2: Get instructions
    let instructions_span = tracing::info_span!("jupiter_get_instructions").entered();
    let instructions = agent.get_instructions(&quote).instrument(instructions_span).await?;
    instructions_span.exit();
    
    // Step 3: Execute transaction
    let execution_span = tracing::info_span!("jupiter_execute_transaction").entered();
    let response = agent.execute_transaction(&instructions).instrument(execution_span).await?;
    execution_span.exit();
    
    // Step 4: Validate result
    let validation_span = tracing::info_span!("jupiter_validate_response").entered();
    response.validate_instructions().instrument(validation_span).await?;
    validation_span.exit();
    
    Ok(response)
}
```

### 📊 **Implementation Timeline**

#### **Phase 6.1: OTEL Infrastructure** (2 days)
- Set up OpenTelemetry provider and exporter
- Create type-aware metrics collector
- Implement custom span attributes for compliance tracking

#### **Phase 6.2: Agent Integration** (2 days)
- Add tracing instrumentation to TypedAgent<T>
- Implement type-specific span creation
- Integrate metrics collection into agent execution

#### **Phase 6.3: Distributed Tracing** (2 days)
- Implement span propagation for multi-step flows
- Add parent-child span relationships
- Create correlation IDs for request tracking

#### **Phase 6.4: Dashboard Integration** (1 day)
- Set up Jaeger/Tempo for trace visualization
- Create custom Grafana dashboards for agent metrics
- Implement alerting for compliance violations

### 🎯 **Key Metrics to Track**

#### **Performance Metrics:**
- **Request Rate**: Total agent requests per operation type
- **Execution Time**: Time taken for each operation type
- **Success Rate**: Percentage of successful executions

#### **Compliance Metrics:**
- **API Source Distribution**: API vs LLM generated responses
- **Validation Rate**: Percentage of API-compliant responses
- **Type Validation Success**: Pass/fail rate for each response type

#### **Behavioral Metrics:**
- **Tool Usage**: Frequency and patterns of tool calls
- **Multi-step Success**: Rate of complex workflow completion
- **Error Patterns**: Types and frequency of errors

### 🎯 **Expected Benefits**

#### **Immediate Improvements:**
- **Real-time Visibility**: See exactly what agents are doing in production
- **Performance Insights**: Identify bottlenecks and optimization opportunities
- **Compliance Monitoring**: Ensure agents follow API-first principles

#### **Long-term Advantages:**
- **Data-Driven Optimization**: Use metrics to improve agent behavior
- **Automated Alerting**: Get notified of compliance violations
- **Historical Analysis**: Track agent performance over time
- **Cross-Model Comparison**: Compare different model performance

### 🎯 Success Criteria
- **100% Coverage**: All agent operations are instrumented
- **Real-time Dashboard**: Live monitoring of agent behavior
- **Compliance Enforcement**: Automatic alerts for API violations
- **Performance Optimization**: Metrics-driven agent improvements

### 🎯 Overall Progress**: 57% Complete (4 of 6 phases)