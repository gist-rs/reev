# 🪸 `reev` Engineering Guidelines

## 🎯 Core Principles

### 1. Code Quality Standards
- **Early Returns**: Use guard clauses to reduce nesting
- **Match Over If-Else**: Use `match` for >1 conditional cases
- **Error Handling**: Replace `unwrap()` with proper error handling using `context()`
- **Constants**: Centralize all magic numbers and hardcoded values
- **Snake Case**: Use snake_case for functions and variables

### 2. Architecture Rules
- **Separation of Concerns**: Core library (`reev-lib`) separate from binaries
- **Thin Binaries**: `main.rs` only handles argument parsing and setup
- **Module Index Files**: `mod.rs` only contains `mod` and `pub use` statements
- **Protocol Abstraction**: Use standardized traits for all protocols

### 3. Development Process
- **Plan-Driven**: Update `PLAN.md` before significant changes
- **Debug with Logs**: Use `tracing` crate, control with `RUST_LOG`
- **Test Coverage**: All features must have benchmarks with 100% success rates
- **Clippy First**: Run `cargo clippy --fix --allow-dirty` before commits

### 4. Jupiter Integration Rules
- **API-Only Instructions**: All Jupiter instructions must come from official API calls
- **No LLM Generation**: LLM is forbidden from generating Jupiter transaction data
- **Exact API Extraction**: Preserve complete API response structure
- **SDK Enforcement**: Use only official Jupiter SDK methods

### 5. Flow System Requirements
- **Step Isolation**: Each flow step executes as separate transaction
- **State Propagation**: Account states flow automatically between steps
- **Agent Consistency**: Deterministic and AI agents handle flows identically
- **Error Isolation**: Step failures don't cascade to other steps

## 🛠️ Production Standards

### Dependencies
- **Error Handling**: `anyhow` for context-rich errors
- **Serialization**: `serde`, `serde_json`, `serde_yaml`
- **CLI**: `clap` for command-line interfaces
- **TUI**: `ratatui` for terminal interfaces
- **Solana**: `solana-client`, `solana-sdk`, `spl-token`
- **LLM**: `rig` for agent-LLM communication
- **Logging**: `tracing` for structured logging

### Testing Requirements
- **Score Validation**: Test 0%, ~50%, ~75%, and 100% score scenarios
- **Anti-False-Positive**: Differentiate failure modes
- **Cross-Agent**: Validate consistency across agent types
- **Flow Testing**: Step-by-step execution validation
- **Regression Testing**: All tests must pass on every change

### Commit Standards
- **Conventional Commits**: Use `feat:`, `fix:`, `refactor:` prefixes
- **Zero Warnings**: No clippy warnings allowed
- **Tests Pass**: All tests must pass before commit
- **Documentation**: Update docs for API changes

## 🎯 Success Criteria

### Production Readiness
- ✅ All benchmarks passing (11/11 examples)
- ✅ Zero clippy warnings
- ✅ Comprehensive test coverage
- ✅ No critical security vulnerabilities
- ✅ Performance within acceptable limits

### Code Quality
- ✅ Centralized configuration
- ✅ Modular architecture
- ✅ Clear error messages
- ✅ Consistent naming conventions
- ✅ Well-documented APIs

The `reev` framework maintains enterprise-grade quality standards while enabling rapid development of blockchain agent evaluation capabilities.