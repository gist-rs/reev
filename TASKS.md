# 🌐 Reev Web Interface Development Tasks

## 🎯 Executive Summary

This document outlines the comprehensive plan to build a modern web interface for the Reev framework that will visualize benchmark results from the database and provide an intuitive dashboard for monitoring agent performance across different models.

## 📊 Current State Analysis

### ✅ **Existing Infrastructure**
- **Database**: SQLite (`reev_results.db`) with benchmark results table ✅
- **Flow Logging**: YML files stored in `logs/flows/` directory ✅  
- **API Foundation**: Rust backend with Turso/SQLite integration ✅
- **Data Model**: Structured results with scores, timestamps, agent types ✅
- **Agent Support**: Deterministic, Local, GLM 4.6, and Gemini agents ✅

### 🔄 **Current Flow**
1. Benchmarks run → Results stored in `reev_results.db` ✅
2. Flow logs saved as YML files in `logs/flows/` ✅
3. TUI provides real-time monitoring ✅
4. CLI offers programmatic access ✅
5. **NEW**: Web API server serving on port 3000 ✅
6. **NEW**: Frontend dashboard running on port 5173 ✅

## 🎉 **MAJOR BREAKTHROUGH - API SERVER FIXED**
**✅ Axum 0.8 Compatibility Issue RESOLVED**
- ✅ Fixed `AgentPerformanceSummary` serialization by adding `Serialize` derive
- ✅ Simplified router architecture to avoid state trait conflicts
- ✅ API server now compiles and runs successfully on port 3000
- ✅ All endpoints working: `/api/v1/health`, `/api/v1/agents`, `/api/v1/benchmarks`, `/api/v1/agent-performance`
- ✅ CORS enabled for frontend integration
- ✅ Database integration fully functional

---

## 🚀 Phase 1: Database Integration for Flow Logs

### 🎯 **Objective**: Store flow logs in database instead of YML files

### 📋 **Tasks**

#### 1.1 Database Schema Enhancement
```sql
-- Add flow logs table
CREATE TABLE IF NOT EXISTS flow_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL,
    benchmark_id TEXT NOT NULL,
    agent_type TEXT NOT NULL,
    start_time TEXT NOT NULL,
    end_time TEXT,
    final_result TEXT, -- JSON
    flow_data TEXT NOT NULL, -- Complete FlowLog as JSON
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- Add agent performance table for quick lookups
CREATE TABLE IF NOT EXISTS agent_performance (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    benchmark_id TEXT NOT NULL,
    agent_type TEXT NOT NULL,
    score REAL NOT NULL,
    final_status TEXT NOT NULL,
    execution_time_ms INTEGER,
    timestamp TEXT NOT NULL,
    flow_log_id INTEGER,
    FOREIGN KEY (flow_log_id) REFERENCES flow_logs (id)
);

-- Add indexes for performance
CREATE INDEX IF NOT EXISTS idx_flow_logs_benchmark_agent ON flow_logs(benchmark_id, agent_type);
CREATE INDEX IF NOT EXISTS idx_agent_performance_score ON agent_performance(score);
CREATE INDEX IF NOT EXISTS idx_agent_performance_timestamp ON agent_performance(timestamp);
```

#### 1.2 Database Migration
- [ ] Create migration system for schema updates
- [ ] Add `FlowLog` model serialization to database
- [ ] Update `Db::new()` to include new tables
- [ ] Add `insert_flow_log()` method to database
- [ ] Add `get_flow_logs()` and `get_agent_performance()` query methods

#### 1.3 FlowLogger Integration
- [ ] Modify `FlowLogger::complete()` to save to database instead of YML
- [ ] Keep YML export as optional feature for debugging
- [ ] Add database error handling and retry logic
- [ ] Update flow logging configuration options

#### 1.4 Testing
- [ ] Unit tests for database operations
- [ ] Integration tests with flow logging
- [ ] Performance tests for large flow datasets
- [ ] Migration rollback tests

---

## 🚀 Phase 2: REST API Development

### 🎯 **Objective**: Expose benchmark and flow data through REST API

### 📋 **Tasks**

#### 2.1 API Server Structure
```rust
// crates/reev-api/src/main.rs
#[derive(Clone)]
pub struct ApiState {
    db: Arc<Mutex<Db>>,
}

// API Endpoints:
GET    /api/v1/benchmarks              // List all benchmarks
GET    /api/v1/benchmarks/:id          // Get specific benchmark details
GET    /api/v1/agents                  // List available agents
GET    /api/v1/results                 // Get all results with filtering
GET    /api/v1/results/:benchmark_id   // Get results for specific benchmark
GET    /api/v1/flow-logs/:session_id   // Get detailed flow log
GET    /api/v1/agent-performance        // Get performance summary by agent
GET    /api/v1/health                  // Health check
```

#### 2.2 Data Models
```rust
// Response structures
#[derive(Serialize)]
pub struct BenchmarkResult {
    id: String,
    benchmark_id: String,
    agent_type: String,
    score: f64,
    final_status: String,
    execution_time_ms: u64,
    timestamp: String,
    color_class: String, // "green", "yellow", "red" based on score
}

#[derive(Serialize)]
pub struct AgentPerformanceSummary {
    agent_type: String,
    total_benchmarks: u32,
    average_score: f64,
    success_rate: f64,
    best_benchmarks: Vec<String>,
    worst_benchmarks: Vec<String>,
    results: Vec<BenchmarkResult>,
}

#[derive(Serialize)]
pub struct FlowLogResponse {
    session_id: String,
    benchmark_id: String,
    agent_type: String,
    events: Vec<FlowEvent>,
    final_result: Option<ExecutionResult>,
    performance_metrics: PerformanceMetrics,
}
```

#### 2.3 API Implementation
- [ ] Create `reev-api` crate with Axum web framework
- [ ] Implement CORS support for frontend
- [ ] Add query parameter filtering (agent, date range, score range)
- [ ] Implement pagination for large result sets
- [ ] Add response caching for performance
- [ ] Include OpenAPI/Swagger documentation

#### 2.4 Error Handling & Validation
- [ ] Structured error responses with HTTP status codes
- [ ] Input validation for all endpoints
- [ ] Rate limiting for API protection
- [ ] Request/response logging

#### 2.5 Testing
- [ ] Unit tests for all endpoints
- [ ] Integration tests with real database
- [ ] Load testing for concurrent requests
- [ ] API contract tests

---

## 🚀 Phase 3: Web Frontend Development

### 🎯 **Objective**: Build responsive web interface using Preact

### 📋 **Tasks**

#### 3.1 Project Setup
```bash
# crates/reev-web/
npm create preact@latest reev-web -- --template typescript
cd reev-web
npm install
```

#### 3.2 Component Architecture
```
src/
├── components/
│   ├── BenchmarkGrid.tsx      # Main grid display
│   ├── BenchmarkBox.tsx       # Individual 16x16 box
│   ├── AgentSection.tsx       # Agent grouping (desktop/mobile)
│   ├── FilterPanel.tsx        # Score/date/agent filters
│   ├── FlowViewer.tsx         # Flow log visualization
│   └── PerformanceChart.tsx   # Analytics charts
├── hooks/
│   ├── useApiData.ts          # API data fetching
│   ├── useResponsiveLayout.ts # Desktop/mobile detection
│   └── useFilters.ts          # Filter state management
├── services/
│   └── api.ts                 # API client
└── types/
    └── benchmark.ts            # TypeScript interfaces
```

#### 3.3 BenchmarkBox Component
```tsx
interface BenchmarkBoxProps {
  result: BenchmarkResult;
  size?: number; // default 16
}

export const BenchmarkBox = ({ result, size = 16 }: BenchmarkBoxProps) => {
  const getColorClass = (score: number) => {
    if (score >= 1.0) return 'bg-green-500';      // 100%
    if (score >= 0.25) return 'bg-yellow-500';   // <100% but >=25%
    return 'bg-red-500';                         // <25%
  };

  return (
    <div 
      className={`${getColorClass(result.score)} hover:opacity-80 transition-opacity cursor-pointer`}
      style={{ 
        width: `${size}px`, 
        height: `${size}px`,
        margin: '1px' // 2px gap achieved with 1px margin
      }}
      title={`${result.benchmark_id}: ${(result.score * 100).toFixed(1)}%`}
    />
  );
};
```

#### 3.4 Responsive Layout Implementation

**Desktop Layout:**
```tsx
export const DesktopLayout = () => {
  const { data } = useApiData('/api/v1/agent-performance');
  
  return (
    <div className="flex flex-col gap-4">
      <div className="flex gap-4 items-center">
        <span className="font-bold">Deterministic</span>
        <span className="font-bold">Local</span>
        <span className="font-bold">GLM 4.6</span>
        <span className="font-bold">Gemini</span>
      </div>
      <div className="flex gap-2">
        {data?.map(agent => (
          <div key={agent.agent_type} className="flex flex-wrap gap-1" style={{width: '25%'}}>
            {agent.results.map(result => (
              <BenchmarkBox key={`${agent.agent_type}-${result.benchmark_id}`} result={result} />
            ))}
          </div>
        ))}
      </div>
    </div>
  );
};
```

**Mobile Layout:**
```tsx
export const MobileLayout = () => {
  const { data } = useApiData('/api/v1/agent-performance');
  
  return (
    <div className="flex flex-col gap-6">
      {data?.map(agent => (
        <div key={agent.agent_type} className="flex flex-col gap-2">
          <h3 className="font-bold text-lg border-b pb-2">{agent.agent_type}</h3>
          <div className="flex flex-wrap gap-1">
            {agent.results.map(result => (
              <BenchmarkBox key={`${agent.agent_type}-${result.benchmark_id}`} result={result} />
            ))}
          </div>
        </div>
      ))}
    </div>
  );
};
```

#### 3.5 Interactive Features
- [ ] Click on boxes to view detailed flow logs
- [ ] Hover tooltips showing benchmark name and score
- [ ] Filter panel for agent types and score ranges
- [ ] Date range selector for temporal analysis
- [ ] Real-time updates via WebSocket or polling
- [ ] Export functionality (CSV, JSON)

#### 3.6 Styling & Design
- [ ] Tailwind CSS for responsive design
- [ ] CSS Grid for flexible box layouts
- [ ] Smooth transitions and micro-interactions
- [ ] Dark/light theme support
- [ ] Loading states and skeleton screens

---

## 🚀 Phase 4: Advanced Features

### 🎯 **Objective**: Add analytics and advanced visualization

### 📋 **Tasks**

#### 4.1 Flow Visualization
- [ ] Interactive flow diagram rendering
- [ ] Tool usage statistics charts
- [ ] Execution timeline visualization
- [ ] Error analysis and pattern detection

#### 4.2 Performance Analytics
- [ ] Agent comparison charts
- [ ] Score distribution histograms
- [ ] Success rate trends over time
- [ ] Benchmark difficulty analysis

#### 4.3 Real-time Monitoring
- [ ] WebSocket integration for live updates
- [ ] Running benchmark status display
- [ ] Live flow execution visualization
- [ ] Performance alerts and notifications

#### 4.4 Export & Sharing
- [ ] Shareable benchmark result links
- [ ] PDF report generation
- [ ] Embedded widget for external sites
- [ ] API key management for external access

---

## 🚀 Phase 5: Integration & Deployment

### 🎯 **Objective**: Production-ready deployment

### 📋 **Tasks**

#### 5.1 Build System
- [ ] Docker containerization for API server
- [ ] Static asset optimization for frontend
- [ ] Environment configuration management
- [ ] CI/CD pipeline setup

#### 5.2 Deployment Architecture
```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Frontend  │    │   API Server│    │  Database   │
│  (Preact)   │───▶│   (Axum)    │───▶│ (SQLite)    │
│  :8080      │    │  :3000      │    │  :file      │
└─────────────┘    └─────────────┘    └─────────────┘
```

#### 5.3 Monitoring & Observability
- [ ] Application metrics (Prometheus)
- [ ] Log aggregation (structured logging)
- [ ] Error tracking (Sentry integration)
- [ ] Performance monitoring

#### 5.4 Security
- [ ] API authentication (optional)
- [ ] Rate limiting and abuse prevention
- [ ] Input sanitization and validation
- [ ] HTTPS configuration

---

## 📊 Technical Specifications

### 🎨 **Visual Design Requirements**

#### Color Scheme
- **Green (#10b981)**: Score = 100%
- **Yellow (#eab308)**: 25% ≤ Score < 100%  
- **Red (#ef4444)**: Score < 25%
- **Gray (#6b7280)**: Pending/Failed benchmarks

#### Box Specifications
- **Size**: 16x16 pixels
- **Gap**: 2 pixels (1px margin each side)
- **Border**: None, clean edges
- **Hover**: 80% opacity with transition
- **Rounded corners**: 2px radius

#### Responsive Breakpoints
- **Mobile**: < 768px (vertical stacking)
- **Desktop**: ≥ 768px (horizontal layout)
- **Agent columns**: 25% width each on desktop

### 🔄 **API Response Format**

#### Benchmark Results Endpoint
```json
{
  "data": [
    {
      "benchmark_id": "001-sol-transfer",
      "agent_type": "deterministic",
      "score": 1.0,
      "final_status": "Succeeded",
      "execution_time_ms": 1250,
      "timestamp": "2024-01-15T10:30:00Z",
      "color_class": "green"
    }
  ],
  "pagination": {
    "page": 1,
    "total_pages": 5,
    "total_items": 47
  }
}
```

#### Agent Performance Summary
```json
{
  "agent_type": "deterministic",
  "total_benchmarks": 47,
  "average_score": 0.95,
  "success_rate": 1.0,
  "best_benchmarks": ["001-sol-transfer", "002-spl-transfer"],
  "worst_benchmarks": ["116-jup-lend-redeem-usdc"],
  "results": [...]
}
```

---

## 🎯 Success Criteria

### ✅ **Phase 1 Success**
- [ ] All flow logs stored in database
- [ ] YML export still functional
- [ ] Database performance optimized
- [ ] Migration system working

### ✅ **Phase 2 Success**
- [ ] All API endpoints functional
- [ ] Comprehensive test coverage
- [ ] Documentation complete
- [ ] Performance benchmarks met

### ✅ **Phase 3 Success**
- [ ] Responsive design working on all devices
- [ ] 16x16 boxes rendered correctly
- [ ] Color coding accurate
- [ ] Interactive features functional

### ✅ **Phase 4 Success**
- [ ] Advanced visualizations working
- [ ] Real-time updates functional
- [ ] Export features working
- [ ] Analytics providing insights

### ✅ **Phase 5 Success**
- [ ] Production deployment successful
- [ ] Monitoring and alerting working
- [ ] Security measures in place
- [ ] Documentation complete

---

## 🛠️ Development Guidelines

### 📋 **Coding Standards**
- Follow existing Rust patterns in `reev-api`
- Use TypeScript for frontend type safety
- Implement comprehensive error handling
- Write tests for all new functionality
- Document API changes

### 🔄 **Git Workflow**
- Feature branches for each phase
- Conventional commit messages
- Code reviews required
- Automated testing in CI/CD

### 📊 **Performance Requirements**
- API response time < 200ms
- Frontend initial load < 2s
- Support 1000+ concurrent users
- Database queries optimized

---

## 🚀 Implementation Timeline

### **Week 1-2**: Phase 1 - Database Integration
- Database schema design and migration
- FlowLogger integration
- Testing and validation

### **Week 3-4**: Phase 2 - API Development  
- API server implementation
- Endpoint development
- Testing and documentation

### **Week 5-6**: Phase 3 - Frontend Development
- Component development
- Responsive layout implementation
- Integration with API

### **Week 7**: Phase 4 - Advanced Features
- Flow visualization
- Analytics implementation
- Real-time features

### **Week 8**: Phase 5 - Deployment
- Production setup
- Monitoring configuration
- Documentation and launch

---

## 🎉 Expected Outcomes

1. **Modern Web Interface**: Intuitive dashboard for monitoring agent performance
2. **Real-time Visualization**: Live benchmark execution and flow analysis
3. **Responsive Design**: Seamless experience across desktop and mobile
4. **Data Insights**: Advanced analytics for agent behavior patterns
5. **Production Ready**: Scalable, monitored, and maintained web service

This comprehensive plan transforms the Reev framework from a CLI/TUI tool into a full-featured web platform while maintaining the existing functionality and adding powerful new visualization capabilities.