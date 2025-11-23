//! Prompt Templates for Flow Generation
//!
//! This module provides structured prompt templates for generating
//! YML flows using the LLM.

/// Template for generating structured YML flows
pub struct FlowPromptTemplate;

impl FlowPromptTemplate {
    /// Build a structured prompt for YML flow generation
    pub fn build_flow_prompt(user_prompt: &str) -> String {
        format!(
            r#"You are a DeFi assistant that generates structured YAML flows from user prompts.

Generate a valid YAML flow that represents the user's intent. The flow should include:

1. flow_id: A unique identifier for the flow (use UUID)
2. user_prompt: The original user request
3. subject_wallet_info: Wallet context information
4. steps: A list of steps to execute
5. ground_truth: Validation assertions

User Prompt: "{user_prompt}"

Please generate a YAML flow with the following structure:

```yaml
flow_id: <UUID>
user_prompt: "<original user prompt>"
subject_wallet_info:
  - pubkey: "placeholder_pubkey"
    lamports: 4000000000
    tokens:
      - mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
        amount: 20000000
steps:
  - prompt: "<description of what this step does>"
    context: "<additional context for this step>"
    expected_tool_calls:
      - tool_name: "<tool to call>"
        critical: true
ground_truth:
  final_state_assertions:
    - type: SolBalanceChange
      pubkey: "placeholder_pubkey"
      expected_change_gte: <expected SOL change in lamports>
      error_tolerance: 0.01
  expected_tool_calls:
    - tool_name: "<tool to call>"
      critical: true
```

Make sure the YAML is valid and properly formatted. Focus on creating a logical flow that achieves the user's goal.
"#
        )
    }
}
