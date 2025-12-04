    /// Total portfolio value in USD
    pub total_value_usd: f64,

dont add test in file, use folder crates/reev-core/tests

con impl, step by step, be concise, no fallback, no legacy, no mock, use serde_yml deserialzed to struct not manually parse

glm client should be in one place, create reev-llm and copy crates/reev-agent/src/providers then consolidate remain

action must be enum, strum, no string allow
        match action {
            "transfer" => PromptAction::Transfer,
            "swap" => PromptAction::Swap,
            "lend" => PromptAction::Lend,
            "borrow" => PromptAction::Borrow,
            "earn" => PromptAction::Earn,
            _ => PromptAction::Unknown,
        }


why extract_amount_from_refined_prompt fn is not same as [@prompt_processor_tests.rs](file:///Users/katopz/git/gist/reev/crates/reev-core/tests/prompt_processor_tests.rs) ?
we should dry to have only one buggable to crates/reev-core/tests/common

which one is correct btw?
