# Reev Protocol Architecture Plan

## 🎯 Why: Clear Protocol Separation for Maintainable AI-Generated Flows

Current protocol implementation faces several challenges:
1. **Duplicate implementations** between `crates/reev-protocols` and `crates/reev-core/src/execution/handlers`
2. **Confusing dependencies** with protocol-specific code mixed into core orchestration
3. **Maintenance burden** keeping two separate implementations in sync
4. **Testing complexity** due to unclear boundaries between components
5. **Difficulty extending** to new protocols without clear patterns

This plan establishes a clear, testable approach to consolidate protocol implementations while maintaining the existing `reev-core` structure for E2E compatibility.

## 🏗️ Architecture Overview

### Core Principle: Protocol Abstraction Layer
Create a generic protocol abstraction that allows plugging in different DeFi protocols while keeping core orchestration logic unchanged.

```
┌─────────────────────────────────────────────────────────────┐
│                   reev-core (Orchestration)            │
├─────────────────────────────────────────────────────────────┤
│  • QueryHandler • Planner • Executor • YMLGenerator      │
│  • BenchmarkScorer • Validation Framework                │
└─────────────────────────────────────────────────────────────┘
                          │
                          │ uses
                          ▼
┌─────────────────────────────────────────────────────────────┐
│               Protocol Abstraction Layer               │
├─────────────────────────────────────────────────────────────┤
│  • ProtocolTraits • GenericExecutor • ErrorHandling       │
│  • ParameterValidation • ResultMapping                  │
└─────────────────────────────────────────────────────────────┘
                          │
                          │ implements
                          ▼
┌─────────────────────────────────────────────────────────────┐
│             Protocol Implementations               │
├─────────────────────────────────────────────────────────────┤
│  • Jupiter • Orca • Marinade • Raydium • Custom    │
└─────────────────────────────────────────────────────────────┘
```

## 📋 Testable Refactoring Groups

### Group 1: Protocol Trait Definition (Priority: High)
**Files**:
- `crates/reev-core/src/protocols/mod.rs`
- `crates/reev-core/src/protocols/traits.rs`

**Goal**: Define generic interfaces for all protocol operations.

**Implementation Steps**:
1. Create `ProtocolExecutor` trait for execution
2. Create `ProtocolValidator` trait for parameter validation
3. Create `ProtocolResultMapper` trait for result standardization

**Tests**:
- Unit tests for trait definitions
- Mock implementations for testing
- E2E tests using mock protocols

### Group 2: Jupiter Protocol Implementation (Priority: High)
**Files**:
- `crates/reev-core/src/protocols/jupiter/mod.rs`
- `crates/reev-core/src/protocols/jupiter/swap.rs`
- `crates/reev-core/src/protocols/jupiter/lend.rs`
- `crates/reev-core/src/protocols/jupiter/earn.rs`

**Goal**: Implement Jupiter protocol using new trait structure.

**Implementation Steps**:
1. Create Jupiter struct implementing protocol traits
2. Consolidate duplicate Jupiter implementations
3. Maintain compatibility with existing E2E tests
4. Add comprehensive error handling

**Tests**:
- Unit tests for each operation
- Integration tests with SURFPOOL
- E2E tests preserving current functionality

### Group 3: Protocol Registry (Priority: High)
**Files**:
- `crates/reev-core/src/protocols/registry.rs`
- `crates/reev-core/src/protocols/factory.rs`

**Goal**: Create protocol discovery and instantiation mechanism.

**Implementation Steps**:
1. Implement protocol registry with dynamic loading
2. Create factory pattern for protocol instantiation
3. Add protocol selection based on operation type
4. Support custom protocol registration

**Tests**:
- Registry population tests
- Factory creation tests
- Dynamic loading tests

### Group 4: Generic Protocol Executor (Priority: Medium)
**Files**:
- `crates/reev-core/src/execution/generic_executor.rs`
- `crates/reev-core/src/execution/parameter_validator.rs`

**Goal**: Create execution layer that works with any protocol.

**Implementation Steps**:
1. Implement generic executor using protocol traits
2. Add parameter validation before execution
3. Standardize result handling across protocols
4. Integrate with existing executor framework

**Tests**:
- Generic executor tests with multiple protocols
- Parameter validation tests
- Result mapping tests

### Group 5: Error Handling Framework (Priority: Medium)
**Files**:
- `crates/reev-core/src/protocols/error_handling.rs`
- `crates/reev-core/src/protocols/recovery.rs`

**Goal**: Standardize error handling across protocols.

**Implementation Steps**:
1. Create protocol-agnostic error types
2. Implement recovery strategies
3. Add error mapping between protocols
4. Integrate with existing error handling

**Tests**:
- Error type conversion tests
- Recovery strategy tests
- Error propagation tests

### Group 6: Configuration System (Priority: Low)
**Files**:
- `crates/reev-core/src/protocols/config.rs`
- `crates/reev-core/src/protocols/settings.rs`

**Goal**: Centralized protocol configuration.

**Implementation Steps**:
1. Create configuration structure for protocols
2. Add protocol-specific settings
3. Implement configuration validation
4. Support environment-based configuration

**Tests**:
- Configuration loading tests
- Validation tests
- Environment override tests

## 🔧 Implementation Strategy

### Phase 1: Foundation (Week 1)
1. Implement Group 1: Protocol Trait Definition
   - Define clear interfaces for all protocols
   - Create testable trait boundaries
   - Document usage patterns

2. Verify E2E Tests Pass
   - Ensure all existing tests still work
   - Document any compatibility issues
   - Create migration plan if needed

### Phase 2: Core Protocol (Week 2)
1. Implement Group 2: Jupiter Protocol
   - Consolidate existing Jupiter implementations
   - Implement new trait structure
   - Maintain backward compatibility

2. Implement Group 3: Protocol Registry
   - Create protocol discovery mechanism
   - Add factory pattern for instantiation
   - Support protocol selection

3. Verify E2E Tests Pass
   - Test all Jupiter operations
   - Ensure registry works correctly
   - Validate factory pattern implementation

### Phase 3: Generic Execution (Week 3)
1. Implement Group 4: Generic Protocol Executor
   - Create protocol-agnostic execution layer
   - Add parameter validation
   - Standardize result handling

2. Implement Group 5: Error Handling Framework
   - Create protocol-agnostic error types
   - Implement recovery strategies
   - Integrate with existing error handling

3. Verify E2E Tests Pass
   - Test generic execution with multiple protocols
   - Validate error handling
   - Test recovery mechanisms

### Phase 4: Configuration (Week 4)
1. Implement Group 6: Configuration System
   - Create centralized configuration
   - Add protocol-specific settings
   - Support environment-based configuration

2. Add Protocol Extension Examples
   - Document adding new protocols
   - Create template implementations
   - Provide best practices guide

3. Verify E2E Tests Pass
   - Test configuration system
   - Validate protocol extensions
   - Test with different configurations

## 📊 Protocol Trait Definition

### Core Protocol Traits

```rust
/// Core trait for all protocol executors
#[async_trait]
pub trait ProtocolExecutor: Send + Sync {
    /// Execute a protocol operation with given parameters
    async fn execute(
        &self,
        operation: &ProtocolOperation,
        context: &ExecutionContext,
    ) -> Result<ProtocolExecutionResult>;
    
    /// Validate parameters before execution
    fn validate_parameters(
        &self,
        operation: &ProtocolOperation,
    ) -> Result<ParameterValidationResult>;
    
    /// Estimate gas costs for the operation
    fn estimate_gas_cost(
        &self,
        operation: &ProtocolOperation,
    ) -> Result<u64>;
    
    /// Get protocol metadata
    fn metadata(&self) -> &ProtocolMetadata;
}

/// Core trait for protocol validators
pub trait ProtocolValidator {
    /// Validate operation parameters
    fn validate_operation(
        &self,
        operation: &ProtocolOperation,
    ) -> Result<ValidationResult>;
    
    /// Validate execution context
    fn validate_context(
        &self,
        context: &ExecutionContext,
    ) -> Result<ValidationResult>;
}

/// Core trait for result mapping
pub trait ProtocolResultMapper {
    /// Map protocol-specific result to standard format
    fn map_result(
        &self,
        protocol_result: &ProtocolSpecificResult,
    ) -> Result<StandardizedResult>;
    
    /// Extract transaction signature from result
    fn extract_signature(
        &self,
        protocol_result: &ProtocolSpecificResult,
    ) -> Result<String>;
}
```

### Protocol Operation Definition

```rust
/// Generic protocol operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolOperation {
    /// Operation type (swap, lend, transfer, etc.)
    pub operation_type: OperationType,
    /// Protocol name (jupiter, orca, etc.)
    pub protocol: String,
    /// Input parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// Execution context
    pub context: Option<ExecutionContext>,
}

/// Operation type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OperationType {
    Swap,
    Lend,
    Borrow,
    Stake,
    Transfer,
    Custom(String),
}
```

## 🎯 Jupiter Protocol Implementation Example

### Jupiter Protocol Structure

```rust
/// Jupiter protocol implementation
pub struct JupiterProtocol {
    /// Jupiter client
    client: Jupiter,
    /// Configuration
    config: JupiterConfig,
}

/// Jupiter-specific configuration
#[derive(Debug, Clone)]
pub struct JupiterConfig {
    /// RPC URL for SURFPOOL
    pub surfpool_rpc_url: Option<String>,
    /// Default slippage tolerance in basis points
    pub default_slippage_bps: u16,
    /// Maximum slippage tolerance in basis points
    pub max_slippage_bps: u16,
}

/// Jupiter-specific swap operation
#[derive(Debug, Clone)]
pub struct JupiterSwapOperation {
    /// Input token mint
    pub input_mint: Pubkey,
    /// Output token mint
    pub output_mint: Pubkey,
    /// Amount to swap
    pub amount: u64,
    /// Slippage tolerance in basis points
    pub slippage_bps: u16,
}

/// Jupiter-specific operation result
#[derive(Debug, Clone)]
pub struct JupiterSwapResult {
    /// Transaction signature
    pub signature: String,
    /// Input amount actually swapped
    pub input_amount: u64,
    /// Output amount received
    pub output_amount: u64,
    /// Price impact
    pub price_impact: Option<f64>,
}
```

### Jupiter Implementation

```rust
#[async_trait]
impl ProtocolExecutor for JupiterProtocol {
    async fn execute(
        &self,
        operation: &ProtocolOperation,
        context: &ExecutionContext,
    ) -> Result<ProtocolExecutionResult> {
        match operation.operation_type {
            OperationType::Swap => self.execute_swap(operation, context).await,
            OperationType::Lend => self.execute_lend(operation, context).await,
            _ => Err(anyhow!("Unsupported operation type for Jupiter protocol")),
        }
    }
    
    fn validate_parameters(
        &self,
        operation: &ProtocolOperation,
    ) -> Result<ParameterValidationResult> {
        match operation.operation_type {
            OperationType::Swap => self.validate_swap_params(operation),
            OperationType::Lend => self.validate_lend_params(operation),
            _ => Err(anyhow!("Unsupported operation type for validation")),
        }
    }
    
    fn estimate_gas_cost(
        &self,
        operation: &ProtocolOperation,
    ) -> Result<u64> {
        // Estimate based on operation complexity
        match operation.operation_type {
            OperationType::Swap => Ok(5_000_000), // 0.005 SOL
            OperationType::Lend => Ok(3_000_000), // 0.003 SOL
            _ => Ok(10_000_000), // 0.01 SOL default
        }
    }
    
    fn metadata(&self) -> &ProtocolMetadata {
        &self.metadata
    }
}

impl JupiterProtocol {
    async fn execute_swap(
        &self,
        operation: &ProtocolOperation,
        context: &ExecutionContext,
    ) -> Result<ProtocolExecutionResult> {
        // Extract Jupiter-specific parameters
        let swap_params = self.extract_swap_params(operation)?;
        
        // Execute swap using Jupiter client
        let swap_result = self.client.swap(swap_params).await?;
        
        // Map to standardized result
        Ok(ProtocolExecutionResult {
            success: true,
            transaction_signature: swap_result.signature.clone(),
            protocol_specific: serde_json::to_value(swap_result)?,
            gas_used: self.estimate_gas_cost(operation)?,
            execution_time_ms: None, // Would be filled by caller
        })
    }
}
```

## 🔄 Migration Strategy

### Backward Compatibility

1. **Preserve Existing Handler Interface**
   - Keep existing handlers as adapters during transition
   - Gradually migrate E2E tests to new protocol system
   - Maintain compatibility with existing tool integrations

2. **Adapter Pattern for Legacy Handlers**
```rust
/// Adapter for legacy handlers
pub struct LegacyHandlerAdapter {
    legacy_handler: Box<dyn LegacyHandler>,
}

#[async_trait]
impl ProtocolExecutor for LegacyHandlerAdapter {
    async fn execute(
        &self,
        operation: &ProtocolOperation,
        context: &ExecutionContext,
    ) -> Result<ProtocolExecutionResult> {
        // Convert to legacy format
        let legacy_params = self.convert_to_legacy_params(operation)?;
        
        // Execute using legacy handler
        let legacy_result = self.legacy_handler.execute(legacy_params, context).await?;
        
        // Convert back to standardized result
        Ok(self.convert_from_legacy_result(legacy_result))
    }
}
```

### Test Preservation

1. **E2E Test Compatibility**
   - All existing E2E tests must continue to pass
   - Implement adapter layer if needed during transition
   - Track test compatibility at each refactoring stage

2. **Incremental Migration**
   - Migrate tests one operation type at a time
   - Document any test changes required
   - Maintain clear mapping between old and new implementations

## 🧪 Testing Strategy

### Unit Tests
- Test each protocol implementation in isolation
- Verify trait implementations
- Test error conditions and edge cases

### Integration Tests
- Test protocol registry and factory
- Verify configuration loading
- Test protocol selection and execution

### E2E Tests
- Ensure all existing E2E tests pass
- Add new tests for generic protocol features
- Verify backward compatibility

### Performance Tests
- Measure execution time with new abstraction layer
- Validate minimal overhead from trait abstraction
- Test resource usage with multiple protocols

## 📈 Benefits

1. **Clear Separation of Concerns**
   - Protocol-specific code isolated from orchestration
   - Clear boundaries between components
   - Easier to understand and maintain

2. **Extensibility**
   - Easy to add new protocols
   - Generic execution framework works with all protocols
   - Plugin-like architecture for protocols

3. **Testability**
   - Protocol implementations can be tested in isolation
   - Core orchestration can be tested with mocks
   - Clear test boundaries

4. **Maintainability**
   - Single implementation of each protocol
   - Reduced duplication
   - Centralized error handling

5. **Performance**
   - Protocol-specific optimizations in implementation
   - Generic framework adds minimal overhead
   - Efficient protocol selection

This plan provides a clear, testable path to consolidate protocol implementations while maintaining E2E test compatibility and enabling future extensibility.