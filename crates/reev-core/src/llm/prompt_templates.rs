//! Prompt Templates for Flow Generation
//!
//! This module provides structured prompt templates for generating
//! YML flows using the LLM.

/// Template for generating structured YML flows
pub struct FlowPromptTemplate;

impl FlowPromptTemplate {
    /// Build a structured prompt for intent and parameter extraction
    pub fn build_flow_prompt(user_prompt: &str) -> String {
        format!(
            r#"You are a DeFi assistant that extracts user intent and parameters from prompts.

Extract the user's intent and key parameters from the following prompt. Respond with a simple JSON object containing:

1. intent: The type of operation (swap, lend, borrow, etc.)
2. parameters: Key parameters for the operation
   - from_token: Source token (e.g., SOL, USDC)
   - to_token: Destination token (for swaps)
   - amount: The amount to operate with
   - percentage: Percentage if specified (e.g., "50%")
3. steps: Brief description of the steps needed

User Prompt: "{user_prompt}"

Please respond with a JSON object in this format:
{{
  "intent": "swap|lend|borrow|etc",
  "parameters": {{
    "from_token": "SOL|USDC|etc",
    "to_token": "USDC|SOL|etc",
    "amount": "1.0|50|etc",
    "percentage": "50|100|null"
  }},
  "steps": ["Step 1 description", "Step 2 description"]
}}

Focus on extracting the core intent and parameters accurately, even if there are typos or variations in phrasing.
"#
        )
    }
}
