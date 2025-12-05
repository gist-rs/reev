fix it one by one with smaller change step by step and stop each step and ask me to con so we can consider commit for each, be concise, no fallback, no legacy, no mock, use serde_json, serde_yml to deserialze untyped to struct as possible. dont add test in file, use tests folder.

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
