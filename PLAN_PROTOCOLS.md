# Reev Protocol Architecture Plan - Two-Stage Approach

## 🎯 Why: Phased Protocol Abstraction for Immediate and Future Needs

This document outlines a two-stage approach to implementing protocol abstraction in Reev, prioritizing immediate needs while keeping the architecture flexible for future protocol additions.

### Current State
- Jupiter operations (swap, lend, earn) implemented with separate handlers
- Marinade operations (stake) and wallet operations (transfer) in development
- Need for benchmarking and testing infrastructure per TASKS.md
- Project constraints: files under 320-512 lines, modular design, incremental implementation

### Future Needs
- Multiple protocol support (Jupiter, Marinade, custom protocols)
- Enhanced validation and error recovery
- Performance optimization and gas estimation
- Protocol-specific configuration and parameters

## 🏗️ Two-Stage Architecture Overview

### Stage 1: Minimal Protocol Interface (Immediate Priority)
**Focus**: Consolidate current operations with a simple, extensible interface

**Timeline**: Weeks 1-2
**Goals**:
1. Create a lightweight protocol abstraction
2. Consolidate existing Jupiter operations without breaking functionality
3. Prepare the foundation for future protocol additions
4. Support immediate benchmarking and testing needs

### Stage 2: Full Protocol Abstraction (Future Priority)
**Focus**: Enhanced features for production-scale multi-protocol support

**Timeline**: Future (triggered by specific needs)
**Goals**:
1. Add advanced protocol features (validation, gas estimation)
2. Implement comprehensive error handling and recovery
3. Optimize performance for high-throughput scenarios
4. Enhance configuration and protocol management

## 📋 Stage 1: Minimal Protocol Interface (Priority: High)

### Core Protocol Executor
**File**: `crates/reev-core/src/protocols/executor.rs`
**Description**: Simple protocol interface for immediate needs

```rust
/// Minimal protocol interface for Stage 1
pub trait ProtocolExecutor {
    type Error;
    type Result;
    
    /// Execute a protocol operation
    async fn execute(&self, operation: &ProtocolOperation) -> Result<Self::Result, Self::Error>;
}

/// Protocol operation definition
pub struct ProtocolOperation {
    pub operation_type: OperationType,
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Protocol operation types
pub enum OperationType {
    Swap,
    Lend,
    Earn,
    Stake,
    Transfer,
    Custom(String),
}
```

### Jupiter Protocol Implementation
**File**: `crates/reev-core/src/protocols/jupiter/mod.rs`
**Description**: Jupiter protocol implementation that wraps existing handlers

```rust
/// Jupiter protocol implementation
pub struct JupiterProtocol {
    swap_handler: Box<dyn SwapHandler>,
    lend_handler: Box<dyn LendHandler>,
    earn_handler: Box<dyn EarnHandler>,
}

impl JupiterProtocol {
    pub fn new(
        swap_handler: Box<dyn SwapHandler>,
        lend_handler: Box<dyn LendHandler>,
        earn_handler: Box<dyn EarnHandler>,
    ) -> Self {
        Self {
            swap_handler,
            lend_handler,
            earn_handler,
        }
    }
}

impl ProtocolExecutor for JupiterProtocol {
    type Error = JupiterError;
    type Result = JupiterResult;
    
    async fn execute(&self, operation: &ProtocolOperation) -> Result<Self::Result, Self::Error> {
        match operation.operation_type {
            OperationType::Swap => {
                // Convert parameters and delegate to existing swap handler
                let params = SwapParameters::from(operation.parameters.clone());
                let result = self.swap_handler.execute(params).await?;
                Ok(JupiterResult::Swap(result))
            }
            OperationType::Lend => {
                // Convert parameters and delegate to existing lend handler
                let params = LendParameters::from(operation.parameters.clone());
                let result = self.lend_handler.execute(params).await?;
                Ok(JupiterResult::Lend(result))
            }
            OperationType::Earn => {
                // Convert parameters and delegate to existing earn handler
                let params = EarnParameters::from(operation.parameters.clone());
                let result = self.earn_handler.execute(params).await?;
                Ok(JupiterResult::Earn(result))
            }
            _ => Err(JupiterError::UnsupportedOperation(operation.operation_type.clone())),
        }
    }
}
```

### Protocol Registry
**File**: `crates/reev-core/src/protocols/registry.rs`
**Description**: Simple protocol registry for future extensibility

```rust
/// Simple protocol registry for Stage 1
pub struct ProtocolRegistry {
    protocols: HashMap<String, Box<dyn ProtocolExecutor<Error = ProtocolError, Result = ProtocolResult>>>,
}

impl ProtocolRegistry {
    pub fn new() -> Self {
        Self {
            protocols: HashMap::new(),
        }
    }
    
    pub fn register<E, R>(&mut self, name: &str, protocol: E) 
    where
        E: ProtocolExecutor<Error = E::Error, Result = E::Result> + 'static,
        E::Error: Into<ProtocolError>,
        E::Result: Into<ProtocolResult>,
    {
        self.protocols.insert(name.to_string(), Box::new(protocol));
    }
    
    pub fn get(&self, name: &str) -> Option<&dyn ProtocolExecutor<Error = ProtocolError, Result = ProtocolResult>> {
        self.protocols.get(name).map(|p| p.as_ref())
    }
}
```

## 📋 Stage 2: Full Protocol Abstraction (Priority: Future)

### Enhanced Protocol Executor
**File**: `crates/reev-core/src/protocols/executor.rs` (extension)
**Description**: Extended protocol interface with advanced features

```rust
/// Enhanced protocol interface for Stage 2
pub trait ProtocolExecutor {
    type Error;
    type Result;
    
    /// Execute a protocol operation (from Stage 1)
    async fn execute(&self, operation: &ProtocolOperation) -> Result<Self::Result, Self::Error>;
    
    /// Validate operation parameters
    fn validate_parameters(&self, operation: &ProtocolOperation) -> Result<(), ValidationError>;
    
    /// Estimate gas cost for operation
    fn estimate_gas_cost(&self, operation: &ProtocolOperation) -> Result<u64, GasEstimationError>;
    
    /// Get protocol metadata
    fn metadata(&self) -> ProtocolMetadata;
}

/// Protocol metadata
pub struct ProtocolMetadata {
    pub name: String,
    pub version: String,
    pub supported_operations: Vec<OperationType>,
    pub configuration_schema: serde_json::Value,
}
```

### Protocol Validator
**File**: `crates/reev-core/src/protocols/validator.rs`
**Description**: Protocol validation framework

```rust
/// Protocol validator trait
pub trait ProtocolValidator {
    /// Validate operation against protocol rules
    fn validate_operation(&self, operation: &ProtocolOperation) -> Result<(), ValidationError>;
    
    /// Validate context against protocol requirements
    fn validate_context(&self, context: &WalletContext) -> Result<(), ValidationError>;
}

/// Default protocol validator implementation
pub struct DefaultProtocolValidator;

impl ProtocolValidator for DefaultProtocolValidator {
    fn validate_operation(&self, operation: &ProtocolOperation) -> Result<(), ValidationError> {
        // Basic validation logic
        match operation.operation_type {
            OperationType::Swap => {
                // Validate swap parameters
                if !operation.parameters.contains_key("input_mint") {
                    return Err(ValidationError::MissingParameter("input_mint".to_string()));
                }
                if !operation.parameters.contains_key("output_mint") {
                    return Err(ValidationError::MissingParameter("output_mint".to_string()));
                }
                if !operation.parameters.contains_key("amount") {
                    return Err(ValidationError::MissingParameter("amount".to_string()));
                }
            }
            OperationType::Lend => {
                // Validate lend parameters
                if !operation.parameters.contains_key("mint") {
                    return Err(ValidationError::MissingParameter("mint".to_string()));
                }
                if !operation.parameters.contains_key("amount") {
                    return Err(ValidationError::MissingParameter("amount".to_string()));
                }
            }
            // ... other operation types
        }
        Ok(())
    }
    
    fn validate_context(&self, context: &WalletContext) -> Result<(), ValidationError> {
        // Basic context validation
        if context.pubkey == Pubkey::default() {
            return Err(ValidationError::InvalidContext("Invalid wallet pubkey".to_string()));
        }
        Ok(())
    }
}
```

### Protocol Result Mapper
**File**: `crates/reev-core/src/protocols/result_mapper.rs`
**Description**: Protocol result mapping framework

```rust
/// Protocol result mapper trait
pub trait ProtocolResultMapper {
    /// Map protocol result to standard format
    fn map_result(&self, result: &ProtocolResult) -> Result<StandardResult, MappingError>;
    
    /// Extract transaction signature from result
    fn extract_signature(&self, result: &ProtocolResult) -> Option<String>;
}

/// Default protocol result mapper
pub struct DefaultProtocolResultMapper;

impl ProtocolResultMapper for DefaultProtocolResultMapper {
    fn map_result(&self, result: &ProtocolResult) -> Result<StandardResult, MappingError> {
        match result {
            ProtocolResult::Jupiter(jupiter_result) => {
                match jupiter_result {
                    JupiterResult::Swap(swap_result) => {
                        Ok(StandardResult {
                            operation_type: OperationType::Swap,
                            success: swap_result.success,
                            signature: swap_result.signature.clone(),
                            details: serde_json::to_value(swap_result).unwrap_or_default(),
                        })
                    }
                    JupiterResult::Lend(lend_result) => {
                        Ok(StandardResult {
                            operation_type: OperationType::Lend,
                            success: lend_result.success,
                            signature: lend_result.signature.clone(),
                            details: serde_json::to_value(lend_result).unwrap_or_default(),
                        })
                    }
                    // ... other result types
                }
            }
            // ... other protocol results
        }
    }
    
    fn extract_signature(&self, result: &ProtocolResult) -> Option<String> {
        match result {
            ProtocolResult::Jupiter(jupiter_result) => {
                match jupiter_result {
                    JupiterResult::Swap(swap_result) => swap_result.signature.clone(),
                    JupiterResult::Lend(lend_result) => lend_result.signature.clone(),
                    // ... other result types
                }
            }
            // ... other protocol results
        }
    }
}
```

## 🔧 Implementation Strategy

### Stage 1: Immediate Implementation (Weeks 1-2)

#### Week 1: Foundation
1. Create minimal `ProtocolExecutor` trait
2. Implement `ProtocolOperation` and `OperationType`
3. Implement `JupiterProtocol` wrapper
4. Write basic tests for the interface

#### Week 2: Integration
1. Implement simple `ProtocolRegistry`
2. Update benchmark runner to use protocol interface
3. Add error handling for protocol operations
4. Write integration tests

### Stage 2: Future Implementation (Triggered by Specific Needs)

#### Week 1: Enhanced Interface
1. Extend `ProtocolExecutor` with validation methods
2. Implement `ProtocolValidator` trait
3. Implement `ProtocolResultMapper` trait
4. Add protocol metadata support

#### Week 2: Advanced Features
1. Implement parameter validation
2. Add gas estimation
3. Enhance error handling
4. Update tests for new features

## 🔄 Migration Strategy

### From Stage 1 to Stage 2
1. Extend existing protocols with new trait methods
2. Implement validation and result mapping for existing protocols
3. Update protocol registry to support enhanced features
4. Migrate tests to cover new functionality

### Backward Compatibility
- Maintain Stage 1 interface functionality in Stage 2
- Provide adapter pattern for legacy code
- Support gradual migration of individual protocols

## 🧪 Testing Strategy

### Stage 1 Tests
```rust
#[tokio::test]
async fn test_jupiter_protocol_swap() {
    // Test Jupiter protocol swap operation
}

#[tokio::test]
async fn test_protocol_registry() {
    // Test protocol registration and retrieval
}

#[tokio::test]
async fn test_protocol_integration() {
    // Test protocol integration with benchmark runner
}
```

### Stage 2 Tests
```rust
#[tokio::test]
async fn test_protocol_validation() {
    // Test protocol validation
}

#[tokio::test]
async fn test_gas_estimation() {
    // Test gas estimation
}

#[tokio::test]
async fn test_result_mapping() {
    // Test result mapping
}
```

## 📈 Benefits

### Stage 1 Benefits
1. **Immediate Value**: Cleaner code structure without major refactoring
2. **Future-Ready**: Easy to add new protocols when needed
3. **Low Risk**: Minimal changes to existing functionality
4. **Testable**: Each component can be tested independently
5. **Aligned with Tasks**: Supports immediate benchmark needs

### Stage 2 Benefits
1. **Enhanced Validation**: Comprehensive parameter and context validation
2. **Gas Estimation**: Cost estimation for operations
3. **Standardized Results**: Consistent result format across protocols
4. **Performance Optimization**: Protocol-specific optimizations
5. **Advanced Error Handling**: Granular error types and recovery strategies

## 📝 Related Documents

- PLAN_CORE_V3.md: Core architecture for production implementation
- PLAN_CORE_BENCHMARK.md: Benchmark requirements for AI-generated flows
- TASKS.md: Implementation tasks for benchmark infrastructure

---

This document outlines a pragmatic two-stage approach to protocol abstraction, prioritizing immediate needs while maintaining flexibility for future enhancements. The approach aligns with the project's constraints and the tasks outlined in TASKS.md.