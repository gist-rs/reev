# OTEL.md: Type-Safe Response Architecture Observability

## 📋 Current Status: Phase 6 - Implementation Ready

This document outlines the OpenTelemetry integration for the type-safe response architecture, providing real-time insights into agent behavior, API compliance, and performance metrics. The integration follows the rig-core example pattern and is ready for implementation in Phase 6.

**Current State**: The framework uses `tracing` for structured logging and has basic health monitoring. OpenTelemetry integration is planned for Phase 6 development with type-safe response architecture.

---

## OpenTelemetry Integration

**Goal:** Instrument the framework to emit standardized OpenTelemetry (OTel) traces for advanced performance analysis and professional observability integration.

**Rationale:**
-   **Type-Aware Observability**: OTel provides real-time visibility into type-safe response handling, API compliance validation, and performance bottlenecks.
-   **Compliance Monitoring**: Track API vs LLM-generated responses in real-time, ensuring agents follow API-first principles.
-   **Performance Optimization**: Identify bottlenecks in response parsing, validation, and instruction extraction through detailed metrics.
-   **Distributed Tracing**: Track multi-step flows with proper span relationships and correlation IDs for complex DeFi operations.

**Current Observability Stack:**
- **Structured Logging**: Comprehensive `tracing` integration with log levels (debug, info, warn, error)
- **Health Monitoring**: Surfpool health checks and service status monitoring
- **Performance Metrics**: Benchmark execution times, scoring results, and agent performance data
- **Database Persistence**: SQLite database with detailed execution traces and results

**Planned OpenTelemetry Integration:**
| Type-Safe Response Concept | OpenTelemetry Concept | OTEL Attributes & Events |
|---------------------------|---------------------|------------------------|
| `TypedAgent<T>` Request | **Root Span** (`agent.request`) | `response_type`, `operation`, `request_id` |
| Response Validation | **Span** (`agent.validate`) | `validation_result`, `api_source`, `instruction_count` |
| API Compliance Check | **Event** (`agent.compliance`) | `api_compliant`, `validation_errors` |
| Instruction Extraction | **Span** (`agent.extract`) | `extraction_method`, `instruction_count` |
| Type-Specific Metrics | **Metrics** (`agent.metrics`) | Counter, Histogram, Gauge per type |

## 🏗️ **Type-Safe OTEL Architecture**

### **Component 1: Type-Aware Tracing Infrastructure**
```rust
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::trace::SdkTracerProvider;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

// 🎯 Type-safe instrumented agent execution
#[tracing::instrument(
    name = "agent_execution",
    fields(
        response_type = std::any::type_name::<T>(),
        operation = T::operation_type(),
        instruction_count = tracing::field::Empty,
        api_source = tracing::field::Empty,
    )
)]
pub async fn execute_typed_request<T: AgentResponse>(request: T::Request) -> Result<T, AgentError> {
    let start = std::time::Instant::now();
    
    // 🎯 OpenTelemetry tracks exact types
    tracing::info!(
        agent_type = std::any::type_name::<T>(),
        request_id = uuid::Uuid::new_v4().to_string(),
        operation = T::operation_type(),
        user_pubkey = request.user_pubkey(),
    );
    
    // Execute with automatic tracing
    let response = typed_agent.call_typed(request).await?;
    
    // 🎯 Record compliance metrics
    let execution_time = start.elapsed();
    tracing::info!(
        execution_time_ms = execution_time.as_millis(),
        instruction_count = response.to_execution_result().transactions.len(),
        validation_result = response.validate_instructions().is_ok(),
        api_source = detect_api_source(&response),
    );
    
    Ok(response)
}
```

### **Component 2: Structured Metrics Collection**
```rust
use opentelemetry::metrics::{Counter, Histogram, Gauge};

// 🎯 Type-aware metrics collector for API compliance tracking
pub struct TypeMetricsCollector {
    request_counter: Counter<u64>,
    execution_histogram: Histogram<f64>,
    validation_gauge: Gauge<u64>,
    api_source_counter: Counter<u64>,
}

impl TypeMetricsCollector {
    pub fn new() -> Self {
        let meter = opentelemetry::global::meter("reev_agent_metrics");
        
        Self {
            request_counter: meter.u64_counter("agent_requests_total")
                .with_description("Total number of agent requests"),
            execution_histogram: meter.f64_histogram("agent_execution_time")
                .with_description("Agent execution time in milliseconds"),
            validation_gauge: meter.u64_gauge("agent_validation_status")
                .with_description("Agent response validation status (1=valid, 0=invalid)"),
            api_source_counter: meter.u64_counter("api_source_counts")
                .with_description("Counts of API vs LLM generated responses"),
        }
    }
    
    pub fn record_request<T: AgentResponse>(&self, response: &T) {
        self.request_counter.add(
            1,
            [
                KeyValue::new("response_type", T::operation_type()),
                KeyValue::new("operation_id", uuid::Uuid::new_v4().to_string()),
            ],
        );
        
        self.execution_histogram.record(
            response.execution_time_ms() as f64,
            [
                KeyValue::new("response_type", T::operation_type()),
                KeyValue::new("instruction_count", response.instruction_count() as u64),
            ],
        );
        
        self.validation_gauge.set(
            if response.validate_instructions().is_ok() { 1 } else { 0 },
            [
                KeyValue::new("response_type", T::operation_type()),
            ],
        );
        
        self.api_source_counter.add(
            1,
            [
                KeyValue::new("api_source", response.detect_api_source()),
                KeyValue::new("response_type", T::operation_type()),
            ],
        );
    }
}
```

### **Component 3: Custom Span Attributes for Compliance**
```rust
use opentelemetry::trace::{Span, SpanKind, Status};

// 🎯 Rich span attributes for compliance tracking
impl<T: AgentResponse> AgentResponse for T {
    fn create_span(&self, operation: &str) -> Span {
        let span = tracing::span!(Level::INFO, operation, kind = SpanKind::Client);
        
        span.set_attribute("response_type", T::operation_type());
        span.set_attribute("instruction_count", self.instruction_count() as u64);
        span.set_attribute("api_compliant", self.validate_instructions().is_ok());
        span.set_attribute("execution_time_ms", self.execution_time_ms());
        span.set_attribute("api_source", self.detect_api_source());
        
        // Add protocol-specific attributes
        if let Some(jupiter_data) = self.jupiter_metadata() {
            span.set_attribute("jupiter_operation", jupiter_data.operation_type);
            span.set_attribute("jupiter_tokens", jupiter_data.token_mints);
        }
        
        span
    }
}
```

### **Component 4: Distributed Tracing for Multi-Step Flows**
```rust
// 🎯 Distributed tracing for Jupiter multi-step flows
#[tracing::instrument(
    name = "jupiter_swap_flow",
    skip_if = true
)]
pub async fn execute_jupiter_swap_flow<T: AgentResponse>(
    agent: &TypedAgent<T>,
    request: JupiterSwapRequest,
) -> Result<T, AgentError> {
    // Step 1: Get quote with span tracking
    let quote_span = tracing::info_span!("jupiter_get_quote").entered();
    let quote = agent.get_quote(&request).instrument(quote_span).await?;
    quote_span.exit();
    
    // Step 2: Get instructions with span tracking
    let instructions_span = tracing::info_span!("jupiter_get_instructions").entered();
    let instructions = agent.get_instructions(&quote).instrument(instructions_span).await?;
    instructions_span.exit();
    
    // Step 3: Execute transaction with span tracking
    let execution_span = tracing::info_span!("jupiter_execute_transaction").entered();
    let response = agent.execute_transaction(&instructions).instrument(execution_span).await?;
    execution_span.exit();
    
    // Step 4: Validate result with span tracking
    let validation_span = tracing::info_span!("jupiter_validate_response").entered();
    response.validate_instructions().instrument(validation_span).await?;
    validation_span.exit();
    
    Ok(response)
}
```