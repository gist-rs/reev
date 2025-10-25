-- Add session_tool_calls table for detailed tool execution tracking
-- Version: 1.1 (Phase 26 - Tool Call Detail Tracking)

-- Table for storing detailed tool execution information per session
CREATE TABLE IF NOT EXISTS session_tool_calls (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL,
    tool_name TEXT NOT NULL,
    start_time INTEGER NOT NULL,
    execution_time_ms INTEGER NOT NULL,
    input_params TEXT NOT NULL, -- JSON string
    output_result TEXT NOT NULL, -- JSON string
    status TEXT NOT NULL CHECK (status IN ('success', 'error', 'timeout')),
    error_message TEXT,
    created_at INTEGER DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY (session_id) REFERENCES execution_sessions (session_id) ON DELETE CASCADE
);

-- Indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_session_tool_calls_session_id ON session_tool_calls(session_id);
CREATE INDEX IF NOT EXISTS idx_session_tool_calls_tool_name ON session_tool_calls(tool_name);
CREATE INDEX IF NOT EXISTS idx_session_tool_calls_status ON session_tool_calls(status);
CREATE INDEX IF NOT EXISTS idx_session_tool_calls_start_time ON session_tool_calls(start_time);
CREATE INDEX IF NOT EXISTS idx_session_tool_calls_session_tool ON session_tool_calls(session_id, tool_name);

-- Update schema version
INSERT OR REPLACE INTO schema_version (version, description) VALUES (
    '1.1',
    'Phase 26: Added session_tool_calls table for detailed tool execution tracking'
);
