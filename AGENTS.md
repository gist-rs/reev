## DO
- DO add test to tests folder.
- Create/refine issues in `ISSUES.md` before fix anything and updated when fixed, keep only last 10 issues.
- When remove old isses do reflect critical one in `REFLECT.md`.
- Try to keep all md under 320-512 lines, keep only important and use short word, less noise, keep revise for compact.
- Follow modular architecture, Keep coding files under 320-512 lines, sparate to smalelr files if need, askfor separation if number of lines exceed limits.
- Always run `cargo clippy --fix --allow-dirty` and fix after done edit for Rust files, for frontend just run diagnostic.
- Ask user for commit when done impl or fixed for each task and run success result without error with git convention e.g. `feat:...`, `fix:...`, `refactor:...` for each task with important list of impl and shrot reflect for later ref, but wait for confirm first.
- `mod.rs` is for index only, keep `main.rs` and `lib.rs` minimal, less or no logic, use `types.rs` for keep struct and impl that not depends on (can decouple from logic when other use it).
- prefer `match` more than `if`.
- use early `return` condition instead of `else`.
- when user say `handover` you must summarize current state, current debug method for current issue, to `HANDOVER.md` for next thread so we can follow up easily, refer to `ISSUES.md` number/title if has.
- use macos cli commad not linux .e.g `timeout` is not exist use `sleep`.
- search for related code, tests, examples to get an idea and current context before impl.
- if user say `regression` try dig to git history to get more idea.
- fix warning diag before commit but be reason about unused because it maybe meant to use but we forget to use, remove if not used
- Always diagnostic and fix error before run server.
- Must use `RUST_LOG=info` and `--quiet` to reduce log noise and better filter the log by target keyword.
- RESTRICT jupiter_earn tool to position/earnings benchmarks (114-*.yml) only, never add to normal agent tool list.
- when stuck, try to find what working and start adding until it break.

## DONT
- DONT Add a test on each file, place it in tests folder.
- DONT run server and get stuck, do run server in background, use cargo watch e.g. `nohup cargo ...`.
k
