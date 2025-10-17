-- Migration 002: Add Performance Indexes
-- Reev Framework - Database Performance Optimization
-- Applied: Phase 25 Performance Optimization

-- =====================================
-- Benchmark Indexes
-- =====================================

-- Index for benchmark name lookups
CREATE INDEX idx_benchmarks_name ON benchmarks(benchmark_name);

-- =====================================
-- Execution Session Indexes
-- =====================================

-- Composite index for benchmark + agent queries
CREATE INDEX idx_execution_sessions_benchmark_agent ON execution_sessions(benchmark_id, agent_type);

-- Index for interface filtering (TUI vs Web)
CREATE INDEX idx_execution_sessions_interface ON execution_sessions(interface);

-- Index for status filtering (running, completed, failed)
CREATE INDEX idx_execution_sessions_status ON execution_sessions(status);

-- Index for time-based queries and ordering
CREATE INDEX idx_execution_sessions_start_time ON execution_sessions(start_time);

-- =====================================
-- Session Log Indexes
-- =====================================

-- Index for log time-based queries
CREATE INDEX idx_session_logs_created_at ON session_logs(created_at);

-- =====================================
-- Agent Performance Indexes
-- =====================================

-- Index for session-based performance lookups
CREATE INDEX idx_agent_performance_session_id ON agent_performance(session_id);

-- Index for prompt-based performance tracking
CREATE INDEX idx_agent_performance_prompt_md5 ON agent_performance(prompt_md5);

-- Index for score-based analytics and sorting
CREATE INDEX idx_agent_performance_score ON agent_performance(score);

-- Index for time-based performance queries
CREATE INDEX idx_agent_performance_timestamp ON agent_performance(timestamp);

-- =====================================
-- Update Schema Version
-- =====================================

-- Mark this migration as applied
INSERT INTO schema_version (version, description) VALUES (
    '002',
    'Added performance indexes for all major tables'
);
