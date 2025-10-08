# PLAN.md: Development Roadmap for `reev` 🪸

**Current Status: Production-Ready Framework with Critical AI Agent Enhancement Needed**  
**Next Focus: Making Local Model Superior to Deterministic Agent**

---

## 🎯 Executive Summary

The `reev` framework has achieved **production-ready status** with comprehensive capabilities for evaluating Solana LLM agents. However, a critical issue has been identified: the local AI model is underperforming compared to the deterministic agent, which defeats the purpose of AI evaluation.

**Current Issue**: 
- ✅ **Deterministic Agent**: 100% success rate with proper multi-step flows
- ❌ **Local Model**: Only 75% success rate, fails to understand multi-step workflows
- **Gap**: Local model generates single instructions instead of required multi-step flows

**Urgent Priority**: Enhance the local model to be smarter, more dynamic, and superior to deterministic execution.

---

## 🚨 Phase 16: Critical AI Agent Enhancement (Current Phase)

### 🎯 **Primary Objective**
Transform the local model from underperforming to superior by implementing advanced AI capabilities that exceed deterministic agent performance.

### 🛠️ **Core Enhancement Areas**

#### **Priority 1: Superior System Prompts & Context Engineering**
- **Enhanced System Prompt**: Create intelligent prompts that understand multi-step DeFi workflows
- **Context-Aware Reasoning**: Enable agent to understand when multiple steps are required
- **Dynamic Flow Detection**: Agent should automatically identify need for multi-step operations
- **Self-Correction**: Agent should recognize when a single step is insufficient and request additional steps

#### **Priority 2: Multi-Turn Conversation Architecture**
- **Rig Integration**: Implement multi-turn agent capabilities using Rig framework
- **Step-by-Step Execution**: Allow agent to break complex operations into sequential steps
- **State Management**: Maintain conversation context across multiple turns
- **Progressive Completion**: Enable agent to validate and continue until full workflow completion

#### **Priority 3: Enhanced Context & Tool Discovery**
- **Rich Financial Context**: Provide comprehensive DeFi protocol information
- **Dynamic Tool Selection**: Agent should discover and select appropriate tools for each step
- **Balance Awareness**: Agent should understand token balances and requirements before operations
- **Error Recovery**: Agent should handle failures and retry with different approaches

### 🎯 **Success Criteria**
- **Superior Performance**: Local model achieves 100% success rate on all flow benchmarks
- **Multi-Step Mastery**: Agent properly sequences swap → lend operations without guidance
- **Adaptive Intelligence**: Agent handles edge cases and unexpected scenarios better than deterministic
- **Demonstrated Superiority**: Local model shows capabilities impossible with deterministic approach

---

## 🏗️ Implementation Strategy

### **Phase 16.1: System Prompt & Context Enhancement** 
- Design comprehensive DeFi system prompts
- Implement rich context injection for financial operations
- Add balance and requirement awareness

### **Phase 16.2: Multi-Turn Architecture Integration**
- Integrate Rig multi-turn agent framework
- Implement step-by-step workflow management
- Add conversation state persistence

### **Phase 16.3: Advanced Tool Integration**
- Enhance Jupiter tool integration with multi-step awareness
- Add balance checking and validation tools
- Implement error recovery and retry mechanisms

### **Phase 16.4: Validation & Performance Testing**
- Comprehensive testing against deterministic benchmarks
- Performance comparison and optimization
- Edge case handling validation

---

## 📊 Expected Outcomes

### **Before Enhancement**
- Local Model: 75% success rate, single-step thinking
- Deterministic: 100% success rate, predictable behavior

### **After Enhancement**
- Local Model: 100%+ success rate, intelligent multi-step reasoning
- Deterministic: 100% success rate, baseline for comparison
- **Result**: Local model demonstrates superior AI capabilities that deterministic cannot achieve

---

## 🔮 Future Roadmap (Post-Phase 16)

### **Phase 17: Advanced Multi-Agent Collaboration**
- Multi-agent workflows for complex DeFi strategies
- Competitive benchmarking between different AI approaches
- Learning and adaptation capabilities

### **Phase 18: Enterprise Features**
- Team collaboration and shared workspaces
- CI/CD integration and automated testing
- Advanced analytics and performance insights

### **Phase 19: Ecosystem Expansion**
- Additional DeFi protocol support (Raydium, Orca, etc.)
- Cross-chain operations and multi-chain strategies
- Community features and public benchmark sharing

---

## 📚 Documentation & Resources

### **📖 Current Documentation**
- **README.md**: Production-ready quick start guide
- **TASKS.md**: Detailed implementation plan for AI enhancement
- **REFLECT.md**: Archive of debugging sessions and insights

### **🎯 Development Guidelines**
- **AI-First Development**: Prioritize agent intelligence over hard-coded logic
- **Superiority Testing**: Ensure AI agent outperforms deterministic approaches
- **Comprehensive Logging**: Detailed tracing for AI decision analysis
- **Multi-Step Validation**: Rigorous testing of complex workflow execution

---

## 🎉 Conclusion

The immediate focus is transforming the local model from a limitation to a demonstration of superior AI intelligence. By implementing advanced system prompts, multi-turn conversations, and enhanced context, the local model should not only match but exceed deterministic agent performance, showcasing the true potential of AI-driven autonomous agents in DeFi environments.

### **🎯 Primary Objective**
Implement intelligent dependency management architecture that separates concerns and provides zero-setup experience for developers while maintaining clean component boundaries.

### **📋 Core Architecture Changes**
- **Component Separation**: `reev-lib` and `reev-agent` have no surfpool dependencies
- **Runner as Orchestrator**: `reev-runner` manages all external dependencies automatically
- **Starter Pack Distribution**: Pre-built binaries for instant setup without compilation
- **Smart Process Management**: Automatic detection and shared instance support

### 🛠️ **Key Features to Implement**

#### **Priority 1: Dependency Management Architecture**
- **Process Manager**: Centralized management of surfpool and reev-agent processes
- **Health Monitoring**: Continuous health checks with automatic recovery mechanisms
- **Service Discovery**: Automatic detection of running processes to avoid duplicates
- **Lifecycle Management**: Proper cleanup and graceful shutdown on exit

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

### 🎯 **Success Criteria**
- **Zero-Setup Experience**: Run benchmarks with automatic dependency management
- **Fast Startup**: Reduce startup time from minutes to seconds with cached binaries
- **Component Independence**: Clean separation allows independent testing and development
- **Developer Friendly**: Clear status indicators and automatic error handling

---

## 📊 Current Architecture & Capabilities

### **🏗️ Framework Components (Production Ready)**
- **`reev-lib`**: Core evaluation engine with complete Jupiter protocol support
- **`reev-runner`**: CLI orchestrator with comprehensive benchmark suite
- **`reev-agent`**: Dual-agent service (deterministic + AI) with advanced tool integration
- **`reev-tui`**: Interactive cockpit with real-time monitoring and analysis

### **🎯 Benchmark Categories (All Passing)**
- **Transaction Benchmarks** (100-series): Real Jupiter protocol operations
- **Flow Benchmarks** (200-series): Multi-step DeFi workflow orchestration  
- **API Benchmarks**: Data retrieval and portfolio management operations

### **🤖 Agent Capabilities (Fully Functional)**
- **Deterministic Agent**: Ground truth generator with perfect instruction quality
- **Local Model Agent**: AI agent with local LLM integration (100% success rate)
- **Cloud Model Agent**: AI agent with cloud API integration (Gemini, etc.)

### **📈 Performance Metrics**
- **Success Rate**: 100% on all benchmark categories with local model
- **Instruction Quality**: Perfect Jupiter SDK integration with real programs
- **Execution Speed**: Fast surfpool simulation with mainnet fork validation
- **Scoring Accuracy**: Granular evaluation of agent reasoning vs. execution

---

## 🔮 Future Roadmap (Post-Phase 16)

### **Phase 17: Advanced Agent Capabilities**
- **Multi-Agent Collaboration**: Multiple agents working together on complex tasks
- **Learning & Adaptation**: Agents that improve performance over time
- **Cross-Chain Operations**: Support for other blockchain networks and protocols

### **Phase 18: Enterprise Features**
- **Team Collaboration**: Shared workspaces and collaborative benchmarking
- **CI/CD Integration**: Automated testing and deployment pipelines
- **Advanced Analytics**: Deep performance insights and agent behavior analysis

### **Phase 19: Ecosystem Expansion**
- **Protocol Expansion**: Support for additional DeFi protocols (Raydium, Orca, etc.)
- **Tool Marketplace**: Extensible tool system for custom protocols
- **Community Features**: Public benchmark sharing and leaderboards

---

## 📚 Documentation & Resources

### **📖 Current Documentation**
- **README.md**: Production-ready quick start guide and usage examples
- **TASKS.md**: Detailed implementation plan for Phase 16 surfpool management
- **REFLECT.md**: Archive of debugging sessions and lessons learned
- **RULES.md**: Engineering guidelines and architectural principles

### **🎯 Development Guidelines**
- **Benchmark-Driven Development**: All features validated through comprehensive benchmarks
- **Real-World Testing**: Mainnet fork validation with actual deployed programs
- **Comprehensive Logging**: Detailed tracing for debugging and performance analysis
- **Modular Architecture**: Clean separation of concerns for maintainability

---

## 🎉 Conclusion

The `reev` framework has successfully evolved from a proof-of-concept to a **production-ready evaluation platform** for Solana LLM agents. With comprehensive Jupiter integration, advanced multi-step workflows, and robust infrastructure, it now serves as the definitive tool for assessing autonomous agent capabilities in realistic blockchain environments.

The upcoming Phase 16 surfpool management improvements will further enhance the developer experience, making the framework even more accessible and efficient for both research and production use cases.