-- Reev Database Schema
-- Current consolidated schema for unified database architecture
-- Version: 1.0 (Phase 25 - Unified Logging System)

-- Core Tables
CREATE TABLE IF NOT EXISTS benchmarks (
    id TEXT PRIMARY KEY,
    benchmark_name TEXT NOT NULL,
    prompt TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at INTEGER DEFAULT (strftime('%s', 'now'))
);

CREATE TABLE IF NOT EXISTS execution_sessions (
    session_id TEXT PRIMARY KEY,
    benchmark_id TEXT NOT NULL,
    agent_type TEXT NOT NULL,
    interface TEXT NOT NULL,
    start_time INTEGER NOT NULL,
    end_time INTEGER,
    status TEXT NOT NULL DEFAULT 'running',
    score REAL,
    final_status TEXT,
    log_file_path TEXT,
    created_at INTEGER DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY (benchmark_id) REFERENCES benchmarks (id)
);

CREATE TABLE IF NOT EXISTS session_logs (
    session_id TEXT PRIMARY KEY,
    content TEXT NOT NULL,
    file_size INTEGER,
    created_at INTEGER DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY (session_id) REFERENCES execution_sessions (session_id)
);

CREATE TABLE IF NOT EXISTS agent_performance (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL,
    benchmark_id TEXT NOT NULL,
    agent_type TEXT NOT NULL,
    score REAL NOT NULL,
    final_status TEXT NOT NULL,
    execution_time_ms INTEGER,
    created_at INTEGER DEFAULT (strftime('%s', 'now')),
    prompt_md5 TEXT,
    FOREIGN KEY (session_id) REFERENCES execution_sessions (session_id),
    FOREIGN KEY (benchmark_id) REFERENCES benchmarks (id)
);

CREATE TABLE IF NOT EXISTS schema_version (
    version TEXT PRIMARY KEY,
    applied_at INTEGER DEFAULT (strftime('%s', 'now')),
    description TEXT
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_benchmarks_name ON benchmarks(benchmark_name);
CREATE INDEX IF NOT EXISTS idx_execution_sessions_benchmark_agent ON execution_sessions(benchmark_id, agent_type);
CREATE INDEX IF NOT EXISTS idx_execution_sessions_interface ON execution_sessions(interface);
CREATE INDEX IF NOT EXISTS idx_execution_sessions_status ON execution_sessions(status);
CREATE INDEX IF NOT EXISTS idx_execution_sessions_start_time ON execution_sessions(start_time);
CREATE INDEX IF NOT EXISTS idx_session_logs_created_at ON session_logs(created_at);
CREATE INDEX IF NOT EXISTS idx_agent_performance_session_id ON agent_performance(session_id);
CREATE INDEX IF NOT EXISTS idx_agent_performance_prompt_md5 ON agent_performance(prompt_md5);
CREATE INDEX IF NOT EXISTS idx_agent_performance_score ON agent_performance(score);
CREATE INDEX IF NOT EXISTS idx_agent_performance_created_at ON agent_performance(created_at);

-- Initial data (skip auto-insertion for compatibility)
-- INSERT OR IGNORE INTO schema_version (version, description) VALUES ('1.0', 'Phase 25: Unified logging system with session management');

CREATE TABLE IF NOT EXISTS execution_sessions (
    session_id TEXT PRIMARY KEY,
    benchmark_id TEXT NOT NULL,
    agent_type TEXT NOT NULL,
    interface TEXT NOT NULL,
    start_time INTEGER NOT NULL,
    end_time INTEGER,
    status TEXT NOT NULL DEFAULT 'running',
    score REAL,
    final_status TEXT,
    log_file_path TEXT,
    created_at INTEGER DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY (benchmark_id) REFERENCES benchmarks (id)
);

CREATE TABLE IF NOT EXISTS session_logs (
    session_id TEXT PRIMARY KEY,
    content TEXT NOT NULL,
    file_size INTEGER,
    created_at INTEGER DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY (session_id) REFERENCES execution_sessions (session_id)
);

CREATE TABLE IF NOT EXISTS agent_performance (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL,
    benchmark_id TEXT NOT NULL,
    agent_type TEXT NOT NULL,
    score REAL NOT NULL,
    final_status TEXT NOT NULL,
    execution_time_ms INTEGER,
    created_at INTEGER DEFAULT (strftime('%s', 'now')),
    prompt_md5 TEXT,
    FOREIGN KEY (session_id) REFERENCES execution_sessions (session_id),
    FOREIGN KEY (benchmark_id) REFERENCES benchmarks (id)
);

CREATE TABLE IF NOT EXISTS session_tool_calls (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL,
    tool_name TEXT NOT NULL,
    start_time INTEGER NOT NULL,
    execution_time_ms INTEGER NOT NULL,
    input_params TEXT NOT NULL,
    output_result TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('in_progress', 'success', 'error', 'timeout')),
    error_message TEXT,
    created_at INTEGER DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY (session_id) REFERENCES execution_sessions (session_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS schema_version (
    version TEXT PRIMARY KEY,
    applied_at INTEGER DEFAULT (strftime('%s', 'now')),
    description TEXT
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_benchmarks_name ON benchmarks(benchmark_name);
CREATE INDEX IF NOT EXISTS idx_execution_sessions_benchmark_agent ON execution_sessions(benchmark_id, agent_type);
CREATE INDEX IF NOT EXISTS idx_execution_sessions_interface ON execution_sessions(interface);
CREATE INDEX IF NOT EXISTS idx_execution_sessions_status ON execution_sessions(status);
CREATE INDEX IF NOT EXISTS idx_execution_sessions_start_time ON execution_sessions(start_time);
CREATE INDEX IF NOT EXISTS idx_session_logs_created_at ON session_logs(created_at);
CREATE INDEX IF NOT EXISTS idx_agent_performance_session_id ON agent_performance(session_id);
CREATE INDEX IF NOT EXISTS idx_agent_performance_prompt_md5 ON agent_performance(prompt_md5);
CREATE INDEX IF NOT EXISTS idx_agent_performance_score ON agent_performance(score);
CREATE INDEX IF NOT EXISTS idx_agent_performance_created_at ON agent_performance(created_at);

-- Indexes for session_tool_calls table
CREATE INDEX IF NOT EXISTS idx_session_tool_calls_session_id ON session_tool_calls(session_id);
CREATE INDEX IF NOT EXISTS idx_session_tool_calls_tool_name ON session_tool_calls(tool_name);
CREATE INDEX IF NOT EXISTS idx_session_tool_calls_status ON session_tool_calls(status);
CREATE INDEX IF NOT EXISTS idx_session_tool_calls_start_time ON session_tool_calls(start_time);
CREATE INDEX IF NOT EXISTS idx_session_tool_calls_session_tool ON session_tool_calls(session_id, tool_name);
CREATE INDEX IF NOT EXISTS idx_session_tool_calls_consolidation ON session_tool_calls(session_id, tool_name, start_time);
CREATE INDEX IF NOT EXISTS idx_session_tool_calls_updated_at ON session_tool_calls(updated_at);

-- Initial data (skip auto-insertion for compatibility)
-- INSERT OR IGNORE INTO schema_version (version, description) VALUES ('1.0', 'Phase 25: Unified logging system with session management');
