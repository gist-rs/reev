use reev_lib::{
    results::{FinalStatus, TestResult},
    trace::TraceStep,
};

/// Main public entry point to render a `TestResult` as an ASCII tree.
pub fn render_result_as_tree(result: &TestResult) -> String {
    let mut output = String::new();

    // 1. Render the main header for the test case result.
    let status_icon = match result.final_status {
        FinalStatus::Succeeded => "✔",
        FinalStatus::Failed => "✗",
    };
    output.push_str(&format!(
        "{} {}: {:?}\n",
        status_icon, result.id, result.final_status
    ));

    // 2. Render each step in the execution trace.
    let steps = &result.trace.steps;
    let num_steps = steps.len();
    for (i, step) in steps.iter().enumerate() {
        let is_last_step = i == num_steps - 1;
        render_step(&mut output, step, i + 1, is_last_step);
    }

    output
}

/// Renders a single `TraceStep`, including its action and observation.
fn render_step(output: &mut String, step: &TraceStep, step_num: usize, is_last_step: bool) {
    let prefix = if is_last_step { "└─ " } else { "├─ " };
    output.push_str(&format!("{}Step {}:\n", prefix, step_num));

    let child_prefix = if is_last_step { "   " } else { "│  " };

    // An action is always the first child of a step, and there's always an observation after it.
    render_action_node(output, step, child_prefix, false);
    // The observation is always the last child of a step.
    render_observation_node(output, step, child_prefix, true);
}

/// Renders the ACTION part of a step as a formatted node in the tree.
fn render_action_node(output: &mut String, step: &TraceStep, prefix: &str, is_last: bool) {
    let connector = if is_last { "└─ " } else { "├─ " };
    let params = step
        .action
        .parameters
        .iter()
        .map(|(k, v)| {
            let s = v.to_string();
            // Don't add extra quotes around simple strings or numbers for readability.
            if v.is_string() || v.is_number() {
                format!("{}: {}", k, s.trim_matches('"'))
            } else {
                format!("{}: {}", k, s)
            }
        })
        .collect::<Vec<_>>()
        .join(", ");
    output.push_str(&format!(
        "{}{}ACTION: {}({})\n",
        prefix, connector, step.action.tool_name, params
    ));
}

/// Renders the OBSERVATION part of a step, including any errors or logs.
fn render_observation_node(output: &mut String, step: &TraceStep, prefix: &str, is_last: bool) {
    let connector = if is_last { "└─ " } else { "├─ " };
    output.push_str(&format!(
        "{}{}OBSERVATION: {}\n",
        prefix, connector, step.observation.last_transaction_status
    ));

    // Determine the prefix for the children of the observation node.
    let child_prefix = if is_last { "   " } else { "│  " };
    let new_prefix = format!("{}{}", prefix, child_prefix);

    let logs = &step.observation.last_transaction_logs;

    // Render the error, if it exists.
    if let Some(error) = &step.observation.last_transaction_error {
        // The error is the last item only if there are no logs.
        let is_last_child = logs.is_empty();
        let error_connector = if is_last_child { "└─ " } else { "├─ " };
        // Take only the first line of the error for a concise tree view.
        let error_text = format!("Error: {}", error.lines().next().unwrap_or(""));
        output.push_str(&format!(
            "{}{}{}\n",
            new_prefix, error_connector, error_text
        ));
    }

    // Render any transaction logs.
    let num_logs = logs.len();
    for (i, log) in logs.iter().enumerate() {
        let is_last_child = i == num_logs - 1;
        let log_connector = if is_last_child { "└─ " } else { "├─ " };
        output.push_str(&format!("{}{}{}\n", new_prefix, log_connector, log));
    }
}
