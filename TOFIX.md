# TOFIX.md

## Remaining Issues

- **Tool calls not being stored to database**: EnhancedOtelLogger captures tool calls during execution, but reev-runner session completion logic is not extracting and storing these tool calls in the session_tool_calls table. The calls remain in memory and are lost when the session ends.

**Fix needed**: Modify reev-runner session completion to extract tool calls from EnhancedOtelLogger and store them in database before session finalization.
