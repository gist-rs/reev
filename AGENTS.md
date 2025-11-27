- DO add test to tests folder.
- Create/refine issues in `ISSUES.md` before fix anything and updated when fixed, keep only last 10 issues.
- Try to keep all md under 320-512 lines, keep only important and use short word, less noise, keep revise for compact.
- Follow modular architecture, Keep coding files under 320-512 lines, sparate to smalelr files if need, askfor separation if number of lines exceed limits.
- Always run `cargo clippy --fix --allow-dirty` and fix after done edit for Rust files, for frontend just run diagnostic.
- Ask user for commit when done impl or fixed for each task and run success result without error with git convention e.g. `feat:...`, `fix:...`, `refactor:...` for each task with important list of impl and shrot reflect for later ref, but wait for confirm first.
- `mod.rs` is for index only, keep `main.rs` and `lib.rs` minimal, less or no logic, use `types.rs` for keep struct and impl that not depends on (can decouple from logic when other use it).
- prefer `match` more than `if`.
- use early `return` condition instead of `else`.
- when user say `handover` you must summarize current state, current debug method for current issue, to `ISSUES.md` for next thread so we can follow up easily, also remove ole done ISSUES.
- use macos cli commad not linux .e.g `timeout` is not exist use `sleep`.
- search for related code, tests, examples to get an idea and current context before impl.
- if user say `regression` try dig to git history to get more idea.
- fix warning diag before commit but be reason about unused because it maybe meant to use but we forget to use, remove if not used
- Always diagnostic and fix error before run server.
- Must use `RUST_LOG=info` and `--quiet` to reduce log noise and better filter the log by target keyword.
- RESTRICT jupiter_earn tool to position/earnings benchmarks (114-*.yml) only, never add to normal agent tool list.
- when stuck, try to find what working and start adding until it break.
- DONT Add a test on each file, place it in tests folder.
- DONT run server and get stuck, do run server in background, use cargo watch e.g. `nohup cargo ...`.

## AI BEHAVIOR PROMPTS
When analyzing plans or code:
- DO read entire document before making suggestions
- DO connect dots between different sections of the plan
- DO ask clarifying questions if unsure about plan sections
- DON'T assume features that aren't explicitly mentioned
- DON'T ignore implementation details in the plan
- DON'T suggest alternatives to approaches explicitly in the plan

## PLAN ANALYSIS PROMPT
"Read PLAN_CORE_V3.md completely, focusing on:
1. How phases connect together (Phase 1 + Phase 2 architecture)
2. Migration strategy sections (what's deprecated vs. what's recommended)
3. YML structure requirements (what fields are needed)
4. Implementation requirements (what needs to be built)

Then identify if current implementation matches these requirements by:
1. Checking if YmlGenerator creates correct YML structure
2. Verifying if flow builders match plan recommendations
3. Ensuring multi-step operations follow V3 approach
4. Confirming no rule-based parsing is used where LLM should decide

DO NOT suggest implementations that contradict these plan sections.
