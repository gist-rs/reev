## DO
- DO add test to tests folder.
- Always diag after edit, fix diag, follow PLAN.md TASKS.md, update plan PLAN.md TASKS.md when complete
- Remove what done from to TOFIX.md, add what to fix in TOFIX.md
- Then fixed add reflect to REFLECT.md, but keep it short as DO and DONT.
- Try to keep all md under 320 lines, keep only important and use short word, less noise, keep revise for compact.
- Follow modular architecture, Keep files under 320-512 lines
- Always run `cargo clippy --fix --allow-dirty` and fix after done edit.
- Ask user for commit when done impl or fixed for each task and run success result without error and warning with git convention e.g. `feat:...`, `fix:...`, `refactor:...` for each task, but wait for confirm first.
- prefer `match` more than `if`.
- use early `return` condition instead of `else`.
- when i say `handover` you must summarize current state and incomplpeted issues to HANDOVER.md for next thread can follow up easily, refer to TASKS.md number if has.

## DONT
- DONT Add a test on each file.
- DONT run server and get stuck, do run server in background, use cargo watch e.g. `nohup cargo watch -w crates/reev-api -x "run -p reev-api --bin reev-api" > logs/reev-api.log 2>&1 &` so server will reflect the latest code.
