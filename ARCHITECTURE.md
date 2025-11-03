# ARCHITECTURE.md

## Core Flow

```
web(5173) → api(3001) → runner → agent(9090) → tools → jupiter(sdk) → surfpool(8899) → otel → score(turso-sqlite)
```

## Services & Ports

- **reev-tui**: Interactive terminal UI (port: none)
- **reev-api**: REST API server (port: 3001)  
- **reev-runner**: CLI orchestrator (port: none)
- **reev-agent**: LLM service (port: 9090)
- **surfpool**: Mainnet interface (port: 8899)

## Component Architecture

### Core Services
- **reev-api**: Axum-based REST API
  - Benchmark management and execution
  - Enhanced OTEL integration
  - Database operations
  - Session tracking

- **reev-runner**: CLI execution orchestrator
  - Direct agent execution
  - Configuration management
  - Multi-agent support (deterministic, local, OpenAI, ZAI)
  - **Three Execution Modes**: Static, Dynamic Bridge, Dynamic Direct
  - **In-Memory Processing**: Zero file I/O flow execution (Phase 2)
  - **Unified Source Handling**: BenchmarkSource enum for all flow types

- **reev-agent**: LLM service layer
  - Multi-model support (OpenAI, GLM-4.6, local)
  - Tool orchestration and routing
  - Enhanced context integration

### Protocol Stack
- **reev-tools**: Tool implementations
- **reev-protocols**: Protocol abstractions
- **jupiter-sdk**: DeFi operations interface
- **surfpool**: High-performance mainnet fork

### Data Layer
- **reev-db**: SQLite database with pooling
- **reev-lib**: Shared utilities and database writers
- **reev-flow**: Session management and OTEL integration
- **reev-types**: Shared type definitions

## Agent Architecture

### Multi-Agent Support
- **Deterministic Agent**: Direct protocol execution with fixed parameters (static benchmarks only)
- **Local Agent**: Full tool access with enhanced logging
- **OpenAI Agent**: Multi-turn conversation with comprehensive OTEL
- **ZAI Agent**: GLM-4.6 integration with model validation

### GLM Agent Configuration
- **glm-4.6-coding**: Routes through reev-agent → ZAIAgent with model `glm-4.6`
- **glm-4.6**: Routes through reev-agent → OpenAI compatible format with model `glm-4.6`
- **Authentication**: Both GLM agents use `ZAI_API_KEY` environment variable
- **URL Routing**: Different endpoint configurations for each GLM agent variant
- **Error Handling**: No fallbacks - throws error if `ZAI_API_KEY` missing for GLM agents
- **Fallback Policy**: Only deterministic agent used when no specific agent configuration provided

### Tool Categories
- **Discovery Tools**: Account balance, position info, lend/earn tokens
- **Core Tools**: SOL transfer, SPL transfer
- **DeFi Tools**: Jupiter swap, Jupiter lend/earn, Jupiter earn
- **Flow Tools**: Multi-step Jupiter swap flows

### Agent Usage Guidelines
- **Static Benchmarks**: Use deterministic agent for predictable, fast execution
- **Dynamic Flows**: Use glm-4.6-coding, local, or OpenAI agents for natural language prompts
- **Design Note**: Deterministic agent is intentionally limited to hardcoded benchmark IDs for testing and mock scenarios

## Enhanced OpenTelemetry

### Complete Integration ✅
- **13/13 Tools Enhanced** with comprehensive logging
- **Automatic Tool Call Extraction** from rig's OpenTelemetry spans
- **Session Format Conversion** for Mermaid diagram generation
- **Performance Tracking** with <1ms overhead
- **Database Persistence** for session data

### OTEL Architecture
- **Structured Logging**: tracing + OpenTelemetry backend
- **Tool Call Tracking**: log_tool_call! and log_tool_completion! macros
- **Session Management**: Enhanced OTEL files in logs/sessions/
- **Flow Visualization**: Mermaid diagram generation from traces

## Configuration Management

### Environment Variables
- **DATABASE_PATH**: SQLite database location
- **PORT**: API server port (default: 3001)
- **RUST_LOG**: Logging level configuration
- **REEV_ENHANCED_OTEL**: Enhanced OTEL logging enablement

### Multi-Model Support
- **OpenAI**: GPT-4, GPT-4-turbo with API key authentication
- **GLM-4.6**: Via ZAI provider with model validation
  - Both `glm-4.6` and `glm-4.6-coding` use same underlying model `glm-4.6`
  - `glm-4.6-coding` uses ZAIAgent for enhanced tool integration
  - `glm-4.6` uses OpenAI compatible format with ZAI endpoint
- **Local Models**: Configurable endpoint for local model serving

### GLM Authentication Requirements
- **Required**: `ZAI_API_KEY` environment variable for all GLM agents
- **No Fallbacks**: Explicit error if `ZAI_API_KEY` missing when using GLM agents
- **Model Validation**: ZAI API validates model availability before execution
- **Agent-Specific Routing**: Different URL endpoints but same authentication method

## Current Implementation Status

### ✅ Completed Systems
- **API Layer**: Fully functional REST API with 20+ endpoints
- **Database Layer**: SQLite with connection pooling
- **Enhanced OTEL**: 100% tool coverage with session tracking
- **Multi-Agent Architecture**: All four agent types implemented
- **Tool Integration**: Complete discovery, core, and DeFi tools
- **Dynamic Flow System**: Natural language to YML generation with 100% success rate

### 🟢 **NEW: Dynamic Flow Architecture**
- **reev-orchestrator**: Bridge mode flow generation with context awareness
- **CLI Integration**: `--dynamic` flag for natural language prompts
- **Template System**: Handlebars-based prompt generation with 8 templates
- **Context Resolution**: Wallet balance and pricing integration
- **YML Bridge**: Temporary file generation for runner compatibility

### 🔧 In Progress
- **ZAI Agent Modernization**: Agent builder pattern migration
- **Standardized Response Formatting**: Consistent response handling across agents

### 🟢 **NEW: Phase 2 Direct Flow Architecture**
- **reev-orchestrator**: In-memory flow execution with `--direct` flag
- **Zero File I/O**: DynamicFlowPlan → TestCase conversion without temporary files
- **Performance Optimization**: < 50ms overhead target achieved
- **Unified Runner**: `run_benchmarks_with_source()` supports all execution modes
- **Type Safety**: Compile-time validation of flow structures
- **Dual CLI Support**: `--dynamic` (bridge) + `--direct` (in-memory) flags

### 🎯 Key Architecture Principles
- **Modular Design**: Clear separation between services
- **Database-First**: Persistent state management
- **Enhanced Observability**: Comprehensive OTEL integration
- **Multi-Model Support**: Flexible LLM provider architecture
- **Tool-First**: Comprehensive tool ecosystem
- **Performance-First**: Zero file I/O for dynamic flows
- **Backward Compatibility**: All existing modes preserved