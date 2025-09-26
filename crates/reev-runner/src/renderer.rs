use ascii_tree::{Tree, write_tree};
use reev_lib::{results::TestResult, trace::TraceStep};

/// Renders a `TestResult` object into a human-readable ASCII tree format.
///
/// This provides a quick, high-level overview of the agent's execution trace
/// directly in the terminal.
pub fn render_result_as_tree(result: &TestResult) -> String {
    let status_icon = if result.final_status == reev_lib::results::FinalStatus::Succeeded {
        "✅"
    } else {
        "❌"
    };
    let root_label = format!("{} {}: {}", status_icon, result.id, result.final_status);

    let mut step_nodes = Vec::new();
    for (i, step) in result.trace.steps.iter().enumerate() {
        step_nodes.push(render_step_node(i + 1, step));
    }

    let tree = Tree::Node(root_label, step_nodes);
    let mut buffer = String::new();
    write_tree(&mut buffer, &tree).unwrap();
    buffer
}

/// Renders a single `TraceStep` into a `Tree` node for the ASCII tree.
fn render_step_node(step_number: usize, step: &TraceStep) -> Tree {
    let step_label = format!("Step {step_number}");

    // Serialize the `AgentAction` wrapper directly. Its custom `Serialize` implementation
    // is designed to produce a human-readable format with Base58 strings for
    // the program_id, all account pubkeys, and the instruction data.
    let action_str = match serde_json::to_string_pretty(&step.action) {
        Ok(json_str) => {
            // Indent the JSON for better readability within the tree
            json_str
                .lines()
                .map(|line| format!("     {line}"))
                .collect::<Vec<_>>()
                .join("\n")
        }
        Err(_) => " [Error serializing action]".to_string(),
    };
    let action_node = Tree::Leaf(vec![format!("ACTION:\n{}", action_str)]);

    let observation_label = format!("OBSERVATION: {}", step.observation.last_transaction_status);
    let mut observation_children = Vec::new();
    if let Some(error) = &step.observation.last_transaction_error {
        observation_children.push(Tree::Leaf(vec![format!("Error: {}", error)]));
    }
    // Display all logs for better debugging
    if !step.observation.last_transaction_logs.is_empty() {
        let logs_str = step.observation.last_transaction_logs.join("\n     ");
        observation_children.push(Tree::Leaf(vec![format!("Logs:\n     {}", logs_str)]));
    }

    let observation_node = Tree::Node(observation_label, observation_children);

    Tree::Node(step_label, vec![action_node, observation_node])
}
