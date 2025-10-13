# 🪸 Reev Implementation Constraints & FYI

## 🎯 **Server Management Architecture**

### 📋 **Manual vs Automatic Server Management**

**User-Managed Services:**
- **Web Frontend**: User runs manually via `npm run dev` in `/web/` directory
  - Port: 5173 (default Vite dev server)
  - Command: `cd web && npm run dev`
  - Status: User responsibility

**Programmatically Managed Services:**
- **API Server**: Must be started automatically via code
  - Port: 3000
  - Binary: `./target/debug/reev-api`
  - Management: Code should handle startup/shutdown
- **Agent Server**: Must be started automatically via code
  - Port: 9090
  - Binary: `./target/debug/reev-agent`
  - Management: Code should handle startup/shutdown
- **Benchmark Runner**: Should orchestrate all dependencies
  - Primary orchestrator for all backend services
  - Handles dependency startup sequence
  - Manages service health checks

### 🔄 **Service Startup Sequence**

1. **User starts**: `cd web && npm run dev` (manual)
2. **Backend services start**: Via benchmark runner or API server (automatic)
   - Database initialization
   - Agent server startup (port 9090)
   - API server startup (port 3000)
   - Health checks for all services

## 🤖 **Agent Configuration Requirements**

### 📋 **Supported Agent Types**
- **Deterministic**: Built-in deterministic agent
- **Local (Qwen3)**: Local LLM with custom configuration
- **GLM 4.6**: Zhipu AI model with API configuration
- **Gemini 2.5 Pro**: Google AI model with API configuration

### ⚙️ **Configuration Interface Requirements**
- **API URL Input**: Text field for custom API endpoints
- **API Key Input**: Password field for secure key entry
- **Connection Testing**: Test button to validate configuration
- **Save/Reset**: Configuration persistence and reset options
- **Validation**: Real-time validation of API URL format and key requirements

### 🔐 **Security Considerations**
- **API Key Storage**: Encrypt stored API keys
- **Transmission**: Use HTTPS for API communication
- **Validation**: Server-side validation of API credentials
- **Access Control**: Restrict API key access to authorized sessions

## 📊 **Database Management**

### 🗄️ **Database Schema**
```sql
-- Core tables already implemented
results              -- Legacy benchmark results
flow_logs           -- Detailed flow execution logs
agent_performance   -- Agent performance summaries
```

### 🔄 **Database Operations**
- **Auto-Creation**: Schema should be created automatically on API server startup
- **Migration Support**: Handle schema changes gracefully
- **Clear Option**: Ability to clear database for fresh testing
- **Sample Data**: Preserve current sample data for UI testing

### 📍 **Database Location**
- **Path**: `db/reev_results.db`
- **Format**: SQLite
- **Backup**: Consider automatic backup for production use

## 🚀 **Real-time Execution Requirements**

### 📡 **Real-time Update Mechanisms**

**Options:**
1. **WebSocket Connection**: Preferred for real-time updates
   - Endpoint: `ws://localhost:3000/ws/benchmarks/{id}`
   - Events: status updates, log entries, completion
   
2. **HTTP Polling**: Fallback option
   - Endpoint: `GET /api/v1/benchmarks/{id}/status`
   - Interval: 1-2 seconds during execution

### 📊 **Execution Status Tracking**
- **Pending**: Benchmark queued but not started
- **Running**: Currently executing (show progress)
- **Completed**: Finished successfully (show results)
- **Failed**: Execution failed (show error details)

### 📝 **Log Streaming Requirements**
- **Execution Trace**: Real-time trace output
- **Transaction Logs**: Detailed transaction information
- **Error Logs**: Error messages and stack traces
- **Performance Metrics**: Execution time, memory usage

## 🎮 **User Interface Requirements**

### 📱 **Layout Structure**
```
┌─────────────────────────────────────────────────────────┐
│ Agent Tabs: [Deterministic] [Local] [GLM 4.6] [Gemini] │
├─────────────────────┬───────────────────────────────────┤
│ Benchmark List      │ Execution Trace / Transaction Log │
│ - [ ] 001-sol-transfer │ Real-time execution details    │
│ - [✔] 002-spl-transfer │ Auto-scrolling log viewer      │
│ - […] 003-jupiter-swap │ Syntax highlighted output      │
│ [Run Selected]      │                                   │
│ [Run All]           │                                   │
├─────────────────────┴───────────────────────────────────┤
│ 16x16 Overview Boxes (current view)                      │
└─────────────────────────────────────────────────────────┘
```

### 🎨 **Visual Requirements**
- **Color Coding**: Green (100%), Yellow (partial), Red (failed), Gray (pending)
- **Status Indicators**: [ ] Pending, […] Running, [✔] Success, [✗] Failed
- **Progress Bars**: Visual progress during execution
- **Auto-scroll**: Logs should auto-scroll to latest content

### ⌨️ **Keyboard Shortcuts**
- `Tab`: Switch between agents
- `↑↓`: Navigate benchmark list
- `Enter`: Run selected benchmark
- `Ctrl+A`: Run all benchmarks
- `Ctrl+S`: Stop execution
- `Ctrl+C`: Open configuration

## 🔧 **Technical Constraints**

### 🏗️ **Architecture Constraints**
- **Frontend**: Preact + TypeScript + Tailwind CSS (fixed)
- **Backend**: Rust with Axum 0.8.4 (fixed)
- **Database**: SQLite (fixed)
- **Communication**: REST API + WebSocket (preferred)

### 📦 **Dependency Management**
- **Frontend Dependencies**: Managed via npm in `/web/`
- **Backend Dependencies**: Managed via Cargo workspace
- **Service Orchestration**: Handle via Rust code, not external tools

### 🚀 **Performance Requirements**
- **API Response Time**: < 200ms for non-execution endpoints
- **Real-time Updates**: < 1 second latency for execution status
- **Concurrent Users**: Support multiple simultaneous executions
- **Memory Usage**: Efficient handling of large log outputs

## 📋 **Development Workflow**

### 🔄 **Development Constraints**
- **No External Process Managers**: Use Rust code for service management
- **Single Binary Deployment**: API server should be deployable as single binary
- **Environment Variables**: Support for configuration via environment
- **Logging**: Structured logging for debugging and monitoring

### 🧪 **Testing Requirements**
- **Integration Tests**: Test full execution flow
- **API Tests**: Test all endpoints with various inputs
- **Frontend Tests**: Component testing for UI interactions
- **Performance Tests**: Load testing for concurrent executions

## 🚨 **Error Handling**

### 📡 **Connection Errors**
- **API Unavailable**: Show clear error messages and retry options
- **WebSocket Disconnect**: Automatic reconnection with fallback to polling
- **Configuration Errors**: Validation and helpful error messages

### 🔧 **Execution Errors**
- **Benchmark Failures**: Detailed error information and logs
- **Agent Errors**: Clear indication of configuration or API issues
- **System Errors**: Graceful degradation and error reporting

---

*This document serves as a reference for implementation constraints and requirements. All development should follow these guidelines to ensure consistency and proper system integration.*