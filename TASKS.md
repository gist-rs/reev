# TASKS.md - Dynamic Flow Implementation Tasks

## Phase 1: Bridge Mode (Week 1-2) - MVP Focus

### Issue #2: reev-orchestrator Crate Setup

#### Task 2.1: Initialize reev-orchestrator Crate
- [ ] Create `Cargo.toml` with dependencies: reev-types, reev-tools, reev-protocols, tokio, serde, anyhow, handlebars, lru
- [ ] Set up `src/lib.rs` with basic module structure
- [ ] Create `src/gateway.rs` for user prompt processing
- [ ] Create `src/context_resolver.rs` for wallet context
- [ ] Create `src/generators/mod.rs`, `src/generators/yml_generator.rs`
- [ ] Create `tests/` directory structure
- [ ] Add feature flags: `dynamic_flows = ["bridge"]`

**Acceptance**: Crate compiles, basic structure in place
**Estimated**: 0.5 days

#### Task 2.2: Context Resolver Implementation
- [ ] Extract token/price data from `protocols/jupiter/jup-sdk/tests/token_test.rs`
- [ ] Implement `WalletContext` struct in reev-types
- [ ] Create `ContextResolver` with Jupiter SDK integration
- [ ] Add parallel context resolution (balance + prices + metadata)
- [ ] Implement LRU cache with TTL (wallet: 5min, prices: 30s)
- [ ] Add OpenTelemetry tracing for context resolution

**Acceptance**: Context resolves < 500ms for typical wallet
**Estimated**: 2 days
**Dependency**: Task 5.1 (Mock Data)

#### Task 2.3: YML Generator Implementation
- [ ] Design YML structure matching existing benchmark format
- [ ] Implement template engine with Handlebars
- [ ] Create base templates for swap, lend, swap+lend
- [ ] Add context variable injection (amount, wallet, prices)
- [ ] Implement temporary file generation in `/tmp/dynamic-{timestamp}.yml`
- [ ] Add validation for generated YML structure

**Acceptance**: Generated YML validates against schema, loads in runner
**Estimated**: 1.5 days
**Dependency**: Task 6.1 (Template System)

#### Task 2.4: Gateway Implementation
- [ ] Implement `OrchestratorGateway` with prompt refinement
- [ ] Add natural language to intent parsing
- [ ] Create flow planner for step generation
- [ ] Integrate context resolver with prompt generation
- [ ] Add error handling and validation

**Acceptance**: "use 50% sol to 1.5x usdc" generates valid YML
**Estimated**: 2 days
**Dependencies**: Tasks 2.2, 2.3

#### Task 2.5: CLI Integration
- [ ] Add `--dynamic` flag to reev CLI
- [ ] Integrate orchestrator with CLI entry point
- [ ] Add wallet pubkey parameter handling
- [ ] Implement temporary file cleanup
- [ ] Add help text and usage examples

**Acceptance**: `reev exec --dynamic "prompt"` works end-to-end
**Estimated**: 1 day
**Dependency**: Task 2.4

### Issue #5: Mock Data System

#### Task 5.1: Mock Data Extraction
- [ ] Analyze `protocols/jupiter/jup-sdk/tests/token_test.rs`
- [ ] Extract common token balance patterns (SOL, USDC, USDT)
- [ ] Extract price response patterns
- [ ] Create `tests/mock_data.rs` with static structures
- [ ] Implement mock wallet context generator
- [ ] Add mock transaction responses

**Acceptance**: Mock data covers 80% of common test scenarios
**Estimated**: 1 day

#### Task 5.2: Integration Test Suite
- [ ] Create `tests/integration_tests.rs`
- [ ] Add tests for basic swap flow generation
- [ ] Add tests for lend flow generation
- [ ] Add tests for swap+lend multi-step flows
- [ ] Add tests for context injection accuracy
- [ ] Add performance tests for context resolution

**Acceptance**: 100% test coverage for Phase 1 features
**Estimated**: 1.5 days
**Dependency**: Task 5.1

### Issue #6: Template System

#### Task 6.1: Template Engine Setup
- [ ] Design template hierarchy structure
- [ ] Create `templates/` directory with base, protocols, scenarios
- [ ] Implement Handlebars template compilation
- [ ] Add template inheritance and partials
- [ ] Create template validation system

**Acceptance**: Template engine compiles and validates templates
**Estimated**: 1 day

#### Task 6.2: Template Creation
- [ ] Create `templates/base/swap.hbs` for generic swap
- [ ] Create `templates/base/lend.hbs` for generic lend
- [ ] Create `templates/protocols/jupiter/` overrides
- [ ] Create `templates/scenarios/swap_then_lend.hbs`
- [ ] Add template documentation and examples

**Acceptance**: Templates generate context-aware prompts
**Estimated**: 1 day
**Dependency**: Task 6.1

#### Task 6.3: Template Caching
- [ ] Implement LRU cache for compiled templates
- [ ] Add template hot-reload for development
- [ ] Add cache hit rate metrics
- [ ] Add template compilation performance tracking

**Acceptance**: Template compilation < 10ms, cache hit rate > 90%
**Estimated**: 0.5 days
**Dependency**: Task 6.1

### Issue #3: Runner Integration

#### Task 3.1: BenchmarkSource Enum Implementation
- [ ] Add `BenchmarkSource` enum to reev-runner
- [ ] Modify `RunBenchmark` struct to use enum
- [ ] Update runner logic to handle both sources
- [ ] Add backward compatibility layer
- [ ] Update CLI argument parsing

**Acceptance**: Existing static YML execution unchanged
**Estimated**: 1 day

#### Task 3.2: Dynamic Flow Execution
- [ ] Add temporary file handling logic
- [ ] Implement flow source detection
- [ ] Add dynamic flow metrics collection
- [ ] Add error handling for generated YML
- [ ] Update OpenTelemetry spans for dynamic flows

**Acceptance**: Dynamic YML executes with < 100ms overhead
**Estimated**: 1 day
**Dependency**: Task 3.1

#### Task 3.3: Feature Flag Integration
- [ ] Add `dynamic_flows` feature to reev-runner
- [ ] Implement "bridge" mode functionality
- [ ] Add feature flag validation
- [ ] Update CLI help text based on features

**Acceptance**: Feature flags control dynamic flow availability
**Estimated**: 0.5 days
**Dependency**: Task 3.1

### Issue #4: Agent Context Enhancement

#### Task 4.1: PromptContext Struct Design
- [ ] Define `PromptContext` in reev-types
- [ ] Add wallet balance, prices, flow state fields
- [ ] Implement serialization for context passing
- [ ] Add context validation

**Acceptance**: Context struct supports all needed data
**Estimated**: 0.5 days

#### Task 4.2: Agent Interface Enhancement
- [ ] Modify `execute_agent` signature to accept context
- [ ] Update `UnifiedGLMAgent` to process context
- [ ] Add context injection into prompt generation
- [ ] Implement context-aware tool selection

**Acceptance**: Agents can process wallet context
**Estimated**: 1.5 days
**Dependency**: Task 4.1

#### Task 4.3: OpenTelemetry Integration
- [ ] Add spans for context processing
- [ ] Track context resolution time
- [ ] Log prompt generation metrics
- [ ] Add flow execution tracing

**Acceptance**: Context processing fully traced
**Estimated**: 1 day
**Dependency**: Task 4.2

## Phase 1 Success Gates

### Technical Validation
- [ ] All dynamic flow patterns (swap, lend, swap+lend) work
- [ ] Context resolution < 500ms for typical wallets
- [ ] 99.9% backward compatibility maintained
- [ ] < 100ms flow execution overhead vs static
- [ ] Template system supports 90% of common patterns

### User Acceptance
- [ ] Natural language prompts work for basic DeFi operations
- [ ] Context-aware prompts adapt to user wallet state
- [ ] Clear error messages with recovery suggestions
- [ ] CLI `--dynamic` flag works seamlessly

### Developer Experience
- [ ] Comprehensive mock-based testing (100% coverage)
- [ ] Clear separation between static and dynamic flows
- [ ] Template inheritance and validation working
- [ ] Performance parity with existing system

## Phase 1 Testing Strategy

### Unit Tests
- [ ] Context resolver with various wallet states
- [ ] Template compilation and rendering
- [ ] YML generation validation
- [ ] Prompt refinement logic

### Integration Tests
- [ ] End-to-end dynamic flow execution
- [ ] Context accuracy verification
- [ ] Performance benchmarking
- [ ] Backward compatibility validation

### Mock Tests
- [ ] Jupiter SDK response mocking
- [ ] Token price simulation
- [ ] Wallet balance scenarios
- [ ] Error condition handling

## Risk Mitigation Tasks

### Performance Risks
- [ ] Implement aggressive caching (Task 2.2, 6.3)
- [ ] Add performance budgets and monitoring
- [ ] Create performance regression tests

### Compatibility Risks
- [ ] Comprehensive backward compatibility testing
- [ ] Feature flag controlled rollout
- [ ] Static flow preservation guarantees

### Integration Risks
- [ ] Clear contract definitions between components
- [ ] Integration tests for all boundaries
- [ ] Error handling and graceful degradation

## Phase 1 Timeline Summary

| Week | Tasks | Focus |
|------|-------|-------|
| Week 1 | 2.1, 2.2, 5.1, 5.2, 6.1 | Foundation & Mock Data |
| Week 1 | 6.2, 6.3, 3.1, 3.3 | Templates & Runner |
| Week 2 | 2.3, 2.4, 4.1, 4.2 | Generation & Agent |
| Week 2 | 2.5, 3.2, 4.3, Validation | Integration & Testing |

**Total Estimated**: 14 days (2 weeks)
**Buffer Time**: 2 days
**Total Phase 1**: 16 days

## Dependencies Graph

```
Task 2.1 (Crate Setup)
├── Task 2.2 (Context Resolver)
│   ├── Task 2.4 (Gateway)
│   │   └── Task 2.5 (CLI)
│   └── Task 4.1 (PromptContext)
│       └── Task 4.2 (Agent Enhancement)
├── Task 2.3 (YML Generator)
│   └── Task 6.1 (Templates)
│       ├── Task 6.2 (Template Creation)
│       └── Task 6.3 (Template Caching)
└── Task 3.1 (Runner Integration)
    └── Task 3.2 (Dynamic Execution)

Task 5.1 (Mock Data)
└── Task 5.2 (Integration Tests)
```

## Phase 1 Deliverables

1. **reev-orchestrator** crate with dynamic flow generation
2. **Enhanced reev-runner** with dynamic flow support
3. **Enhanced reev-agent** with context awareness
4. **Template system** with 90% pattern coverage
5. **Mock data system** with 100% test coverage
6. **CLI integration** with `--dynamic` flag
7. **OpenTelemetry integration** for flow tracing
8. **Comprehensive test suite** with no external dependencies

## Success Metrics for Phase 1

### Quantitative
- Dynamic flow success rate: ≥ 95% (matching static)
- Context resolution time: < 500ms (95th percentile)
- Flow execution overhead: < 100ms
- Template coverage: 90% of common patterns
- Test coverage: 100% for new features
- Backward compatibility: 99.9%

### Qualitative
- User can type natural language for basic DeFi operations
- Prompts adapt to actual wallet state
- Clear error messages with recovery suggestions
- Seamless CLI experience with `--dynamic` flag
- Developer can easily add new flow patterns
- Performance parity with existing system

## Next Phase Preparation

After Phase 1 completion, prepare for:
1. **Phase 2**: Direct in-memory flow execution
2. **Phase 3**: Recovery mechanisms and non-critical steps
3. **Performance optimization** based on Phase 1 metrics
4. **User feedback integration** and template refinement
5. **Documentation** and migration guides