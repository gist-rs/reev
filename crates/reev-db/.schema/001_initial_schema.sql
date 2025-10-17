-- Migration 001: Initial Database Schema
-- Reev Framework - Unified Database Architecture
-- Applied: Phase 24-25 Implementation

-- =====================================
-- Core Tables
-- =====================================

-- Benchmarks table with MD5 hash-based deduplication
CREATE TABLE benchmarks (
    id TEXT PRIMARY KEY,                    -- MD5 of benchmark_name:prompt
    benchmark_name TEXT NOT NULL,
    prompt TEXT NOT NULL,
    content TEXT NOT NULL,                  -- Full YAML content
    created_at INTEGER DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER DEFAULT (strftime('%s', 'now'))
);

-- Execution sessions for unified tracking (TUI and Web)
CREATE TABLE execution_sessions (
    session_id TEXT PRIMARY KEY,
    benchmark_id TEXT NOT NULL,
    agent_type TEXT NOT NULL,
    interface TEXT NOT NULL,                -- 'tui' or 'web'
    start_time INTEGER NOT NULL,
    end_time INTEGER,
    status TEXT NOT NULL DEFAULT 'running',
    score REAL,
    final_status TEXT,
    log_file_path TEXT,
    created_at INTEGER DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY (benchmark_id) REFERENCES benchmarks (id)
);

-- Complete session logs with JSON content
CREATE TABLE session_logs (
    session_id TEXT PRIMARY KEY,
    content TEXT NOT NULL,                  -- Full JSON log from SessionFileLogger
    file_size INTEGER,
    created_at INTEGER DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY (session_id) REFERENCES execution_sessions (session_id)
);

-- Agent performance metrics for analytics
CREATE TABLE agent_performance (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL,
    benchmark_id TEXT NOT NULL,
    agent_type TEXT NOT NULL,
    score REAL NOT NULL,
    final_status TEXT NOT NULL,
    execution_time_ms INTEGER,
    timestamp INTEGER NOT NULL,
    prompt_md5 TEXT,
    FOREIGN KEY (session_id) REFERENCES execution_sessions (session_id),
    FOREIGN KEY (benchmark_id) REFERENCES benchmarks (id)
);

-- Schema version tracking
CREATE TABLE schema_version (
    version TEXT PRIMARY KEY,
    applied_at INTEGER DEFAULT (strftime('%s', 'now')),
    description TEXT
);

-- Insert initial version
INSERT INTO schema_version (version, description) VALUES (
    '001',
    'Initial schema with unified session management'
);
