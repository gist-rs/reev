//! # Mermaid State Diagram Generator
//!
//! This module converts parsed flow execution data into Mermaid state diagrams
//! for visualizing agent decision flows and tool execution patterns.

use crate::flow::visualization::log_parser::{FlowExecution, FlowStep, StepType};
use std::collections::HashMap;

/// Generates Mermaid state diagrams from flow execution data
pub struct MermaidStateDiagramGenerator {
    /// Configuration options for diagram generation
    config: DiagramConfig,
}

/// Configuration for diagram generation
#[derive(Debug, Clone)]
pub struct DiagramConfig {
    /// Include timing information in the diagram
    pub include_timing: bool,
    /// Include tool parameters in the diagram
    pub include_parameters: bool,
    /// Maximum depth of nested states to show
    pub max_depth: usize,
    /// Whether to show error states
    pub show_errors: bool,
    /// Whether to group tools by category
    pub group_tools: bool,
}

impl Default for DiagramConfig {
    fn default() -> Self {
        Self {
            include_timing: true,
            include_parameters: false,
            max_depth: 3,
            show_errors: true,
            group_tools: true,
        }
    }
}

impl MermaidStateDiagramGenerator {
    /// Create a new generator with default configuration
    pub fn new() -> Self {
        Self {
            config: DiagramConfig::default(),
        }
    }

    /// Create a new generator with custom configuration
    pub fn with_config(config: DiagramConfig) -> Self {
        Self { config }
    }

    /// Generate a Mermaid state diagram from flow execution data
    pub fn generate_diagram(
        &self,
        executions: &[FlowExecution],
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut diagram = String::new();

        // Start with state diagram header
        diagram.push_str("stateDiagram-v2\n");
        diagram.push_str("    [*] --> StartAgent\n\n");

        // Process each execution
        for (i, execution) in executions.iter().enumerate() {
            let execution_name = format!("Execution{}", i + 1);
            diagram.push_str(&format!(
                "    state \"{execution_name}\" as {execution_name}\n"
            ));
            diagram.push_str(&self.generate_execution_diagram(execution, &execution_name)?);
            diagram.push('\n');
        }

        // Add transitions between executions if multiple
        if executions.len() > 1 {
            for i in 0..executions.len() - 1 {
                diagram.push_str(&format!("    Execution{} --> Execution{}\n", i + 1, i + 2));
            }
        }

        // Add final transition to end state
        if !executions.is_empty() {
            diagram.push_str(&format!("    Execution{} --> [*]\n", executions.len()));
        }

        // Add styling
        diagram.push_str(&self.generate_styling(executions)?);

        Ok(diagram)
    }

    /// Generate diagram for a single execution
    fn generate_execution_diagram(
        &self,
        execution: &FlowExecution,
        execution_name: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut diagram = String::new();

        // Create a state for this execution
        diagram.push_str(&format!("        {execution_name}\n"));
        diagram.push_str("            [*] --> AgentStart\n");

        let mut current_state = "AgentStart".to_string();
        let mut tool_states = HashMap::new();
        let mut state_counter = 0;

        for step in &execution.steps {
            match &step.step_type {
                StepType::AgentStart => {
                    diagram.push_str(&format!(
                        "            state \"Agent Start\\nModel: {}\" as AgentStart\n",
                        execution.agent
                    ));
                }
                StepType::ToolCall => {
                    if let Some(tool_name) = &step.tool_name {
                        state_counter += 1;
                        let tool_state = format!("Tool{state_counter}");

                        // Create tool state
                        let tool_label = self.create_tool_state_label(step);
                        diagram.push_str(&format!(
                            "            state \"{tool_label}\" as {tool_state}\n"
                        ));

                        // Add transition from previous state
                        diagram
                            .push_str(&format!("            {current_state} --> {tool_state}\n"));

                        current_state = tool_state.clone();
                        tool_states.insert(tool_name.clone(), tool_state);
                    }
                }
                StepType::ToolComplete => {
                    if let Some(tool_name) = &step.tool_name {
                        if let Some(_tool_state) = tool_states.get(tool_name) {
                            // Add completion state
                            state_counter += 1;
                            let complete_state = format!("Complete{state_counter}");

                            let completion_label = if self.config.include_timing {
                                if let Some(duration) = step.duration_ms {
                                    format!("{tool_name} Complete\\n({duration}ms)")
                                } else {
                                    format!("{tool_name} Complete")
                                }
                            } else {
                                format!("{tool_name} Complete")
                            };

                            diagram.push_str(&format!(
                                "            state \"{completion_label}\" as {complete_state}\n"
                            ));

                            diagram.push_str(&format!(
                                "            {current_state} --> {complete_state}\n"
                            ));
                            current_state = complete_state;
                        }
                    }
                }
                StepType::ToolError => {
                    if self.config.show_errors {
                        state_counter += 1;
                        let error_state = format!("Error{state_counter}");

                        let error_label = if let Some(tool_name) = &step.tool_name {
                            format!("{tool_name} Error")
                        } else {
                            "Error".to_string()
                        };

                        diagram.push_str(&format!(
                            "            state \"{error_label}\" as {error_state}\n"
                        ));

                        diagram
                            .push_str(&format!("            {current_state} --> {error_state}\n"));
                        current_state = error_state;
                    }
                }
                StepType::AgentEnd => {
                    diagram.push_str("            state \"Agent End\" as AgentEnd\n");
                    diagram.push_str(&format!("            {current_state} --> AgentEnd\n"));
                    current_state = "AgentEnd".to_string();
                }
                _ => {} // Skip other step types for now
            }
        }

        // If we didn't end at AgentEnd, add it
        if current_state != "AgentEnd" {
            diagram.push_str("            state \"Agent End\" as AgentEnd\n");
            diagram.push_str(&format!("            {current_state} --> AgentEnd\n"));
        }

        Ok(diagram)
    }

    /// Create a label for tool state
    fn create_tool_state_label(&self, step: &FlowStep) -> String {
        let mut label = step.tool_name.clone().unwrap_or("Unknown Tool".to_string());

        if self.config.include_parameters && !step.parameters.is_empty() {
            label.push_str("\\n");
            let params: Vec<String> = step
                .parameters
                .iter()
                .take(3) // Limit to prevent overcrowding
                .map(|(k, v)| format!("{}: {}", k, self.truncate_value(v)))
                .collect();
            label.push_str(&params.join(", "));
        }

        label
    }

    /// Truncate long values for display
    fn truncate_value(&self, value: &str) -> String {
        if value.len() > 20 {
            format!("{}...", &value[..17])
        } else {
            value.to_string()
        }
    }

    /// Generate styling for the diagram
    fn generate_styling(
        &self,
        executions: &[FlowExecution],
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut styling = String::new();
        styling.push_str("\n");

        // Define styles for different tool types
        let tool_types = self.identify_tool_types(executions);

        for (i, (tool_type, tools)) in tool_types.iter().enumerate() {
            let color = self.get_tool_type_color(i);
            for tool in tools {
                styling.push_str(&format!(
                    "    classDef {}{} fill:{}\n",
                    tool_type,
                    tools.iter().position(|t| t == tool).unwrap_or(0),
                    color
                ));
            }
        }

        // Apply styles to specific states
        for execution in executions {
            for step in &execution.steps {
                if let Some(tool_name) = &step.tool_name {
                    let tool_type = self.categorize_tool(tool_name);
                    styling.push_str(&format!("    class {tool_type}{tool_name} {tool_type}\n"));
                }
            }
        }

        Ok(styling)
    }

    /// Identify different types of tools used
    fn identify_tool_types(&self, executions: &[FlowExecution]) -> Vec<(String, Vec<String>)> {
        let mut tool_types: HashMap<String, Vec<String>> = HashMap::new();

        for execution in executions {
            for step in &execution.steps {
                if let Some(tool_name) = &step.tool_name {
                    let tool_type = self.categorize_tool(tool_name);
                    tool_types
                        .entry(tool_type.clone())
                        .or_default()
                        .push(tool_name.clone());
                }
            }
        }

        tool_types.into_iter().collect()
    }

    /// Categorize a tool by its function
    fn categorize_tool(&self, tool_name: &str) -> String {
        if tool_name.contains("swap") || tool_name.contains("jupiter") {
            "Swap".to_string()
        } else if tool_name.contains("transfer") {
            "Transfer".to_string()
        } else if tool_name.contains("balance") {
            "Discovery".to_string()
        } else if tool_name.contains("lend")
            || tool_name.contains("deposit")
            || tool_name.contains("withdraw")
        {
            "Lending".to_string()
        } else {
            "Other".to_string()
        }
    }

    /// Get color for tool type
    fn get_tool_type_color(&self, index: usize) -> &'static str {
        match index % 6 {
            0 => "#ff6b6b", // Red
            1 => "#4ecdc4", // Teal
            2 => "#45b7d1", // Blue
            3 => "#f9ca24", // Yellow
            4 => "#6c5ce7", // Purple
            5 => "#a29bfe", // Light purple
            _ => "#dfe6e9", // Gray
        }
    }
}

impl Default for MermaidStateDiagramGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flow::visualization::log_parser::{FlowStep, StepType};
    use chrono::Utc;

    #[test]
    fn test_simple_diagram_generation() {
        let generator = MermaidStateDiagramGenerator::new();

        let execution = FlowExecution {
            execution_id: "test_exec".to_string(),
            steps: vec![
                FlowStep {
                    id: "step_1".to_string(),
                    step_type: StepType::AgentStart,
                    tool_name: None,
                    agent_name: Some("test_agent".to_string()),
                    timestamp: Utc::now().to_rfc3339(),
                    duration_ms: None,
                    parameters: HashMap::new(),
                    result: None,
                    metadata: HashMap::new(),
                },
                FlowStep {
                    id: "step_2".to_string(),
                    step_type: StepType::ToolCall,
                    tool_name: Some("jupiter_swap".to_string()),
                    agent_name: Some("test_agent".to_string()),
                    timestamp: Utc::now().to_rfc3339(),
                    duration_ms: None,
                    parameters: HashMap::new(),
                    result: None,
                    metadata: HashMap::new(),
                },
            ],
            agent: "test_agent".to_string(),
            start_time: Utc::now().to_rfc3339(),
            end_time: Some(Utc::now().to_rfc3339()),
            total_duration_ms: Some(1000),
            tools_used: vec!["jupiter_swap".to_string()],
            status: crate::flow::visualization::log_parser::ExecutionStatus::Success,
        };

        let diagram = generator.generate_diagram(&[execution]).unwrap();

        assert!(diagram.contains("stateDiagram-v2"));
        assert!(diagram.contains("jupiter_swap"));
        assert!(diagram.contains("AgentStart"));
    }
}
