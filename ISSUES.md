# Issues

## Issue #8: Dynamic Flow API Implementation

**Priority**: 🟡 **MEDIUM**
**Status**: 📝 **PLANNED**
**Assigned**: TBD
**Component**: reev-api, reev-orchestrator Integration

### 🎯 **Problem Statement**

The reev system has fully functional dynamic flow capabilities via CLI (bridge/direct/recovery modes), but these features are not accessible through the REST API. Users can only execute static benchmarks via the current API endpoints.

### 📋 **Current Status**

**✅ Implemented (CLI Only)**:
- Dynamic flow generation from natural language prompts
- Bridge mode: Temporary YML file generation
- Direct mode: Zero file I/O in-memory execution
- Recovery mode: Enterprise-grade failure handling with 3 strategies
- Context resolution with wallet balance and pricing
- Template system with caching and inheritance

**❌ Missing (API Endpoints)**:
- `POST /api/v1/benchmarks/execute-dynamic` - Bridge mode execution
- `POST /api/v1/benchmarks/execute-direct` - Direct mode execution  
- `POST /api/v1/benchmarks/execute-recovery` - Recovery mode execution
- `GET /api/v1/metrics/recovery` - Recovery performance metrics

**✅ Existing Polling Infrastructure**:
- `GET /api/v1/benchmarks/{id}/status/{execution_id}` - Execution status polling
- `GET /api/v1/benchmarks/{id}/status` - Most recent execution status
- `GET /api/v1/flows/{session_id}` - Flow diagram with stateDiagram visualization
- `GET /api/v1/flow-logs/{benchmark_id}` - Flow execution logs
- `GET /api/v1/execution-logs/{benchmark_id}` - Execution trace logs
- ExecutionState and ExecutionStatus enums for state tracking

### 🏗️ **Required Implementation**

#### Phase 4.1: Dynamic Flow Endpoints
```rust
// Add to reev-api/src/handlers/dynamic_flows.rs
pub async fn execute_dynamic_flow(
    State(state): State<ApiState>,
    Json(request): Json<DynamicFlowRequest>,
) -> Result<Json<ExecutionResponse>, ApiError>

pub async fn execute_direct_flow(
    State(state): State<ApiState>,
    Json(request): Json<DynamicFlowRequest>,
) -> Result<Json<ExecutionResponse>, ApiError>

pub async fn execute_recovery_flow(
    State(state): State<ApiState>,
    Json(request): Json<RecoveryFlowRequest>,
) -> Result<Json<ExecutionResponse>, ApiError>
```

#### Phase 4.2: API Dependencies
```toml
# Add to crates/reev-api/Cargo.toml
[dependencies]
reev-orchestrator = { path = "../reev-orchestrator" }
```

#### Phase 4.3: Session Management Enhancement
```rust
// Extend existing session tracking for dynamic flows
// Existing: get_flow(), get_execution_status(), ExecutionState struct
// Add: caching headers, frequency recommendations, dynamic flow integration
```

### 🔄 **Integration Points**

1. **reev-api → reev-orchestrator**: Use existing gateway functions
2. **Request Validation**: Leverage existing flow planning and context resolution
3. **Session Tracking**: Integrate with reev-flow session management
4. **Error Handling**: Use existing recovery and atomic execution patterns
5. **OpenTelemetry**: Extend current tracing for API-based flow execution

### 📊 **Success Criteria**

- [ ] All dynamic flow modes accessible via REST API
- [ ] Real-time session management and monitoring
- [ ] Full recovery system integration via API
- [ ] Live flow visualization and Mermaid diagram generation
- [ ] Backward compatibility with existing static endpoints
- [ ] Comprehensive error handling and status reporting

### ⚠️ **Blockers & Dependencies**

**Technical Blockers**:
- None - all underlying functionality (reev-orchestrator) is production-ready

**Required Dependencies**:
- Add `reev-orchestrator` dependency to `reev-api/Cargo.toml`
- Polling for session updates
- Enhanced request validation and security middleware

**Integration Requirements**:
- Must work seamlessly with existing static benchmark system
- Preserve all current CLI functionality and performance characteristics
- Maintain backward compatibility with existing API clients
- Enhance existing polling infrastructure: add caching headers, document frequency recommendations
- Extend existing ExecutionState tracking to support dynamic flow sessions

### 📈 **Impact Assessment**

**User Impact**: High - Enables web-based access to dynamic flow capabilities
**Development Impact**: Medium - Well-defined integration points with existing code
**Operational Impact**: Low - No changes to existing static benchmark workflow

**Estimated Effort**: 2-3 weeks (Phase 4 implementation)
**Priority**: Medium - CLI implementation provides core functionality, API enables broader adoption

### 🗓️ **Timeline**

**Week 1**: Basic dynamic flow endpoints (execute-dynamic, execute-direct)
**Week 1**: Basic dynamic flow endpoints (execute-dynamic, execute-direct)
**Week 2**: Recovery endpoints and enhanced session management with caching headers
**Timeline reduced to 1-2 weeks due to comprehensive existing polling infrastructure

---

*Last Updated: Current*
*Related Files*: TASKS.md, ARCHITECTURE.md, crates/reev-api/Cargo.toml
*Dependencies*: reev-orchestrator integration, reev-flow session management
