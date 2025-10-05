//! # Flow-Aware Tools Module
//!
//! This module provides flow-aware implementations of the core tools.
//! These tools are enhanced with embeddings and context awareness
//! for use in multi-step flow orchestration with RAG.

pub mod jupiter_swap_flow;

pub use jupiter_swap_flow::JupiterSwapFlowTool;

/// Trait for flow-aware tools with embedding support
pub trait FlowTool: rig::tool::Tool {
    /// Get the name of the tool
    fn name(&self) -> String {
        Self::NAME.to_string()
    }

    /// Get a description of the tool
    fn description(&self) -> String;

    /// Get embedding documents for RAG-based tool discovery
    fn embedding_docs(&self) -> Vec<String>;

    /// Get tool metadata for context
    fn metadata(&self) -> std::collections::HashMap<String, String> {
        std::collections::HashMap::new()
    }
}

/// Default implementation for tools that implement the standard rig::tool::Tool
impl<T> FlowTool for T
where
    T: rig::tool::Tool,
    T: Default,
{
    fn name(&self) -> String {
        Self::NAME.to_string()
    }

    fn description(&self) -> String {
        // Default description - tools should override this
        format!("Tool: {}", Self::NAME)
    }

    fn embedding_docs(&self) -> Vec<String> {
        // Default embedding docs - tools should override this
        vec![self.description()]
    }

    fn metadata(&self) -> std::collections::HashMap<String, String> {
        // Default metadata - tools can override this
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("tool_type".to_string(), "flow_aware".to_string());
        metadata.insert("name".to_string(), self.name());
        metadata
    }
}
