# Database Schema Directory

This directory contains the database schema definitions and migrations for the Reev framework.

## 📁 Structure

```
.schema/
├── README.md                           # This file
├── 001_initial_schema.sql              # Initial database schema
├── 002_add_indexes.sql                 # Performance indexes
└── current_schema.sql                  # Consolidated current schema
```

## 🗄️ Schema Overview

### Core Tables

1. **`benchmarks`** - Benchmark definitions with MD5-based deduplication
2. **`execution_sessions`** - Unified session tracking for TUI and Web interfaces
3. **`session_logs`** - Complete JSON logs from SessionFileLogger
4. **`agent_performance`** - Performance metrics and analytics

### Key Features

- **Unified Session Management**: Single session tracking for both TUI and Web interfaces
- **Foreign Key Constraints**: Referential integrity across all tables
- **Performance Indexes**: Optimized queries for all common access patterns
- **MD5 Deduplication**: Prevent duplicate benchmarks using content hashing
- **JSON Logging**: Structured session logs with database persistence

## 🔄 Migration System

### Naming Convention
- Format: `XXX_description.sql`
- XXX: Sequential 3-digit number (001, 002, etc.)
- description: Brief, snake_case description of changes

### Applying Migrations
Migrations are currently applied through the consolidated schema in `current_schema.sql`. 
Future versions may implement a proper migration runner.

### Schema Versioning
- No active migration system implemented
- Schema managed through consolidated `current_schema.sql` file
- Version tracking handled in documentation rather than database

## 🛠️ Development Guidelines

### Adding New Tables
1. Create new migration file (next sequential number)
2. Define CREATE TABLE statement with proper constraints
3. Add appropriate indexes in separate migration
4. Update `current_schema.sql` with consolidated changes

### Adding Indexes
1. Create new migration for index additions
2. Follow naming convention: `idx_table_column(s)`
3. Consider query patterns and performance impact
4. Update `current_schema.sql`

### Schema Changes
1. Always create new migration - never modify existing ones
2. Use `IF NOT EXISTS` for backward compatibility
3. Test migrations on fresh and existing databases
4. Update documentation and version tracking

## 📊 Current Schema Version

**Version**: 1.0 (Phase 25 - Unified Logging System)

**Features**:
- Unified session management across TUI/Web interfaces
- SessionFileLogger integration with JSON persistence
- Performance optimization with comprehensive indexing
- Foreign key constraints for data integrity

## 🔍 Usage

### In Development
The schema is loaded at compile time using Rust's `include_str!` macro:
```rust
const CURRENT_SCHEMA: &str = include_str!("../.schema/current_schema.sql");
```

### Database Initialization
1. Load consolidated schema from `current_schema.sql`
2. Execute all CREATE and INDEX statements
3. Verify schema version in `schema_version` table
- Initialize database health checks

### Schema Validation
- Compare deployed schema with `current_schema.sql`
- Verify all indexes exist and are being used
- Check foreign key constraints are enforced
- Validate data types and constraints

## 🚀 Future Enhancements

- **Migration System**: Consider implementing proper migration runner if needed
- **Schema Diff Tools**: Automated change detection
- **Environment-specific schemas**: Dev/staging/prod variations

---

*This schema supports the Reev framework's unified logging system and modernized database architecture implemented in Phase 25.*