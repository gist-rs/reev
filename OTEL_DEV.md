# OTEL Development & Testing Guide

## 🚀 Quick Start

### **ALWAYS Run Server in Background First!**
```bash
# Start API server in background (REQUIRED for all testing)
nohup bash -c 'REEV_TRACE_FILE=traces_server.log RUST_LOG=info cargo run -p reev-api' > server_output.log 2>&1 &

# Wait for server to start
sleep 20

# Verify server is running
curl -X GET http://localhost:3001/api/health
```

⚠️ **WARNING**: If you don't run server in background, all curl commands will hang forever waiting for the server!

---

## 🛠️ Implementation Guide

### **Pattern for Adding Enhanced OTEL Logging**

For any tool that needs enhanced logging, follow this exact pattern:

#### **1. Add Required Imports**
```rust
use std::time::Instant;
use reev_flow::{log_tool_call, log_tool_completion};
```

#### **2. Ensure Args Struct Implements Serialize**
```rust
#[derive(Deserialize, Debug, Serialize)]  // Add Serialize!
pub struct YourToolArgs {
    pub field1: String,
    pub field2: Option<u64>,
}
```

#### **3. Add Logging to call() Function**
```rust
async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
    let start_time = Instant::now();
    
    // 🎯 Add enhanced logging at START
    log_tool_call!(Self::NAME, &args);
    
    info!("[{}] Starting tool execution with OpenTelemetry tracing", Self::NAME);
    
    // ... existing tool logic ...
    
    match result {
        Ok(output) => {
            let execution_time = start_time.elapsed().as_millis() as u64;
            
            // 🎯 Add enhanced logging at SUCCESS
            log_tool_completion!(Self::NAME, execution_time, &output, true);
            
            info!("[{}] Tool execution completed in {}ms", Self::NAME, execution_time);
            Ok(output)
        }
        Err(e) => {
            let execution_time = start_time.elapsed().as_millis() as u64;
            let error_data = json!({"error": e.to_string()});
            
            // 🎯 Add enhanced logging at ERROR
            log_tool_completion!(Self::NAME, execution_time, &error_data, false);
            
            error!("[{}] Tool execution failed in {}ms: {}", Self::NAME, execution_time, e);
            Err(e)
        }
    }
}
```

---

## 🧪 Testing Guide

### **Test Categories**

#### **1. Deterministic Agent Testing**
```bash
# Ensure server is running in background
curl -X GET http://localhost:3001/api/health

# Test deterministic agent via API
curl -X POST http://localhost:3001/api/v1/benchmarks/001-sol-transfer/run \
  -H "Content-Type: application/json" \
  -d '{"agent": "deterministic"}'

# Check enhanced OTEL logs
find logs/sessions/ -name "*deterministic*" -exec cat {} \;

# Verify session file created
find logs/sessions/ -name "session_*$(curl -s http://localhost:3001/api/v1/benchmarks/001-sol-transfer | jq -r '.latest_execution_id')*" -exec cat {} \;
```

#### **2. Local Agent Tool Testing**
```bash
# Test with enhanced logging enabled
curl -X POST http://localhost:3001/api/v1/benchmarks/100-jup-swap-sol-usdc/run \
  -H "Content-Type: application/json" \
  -d '{"agent": "local"}'

# Wait for completion
sleep 30

# Check enhanced OTEL logs for specific tool
grep -i "jupiter_swap" logs/sessions/enhanced_otel_*.jsonl

# Verify tool call structure
jq '.tool_name' logs/sessions/enhanced_otel_*.jsonl | sort | uniq
```

#### **3. Discovery Tools Testing**
```bash
# Test account balance tool
curl -X POST http://localhost:3001/api/v1/benchmarks/001-sol-transfer/run \
  -H "Content-Type: application/json" \
  -d '{"agent": "local"}'

# Check for balance tool logs
grep -i "get_account_balance" logs/sessions/enhanced_otel_*.jsonl

# Verify tool parameters were logged
jq 'select(.tool_name == "get_account_balance") | .tool_input' logs/sessions/enhanced_otel_*.jsonl
```

---

## 🔍 Debugging Guide

### **Common Issues & Solutions**

#### **Issue #1: Server Not Running**
**Symptoms**: `curl` commands hang forever
**Solution**: 
```bash
# Always check if server is running first
curl -X GET http://localhost:3001/api/health || echo "Server not running!"

# Start server in background
nohup bash -c 'REEV_TRACE_FILE=traces_server.log RUST_LOG=info cargo run -p reev-api' > server_output.log 2>&1 &
```

#### **Issue #2: No Enhanced OTEL Logs Generated**
**Symptoms**: Session files created but `events: []` empty
**Debugging**:
```bash
# Check if enhanced logging is enabled
grep -i "enhanced_otel" server_output.log

# Check for logger initialization messages
grep -i "EnhancedOtelLogger" server_output.log

# Verify log file creation
ls -la logs/sessions/enhanced_otel_*.jsonl

# Check log file contents
tail -20 logs/sessions/enhanced_otel_*.jsonl
```

#### **Issue #3: Enhanced Logging Compile Errors**
**Symptoms**: `trait bound Serialize is not satisfied` errors
**Solution**:
```rust
// Add Serialize to Args struct
#[derive(Deserialize, Debug, Serialize)]  // Missing Serialize!
pub struct ToolArgs {
    pub field: String,
}
```

#### **Issue #4: Tool Not Detected in Trace Extraction**
**Symptoms**: Tool runs but doesn't appear in Mermaid diagrams
**Solution**: Add detection pattern in `reev-lib/src/otel_extraction/mod.rs`:
```rust
fn extract_tool_name_from_span(span: &OtelSpanData) -> Option<String> {
    // Add your tool pattern here
    if span_name.contains("your_tool_name") {
        return Some("your_tool_name".to_string());
    }
    // ... existing patterns ...
}
```

---

## 📊 Testing Checklist

### **Pre-Test Checklist**
- [ ] API server running in background (`curl localhost:3001/api/health`)
- [ ] Enhanced logging enabled (`REEV_ENHANCED_OTEL=1`)
- [ ] Trace file configured (`REEV_TRACE_FILE=traces.log`)
- [ ] Rust log level set (`RUST_LOG=info`)

### **Post-Test Verification**
- [ ] Session file created (`logs/sessions/enhanced_otel_*.jsonl`)
- [ ] Tool events generated (non-empty `events` array)
- [ ] Tool parameters logged correctly
- [ ] Execution time tracked
- [ ] Success/error status recorded
- [ ] Tool appears in Mermaid diagrams

### **API Endpoints for Testing**
```bash
# Health check
curl -X GET http://localhost:3001/api/health

# List benchmarks
curl -X GET http://localhost:3001/api/v1/benchmarks

# Run benchmark
curl -X POST http://localhost:3001/api/v1/benchmarks/{id}/run \
  -H "Content-Type: application/json" \
  -d '{"agent": "local"}'

# Get execution trace
curl -X GET http://localhost:3001/api/v1/execution-logs/{id}?execution_id={exec_id}

# Get flow logs (enhanced OTEL)
curl -X GET http://localhost:3001/api/v1/flow-logs/{id}

# Get all flows (Mermaid diagrams)
curl -X GET http://localhost:3001/api/v1/flows
```

---

## 🎯 Success Metrics

### **OTEL Coverage Targets**
- **Deterministic Agents**: 100% (3/3 implemented)
- **Discovery Tools**: 100% (3/3 implemented) ✅
- **Flow Tools**: 0% (0/1 implemented) ❌
- **Jupiter Lend/Earn**: 0% (0/4 implemented) ❌
- **SPL Tools**: 0% (0/1 implemented) ❌
- **Core Tools**: 100% (3/3 implemented) ✅

### **Performance Targets**
- **Enhanced Logging Overhead**: <1ms per tool call
- **Session File Creation**: <100ms after tool execution
- **Trace Extraction**: Complete within 50ms
- **Mermaid Generation**: <200ms per flow

---

## 🚨 Important Reminders

### **ALWAYS REMEMBER**
1. **Server First**: Always start API server in background before any testing
2. **Background Jobs**: Use `nohup` and `&` to prevent hanging
3. **Wait Times**: Allow 20-30 seconds for server startup and execution
4. **Log Checking**: Always verify logs in `logs/sessions/` directory
5. **Enhanced Logging**: Must be enabled (`REEV_ENHANCED_OTEL=1`)

### **Environment Variables**
```bash
export REEV_ENHANCED_OTEL=1        # Enable enhanced logging (default)
export REEV_TRACE_FILE=traces.log   # Trace output file
export RUST_LOG=info               # Rust log level
export RUST_LOG=debug              # For detailed debugging
```

### **File Locations**
- **Server Logs**: `server_output.log`
- **Enhanced OTEL**: `logs/sessions/enhanced_otel_*.jsonl`
- **Session Files**: `logs/sessions/session_*.json`
- **Trace Files**: `traces_server.log`

---

## 🔄 Development Workflow

### **1. Implementation**
1. Add enhanced logging pattern to target tool
2. Ensure Args struct has `Serialize` derive
3. Test compilation (`cargo check -p reev-tools`)
4. Commit changes

### **2. Testing**
1. Start API server in background
2. Run benchmark via API
3. Verify enhanced OTEL logs generated
4. Check tool appears in flow diagrams

### **3. Debugging**
1. Check server logs for initialization messages
2. Verify enhanced logging macros are called
3. Confirm trace extraction patterns
4. Validate Mermaid diagram generation

### **4. Verification**
1. Test multiple benchmarks
2. Verify complete tool coverage
3. Check performance impact
4. Update documentation

This guide ensures consistent OTEL implementation and prevents common testing pitfalls.