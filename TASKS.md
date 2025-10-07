# 🚀 Reev Tasks and Implementation Plan

## 📋 Current Status: Phase 16A - Smart Dependency Management (Current Priority)

### ✅ **Production Framework Achievements (Recent Phases)**:
- **🤖 AI Agent Integration**: End-to-end AI agent evaluation framework (100% success rates)
- **🔧 Complete Tool System**: Single-step tools (sol_transfer, spl_transfer, jupiter_swap, jupiter_lend)
- **📊 Comprehensive Benchmark Suite**: Transaction, flow, and API benchmarks (001-200+)
- **🧪 Robust Test Architecture**: Deterministic vs LLM tests with automatic service management
- **🔄 Advanced Multi-Step Flows**: Real multi-step workflows with Jupiter protocol integration
- **🌐 Real AI Integration**: Local LLM servers (LM Studio, Ollama) and cloud APIs (Gemini)
- **⚡ Real On-Chain Execution**: Authentic Solana transactions on forked mainnet
- **🎯 Complete Jupiter Stack**: Full protocol integration (swap, lend, mint/redeem, positions, earnings)

### 🚀 **Current Production Capabilities**:
- **100% Success Rate**: All benchmarks passing with both deterministic and AI agents
- **Real Jupiter Integration**: Complete protocol stack with mainnet fork validation
- **Multi-Step Orchestration**: Complex DeFi workflows with automatic tool selection
- **Professional Infrastructure**: TUI cockpit, database persistence, comprehensive logging
- **Advanced Scoring**: Granular instruction evaluation + on-chain execution metrics

---

## 🚀 Phase 16A: Smart Dependency Management (Current Priority)

### **🎯 Primary Objective**
Implement intelligent dependency management architecture that separates concerns and provides zero-setup experience for developers while maintaining clean component boundaries.

### **📋 Core Architecture Changes**
- **Component Separation**: `reev-lib` and `reev-agent` have no surfpool dependencies
- **Runner as Orchestrator**: `reev-runner` manages all external dependencies automatically
- **Starter Pack Distribution**: Pre-built binaries for instant setup without compilation
- **Smart Process Management**: Automatic detection and shared instance support

### **🏗️ Implementation Plan**:
---

### **Priority 1: Dependency Management Architecture**
- [ ] **16.1.1**: Create core dependency manager
  ```rust
  // crates/reev-runner/src/dependency_manager.rs
  pub struct DependencyManager {
      reev_agent_process: Option<Child>,
      surfpool_process: Option<Child>,
      config: DependencyConfig,
  }
  
  impl DependencyManager {
      pub fn new() -> Self;
      pub async fn ensure_dependencies(&mut self) -> Result<DependencyUrls>;
      pub async fn start_reev_agent(&mut self) -> Result<()>;
      pub async fn start_surfpool(&mut self) -> Result<()>;
      pub async fn cleanup(&mut self) -> Result<()>;
  }
  ```
- [ ] **16.1.2**: Implement process detection utilities
  ```rust
  // crates/reev-runner/src/process_detector.rs
  pub struct ProcessDetector;
  
  impl ProcessDetector {
      pub fn is_process_running(process_name: &str) -> Result<bool>;
      pub fn find_process_pid(process_name: &str) -> Option<u32>;
      pub fn get_process_ports(pid: u32) -> Result<Vec<u16>>;
  }
  ```
- [ ] **16.1.3**: Add health monitoring for dependencies
  ```rust
  // crates/reev-runner/src/health_monitor.rs
  pub struct HealthMonitor {
      pub async fn check_service_health(&self, service_name: &str) -> Result<ServiceHealth>;
      pub async fn wait_for_health(&self, service_name: &str, timeout: Duration) -> Result<()>;
      pub async fn start_monitoring(&self) -> Result<()>
  }
  ```
- [ ] **16.1.4**: Implement lifecycle management
  ```rust
  // crates/reev-runner/src/lifecycle_manager.rs
  pub struct LifecycleManager {
      pub async fn graceful_shutdown(&mut self) -> Result<()>;
      pub async fn force_shutdown(&mut self) -> Result<()>;
      pub fn setup_signal_handlers(&self);
  }
  ```

#### **Priority 2: Starter Pack System**
- **Binary Distribution**: Platform-specific pre-built binaries (Linux, macOS, Windows)
- **GitHub Integration**: Automatic download from GitHub releases when available
- **Local Caching**: Store binaries in `.surfpool/cache/` for instant reuse
- **Fallback Building**: Build from source only when binaries are unavailable

#### **Priority 3: Smart Installation**
- **Platform Detection**: Automatic detection of OS architecture and platform
- **Version Management**: Check for updates and manage version compatibility
- **Integrity Verification**: Verify downloaded binaries with checksums
- **Extraction & Setup**: Automatic extraction to `.surfpool/installs/` with symlinks

#### **Priority 4: Process Orchestration**
- **Sequential Startup**: Start reev-agent first, then surfpool with health verification
- **Port Management**: Automatic port allocation and conflict resolution
- **Shared Instances**: Allow multiple runner processes to use same services
- **Cleanup Handling**: Proper termination of all processes on graceful shutdown

### **🎯 Success Criteria**
- **Zero-Setup Experience**: Run benchmarks with automatic dependency management
- **Fast Startup**: Reduce startup time from minutes to seconds with cached binaries
- **Component Independence**: Clean separation allows independent testing and development
- **Developer Friendly**: Clear status indicators and automatic error handling

### **🎯 Success Criteria**:
- **Zero-Setup Experience**: Developers run benchmarks without manual surfpool setup
- **Fast Startup**: Reduce startup time from minutes to seconds with binary caching
- **Resource Efficiency**: Shared surfpool instances reduce memory and CPU usage
- **Developer Friendly**: Clear status indicators and error messages for troubleshooting

---

## ✅ Recently Completed: Phase 15 - Advanced Multi-Step Workflows

### **🎯 Objective Achieved**:
Enable LLM agents to orchestrate multiple tools in sequence to complete complex DeFi workflows.

### **🏗️ Major Accomplishments**:
- **Flow Benchmark Architecture** (200-series): Multi-step DeFi operations with automatic orchestration
- **RAG-Based Flow Agent**: Vector store integration for dynamic tool selection
- **Enhanced Tool System**: Jupiter swap and lending protocols with flow awareness
- **Real Jupiter SDK Integration**: Complete replacement of public API calls with local surfpool interaction

### **📊 Production Results**:
- **Complete Pipeline**: Runner → Environment → Agent → LLM → Scoring working end-to-end
- **Real AI Integration**: Successfully tested with local models and cloud APIs
- **Complex Operations**: Jupiter swap + lend workflows executing flawlessly
- **Infrastructure Validation**: Automatic service management and error handling verified

---

### **Priority 2: Binary Optimization & GitHub Integration (High)**
- [ ] **16.2.2**: Implement surfpool binary detection and caching
  ```rust
  // crates/reev-runner/src/binary_manager.rs
  pub struct BinaryManager {
      pub async fn get_cached_binary() -> Result<Option<PathBuf>>
      pub async fn download_and_cache() -> Result<PathBuf>
      pub async fn is_cached_binary_valid() -> Result<bool>
  }
  ```
- [ ] **16.2.3**: Create .surfpool directory management
  ```rust
  // crates/reev-runner/src/binary_manager.rs
  const SURFPOOL_CACHE_DIR: &str = ".surfpool/cache";
  
  impl BinaryManager {
      fn ensure_cache_dir() -> Result<PathBuf>
      fn cleanup_old_binaries() -> Result<()>
      fn get_binary_path(version: &str) -> PathBuf
  }
  ```
- [ ] **16.2.4**: Add fallback build mechanism
  ```rust
  // crates/reev-runner/src/binary_manager.rs
  pub async fn get_or_build_surfpool() -> Result<PathBuf> {
      // 1. Try cached binary
      if let Some(cached) = get_cached_binary().await? {
          return Ok(cached);
      }
      // 2. Try download from GitHub
      if let Ok(downloaded) = download_and_cache().await {
          return Ok(downloaded);
      }
      // 3. Fallback to build from source
      build_from_source().await
  }
  ```
---

### **Priority 3: Service Discovery & Health Monitoring (Medium)**
- [ ] **16.3.1**: Implement dependency health check service
  ```rust
  // crates/reev-runner/src/health_monitor.rs
  pub struct HealthMonitor {
      pub async fn check_service_health(&self, service_name: &str) -> Result<ServiceHealth>
      pub async fn start_monitoring(&self) -> Result<()>
      pub fn get_status(&self) -> HealthStatus
  }
  ```
- [ ] **16.3.2**: Add port management and conflict resolution
  ```rust
  // crates/reev-runner/src/port_manager.rs
  pub struct PortManager {
      pub async fn find_available_port() -> Result<u16>
      pub async fn is_port_available(port: u16) -> bool
      pub async fn reserve_port() -> Result<u16>
  }
  ```
- [ ] **16.3.3**: Create service registry for dependency management
  ```rust
  // crates/reev-runner/src/service_registry.rs
  pub struct ServiceRegistry {
      pub async fn register_service(&mut self, service: DependencyService)
      pub async fn get_service(&self, id: &str) -> Option<&DependencyService>
      pub async fn list_services(&self) -> Vec<&DependencyService>
  }
  ```

### **Priority 4: Developer Experience Enhancements (Low)**
- [ ] **16.4.1**: Add dependency status indicators to CLI/TUI
  ```rust
  // crates/reev-runner/src/display/dependency_status.rs
  pub fn display_dependency_status(handle: &DependencyHandle) -> String
  pub fn display_dependency_metrics(metrics: &DependencyMetrics) -> String
  ```
- [ ] **16.4.2**: Implement dependency configuration management
  ```rust
  // crates/reev-runner/src/config.rs
  pub struct DependencyConfig {
      pub auto_start: bool,
      pub prefer_binary: bool,
      pub cache_duration: Duration,
      pub health_check_interval: Duration,
      pub shared_instances: bool,
  }
  ```
- [ ] **16.4.3**: Add dependency log viewer integration
  ```rust
  // crates/reev-tui/src/components/dependency_logs.rs
  pub struct DependencyLogsComponent {
      pub fn new() -> Self
      pub fn update_logs(&mut self, logs: Vec<String>)
      pub fn render_logs(&self, frame: &mut Frame)
  }
  ```

### **Priority 5: Integration & Testing (High)**
- [ ] **16.5.1**: Update runner to use dependency manager
  ```rust
  // crates/reev-runner/src/main.rs
  impl DependencyManager {
      pub async fn ensure_dependencies(&mut self) -> Result<()>
      // Auto-start reev-agent and surfpool as needed
  }
  
  pub fn get_dependency_urls(&self) -> DependencyUrls {
      DependencyUrls {
          reev_agent: "http://localhost:9090/gen/tx",
          surfpool_rpc: "http://localhost:8899",
      }
  }
  ```
- [ ] **16.5.2**: Add dependency management benchmarks
  ```yaml
  # benchmarks/900-dependency-management.yml
  id: 900-dependency-auto-start
  description: Test automatic dependency startup and management
  initial_state: []
  prompt: "Start all dependencies automatically and verify they're healthy"
  ground_truth:
      success_criteria:
          - type: "dependencies_running"
          description: "All dependencies should be running after test"
          required: true
  ```
- [ ] **16.5.3**: Add integration tests for dependency manager
  ```rust
  // crates/reev-runner/tests/dependency_manager_test.rs
  #[tokio::test]
  async fn test_auto_start_dependencies() -> Result<()>
  #[tokio::test]
  async fn test_binary_caching() -> Result<>()
  #[tokio::test]
  async fn test_shared_instances() -> Result<>()
  ```

### **Priority 6: Documentation & Examples (Medium)**
- [ ] **16.6.1**: Update README.md with automatic dependency setup
- [ ] **16.6.2**: Create dependency management examples
  ```rust
  // examples/dependency_management.rs
  async fn main() -> Result<()> {
      let manager = DependencyManager::new();
      let urls = manager.ensure_dependencies().await?;
      println!("Dependencies running: reev-agent={}, surfpool={}", 
          urls.reev_agent, urls.surfpool_rpc);
  }
  ```
- [ ] **16.6.3**: Add troubleshooting guide for dependency issues
- [ ] **16.6.4**: Update API documentation for dependency management
```


---

## 🚀 **Next Steps - PHASE 16**

1. **Advanced Multi-Step Workflows**: Compound strategies and arbitrage with real integration
2. **Enhanced Error Recovery**: Better handling of external service dependencies
3. **Performance Optimization**: Improve instruction generation and account preloading
4. **Expanded Benchmark Suite**: 201-COMPOUND, 202-ARBITRAGE with real Jupiter APIs
5. **Production Deployment**: Framework for production multi-step agent evaluation
6. **Community Examples**: Real workflows contributed by the community
