// TransactionLog component for detailed transaction log viewing

import { useState, useCallback, useEffect, useRef } from "preact/hooks";
import { apiClient } from "../services/api";

interface TransactionLogProps {
  benchmarkId: string | null;
  execution: any;
  isRunning: boolean;
}

export function TransactionLog({
  benchmarkId,
  execution,
  isRunning,
}: TransactionLogProps) {
  const [flowLog, setFlowLog] = useState<any>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [autoRefresh, setAutoRefresh] = useState(true);
  const logContainerRef = useRef<HTMLDivElement>(null);
  const [shouldScrollToBottom, setShouldScrollToBottom] = useState(true);

  // Load flow logs from database and execution state
  const loadFlowLog = useCallback(async () => {
    if (!benchmarkId) return;

    setLoading(true);
    setError(null);

    try {
      // Try to get flow logs from database first
      const logData = await apiClient.getFlowLog(benchmarkId);
      setFlowLog(logData);
    } catch (err) {
      // If database fails, try to get from current execution state
      setError(err instanceof Error ? err.message : "Failed to load flow log");
    } finally {
      setLoading(false);
    }
  }, [benchmarkId]);

  // Update flow log from execution state when running
  useEffect(() => {
    if (isRunning && execution?.logs) {
      const newLogs = execution.logs
        .split("\n")
        .filter((line) => line.trim() !== "");
      setFlowLog((prev) => {
        // Append new logs instead of replacing
        if (prev && prev.events) {
          const existingEvents = prev.events;
          const newEvents = newLogs.filter(
            (log) => !existingEvents.includes(log),
          );
          return {
            ...prev,
            events: [...existingEvents, ...newEvents],
            final_result: {
              status: execution.status,
              progress: execution.progress,
              trace: execution.trace,
            },
          };
        }
        return {
          events: newLogs,
          final_result: {
            status: execution.status,
            progress: execution.progress,
            trace: execution.trace,
          },
        };
      });
    }
  }, [isRunning, execution]);

  // Auto-refresh for running executions
  useEffect(() => {
    if (!autoRefresh || !isRunning || !benchmarkId) return;

    const interval = setInterval(loadFlowLog, 2000);
    return () => {
      clearInterval(interval);
      if (import.meta.env.DEV) {
        console.log("Cleared TransactionLog polling interval");
      }
    };
  }, [autoRefresh, isRunning, benchmarkId, loadFlowLog]);

  // Auto-scroll to bottom when new content is added
  useEffect(() => {
    if (shouldScrollToBottom && logContainerRef.current) {
      const container = logContainerRef.current;
      container.scrollTop = container.scrollHeight;
    }
  }, [flowLog, shouldScrollToBottom]);

  // Handle scroll to detect if user is manually scrolling
  const handleScroll = useCallback(() => {
    if (logContainerRef.current) {
      const container = logContainerRef.current;
      const isAtBottom =
        container.scrollHeight - container.scrollTop <=
        container.clientHeight + 10;
      setShouldScrollToBottom(isAtBottom);
    }
  }, []);

  // Load on mount and when execution changes
  useEffect(() => {
    loadFlowLog();
  }, [loadFlowLog]);

  const clearLogs = useCallback(() => {
    setFlowLog(null);
    setError(null);
  }, []);

  const exportLogs = useCallback(() => {
    const data = JSON.stringify(flowLog, null, 2);
    const blob = new Blob([data], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `flow-log-${benchmarkId}-${execution?.id || "unknown"}.json`;
    a.click();
    URL.revokeObjectURL(url);
  }, [flowLog, benchmarkId]);

  const getStatusColor = (status: string) => {
    switch (status) {
      case "success":
        return "text-green-600 dark:text-green-400 bg-green-50 dark:bg-green-900/20 border-green-200 dark:border-green-700";
      case "failed":
        return "text-red-600 dark:text-red-400 bg-red-50 dark:bg-red-900/20 border-red-200 dark:border-red-700";
      case "pending":
        return "text-yellow-600 dark:text-yellow-400 bg-yellow-50 dark:bg-yellow-900/20 border-yellow-200 dark:border-yellow-700";
      default:
        return "text-gray-600 dark:text-gray-400 bg-gray-50 dark:bg-gray-900/50 border-gray-200 dark:border-gray-700";
    }
  };

  const formatFlowLog = (logData: any) => {
    if (!logData) return "";

    // Limit processing to prevent memory issues
    const MAX_ITEMS = 100;
    const MAX_STRING_LENGTH = 10000;

    // Handle array of mixed JSON strings and YAML strings
    if (Array.isArray(logData)) {
      const limitedArray = logData.slice(0, MAX_ITEMS);
      return (
        limitedArray
          .map((item, index) => {
            // If it's a JSON string, parse and extract readable information
            if (typeof item === "string" && item.startsWith("{")) {
              try {
                const parsed = JSON.parse(item);

                // Extract key information from JSON flow log
                let result = `📊 Flow Log Entry ${index + 1}\n`;

                if (parsed.session_id) {
                  result += `  Session: ${parsed.session_id}\n`;
                }
                if (parsed.benchmark_id) {
                  result += `  Benchmark: ${parsed.benchmark_id}\n`;
                }
                if (parsed.agent_type) {
                  result += `  Agent: ${parsed.agent_type}\n`;
                }
                if (parsed.start_time && parsed.end_time) {
                  const start = new Date(
                    parsed.start_time.secs_since_epoch * 1000,
                  );
                  const end = new Date(parsed.end_time.secs_since_epoch * 1000);
                  const duration = (end.getTime() - start.getTime()) / 1000;
                  result += `  Duration: ${duration.toFixed(2)}s\n`;
                }
                if (parsed.final_result) {
                  result += `  Status: ${parsed.final_result.success ? "✅ Success" : "❌ Failed"}\n`;
                  result += `  Score: ${(parsed.final_result.score * 100).toFixed(1)}%\n`;
                  if (parsed.final_result.statistics) {
                    result += `  LLM Calls: ${parsed.final_result.statistics.total_llm_calls || 0}\n`;
                    result += `  Tool Calls: ${parsed.final_result.statistics.total_tool_calls || 0}\n`;
                  }
                }
                result += ``;
                return result;
              } catch {
                // If parsing fails, treat as raw string but limit length
                const truncatedItem =
                  item.length > MAX_STRING_LENGTH
                    ? item.substring(0, MAX_STRING_LENGTH) + "... (truncated)"
                    : item;
                return `⚠️ Entry ${index + 1} (Parse Error)\n${truncatedItem.substring(0, 200)}...\n`;
              }
            }

            // If it's a YAML/formatted string with trace data, format it nicely
            if (typeof item === "string") {
              const truncatedItem =
                item.length > MAX_STRING_LENGTH
                  ? item.substring(0, MAX_STRING_LENGTH) + "... (truncated)"
                  : item;

              // Check if it's the YAML trace format with prompt/steps
              if (
                item.includes("id:") &&
                item.includes("prompt:") &&
                item.includes("trace:")
              ) {
                // Parse YAML-like content for better formatting
                const lines = truncatedItem.split("\n");
                let result = `🔍 Execution Trace ${index + 1}\n`;

                for (const line of lines) {
                  if (line.startsWith("id:")) {
                    result += `  📋 ${line}\n`;
                  } else if (line.startsWith("prompt:")) {
                    result += `  💭 ${line}\n`;
                  } else if (line.startsWith("final_status:")) {
                    result += `  🏁 ${line}\n`;
                  } else if (line.startsWith("score:")) {
                    result += `  📊 ${line}\n`;
                  } else if (line.startsWith("trace:")) {
                    result += `  🔍 ${line}\n`;
                  } else if (line.startsWith("  steps:")) {
                    result += `  📝 ${line}\n`;
                  } else if (line.startsWith("    -")) {
                    result += `    ${line}\n`;
                  } else if (line.startsWith("      ")) {
                    result += `     ${line}\n`;
                  } else if (line.trim()) {
                    result += `  ${line}\n`;
                  }
                }
                return result;
              }

              // If it's already formatted with separators, keep it
              if (item.includes("---")) {
                return truncatedItem;
              }

              // Otherwise format as a simple entry
              return `📄 Entry ${index + 1}\n${truncatedItem}\n`;
            }

            // If it's an object, format it with size limit
            if (typeof item === "object") {
              const jsonString = JSON.stringify(item, null, 2);
              const truncatedJson =
                jsonString.length > MAX_STRING_LENGTH
                  ? jsonString.substring(0, MAX_STRING_LENGTH) +
                    "... (truncated)"
                  : jsonString;
              return `📋 Object Entry ${index + 1}\n${truncatedJson}\n`;
            }

            return `📝 Entry ${index + 1}\n${String(item).substring(0, MAX_STRING_LENGTH)}\n`;
          })
          .join("\n") +
        (logData.length > MAX_ITEMS
          ? `\n\n... and ${logData.length - MAX_ITEMS} more entries (truncated for performance)`
          : "")
      );
    }

    // If it's a single JSON string, try to parse it
    if (typeof logData === "string" && logData.startsWith("{")) {
      try {
        const parsed = JSON.parse(logData);
        if (parsed.final_result && parsed.final_result.trace) {
          return parsed.final_result.trace;
        }
        return JSON.stringify(parsed, null, 2);
      } catch {
        return logData;
      }
    }

    // If it's a plain string, return it
    if (typeof logData === "string") {
      return logData;
    }

    // Fallback to JSON format
    return JSON.stringify(logData, null, 2);
  };

  if (!benchmarkId) {
    return (
      <div className="p-4 bg-white dark:bg-gray-800 border dark:border-gray-700 rounded-lg">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-3">
          Transaction Log
        </h3>
        <div className="text-gray-500 dark:text-gray-400 text-center py-8">
          <svg
            class="w-12 h-12 mx-auto mb-3 text-gray-300 dark:text-gray-500"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
            ></path>
          </svg>
          <p>Select a benchmark execution to view flow logs</p>
        </div>
      </div>
    );
  }

  return (
    <div className="p-4 bg-white dark:bg-gray-800 border dark:border-gray-700 rounded-lg w-full min-w-0">
      {/* Header */}
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
          Transaction Log
        </h3>
        <div className="flex items-center space-x-2">
          {isRunning && (
            <div className="flex items-center text-xs text-green-600 dark:text-green-400">
              <div className="w-2 h-2 bg-green-500 rounded-full mr-1 animate-pulse"></div>
              Live
            </div>
          )}
          <button
            onClick={() => setAutoRefresh(!autoRefresh)}
            className={`px-2 py-1 text-xs rounded ${
              autoRefresh
                ? "bg-green-100 dark:bg-green-900/20 text-green-700 dark:text-green-400 border border-green-200 dark:border-green-700"
                : "bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 border border-gray-200 dark:border-gray-700"
            }`}
          >
            Auto-refresh
          </button>
        </div>
      </div>

      {/* Controls */}
      <div className="flex items-center space-x-2 mb-4">
        <div className="flex-1 text-sm text-gray-600 dark:text-gray-400">
          Flow Log YAML/ASCII Tree
        </div>
        <button
          onClick={clearLogs}
          className="px-3 py-1 text-sm bg-red-100 dark:bg-red-900/20 text-red-700 dark:text-red-400 rounded hover:bg-red-200 dark:hover:bg-red-900/30 transition-colors"
        >
          Clear
        </button>
        <button
          onClick={exportLogs}
          disabled={!flowLog}
          className="px-3 py-1 text-sm bg-blue-100 dark:bg-blue-900/20 text-blue-700 dark:text-blue-400 rounded hover:bg-blue-200 dark:hover:bg-blue-900/30 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Export
        </button>
      </div>

      {/* Transaction List */}
      <div className="border dark:border-gray-700 rounded-lg w-full flex-1 flex flex-col min-h-0">
        {loading ? (
          <div className="flex items-center justify-center py-8">
            <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-blue-500 mr-2"></div>
            <span className="text-sm text-gray-600 dark:text-gray-400">
              Loading transactions...
            </span>
          </div>
        ) : error ? (
          <div className="p-4 text-center">
            <div className="text-red-500 dark:text-red-400 mb-2">
              <svg
                class="w-8 h-8 mx-auto"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                />
              </svg>
            </div>
            <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2">
              Failed to load flow logs
            </h3>
            <p className="text-gray-600 dark:text-gray-400 mb-4">{error}</p>
            <button
              onClick={loadFlowLog}
              className="px-3 py-1 text-sm bg-red-600 text-white rounded hover:bg-red-700 transition-colors"
            >
              Retry
            </button>
          </div>
        ) : !flowLog ? (
          <div className="p-4 text-center text-gray-500 dark:text-gray-400">
            No flow log data available for this execution
          </div>
        ) : (
          <div className="p-4 flex flex-col flex-1 min-h-0">
            <div className="text-xs font-medium text-gray-700 dark:text-gray-300 mb-2 flex-shrink-0 flex-wrap">
              <span className="break-words">Transaction Log (Real-time):</span>
              {isRunning && (
                <span className="ml-2 text-green-600 dark:text-green-400 animate-pulse">
                  ● Live
                </span>
              )}
              <button
                onClick={() => setShouldScrollToBottom(true)}
                className="ml-2 text-blue-500 hover:text-blue-600 text-xs whitespace-nowrap"
              >
                {shouldScrollToBottom
                  ? "📌 Auto-scroll"
                  : "📌 Scroll to bottom"}
              </button>
            </div>
            <div
              ref={logContainerRef}
              onScroll={handleScroll}
              className="overflow-auto border border-gray-300 dark:border-gray-700 rounded min-w-0 flex-1"
            >
              <pre className="text-xs bg-gray-900 dark:bg-black text-green-400 dark:text-green-300 p-4 font-mono leading-relaxed whitespace-pre min-w-max">
                {formatFlowLog(flowLog)}
              </pre>
            </div>
            {isRunning && (
              <div className="mt-2 text-xs text-blue-400 dark:text-blue-300 text-center flex-shrink-0">
                Executing: {benchmarkId} - Progress: {execution?.progress || 0}%
              </div>
            )}
          </div>
        )}
      </div>

      {/* Footer Info */}
      <div className="mt-3 text-xs text-gray-500 flex items-center justify-between">
        <span>Flow Log Data</span>
        <span>
          Benchmark: {benchmarkId} | Execution: {execution?.id || "N/A"}
        </span>
      </div>
    </div>
  );
}
