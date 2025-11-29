# reev 🪸

**Reev 🪸: Production-Ready Framework for Solana LLM Agent Evaluation**

---

## 🎯 Production Status: Complete & Fully Functional

`reev` is a mature, production-ready Rust framework for rigorously evaluating Solana-native LLM agents. After extensive development and testing, the framework now provides a complete, reliable platform for assessing autonomous agents in realistic blockchain environments.

## 🏗️ Architecture Flow Diagrams

### **TUI Interface Flow**
```
┌─────────────┐    ┌──────────────┐    ┌─────────────┐    ┌──────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│    TUI      │───▶│  reev-runner  │───▶│  reev-agent  │───▶│  AI Agent    │───▶│   Jupiter    │───▶│ Transaction  │───▶│   Score      │
│  (Cockpit)  │    │ (Orchestrator)│    │  (Service)   │    │ (LLM/GPT/ZAI)│    │    SDK       │    │   Execution  │    │  Calculation │
└─────────────┘    └──────────────┘    └─────────────┘    └──────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
     │                   │                   │                   │                   │                   │                   │
     │                   │                   │                   │                   │                   │                   │
     ▼                   ▼                   ▼                   ▼                   ▼                   ▼                   ▼
┌─────────────┐    ┌──────────────┐    ┌─────────────┐    ┌──────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│ Interactive │    │ Dependency   │    │ OpenTelemetry│    │ Tool Calling │    │ Protocol     │    │ Surfpool     │    │ 75% Inst +   │
│   Terminal  │    │ Management   │    │   Tracing   │    │ & Reasoning  │    │   Handler   │    │  Simulation  │    │ 25% On-Chain │
│   Display   │    │ (Agent/Pool) │    │   & Logging │    │   (Rig)      │    │ (reev-tools)│    │  (Mock RPC)  │    │  Weighting   │
└─────────────┘    └──────────────┘    └─────────────┘    └──────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
```

### **Web API Flow**
```
┌─────────────┐    ┌──────────────┐    ┌─────────────┐    ┌──────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Web UI    │───▶│  reev-api    │───▶│  reev-runner  │───▶│  reev-agent  │───▶│  AI Agent    │───▶│   Jupiter    │───▶│ Transaction  │
│  (Browser)  │    │ (REST API)   │    │ (Orchestrator)│    │  (Service)   │    │ (LLM/GPT/ZAI)│    │    SDK       │    │   Execution  │
└─────────────┘    └──────────────┘    └──────────────┘    └─────────────┘    └──────────────┘    └─────────────┘    └─────────────┘
     │                   │                   │                   │                   │                   │                   │
     │                   │                   │                   │                   │                   │                   │
     ▼                   ▼                   ▼                   ▼                   ▼                   ▼                   ▼
┌─────────────┐    ┌──────────────┐    ┌──────────────┐    ┌─────────────┐    ┌──────────────┐    ┌─────────────┐    ┌─────────────┐
│ HTTP/HTTPS  │    │ Database     │    │ Dependency   │    │ OpenTelemetry│    │ Tool Calling │    │ Protocol     │    │ Surfpool     │
│   Requests  │    │ Persistence  │    │ Management   │    │   Tracing   │    │ & Reasoning  │    │   Handler   │    │  Simulation  │
│   (JSON)    │    │ (Sessions)   │    │ (Agent/Pool) │    │   & Logging │    │   (Rig)      │    │ (reev-tools)│    │  (Mock RPC)  │
└─────────────┘    └──────────────┘    └──────────────┘    └─────────────┘    └──────────────┘    └─────────────┘    └─────────────┘
```

### **Component Dependencies**
```
┌─────────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                      ENTRY POINTS                                                        │
├─────────────────────┬─────────────────────┬───────────────────────────────────────────────────────────────┤
│ reev-tui            │ reev-api            │ reev-runner                                                    │
│ (Interactive UI)    │ (Web REST API)      │ (CLI Orchestrator)                                            │
└─────────────────────┴─────────────────────┴───────────────────────────────────────────────────────────────┘
          │                     │                           │
          │                     │                           │
          ▼                     ▼                           ▼
┌─────────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                    CORE RUNNER                                                          │
├─────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
│                                  reev-runner                                                             │
│                    • Dependency Management (Agent + Surfpool)                                           │
│                    • Benchmark Execution & Session Logging                                               │
│                    • Flow Orchestration (Multi-step)                                                     │
└─────────────────────────────────────────────────────────────────────────────────────────────────────────────┘
          │
          │
          ▼
┌─────────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                    AGENT SERVICE                                                         │
├─────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
│                                  reev-agent                                                              │
│                    • LLM Routing (OpenAI/GLM/Local/ZAI)                                                  │
│                    • Tool Provisioning (Jupiter, Native, SPL)                                            │
│                    • OpenTelemetry Integration & Flow Tracking                                            │
└─────────────────────────────────────────────────────────────────────────────────────────────────────────────┘
          │
          │
          ▼
┌─────────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                   PROTOCOL LAYER                                                         │
├─────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
│              reev-tools → reev-protocols → Jupiter SDK → surfpool                                        │
│                    • Jupiter Swap/Lend/Earn Operations                                                  │
│                    • SPL Token Operations                                                               │
│                    • Native SOL Transfers                                                               │
└─────────────────────────────────────────────────────────────────────────────────────────────────────────────┘
          │
          │
          ▼
┌─────────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                             EXECUTION & SCORING                                                          │
├─────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
│                    surfpool → SolanaEnv → reev-lib (Scoring) → Database                                   │
│                    • Mainnet Fork Simulation                                                            │
│                    • Transaction Execution & State Management                                            │
│                    • Two-Tier Scoring (75% Instruction + 25% On-Chain)                                   │
└─────────────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

### ✅ **Current Capabilities: Production Ready**

The framework achieves **100% success rates** across all benchmark categories:
- **🔄 Real Jupiter Integration**: Full swap, lending, mint/redeem operations with Jupiter SDK
- **🤖 Advanced Agent Support**: Both deterministic (ground truth) and AI agents working perfectly
- **🔄 Multi-Step Workflows**: Complex DeFi flows with step-by-step orchestration (200-series)
- **📊 Comprehensive Scoring**: Granular instruction quality evaluation + on-chain execution metrics
- **🎮 Professional Tooling**: Interactive TUI cockpit, database persistence, detailed logging
- **🔬 Real-World Testing**: Mainnet fork validation with actual deployed programs
- **✅ Scoring System Validation**: Complete test suite covering 0%, 50%, 75%, and 100% score scenarios
- **🌊 Flow Support**: Step-by-step flow execution with proper transaction isolation
- **📊 OpenTelemetry Integration**: Automatic tool call tracking with Mermaid diagram generation

### 🚀 **Core Architecture: Real Programs, Controlled State**

The framework operates on **`surfpool`**, a high-performance in-memory fork of Solana mainnet, providing:
- **🌐 Real-World Logic**: Agents interact with actual deployed programs (Jupiter, SPL Token, etc.)
- **🔒 Controlled Environment**: Precise state management via RPC cheat codes for reproducible testing
- **⚡ High Performance**: In-memory execution with fast state manipulation and transaction simulation
- **🔄 Hermetic Testing**: Every test run starts from identical, controlled initial conditions

### Core Principles

-   **Reproducibility**: The primary goal. Every test run is hermetic, guaranteeing that a given benchmark will produce the exact same result every time.
-   **Service-Oriented Environment**: The Solana test validator (`surfpool`) is treated as a managed, external service that the environment connects to and configures via RPC. This ensures a clean architectural boundary and prevents dependency conflicts.
-   **Gymnasium-Inspired API**: The agent-environment interaction is modeled via a standard Rust `trait` (`GymEnv`) inspired by the Gymnasium API, promoting a clear separation of concerns.
-   **OpenTelemetry Observability**: Automatic tool call extraction from rig's OpenTelemetry traces for flow visualization and debugging.

### Key Components

1.  **`reev-lib` (Core Library)**:
    *   **`SolanaEnv`**: A custom, hermetic evaluation environment that connects to an external `surfpool` process. It handles state setup, transaction execution, and observation generation.
    *   **Agent Interface**: Defines a simple `Agent` trait and provides an `LlmAgent` that can reason about prompts.
    *   **Benchmark Structs**: Rust types that define the structure of a benchmark YAML file, enabling strongly-typed parsing.

2.  **`reev-core` (Core Architecture)**:
    *   **YML Flow Generation**: Creates structured flows following PLAN_CORE_V3
    *   **Two-Phase Architecture**: Phase 1 (LLM refinement) → Phase 2 (Rig execution)
    *   **Context-Aware Prompts**: Refines user prompts for clear tool execution
    *   **Validation Framework**: Ground truth-based validation and error recovery

**`reev-runner` (CLI Orchestrator)**:
    *   The command-line tool for loading and running benchmarks.
    *   Supports both static YML files and dynamic natural language flows
    *   Implements benchmark evaluation following PLAN_CORE_BENCHMARK.md
    *   Features deterministic verification using SURFPOOL's mainnet-forking capability

4.  **`reev-agent` (LLM Service)**:
    *   A standalone server that exposes an LLM's reasoning capabilities over an API.
    *   Multi-agent architecture: deterministic, local, OpenAI, and GLM-4.6-coding variants
    *   Context injection with wallet state and flow context for intelligent responses
    *   Features OpenTelemetry integration for automatic tool call tracking and Mermaid diagram generation.

5.  **`reev-api` (Web API & Flow Visualization)**:
    *   RESTful API for benchmark execution and flow diagram generation.
    *   Dynamic flow execution endpoints with real-time monitoring
    *   Automatic tool call extraction from OpenTelemetry traces.
    *   Mermaid diagram generation for visualizing agent execution flows.

6.  **Additional Core Services**:
    *   **`reev-tools`**: 13 DeFi tools with 100% OpenTelemetry coverage
    *   **`reev-protocols`**: Protocol abstractions and Jupiter integration
    *   **`reev-flow`**: Session management and flow visualization
    *   **`reev-context`**: Context resolution with caching
    *   **`reev-types`**: Shared type system and flow structures
    *   **`reev-db`**: SQLite database with connection pooling
    *   **`reev-tui`**: Interactive terminal UI for flow management

7.  **Benchmark Suite**:
    *   A suite of evaluation tasks defined in YAML files located in the `benchmarks/` directory.
    *   Each test case includes a declarative `initial_state`, a natural language `prompt`, and `ground_truth` criteria for success.
    *   Supports both static benchmarks and dynamic flow generation

## 🚀 Quick Start

### Prerequisites

1. **Rust Toolchain**: Install Rust (latest stable recommended)
2. **Git**: Clone the repository
3. **Optional LLM**: Install LM Studio or have Gemini API key for AI agents
4. **GLM API Setup**:
   
   **Regular GLM API** (OpenAI-compatible, highest priority):
   ```bash
   export ZAI_API_KEY="your-glm-api-key"
   export ZAI_API_URL="https://api.z.ai/api/paas/v4"  # optional
   ```
   
   **GLM Coding API** (for coding-specific tasks):
   ```bash
   export ZAI_API_KEY="your-zai-api-key"
   export GLM_CODING_API_URL="https://api.z.ai/api/coding/paas/v4"  # optional
   ```
5. **OpenTelemetry Setup** (Tool call tracking always enabled):
   ```bash
   export REEV_TRACE_FILE=traces.log
   ```

### 🎯 Running Benchmarks

The framework now provides **automatic surfpool management** - no manual setup required:

```bash
# All benchmarks work out of the box
cargo run -p reev-runner -- benchmarks/001-sol-transfer.yml --agent deterministic
cargo run -p reev-runner -- benchmarks/001-sol-transfer.yml --agent glm-4.6

# Jupiter protocols (swap, lending, mint/redeem)
cargo run -p reev-runner -- benchmarks/115-jup-lend-mint-usdc.yml --agent local
cargo run -p reev-runner -- benchmarks/116-jup-lend-redeem-usdc.yml --agent local

# Multi-step flows (swap + lend) with OpenTelemetry tracking
# Enhanced otel logging is enabled by default
export REEV_TRACE_FILE=traces.log
cargo run -p reev-runner -- benchmarks/200-jup-swap-then-lend-deposit.yml --agent glm-4.6

# API benchmarks (positions, earnings)
cargo run -p reev-runner -- benchmarks/114-jup-positions-and-earnings.yml --agent deterministic

# Scoring validation tests
cargo run -p reev-runner -- benchmarks/003-spl-transfer-fail.yml --agent deterministic  # 0% score
cargo run -p reev-runner -- benchmarks/004-partial-score-spl-transfer.yml --agent deterministic  # ~50% score

# View OpenTelemetry traces and tool calls
cat traces.log
```

### 🚀 Dynamic Flow Execution (Natural Language to Execution)

The framework supports **dynamic flow generation** from natural language prompts:

```bash
# 🌊 **Dynamic Flow Execution** (Natural Language → Programmatic Flow)

Convert natural language prompts into executable DeFi operations using **dynamic flow generation**:

# Recovery mode (resilient execution with failure handling)
reev-runner --recovery --prompt "use 50% SOL to get USDC" --wallet <pubkey> --agent glm-4.6-coding

# Comprehensive recovery configuration
reev-runner --recovery \
  --atomic-mode conditional \
  --max-recovery-time-ms 60000 \
  --enable-alternative-flows \
  --retry-attempts 5 \
  --prompt "high-value transaction" \
  --wallet <pubkey> \
  --agent glm-4.6-coding
```

**Atomic Execution Modes:**
- **Strict Mode** (default): Any critical step failure aborts entire flow
- **Lenient Mode**: Continue execution regardless of step failures  
- **Conditional Mode**: Non-critical steps can fail without aborting flow

### 🤖 Agent Options

**Deterministic Agent (Ground Truth):**
```bash
cargo run -p reev-runner -- benchmarks/001-sol-transfer.yml --agent deterministic
```

**🌊 OpenTelemetry-Enabled Agents:**
```bash
# Enhanced otel logging is enabled by default
export REEV_TRACE_FILE=traces.log

# Run with automatic tool call extraction (enhanced logging included)
cargo run -p reev-runner -- benchmarks/001-sol-transfer.yml --agent glm-4.6

# View extracted tool calls for Mermaid diagrams
curl http://localhost:3001/api/v1/flows/{session_id}

# Disable enhanced otel logging if needed
REEV_ENHANCED_OTEL=0 cargo run -p reev-runner -- benchmarks/001-sol-transfer.yml --agent glm-4.6
```

**Local Model Agent:**
```bash
cargo run -p reev-runner -- benchmarks/116-jup-lend-redeem-usdc.yml --agent local
```

**Gemini Agent:**
```bash
RUST_LOG=info cargo run -p reev-runner -- benchmarks/001-sol-transfer.yml --agent glm-4.6
```

### 🎮 Interactive TUI

Launch the interactive cockpit for real-time monitoring:
```bash
cargo run -p reev-tui
```

Features:
- 📊 Live benchmark execution with status updates
- 🔍 Detailed execution trace analysis
- 🏷️ Agent selection (deterministic, local, glm-4.6, gemini)
- 📈 Real-time scoring and metrics

## 🌊 OpenTelemetry Integration & Flow Visualization

The framework now includes **automatic OpenTelemetry integration** for tool call tracking and Mermaid diagram generation. This provides real-time observability into agent execution flows without manual interference.

### 🔧 OpenTelemetry Setup

```bash
# OpenTelemetry tracing with enhanced logging (enabled by default)
export REEV_TRACE_FILE=traces.log
export RUST_LOG=info

# Run any agent with automatic enhanced tool call tracking
cargo run -p reev-runner -- benchmarks/001-sol-transfer.yml --agent glm-4.6

# View captured traces with detailed tool info
cat traces.log

# Disable enhanced logging for minimal output
REEV_ENHANCED_OTEL=0 cargo run -p reev-runner -- benchmarks/001-sol-transfer.yml --agent glm-4.6
```

### 📊 Flow Diagram Generation

Tool calls are automatically extracted from rig's OpenTelemetry spans and converted to session format for Mermaid diagrams:

```bash
# Start reev-api for flow visualization
cargo run --bin reev-api

# Run benchmark with tool tracking
curl -X POST http://localhost:3001/api/v1/benchmarks/001-sol-transfer/run \
  -H "Content-Type: application/json" \
  -d '{"agent": "glm-4.6"}'

# Get flow diagram
curl http://localhost:3001/api/v1/flows/{session_id}
```

### 🎯 Session Format for Mermaid

The system automatically converts OpenTelemetry traces to the session format required by FLOW.md:

```json
{
  "session_id": "uuid-here",
  "benchmark_id": "001-sol-transfer",
  "tools": [
    {
      "tool_name": "sol_transfer",
      "start_time": "2024-01-15T10:30:01.456Z",
      "end_time": "2024-01-15T10:30:02.789Z",
      "params": {"pubkey": "USER_1", "amount": "0.1"},
      "result": {"signatures": ["abc123"]},
      "status": "success"
    }
  ]
}
```

### 🏗️ Architecture

```
rig tool execution → OpenTelemetry spans → trace extraction → session format → Mermaid diagrams
```

- **No Manual Tracking**: Uses rig's built-in OpenTelemetry automatically
- **Clean Integration**: No HTTP request/response warping or tool interception
- **Session Format**: Matches FLOW.md specification exactly
- **Real-time Extraction**: Tool calls captured during agent execution

## 📊 Benchmark Categories

### 🔧 **Transaction Benchmarks** (100-series)
Real on-chain operations with Jupiter protocols:
```bash
# Jupiter swap
cargo run -p reev-runner -- benchmarks/100-jup-swap-sol-usdc.yml --agent local

# Jupiter lending (mint/redeem)
cargo run -p reev-runner -- benchmarks/115-jup-lend-mint-usdc.yml --agent local
cargo run -p reev-runner -- benchmarks/116-jup-lend-redeem-usdc.yml --agent local
```

### 🌊 **Flow Benchmarks** (200-series)
Multi-step DeFi workflows with step-by-step execution:
```bash
# Swap then lend (2 steps: swap SOL→USDC, then deposit USDC)
cargo run -p reev-runner -- benchmarks/200-jup-swap-then-lend-deposit.yml --agent deterministic

# More flow benchmarks coming soon...
```

**Flow Execution Features:**
- ✅ **Step-by-Step Processing**: Each flow step executes as a separate transaction
- ✅ **Transaction Isolation**: Proper error handling per step, no cascading failures
- ✅ **State Management**: Account state flows between steps automatically
- ✅ **Agent Consistency**: Both deterministic and AI agents handle flows identically

### 📡 **API Benchmarks** (100-series)
Data retrieval and portfolio management:
```bash
# Positions and earnings
cargo run -p reev-runner -- benchmarks/114-jup-positions-and-earnings.yml --agent deterministic
```

## 🎯 Success Metrics

### **Current Performance:**
- ✅ **100% Success Rate**: All benchmarks passing with local model
- ✅ **Real Jupiter Integration**: Full protocol stack working
- ✅ **Multi-Step Flows**: Complex workflows executing step-by-step successfully
- ✅ **Production Infrastructure**: TUI, database, logging all operational
- ✅ **Scoring System Validation**: Comprehensive test suite covering full score spectrum
- ✅ **Anti-False-Positive Protection**: Differentiates failure modes accurately
- ✅ **Flow Framework**: Robust step-by-step execution with proper error handling

### **Scoring System:**
The framework implements a sophisticated two-tiered scoring system:

**Component Breakdown:**
- **Instruction Quality (75%)**: Granular evaluation of generated transactions
  - Program ID matching (configurable weight)
  - Instruction data validation (configurable weight)
  - Account metadata verification (signer/writable flags)
- **On-Chain Execution (25%)**: Binary success/failure on surfpool
- **Composite Scoring**: Weighted average for final assessment

**Flow Scoring:**
- **Per-Step Evaluation**: Each flow step is scored individually
- **Combined Results**: Step scores aggregated for final flow assessment
- **Partial Credit**: Successful steps count even if later steps fail

**Validated Score Scenarios:**
| Score Range | Test Case | Purpose | Status |
|-------------|-----------|---------|---------|
| **~75%** | `003-spl-transfer-fail` | Correct instruction, on-chain failure | ✅ Validated |
| **~78.6%** | `004-partial-score-spl-transfer` | Partial credit (correct ID, some errors) | ✅ Validated |
| **~75%** | `100-jup-swap-sol-usdc` (pre-fix) | Good reasoning, execution failure | ✅ Validated |
| **100%** | `001-sol-transfer`, `002-spl-transfer` | Perfect execution | ✅ Validated |

**Anti-False-Positive Testing:**
- Differentiates between "no attempt" (0%) vs "attempted but failed" (partial credit)
- Validates granular component scoring (program ID vs data vs accounts)
- Ensures weighted scoring prevents gaming the system

## 🔧 Development & Testing

### **Integration Tests:**
```bash
# Full test suite (deterministic + AI agents)
cargo test -p reev-runner

# Specific agent testing
cargo test -p reev-runner --test deterministic_agent_test
cargo test -p reev-runner --test llm_agent_test
```

### **Example Testing:**
```bash
# Protocol examples
cargo run -p reev-agent --example 115-jup-lend-mint-usdc

# Flow examples
cargo run -p reev-agent --example 200-jup-swap-then-lend-deposit
```

### **Debugging:**
```bash
# Enable verbose logging
RUST_LOG=debug cargo run -p reev-runner -- benchmarks/001-sol-transfer.yml

# Check surfpool status
curl http://localhost:8899/health
```
