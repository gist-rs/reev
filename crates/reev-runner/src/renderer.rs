use ascii_tree::{Tree, write_tree};
use reev_lib::{results::TestResult, trace::TraceStep};
use solana_sdk::{bs58, instruction::AccountMeta};

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
    let score_percent = result.score * 100.0;
    let root_label = format!(
        "{} {} (Score: {:.1}%): {}",
        status_icon, result.id, score_percent, result.final_status
    );

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
/// - `is_signer`: `true` -> `🖋️`, `false` -> `🖍️`
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

    // The `action` is now a Vec<AgentAction>. We handle all cases gracefully.
    let action_str = if let Some(first_action) = step.action.first() {
        let instruction = &first_action.0;
        let program_id = format!("Program ID: {}", instruction.program_id);
        let accounts_str = format_accounts(&instruction.accounts);
        let data_str = format!(
            "Data (Base58): {}",
            bs58::encode(&instruction.data).into_string()
        );

        let mut output =
            format!("     {program_id}\n     Accounts:\n{accounts_str}\n     {data_str}");

        // If there are more instructions, add a note to indicate a multi-instruction transaction.
        if step.action.len() > 1 {
            output.push_str(&format!(
                "\n     (+ {} more instructions in this transaction)",
                step.action.len() - 1
            ));
        }
        output
    } else {
        "     No action was taken.".to_string()
    };

    let action_node = Tree::Leaf(vec![format!("ACTION:\n{}", action_str)]);

    // Generate inline transaction logs for this step
    let mut transaction_children = Vec::new();
    let logs = &step.observation.last_transaction_logs;
    if !logs.is_empty() {
        let parsed_logs = parse_transaction_logs_inline(logs);
        for entry in &parsed_logs {
            render_log_entry_inline(&mut transaction_children, entry, 1);
        }
    }

    let transaction_node = Tree::Node("TRANSACTION LOGS:".to_string(), transaction_children);

    // Check if this is a special benchmark type by looking at the info field
    let observation_label = match step.info.get("type").and_then(|v| v.as_str()) {
        Some("flow_benchmark") => "OBSERVATION: Flow Completed".to_string(),
        Some("api_benchmark") => "OBSERVATION: API Query Completed".to_string(),
        _ => format!("OBSERVATION: {}", step.observation.last_transaction_status),
    };

    let mut observation_children = Vec::new();
    if let Some(error) = &step.observation.last_transaction_error {
        observation_children.push(Tree::Leaf(vec![format!("Error: {}", error)]));
    } else if let Some(message) = step.info.get("message").and_then(|v| v.as_str()) {
        observation_children.push(Tree::Leaf(vec![message.to_string()]));
    }

    let observation_node = Tree::Node(observation_label, observation_children);

    Tree::Node(
        step_label,
        vec![action_node, transaction_node, observation_node],
    )
}

/// Parsed transaction log entry for inline rendering
#[derive(Debug, Clone)]
struct LogEntryInline {
    level: usize,
    program_id: Option<String>,
    program_name: Option<String>,
    instruction: Option<String>,
    log_message: Option<String>,
    compute_units: Option<u64>,
    is_success: bool,
    is_last_child: bool,
}

/// Parse raw transaction log lines into structured entries (inline version)
fn parse_transaction_logs_inline(logs: &[String]) -> Vec<LogEntryInline> {
    let mut entries = Vec::new();
    let mut program_stack: Vec<(String, usize)> = Vec::new();

    // Pre-compile regex patterns for performance
    let program_invoke_regex =
        regex::Regex::new(r"Program ([a-zA-Z0-9]+) invoke \[(\d+)\]").unwrap();
    let compute_units_regex = regex::Regex::new(r"consumed (\d+) of (\d+) compute units").unwrap();

    for log_line in logs {
        let trimmed = log_line.trim();

        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }

        // Parse program invocation
        if let Some(caps) = program_invoke_regex.captures(trimmed) {
            let program_id = caps[1].to_string();
            let level = caps[2].parse::<usize>().unwrap_or(0);

            // Update stack for nesting
            while program_stack.len() > level {
                program_stack.pop();
            }
            program_stack.push((program_id.clone(), entries.len()));

            entries.push(LogEntryInline {
                level,
                program_id: Some(program_id.clone()),
                program_name: get_program_name_inline(&program_id),
                instruction: None,
                log_message: None,
                compute_units: None,
                is_success: false,
                is_last_child: false,
            });
            continue;
        }

        // Parse compute units
        if let Some(caps) = compute_units_regex.captures(trimmed) {
            let compute_units = caps[1].parse::<u64>().unwrap_or(0);
            // Find the most recent entry that doesn't have compute units yet
            for entry in entries.iter_mut().rev() {
                if entry.compute_units.is_none() {
                    entry.compute_units = Some(compute_units);
                    break;
                }
            }
            continue;
        }

        // Parse instruction log
        if trimmed.contains("Program log: Instruction:") {
            let instruction = trimmed
                .replace("Program log: Instruction:", "")
                .trim()
                .to_string();
            if let Some(entry) = entries.last_mut() {
                entry.instruction = Some(instruction);
            }
            continue;
        }

        // Parse program log
        if trimmed.contains("Program log:") && !trimmed.contains("Instruction:") {
            let log_msg = trimmed.replace("Program log:", "").trim().to_string();
            if let Some(entry) = entries.last_mut() {
                entry.log_message = Some(log_msg);
            }
            continue;
        }

        // Parse program success
        if trimmed.contains("Program") && trimmed.contains("success") {
            if let Some(program_id) = extract_program_id_from_success_inline(trimmed) {
                // Check if this program already has a success entry to avoid duplicates
                let has_success = entries.iter().any(|entry| {
                    entry.program_id.as_ref() == Some(&program_id) && entry.is_success
                });

                if !has_success {
                    // Find the most recent program invocation with this ID
                    if let Some(program_level) = entries
                        .iter()
                        .rev()
                        .find(|entry| {
                            entry.program_id.as_ref() == Some(&program_id) && !entry.is_success
                        })
                        .map(|entry| entry.level)
                    {
                        // Add success as a child of this program
                        entries.push(LogEntryInline {
                            level: program_level + 1, // Child level
                            program_id: Some(program_id.clone()),
                            program_name: get_program_name_inline(&program_id),
                            instruction: None,
                            log_message: None,
                            compute_units: None,
                            is_success: true,
                            is_last_child: false,
                        });
                    }
                }
            }
            continue;
        }
    }

    // Mark last children for proper tree rendering
    mark_last_children_inline(&mut entries);
    entries
}

/// Get human-readable program name from program ID (inline version)
fn get_program_name_inline(program_id: &str) -> Option<String> {
    match program_id {
        "11111111111111111111111111111111" => Some("System".to_string()),
        "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA" => Some("SPL Token".to_string()),
        "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL" => Some("Associated Token".to_string()),
        "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4" => Some("Jupiter Router".to_string()),
        "TessVdML9pBGgG9yGks7o4HewRaXVAMuoVj4x83GLQH" => Some("Tessellate".to_string()),
        "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM" => Some("Serum DEX".to_string()),
        _ => None,
    }
}

/// Extract program ID from success message (inline version)
fn extract_program_id_from_success_inline(message: &str) -> Option<String> {
    regex::Regex::new(r"Program ([a-zA-Z0-9]+) success")
        .unwrap()
        .captures(message)
        .map(|caps| caps[1].to_string())
}

/// Mark which entries are the last child of their parent (inline version)
fn mark_last_children_inline(entries: &mut [LogEntryInline]) {
    let mut i = 0;
    while i < entries.len() {
        let current_level = entries[i].level;
        let mut j = i + 1;

        // Find next entry at same or higher level
        while j < entries.len() && entries[j].level > current_level {
            j += 1;
        }

        // If no next entry at same level, this is last child
        if j >= entries.len() || entries[j].level < current_level {
            entries[i].is_last_child = true;
        }

        i += 1;
    }
}

/// Render a single log entry as ASCII tree for execution trace (inline version)
fn render_log_entry_inline(children: &mut Vec<Tree>, entry: &LogEntryInline, _base_indent: usize) {
    // Build proper tree indentation with vertical connectors
    let prefix = build_tree_prefix_inline(entry.level, entry.is_last_child);

    // Get appropriate icon
    let icon = get_program_icon_inline(&entry.program_id);

    // Render program invocation
    if entry.program_name.is_some() && !entry.is_success {
        let label = format!(
            "{}{} {} ({})",
            prefix,
            icon,
            entry.program_name.as_deref().unwrap_or("Unknown"),
            entry.program_id.as_deref().unwrap_or("")
        );

        let mut sub_children = Vec::new();

        // Build child prefix (adds vertical connector for this node's children)
        let child_prefix = build_child_prefix_inline(entry.level, entry.is_last_child);

        // Render instruction if present
        if let Some(instruction) = &entry.instruction {
            sub_children.push(Tree::Leaf(vec![format!(
                "{}├─ 📋 Instruction: {}",
                child_prefix, instruction
            )]));
        }

        // Render log message if present
        if let Some(log_msg) = &entry.log_message {
            if log_msg.starts_with("Please upgrade") {
                sub_children.push(Tree::Leaf(vec![format!(
                    "{}├─ ⚠️  Log: {}",
                    child_prefix, log_msg
                )]));
            } else {
                sub_children.push(Tree::Leaf(vec![format!(
                    "{}├─ 📝 Log: {}",
                    child_prefix, log_msg
                )]));
            }
        }

        // Render success if present
        if let Some(cu) = entry.compute_units {
            sub_children.push(Tree::Leaf(vec![format!(
                "{}└─ ✅ Success ({} CU)",
                child_prefix, cu
            )]));
        }

        children.push(Tree::Node(label, sub_children));
    }
    // Render standalone success entry
    else if entry.is_success {
        let label = if let Some(cu) = entry.compute_units {
            format!("{prefix}{icon} ✅ Success ({cu} CU)")
        } else {
            format!("{prefix}{icon} ✅ Success")
        };
        children.push(Tree::Leaf(vec![label]));
    }
}

/// Build proper tree prefix with vertical connectors for each level (inline version)
fn build_tree_prefix_inline(level: usize, is_last_child: bool) -> String {
    if level == 0 {
        return String::new();
    }

    let mut prefix = String::new();

    // For each level above the current one, add appropriate connector
    for i in 0..level {
        if i == level - 1 {
            // This is the direct parent level
            if is_last_child {
                prefix.push_str("└─ ");
            } else {
                prefix.push_str("├─ ");
            }
        } else {
            // Higher levels need vertical connectors if they have siblings below
            prefix.push_str("│  ");
        }
    }

    prefix
}

/// Build prefix for child elements (adds vertical connector for current node) (inline version)
fn build_child_prefix_inline(level: usize, is_last_child: bool) -> String {
    if level == 0 {
        return "  ".to_string();
    }

    let mut prefix = String::new();

    // For each level, add appropriate connector
    for i in 0..level {
        if i == level - 1 {
            // This is the current level - children need continuation
            if is_last_child {
                prefix.push_str("   ");
            } else {
                prefix.push_str("│  ");
            }
        } else {
            // Higher levels need vertical connectors
            prefix.push_str("│  ");
        }
    }

    prefix
}

/// Get appropriate icon for program type (inline version)
fn get_program_icon_inline(program_id: &Option<String>) -> &'static str {
    match program_id.as_deref() {
        Some("11111111111111111111111111111111") => "🔹",
        Some("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") => "🪙",
        Some("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL") => "🏦",
        Some("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4") => "🚀",
        Some("TessVdML9pBGgG9yGks7o4HewRaXVAMuoVj4x83GLQH") => "🔸",
        Some("9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM") => "📈",
        Some(_) => "📄",
        None => "❓",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reev_lib::agent::{AgentAction, AgentObservation};
    use reev_lib::trace::TraceStep;
    use solana_sdk::instruction::{AccountMeta, Instruction};
    use solana_sdk::pubkey::Pubkey;
    use std::str::FromStr;

    #[test]
    fn test_inline_transaction_log_parsing() {
        let logs = vec![
            "Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL invoke [1]".to_string(),
            "Program log: CreateIdempotent".to_string(),
            "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]".to_string(),
            "Program log: Instruction: GetAccountDataSize".to_string(),
            "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 1569 of 997595 compute units".to_string(),
            "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success".to_string(),
            "Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL consumed 19315 of 1003000 compute units".to_string(),
            "Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL success".to_string(),
        ];

        let parsed = parse_transaction_logs_inline(&logs);

        // Should have at least 2 entries: AToken invoke and Token invoke
        assert!(parsed.len() >= 2);

        // Check first entry (AToken invoke)
        assert_eq!(parsed[0].level, 1);
        assert_eq!(
            parsed[0].program_id,
            Some("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL".to_string())
        );
        assert_eq!(parsed[0].program_name, Some("Associated Token".to_string()));
        assert_eq!(parsed[0].log_message, Some("CreateIdempotent".to_string()));

        // Check second entry (Token invoke)
        assert_eq!(parsed[1].level, 2);
        assert_eq!(
            parsed[1].program_id,
            Some("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string())
        );
        assert_eq!(
            parsed[1].instruction,
            Some("GetAccountDataSize".to_string())
        );
        assert_eq!(parsed[1].compute_units, Some(1569));

        // Check that we have success entries
        let success_entries: Vec<_> = parsed.iter().filter(|e| e.is_success).collect();
        assert!(!success_entries.is_empty());
    }

    #[test]
    fn test_execution_trace_with_transaction_logs() {
        // Create a mock trace step with transaction logs
        let logs = vec![
            "Program 11111111111111111111111111111111 invoke [0]".to_string(),
            "Program log: Instruction: Transfer".to_string(),
            "Program 11111111111111111111111111111111 consumed 2000 of 200000 compute units"
                .to_string(),
            "Program 11111111111111111111111111111111 success".to_string(),
        ];

        let step = TraceStep {
            thought: None,
            action: vec![AgentAction(Instruction::new_with_bytes(
                Pubkey::from_str("11111111111111111111111111111111").unwrap(),
                &[],
                vec![AccountMeta::new(
                    Pubkey::from_str("11111111111111111111111111111111").unwrap(),
                    true,
                )],
            ))],
            observation: AgentObservation {
                last_transaction_status: "Success".to_string(),
                last_transaction_logs: logs,
                last_transaction_error: None,
                account_states: std::collections::HashMap::new(),
                key_map: std::collections::HashMap::new(),
            },
            info: serde_json::json!({}),
        };

        let result = TestResult {
            id: "test-benchmark".to_string(),
            prompt: "Test prompt".to_string(),
            final_status: reev_lib::results::FinalStatus::Succeeded,
            score: 1.0,
            trace: reev_lib::trace::ExecutionTrace {
                steps: vec![step],
                prompt: "Test prompt".to_string(),
            },
        };

        // This should render without panicking
        let rendered = render_result_as_tree(&result);

        // Should contain the transaction logs section
        assert!(rendered.contains("TRANSACTION LOGS:"));
        assert!(rendered.contains("🔹 System"));
        assert!(rendered.contains("📋 Instruction: Transfer"));
        assert!(rendered.contains("✅ Success (2000 CU)"));
    }
}
