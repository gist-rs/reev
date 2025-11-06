#!/bin/bash

# Debug script to check integration test database state
echo "🔍 Debugging Integration Test Database State"
echo "========================================="

# Find latest test database
DB_FILE=$(find . -name "test_db_integration_*.db" 2>/dev/null | sort | tail -1)

if [ -z "$DB_FILE" ]; then
    echo "❌ No test database found"
    exit 1
fi

echo "📊 Database: $DB_FILE"
echo ""

# Check execution_states table
echo "📋 Execution States:"
sqlite3 "$DB_FILE" "SELECT execution_id, status, created_at, updated_at FROM execution_states ORDER BY created_at DESC;"

echo ""
echo "🔍 Looking for integration test execution..."
sqlite3 "$DB_FILE" "SELECT execution_id, status, CASE
    WHEN status = 0 THEN 'Queued'
    WHEN status = 1 THEN 'Running'
    WHEN status = 2 THEN 'Completed'
    WHEN status = 3 THEN 'Failed'
    ELSE 'Unknown'
END as status_text
FROM execution_states
WHERE execution_id LIKE '%integration-test%'
ORDER BY created_at DESC;"

echo ""
echo "📊 Total execution states:"
sqlite3 "$DB_FILE" "SELECT COUNT(*) as total_count FROM execution_states;"

echo ""
echo "📁 Session files created:"
find logs/sessions -name "*integration-test*" -type f -exec ls -la {} \;

echo ""
echo "📋 Latest session file content (last 20 lines):"
LATEST_SESSION=$(find logs/sessions -name "session_integration-test-*.json" -type f | sort | tail -1)
if [ -n "$LATEST_SESSION" ]; then
    tail -20 "$LATEST_SESSION"
else
    echo "No session file found"
fi
