## DO
- DO add test to tests folder.
- Always diag after edit, fix diag error, follow PLAN.md TASKS.md, update plan PLAN.md TASKS.md when complete
- Add what to fix in TOFIX.md for fix anything and updated when fixed.
- Then fixed add reflect to REFLECT.md, but keep it short.
- Try to keep all md under 320 lines, keep only important and use short word, less noise, keep revise for compact.
- Follow modular architecture, Keep coding files under 320-512 lines, sparate to smalelr files if need.
- Always run `cargo clippy --fix --allow-dirty` and fix after done edit.
- Ask user for commit when done impl or fixed for each task and run success result without error with git convention e.g. `feat:...`, `fix:...`, `refactor:...` for each task, but wait for confirm first.
- use `types.rs`, `mod.rs` is for index only, keep `main.rs` and `lib.rs` minimal, less or no logic.
- prefer `match` more than `if`.
- use early `return` condition instead of `else`.
- when user say `handover` you must summarize current state and incomplpeted issues to HANDOVER.md for next thread can follow up easily, refer to TASKS.md number if has.
- use macos cli commad
- search for related code, tests, examples to get an idea and current context before impl
- fix warning diag but be reason about unused because maybe it meant to use, remove if not used

## DONT
- DONT Add a test on each file, place it in tests folder.
- DONT run server and get stuck, do run server in background, use cargo watch e.g. `nohup cargo watch -w crates/reev-api -x "run -p reev-api --bin reev-api" > logs/reev-api.log 2>&1 &` so server will reflect the latest code.
