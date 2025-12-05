fix it one by one with smaller change step by step and stop each step and ask me to con so we can consider commit for each, be concise, no fallback, no legacy, no mock in prod code, use serde_json, serde_yml to deserialze untyped to struct as possible. dont add test in file, use tests folder.
---
action must be enum, strum, no string allow
        match action {
            "transfer" => PromptAction::Transfer,
            "swap" => PromptAction::Swap,
            "lend" => PromptAction::Lend,
            "borrow" => PromptAction::Borrow,
            "earn" => PromptAction::Earn,
            _ => PromptAction::Unknown,
        }

    /// Total portfolio value in USD
    pub total_value_usd: f64,
---
```
// Convert amount string to u64
let amount_u64 = amount.parse::<f64>().unwrap_or(0.0) as u64;

let params = serde_json::json!({
    "input_mint": input_mint,
    "output_mint": output_mint,
    "input_amount": amount,
    "input_amount": amount_u64,
});
```
something



ai messup crates/reev-core/src/execution/rig_agent/mod.rs after impl PLAN_LLM.md
can you check with crates/reev-core/src/execution/rig_agent/mod_old.rs what logic is missing?
ideally it should modify only zai_client related but somehow it's mess up all the logic
or maybe just cp old file and mod only zai_client related
```
cargo test -p reev-core --test e2e_simple_transfer -- --nocapture
```
to proove it work after edit
