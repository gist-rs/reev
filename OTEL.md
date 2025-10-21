# OTEL.md: OpenTelemetry Integration for Tool Call Extraction

## 📋 Current Status: ✅ IMPLEMENTATION COMPLETE

This document outlines the completed OpenTelemetry integration for tool call extraction and Mermaid diagram generation. The system now automatically extracts tool calls from rig's OpenTelemetry traces and converts them to session format for flow visualization.

**Current State**: ✅ **Full OpenTelemetry integration implemented** - Tool calls are automatically captured from rig's spans and converted to session format for Mermaid diagrams.

---

## ✅ Completed OpenTelemetry Integration

**Goal:** ✅ **ACHIEVED** - Instrument the framework to extract tool calls from rig's OpenTelemetry traces for Mermaid diagram generation.

**Implementation:**
-   **Tool Call Extraction**: Automatic extraction of tool calls from rig's OpenTelemetry spans
-   **Session Format Conversion**: Convert traces to FLOW.md session format for Mermaid diagrams
-   **Real-time Tracking**: Tool calls captured during agent execution without manual interference
-   **Clean Architecture**: No manual tool tracking - relies on rig's built-in OpenTelemetry integration

**Key Components Implemented:**
-   `reev-lib/src/otel_extraction/mod.rs` - Trace extraction layer
-   `extract_current_otel_trace()` - Extract current trace from global tracer
-   `parse_otel_trace_to_tools()` - Convert spans to tool call format
-   `convert_to_session_format()` - Convert to Mermaid session format

**✅ Completed OpenTelemetry Architecture:**
- **Structured Logging**: Comprehensive `tracing` integration with OpenTelemetry backend
- **Tool Call Extraction**: Automatic extraction from rig's OpenTelemetry spans
- **Session Format**: Standardized format for Mermaid diagram generation
- **Clean Integration**: No manual tracking - relies on rig framework

**✅ Implemented OpenTelemetry Integration:**
| Agent Tool Call | OpenTelemetry Concept | Session Format Output |
|-----------------|---------------------|----------------------|
| `sol_transfer` execution | **Span** (`sol_transfer`) | `{tool_name: "sol_transfer", params: {...}, result: {...}}` |
| `jupiter_swap` execution | **Span** (`jupiter_swap`) | `{tool_name: "jupiter_swap", params: {...}, result: {...}}` |
| Tool result | **Span Attributes** | `{status: "success|error", execution_time_ms: 100}` |
| Error handling | **Span Status** | `{status: "error", error_message: "..."}` |
| Session flow | **Trace Context** | `{session_id: "...", tools: [...]}` |

## 🏗️ **✅ Implemented OpenTelemetry Architecture**

### **Component 1: OpenTelemetry Trace Extraction Layer**
```rust
// ✅ COMPLETED: Trace extraction from rig's OpenTelemetry
use reev_lib::otel_extraction::{
    extract_current_otel_trace, 
    parse_otel_trace_to_tools,
    convert_to_session_format
};

// 🎯 Extract tool calls from current OpenTelemetry trace context
pub fn extract_tool_calls_for_mermaid() -> Vec<SessionToolData> {
    if let Some(trace) = extract_current_otel_trace() {
        let tool_calls = parse_otel_trace_to_tools(trace);
        convert_to_session_format(tool_calls)
    } else {
        vec![]
    }
}

// 🎯 Agent implementation with OpenTelemetry extraction
impl GlmAgent {
    pub async fn run_with_otel_extraction(&self, payload: LlmRequest) -> Result<String> {
        // Execute agent with rig's automatic OpenTelemetry tracing
        let response = self.agent.prompt(&enhanced_request).await?;
        
        // Extract tool calls from OpenTelemetry traces
        let tool_calls = extract_tool_calls_for_mermaid();
        info!("Extracted {} tool calls from OpenTelemetry", tool_calls.len());
        
        // Return response with tool call data for Mermaid diagrams
        Ok(format_response_with_tools(response, tool_calls))
    }
}
```

### **Component 2: Session Format for Mermaid Diagrams**
```rust
// ✅ COMPLETED: Session format matching FLOW.md specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionToolData {
    pub tool_name: String,           // "sol_transfer", "jupiter_swap"
    pub start_time: SystemTime,      // Tool execution start
    pub end_time: SystemTime,        // Tool execution end
    pub params: serde_json::Value,   // Tool parameters
    pub result: serde_json::Value,   // Tool result data
    pub status: String,              // "success", "error"
}

// ✅ COMPLETED: Conversion from OpenTelemetry to session format
impl From<OtelSpanData> for SessionToolData {
    fn from(span: OtelSpanData) -> Self {
        SessionToolData {
            tool_name: extract_tool_name_from_span(&span),
            start_time: span.start_time,
            end_time: span.end_time.unwrap_or(span.start_time),
            params: extract_params_from_span(&span),
            result: extract_result_from_span(&span),
            status: span.status,
        }
    }
}

// ✅ COMPLETED: Tool call extraction for Mermaid generation
pub fn generate_mermaid_from_otel(session_id: &str) -> Option<String> {
    let tools = extract_tool_calls_for_mermaid();
    if tools.is_empty() {
        return None;
    }
    
    let mut mermaid = String::from("stateDiagram-v2\n");
    mermaid.push_str(&format!("    [*] --> {}\n", tools[0].tool_name));
    
    for i in 0..tools.len() - 1 {
        mermaid.push_str(&format!(
            "    {} --> {}\n", 
            tools[i].tool_name, 
            tools[i + 1].tool_name
        ));
    }
    
    Some(mermaid)
}
```

### **Component 3: Environment Configuration**
```rust
// ✅ COMPLETED: Environment variables for OpenTelemetry
pub fn init_otel_for_tool_extraction() -> Result<(), Box<dyn std::error::Error>> {
        // OpenTelemetry is always enabled
        // Initialize flow tracing with stdout exporter
        reev_flow::init_flow_tracing()?;
    
        // Initialize OpenTelemetry extraction
        reev_lib::otel_extraction::init_otel_extraction()?;
    
        info!("OpenTelemetry enabled for tool call extraction");
        info!("Tool calls will be automatically captured from rig's spans");
    
    Ok(())
}

// ✅ COMPLETED: Configuration for trace file output
pub struct OtelConfig {
    pub trace_file: String,
}

impl OtelConfig {
    pub fn from_env() -> Self {
        Self {
            trace_file: std::env::var("REEV_TRACE_FILE")
                .unwrap_or_else(|_| "traces.log".to_string()),
        }
    }
}
```

### **Component 4: Integration with All Agents**
```rust
// ✅ COMPLETED: OpenTelemetry extraction integrated with all agents

// GLM Agent with OpenTelemetry extraction
impl GlmAgent {
    pub async fn run(&self, payload: LlmRequest) -> Result<String> {
        // Execute with rig's automatic OpenTelemetry tracing
        let response = self.agent.prompt(&enhanced_request).await?;
        
        // 🌊 Extract tool calls from OpenTelemetry traces
        info!("[GlmAgent] Extracting tool calls from OpenTelemetry traces");
        
        if let Some(otel_trace) = extract_current_otel_trace() {
            let tool_calls = parse_otel_trace_to_tools(otel_trace);
            let session_tools = convert_to_session_format(tool_calls);
            info!("[GlmAgent] Extracted {} tools for Mermaid diagram", session_tools.len());
        }
        
        Ok(response)
    }
}

// OpenAI Agent with OpenTelemetry extraction  
impl OpenAIAgent {
    pub async fn run(&self, payload: LlmRequest) -> Result<String> {
        // Execute with rig's automatic OpenTelemetry tracing
        let response = self.agent.prompt(&enhanced_request).await?;
        
        // 🌊 Extract tool calls from OpenTelemetry traces
        let tool_calls = if let Some(otel_trace) = extract_current_otel_trace() {
            reev_lib::otel_extraction::parse_otel_trace_to_tools(otel_trace)
        } else {
            vec![]
        };
        
        info!("[OpenAIAgent] Tool calls captured via OpenTelemetry: {}", tool_calls.len());
        Ok(response)
    }
}
```