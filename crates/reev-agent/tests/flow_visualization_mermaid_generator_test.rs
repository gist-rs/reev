use chrono::Utc;
use reev_agent::flow::visualization::log_parser::{
    ExecutionStatus, FlowExecution, FlowStep, StepType,
};
use reev_agent::flow::visualization::mermaid_generator::MermaidStateDiagramGenerator;
use std::collections::HashMap;

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
        status: ExecutionStatus::Success,
    };

    let diagram = generator.generate_diagram(&[execution]).unwrap();

    assert!(diagram.contains("stateDiagram-v2"));
    assert!(diagram.contains("jupiter_swap"));
    assert!(diagram.contains("AgentStart"));
}
