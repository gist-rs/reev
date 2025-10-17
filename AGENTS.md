- always diag after edit, fix diag, follow PLAN.md TASKS.md, update plan PLAN.md TASKS.md when complete
- remove what done from to TOFIX.md, add what to fix in TOFIX.md
- when fixed add reflect to REFLECT.md
- keep all md under 320 lines, keep only important and use short word, less noise.
- Follow modular architecture, Keep files under 320-512 lines
- ask user for commit when done impl or fixed and run success result without error and warning with git convention e.g. `feat:...`, `fix:...`, `refactor:...` for each task, but wait for confirm first.
- run `cargo clippy --fix --allow-dirty`  before commit.
- don't add a test on each file, do add test to tests folder.
- prefer `match` more than `if`.
- use early `return` condition instead of `else`.
- don't run server and get stuck, do run server in background, use cargo watch e.g. `nohup cargo watch -w crates/reev-api -x "run -p reev-api --bin reev-api" > logs/reev-api.log 2>&1 &` so server will reflect the latest code.
- always diag!

## 🔄 Handover Process

### When Taking Over Development
1. **Read HANDOVER.md first** - Contains complete project state and architecture
2. **Review PLAN.md and TASKS.md** - Understand current progress and next priorities
3. **Check compilation status** - Run `cargo check` to identify any issues
4. **Run critical tests** - `cargo test -p reev-db --test session_management`
5. **Update documentation** - Reflect any changes in planning docs

### During Development
1. **Maintain session consistency** - TUI and Web must produce identical DB records
2. **Use unified database interface** - All DB operations through modular writer
3. **Test interface consistency** - Verify TUI/Web behavior matches

### When Completing Work
1. **Update HANDOVER.md** - Reflect current state and remaining issues
2. **Update PLAN.md/TASKS.md** - Mark completed phases, update progress
3. **Add to REFLECT.md** - Document learnings and architectural decisions
4. **Run full test suite** - Ensure no regressions before handover
5. **Create commit with clear message** - Wait for confirmation before pushing

### Handover Checklist
- [ ] HANDOVER.md updated with current state
- [ ] All compilation errors resolved
- [ ] Session management tests passing
- [ ] PLAN.md and TASKS.md reflect current progress
- [ ] No files exceed 512 lines
- [ ] TUI and Web interfaces tested for consistency
