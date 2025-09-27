use ascii_tree::{Tree, write_tree};
use reev_lib::{results::TestResult, trace::TraceStep};
use solana_sdk::instruction::AccountMeta;

/// Renders a `TestResult` object into a human-readable ASCII tree format.
///
/// This provides a quick, high-level overview of the agent's execution trace
/// directly in the terminal.
pub fn render_result_as_tree(result: &TestResult) -> String {
    let status_icon = if result.final_status == reev_lib::results::FinalStatus::Succeeded {
        "⭕️"
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

/// Formats the accounts of a transaction into a compact, readable format.
///
/// - `is_signer`: `true` -> `💲`, `false` -> `✖️`
/// - `is_writable`: `true` -> `➕`, `false` -> `➖`
fn format_accounts(accounts: &[AccountMeta]) -> String {
    accounts
        .iter()
        .enumerate()
        .map(|(i, account)| {
            let signer_icon = if account.is_signer {
                "🖋️"
            } else {
                "🖍️"
            };
            let writable_icon = if account.is_writable { "➕" } else { "➖" };
            format!(
                "     [{:2}] {} {} {}",
                i, signer_icon, writable_icon, account.pubkey
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Renders a single `TraceStep` into a `Tree` node for the ASCII tree.
fn render_step_node(step_number: usize, step: &TraceStep) -> Tree {
    let step_label = format!("Step {step_number}");

    // Manually format the AgentAction for a more compact view instead of using JSON.
    let instruction = &step.action.0;
    let program_id = format!("Program ID: {}", instruction.program_id);
    let accounts_str = format_accounts(&instruction.accounts);
    let data_str = format!(
        "Data (Base58): {}",
        bs58::encode(&instruction.data).into_string()
    );

    let action_str = format!("     {program_id}\n     Accounts:\n{accounts_str}\n     {data_str}");

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
