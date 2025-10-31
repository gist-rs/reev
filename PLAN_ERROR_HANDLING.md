# Plan: Graceful Error Handling for Flow Benchmarks

## Overview

This plan addresses critical issues with flow benchmark error handling that cause hard failures and frontend hanging. Focus on graceful degradation, proper error logging, and frontend state management.

## Current Problems

### Issue #1: Invalid Share Parameter (Benchmark 116)
- **Symptom**: LLM passes `-1` as shares to `jupiter_lend_earn_redeem`
- **Root Cause**: `query_token_balance()` returns `-1` on failure instead of proper error
- **Impact**: Agent crashes with "Number is not a valid u64"

### Issue #2: Insufficient Funds (Benchmark 200)
- **Symptom**: Agent tries to deposit 394M tokens when balance is 0
- **Root Cause**: No pre-validation of available balance before operation
- **Impact**: "Insufficient funds" error stops entire flow

### Issue #3: Hard Failure Propagation
- **Symptom**: Flow step failures crash entire benchmark suite
- **Root Cause**: Errors propagate up without recovery mechanisms
- **Impact**: Frontend gets stuck waiting for completion that never comes

## Solution Architecture

### 1. Soft Error Handling Strategy

#### Core Principle: "Fail Gracefully, Don't Crash"
- Catch errors at each flow step level
- Log comprehensive error information
- Continue with next benchmark instead of stopping entire suite
- Provide frontend with clear failure state

#### Error Classification
```rust
#[derive(Debug, Clone)]
pub enum FlowStepError {
    Recoverable { message: String, can_retry: bool },
    NonRecoverable { message: String, error_code: String },
    Validation { field: String, value: String, constraint: String }
}
```

### 2. Enhanced Session Logging

#### Session File Error Schema
```json
{
  "session_id": "...",
  "error": {
    "step": 2,
    "benchmark_id": "116-jup-lend-redeem-usdc",
    "error_type": "VALIDATION_ERROR",
    "message": "Invalid shares parameter: -1 is not a valid u64",
    "tool": "jupiter_lend_earn_redeem",
    "arguments": {"shares": -1, "asset": "..."},
    "recovery_attempted": true,
    "recovery_result": "Balance query failed, using 0 shares"
  }
}
```

### 3. Balance Query Fixes

#### Token Balance Query Enhancement
```rust
pub async fn query_token_balance(token_account: &str) -> Result<u64> {
    match get_balance_from_rpc(token_account).await {
        Ok(balance) => Ok(balance),
        Err(e) => {
            // Log error details for debugging
            tracing::warn!("Token balance query failed for {}: {}", token_account, e);
            
            // Return 0 instead of error - better than -1
            // This allows flow to continue with reasonable behavior
            Ok(0)
        }
    }
}
```

#### Pre-Operation Validation
```rust
impl JupiterLendEarnRedeemTool {
    async fn validate_balance(&self, asset: &str, required_amount: u64) -> Result<bool> {
        let user_wallet = self.key_map.get("USER_WALLET_PUBKEY")?;
        let current_balance = self.query_user_balance(user_wallet, asset).await?;
        
        if current_balance < required_amount {
            return Err(JupiterError::InsufficientFunds {
                available: current_balance,
                required: required_amount,
            });
        }
        
        Ok(true)
    }
}
```

### 4. Flow Step Error Recovery

#### Modified Flow Execution
```rust
async fn run_flow_benchmark(...) -> Result<TestResult> {
    let mut flow_result = FlowExecutionResult::new();
    
    for step in flow_steps.iter() {
        match execute_flow_step(step).await {
            Ok(step_result) => {
                flow_result.add_successful_step(step_result);
            }
            Err(step_error) => {
                // Log comprehensive error to session
                log_step_error_to_session(&step_error, &session_logger).await;
                
                // Determine if we can recover
                if let Some(recovered_result) = attempt_step_recovery(&step_error).await {
                    flow_result.add_recovered_step(recovered_result);
                } else {
                    flow_result.add_failed_step(step_error);
                }
                
                // Continue to next step instead of crashing
                continue;
            }
        }
    }
    
    // Calculate score based on successful steps
    let final_score = calculate_flow_score(&flow_result);
    Ok(TestResult::with_score(flow_result, final_score))
}
```

### 5. Frontend State Management

#### Execution State Transitions
```typescript
interface ExecutionState {
  status: 'queued' | 'running' | 'succeeded' | 'failed' | 'soft_failed';
  error?: {
    step: number;
    message: string;
    recoverable: boolean;
  };
  progress: {
    current_step: number;
    total_steps: number;
    successful_steps: number;
  };
}
```

#### Frontend Error Handling
- Display soft failures clearly without crashing
- Show which steps failed and why
- Allow continuing to next benchmark
- Provide recovery suggestions to user

## Implementation Phases

### Phase 1: Foundation (Week 1)
1. **Error Classification System**
   - Define error types and recovery strategies
   - Implement error enum and traits
   - Add error serialization for session logs

2. **Balance Query Fixes**
   - Fix `query_token_balance()` to return 0 on errors
   - Add comprehensive logging for failed queries
   - Implement balance validation helpers

3. **Session Error Logging**
   - Extend session schema to include error details
   - Add error logging functions to flow logger
   - Update session file format

### Phase 2: Flow Recovery (Week 2)
1. **Soft Error Handling**
   - Modify `run_flow_benchmark()` for step-level error catching
   - Implement recovery attempt logic
   - Add step result accumulation

2. **Pre-Validation Layer**
   - Add balance checks before Jupiter operations
   - Validate tool parameters before API calls
   - Implement parameter sanitization

3. **Enhanced Error Context**
   - Capture full context when errors occur
   - Log tool arguments and system state
   - Add timing and performance metrics

### Phase 3: Frontend Integration (Week 3)
1. **State Management Update**
   - Update execution state schema
   - Implement soft failure states
   - Add progress tracking for flows

2. **UI Error Display**
   - Design error display components
   - Show step-by-step failures
   - Provide recovery suggestions

3. **API Updates**
   - Update execution logs API to include error details
   - Add error recovery endpoints
   - Implement proper status transitions

### Phase 4: Advanced Features (Week 4)
1. **Retry Logic**
   - Add configurable retry for transient errors
   - Implement exponential backoff
   - Track retry attempts and outcomes

2. **Smart Recovery**
   - Fallback operation strategies
   - Alternative tool selection
   - Context-aware error resolution

3. **Monitoring & Analytics**
   - Error rate tracking
   - Recovery success metrics
   - Performance impact analysis

## Testing Strategy

### Unit Tests
```rust
#[tokio::test]
async fn test_balance_query_error_handling() {
    // Mock RPC failure
    let mock_rpc = setup_failing_rpc();
    
    // Should return 0, not error or -1
    let balance = query_token_balance_with_client("invalid_account", &mock_rpc).await;
    assert_eq!(balance, Ok(0));
}

#[tokio::test] 
async fn test_flow_step_recovery() {
    let step = create_failing_flow_step();
    let result = execute_flow_step_with_recovery(&step).await;
    
    assert!(result.is_ok());
    assert!(result.unwrap().error_recovered);
}
```

### Integration Tests
- Test benchmarks 116 and 200 end-to-end
- Verify frontend doesn't hang on failures
- Confirm error information appears in session logs
- Test soft failure state transitions

### Load Testing
- Multiple concurrent flow failures
- Resource exhaustion scenarios
- Network failure simulation
- Database connection issues

## Success Metrics

### Technical Metrics
- Zero frontend hangs on flow failures
- 100% error information captured in session logs
- <5% error propagation to top level
- >90% of recoverable errors handled gracefully

### User Experience Metrics
- Clear error messages for all failure types
- Progress visibility even during failures
- Fast recovery from common issues
- Confidence in system stability

### Operational Metrics
- Reduced support tickets for flow failures
- Faster debugging with enhanced error context
- Better system resilience under load
- Improved monitoring capabilities

## Risk Assessment

### Technical Risks
- **Complex Error Logic**: Error classification might become too complex
- **Performance Impact**: Additional error handling overhead
- **State Inconsistency**: Recovery might leave system in inconsistent state

### Mitigation Strategies
- Keep error classification simple and well-documented
- Profile performance impact of error handling
- Implement state validation after recovery attempts

### Rollback Plan
- Feature flag for new error handling
- Revert to simple hard failure mode if issues arise
- Maintain backward compatibility during transition

## Timeline & Dependencies

### Critical Path
1. **Week 1**: Balance query fixes + basic error logging
2. **Week 2**: Flow step error handling + recovery logic  
3. **Week 3**: Frontend state management + API updates
4. **Week 4**: Advanced features + monitoring

### Dependencies
- No external dependencies required
- Frontend team coordination for state management
- Testing environment setup for error scenarios
- Documentation updates for new error handling

### Deliverables
- Enhanced flow benchmark execution
- Improved error logging and session storage
- Frontend error handling improvements
- Comprehensive test coverage
- Monitoring and alerting setup