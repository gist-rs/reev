# Reev Core Issue #105: RigAgent Enhancement Tasks

## Overview

This document outlines the remaining tasks for Issue #105 (RigAgent Enhancement), which is currently marked as PARTIALLY COMPLETED. The focus is on improving context passing between operations, enhancing prompt engineering for complex scenarios, and adding comprehensive tool execution validation.

---

## Task 1: Improve Context Passing Between Operations (NOT STARTED)

### Current Implementation:
- Basic context passing via YmlContextBuilder in `/crates/reev-core/src/execution/context_builder/mod.rs`
- Issue #121 completed balance change tracking and constraints generation
- However, more complex scenarios still need better wallet state updates

### What Needs Enhancement:

#### 1.1 Enhanced Operation History Tracking
**Location:** `/crates/reev-core/src/execution/context_builder/mod.rs`

**Implementation Details:**
```rust
// Add to YmlContextBuilder
pub struct YmlContextBuilder {
    wallet_context: WalletContext,
    previous_results: Vec<StepResult>,
    operation_history: Vec<OperationHistory>, // New field
    step_constraints: HashMap<String, Constraint>, // New field
}

// New struct to track operation history
pub struct OperationHistory {
    pub operation_type: String,
    pub input_amount: Option<f64>,
    pub input_mint: Option<String>,
    pub output_amount: Option<f64>,
    pub output_mint: Option<String>,
    pub success: bool,
    pub error_message: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
```

**Tasks:**
1. Add `OperationHistory` struct to track each operation's inputs and outputs
2. Implement `with_operation_history()` method in YmlContextBuilder
3. Update context generation to include operation history
4. Add method to calculate available balance after previous operations
5. Implement balance validation for upcoming operations

#### 1.2 Step-Specific Constraints
**Location:** `/crates/reev-core/src/execution/context_builder/mod.rs`

**Implementation Details:**
```rust
// New struct for step-specific constraints
pub struct StepConstraint {
    pub constraint_type: ConstraintType,
    pub value: serde_json::Value,
    pub applicable_steps: Vec<usize>,
}

pub enum ConstraintType {
    MaximumAmount(f64),
    MinimumAmount(f64),
    RequiredMint(String),
    ExcludedMint(String),
    PriceSlippage(f64),
}
```

**Tasks:**
1. Implement `StepConstraint` struct and `ConstraintType` enum
2. Add constraint application logic in context generation
3. Update `YmlOperationContext` to include applicable constraints
4. Implement constraint validation before tool execution

#### 1.3 Dynamic Context Updates
**Location:** `/crates/reev-core/src/execution/rig_agent/mod.rs`

**Implementation Details:**
```rust
// Add to RigAgent
impl RigAgent {
    // New method to update context after each operation
    async fn update_context_after_execution(
        &self,
        context: &mut YmlOperationContext,
        tool_result: &serde_json::Value,
        tool_name: &str,
    ) -> Result<()> {
        // Extract balance changes from tool result
        // Update wallet context with new balances
        // Add operation to history
        // Generate constraints for next steps
    }
}
```

**Tasks:**
1. Implement `update_context_after_execution()` method in RigAgent
2. Add balance change extraction from tool results
3. Implement wallet context update logic
4. Add operation history tracking
5. Generate constraints for next steps based on current state

---

## Task 2: Enhance Prompt Engineering for Complex Scenarios (NOT STARTED)

### Current Implementation:
- Basic prompt engineering in `/crates/reev-core/src/execution/rig_agent/prompting.rs`
- Simple handling of multi-step operations with "then" and "and"
- Limited handling of complex conditional operations

### What Needs Enhancement:

#### 2.1 Complex Operation Detection
**Location:** `/crates/reev-core/src/execution/rig_agent/prompting.rs`

**Implementation Details:**
```rust
// Add to MultiStepHandler trait
trait MultiStepHandler {
    // Existing methods...
    
    // New method to detect complex operations
    fn detect_complex_operations(&self, prompt: &str) -> Result<Vec<Operation>>;
    
    // New method to parse conditional statements
    fn parse_conditional_operations(&self, prompt: &str) -> Result<Vec<ConditionalOperation>>;
    
    // New method to resolve ambiguous operations
    fn resolve_ambiguous_operations(
        &self,
        prompt: &str,
        context: &YmlOperationContext,
    ) -> Result<Vec<ResolvedOperation>>;
}
```

**Tasks:**
1. Implement `detect_complex_operations()` method to identify non-linear operation sequences
2. Add support for conditional operations ("if/then" statements)
3. Implement `resolve_ambiguous_operations()` to handle prompts with multiple interpretations
4. Add context-aware disambiguation based on wallet state

#### 2.2 Context-Aware Prompt Refinement
**Location:** `/crates/reev-core/src/execution/rig_agent/prompting.rs`

**Implementation Details:**
```rust
// New struct for prompt refinement
pub struct PromptRefiner {
    context_awareness: bool,
    ambiguity_resolution: bool,
    conditional_support: bool,
}

impl PromptRefiner {
    // Refine prompt based on current context
    pub fn refine_with_context(
        &self,
        prompt: &str,
        context: &YmlOperationContext,
    ) -> Result<String>;
    
    // Add clarity to ambiguous parts of prompt
    pub fn resolve_ambiguity(
        &self,
        prompt: &str,
        ambiguous_parts: Vec<String>,
        context: &YmlOperationContext,
    ) -> Result<String>;
}
```

**Tasks:**
1. Implement `PromptRefiner` struct with context awareness
2. Add logic to refine prompts based on current wallet state
3. Implement ambiguity resolution for unclear operations
4. Add conditional statement handling in prompts

#### 2.3 Enhanced Multi-Step Processing
**Location:** `/crates/reev-core/src/execution/rig_agent/prompting.rs`

**Implementation Details:**
```rust
// Enhanced operation extraction
fn extract_operations_enhanced(
    &self,
    prompt: &str,
    context: &YmlOperationContext,
) -> Result<Vec<EnhancedOperation>> {
    // Parse complex operation sequences
    // Handle conditional operations
    // Resolve ambiguities based on context
    // Extract operation dependencies
}

// New struct for enhanced operations
pub struct EnhancedOperation {
    pub operation: String,
    pub conditions: Vec<OperationCondition>,
    pub dependencies: Vec<usize>,
    pub priority: u8,
    pub estimated_gas_cost: Option<f64>,
}
```

**Tasks:**
1. Implement enhanced operation extraction with dependency tracking
2. Add conditional operation support
3. Implement operation prioritization based on dependencies
4. Add estimated gas cost calculation for operations

---

## Task 3: Add Tool Execution Validation (NOT STARTED)

### Current Implementation:
- Minimal validation in `/crates/reev-core/src/execution/rig_agent/tool_execution.rs`
- Basic error handling without recovery mechanisms
- No validation against ground truth

### What Needs Enhancement:

#### 3.1 Parameter Validation Framework
**Location:** `/crates/reev-core/src/execution/rig_agent/validation.rs` (new file)

**Implementation Details:**
```rust
// New file for validation framework
pub struct ParameterValidator {
    wallet_context: WalletContext,
    constraints: Vec<Constraint>,
}

impl ParameterValidator {
    // Validate parameters before tool execution
    pub fn validate_parameters(
        &self,
        tool_name: &str,
        parameters: &serde_json::Value,
    ) -> Result<ValidatedParameters>;
    
    // Check if parameters meet constraints
    pub fn check_constraints(
        &self,
        tool_name: &str,
        parameters: &serde_json::Value,
    ) -> Result<ConstraintCheckResult>;
}

// Result of parameter validation
pub struct ValidatedParameters {
    pub original: serde_json::Value,
    pub validated: serde_json::Value,
    pub warnings: Vec<String>,
    pub adjustments: Vec<ParameterAdjustment>,
}
```

**Tasks:**
1. Create `/crates/reev-core/src/execution/rig_agent/validation.rs` file
2. Implement `ParameterValidator` struct with validation logic
3. Add parameter constraint checking
4. Implement parameter adjustment for invalid inputs
5. Add validation result reporting

#### 3.2 Result Validation Framework
**Location:** `/crates/reev-core/src/execution/rig_agent/validation.rs`

**Implementation Details:**
```rust
// Result validation against ground truth
pub struct ResultValidator {
    ground_truth: Option<GroundTruth>,
    tolerance_map: HashMap<String, f64>, // tool_name -> tolerance
}

impl ResultValidator {
    // Validate tool execution result
    pub fn validate_result(
        &self,
        tool_name: &str,
        result: &serde_json::Value,
    ) -> Result<ValidationReport>;
    
    // Check if result matches ground truth within tolerance
    pub fn check_against_ground_truth(
        &self,
        tool_name: &str,
        result: &serde_json::Value,
    ) -> Result<GroundTruthCheck>;
}

// Validation report
pub struct ValidationReport {
    pub tool_name: String,
    pub success: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<String>,
    pub ground_truth_match: Option<bool>,
}
```

**Tasks:**
1. Implement `ResultValidator` struct in validation.rs
2. Add ground truth comparison logic
3. Implement tolerance-based validation
4. Add validation report generation
5. Integrate result validation with tool execution

#### 3.3 Error Recovery Mechanisms
**Location:** `/crates/reev-core/src/execution/rig_agent/error_recovery.rs` (new file)

**Implementation Details:**
```rust
// New file for error recovery
pub struct ErrorRecoveryEngine {
    max_retries: u8,
    backoff_strategy: BackoffStrategy,
    alternative_tools: HashMap<String, Vec<String>>, // tool_name -> alternatives
}

impl ErrorRecoveryEngine {
    // Attempt to recover from execution error
    pub async fn recover_from_error(
        &self,
        error: &ExecutionError,
        tool_name: &str,
        parameters: &serde_json::Value,
        context: &YmlOperationContext,
    ) -> Result<RecoveryAttempt>;
    
    // Implement retry with backoff
    pub async fn retry_with_backoff(
        &self,
        tool_name: &str,
        parameters: &serde_json::Value,
        retry_count: u8,
    ) -> Result<serde_json::Value>;
}

// Recovery attempt result
pub struct RecoveryAttempt {
    pub success: bool,
    pub adjusted_parameters: Option<serde_json::Value>,
    pub alternative_tool: Option<String>,
    pub retry_suggested: bool,
    pub error_message: Option<String>,
}
```

**Tasks:**
1. Create `/crates/reev-core/src/execution/rig_agent/error_recovery.rs` file
2. Implement `ErrorRecoveryEngine` struct following PLAN_CORE_V3
3. Add specific recovery strategies for each error type
4. Implement retry logic with exponential backoff
5. Add alternative tool selection for persistent failures

#### 3.4 Tool Execution Integration
**Location:** `/crates/reev-core/src/execution/rig_agent/mod.rs`

**Implementation Details:**
```rust
// Enhanced execute_step_with_rig_and_history method
pub async fn execute_step_with_rig_and_history(
    &self,
    step: &YmlStep,
    wallet_context: &WalletContext,
    previous_results: &[StepResult],
) -> Result<StepResult> {
    // Existing code...
    
    // New: Validate parameters before execution
    let validator = ParameterValidator::new(wallet_context.clone(), constraints);
    let validated_params = validator.validate_parameters(tool_name, &parameters)?;
    
    // Execute tool with error recovery
    let result = self.execute_with_recovery(
        tool_name,
        &validated_params.validated,
        &context,
    ).await?;
    
    // New: Validate result against ground truth
    let result_validator = ResultValidator::new(step.ground_truth.clone());
    let validation_report = result_validator.validate_result(tool_name, &result)?;
    
    // Update context based on results
    self.update_context_after_execution(&mut context, &result, tool_name).await?;
    
    // Return step result with validation information
    Ok(StepResult {
        // Existing fields...
        validation_report: Some(validation_report),
        adjusted_parameters: Some(validated_params.adjustments),
    })
}
```

**Tasks:**
1. Update `execute_step_with_rig_and_history` method in RigAgent
2. Integrate parameter validation before tool execution
3. Add error recovery to tool execution
4. Integrate result validation after execution
5. Update context based on execution results

---

## Implementation Priority

### Phase 1 (Immediate - Week 1)
1. **Parameter Validation Framework** - Essential for robust execution
2. **Error Recovery Mechanisms** - Critical for production use
3. **Dynamic Context Updates** - Required for multi-step operations

### Phase 2 (Short-term - Week 2)
1. **Result Validation Framework** - Important for correctness
2. **Step-Specific Constraints** - Enhances operation reliability
3. **Enhanced Wallet State Updates** - Improves accuracy

### Phase 3 (Medium-term - Week 3-4)
1. **Complex Operation Detection** - Handles advanced scenarios
2. **Context-Aware Prompt Refinement** - Improves user experience
3. **Enhanced Multi-Step Processing** - Completes multi-step support

## Testing Strategy

### Unit Tests
- Parameter validation with various inputs
- Error recovery with simulated failures
- Context updates with mock operations

### Integration Tests
- End-to-end multi-step operations with validation
- Error recovery in real scenarios
- Complex prompt handling

### Regression Tests
- Ensure existing functionality remains intact
- Performance benchmarks for enhanced validation
- Stress testing with complex operation sequences

## Success Metrics

1. **Error Reduction**: 90% reduction in execution failures
2. **Validation Accuracy**: 95% correct validation of results
3. **Recovery Success**: 80% successful recovery from errors
4. **Complexity Handling**: Support for 3+ step complex operations
5. **Performance Impact**: Less than 10% overhead from validation

## Dependencies

1. **Issue #102**: Error Recovery Engine - For comprehensive error handling
2. **Issue #112**: Comprehensive Error Recovery - For advanced recovery strategies
3. **Issue #121**: YML Context - For enhanced context passing
4. **Issue #124**: RigAgent Tool Selection - For improved tool selection
5. **PLAN_CORE_V3**: For architectural alignment

## Deliverables

1. Enhanced RigAgent with comprehensive validation
2. Error recovery framework implementation
3. Improved context passing between operations
4. Advanced prompt engineering for complex scenarios
5. Comprehensive test suite for all new functionality
6. Documentation and usage examples
7. Performance benchmarks and optimization
8. Integration with existing architecture components

---

**Last Updated**: 2025-06-17
**Issue**: #105 RigAgent Enhancement
**Status**: PARTIALLY COMPLETED