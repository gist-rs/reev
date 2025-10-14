// TransactionLog component for detailed transaction log viewing

import { useState, useCallback, useEffect } from "preact/hooks";
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

  // Load flow logs from database and execution state
  const loadFlowLog = useCallback(async () => {
    if (!benchmarkId) return;

    // Prevent unnecessary reloads if we already have data for running executions
    if (isRunning && flowLog) return;

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
  }, [benchmarkId, isRunning, flowLog]);

  // Update flow log from execution state when running
  useEffect(() => {
    if (isRunning && execution?.trace) {
      // During execution, show the current trace
      setFlowLog(execution.trace);
    } else if (!isRunning && execution?.trace) {
      // When execution completes, load the full flow log from database
      loadFlowLog();
    }
  }, [isRunning, execution, loadFlowLog]);

  // Auto-refresh for running executions - optimized to prevent full refresh
  useEffect(() => {
    if (!autoRefresh || !isRunning || !benchmarkId) return;

    const interval = setInterval(() => {
      // Only refresh if we have new data, not on every interval
      loadFlowLog();
    }, 3000); // Increased interval to reduce refresh frequency
    return () => clearInterval(interval);
  }, [autoRefresh, isRunning, benchmarkId, loadFlowLog]);

  // Load on mount and when benchmark changes, but not on every render
  useEffect(() => {
    if (benchmarkId) {
      loadFlowLog();
    }
  }, [benchmarkId]); // Removed loadFlowLog dependency to prevent refresh loops

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
        return "text-green-600 bg-green-50 border-green-200";
      case "failed":
        return "text-red-600 bg-red-50 border-red-200";
      case "pending":
        return "text-yellow-600 bg-yellow-50 border-yellow-200";
      default:
        return "text-gray-600 bg-gray-50 border-gray-200";
    }
  };

  const formatFlowLog = (logData: any) => {
    if (!logData) return "";

    // If it's already a formatted string (from execution trace), return it
    if (
      typeof logData === "string" &&
      logData.includes("└─") &&
      logData.includes("ACTION:")
    ) {
      return logData;
    }

    // Handle array of mixed JSON strings and YAML strings
    if (Array.isArray(logData)) {
      return logData
        .map((item, index) => {
          // If it's a JSON string, parse and format it
          if (typeof item === "string" && item.startsWith("{")) {
            try {
              const parsed = JSON.parse(item);
              if (parsed.final_result && parsed.final_result.trace) {
                // This is a flow log with trace data
                return `╭───────── Execution ${index + 1} ─────────╮\n${parsed.final_result.trace}\n╰─────────────────────────────────────╯\n`;
              }
              return `╭───────── Entry ${index + 1} ──────────╮\n${JSON.stringify(parsed, null, 2)}\n╰─────────────────────────────────────╯\n`;
            } catch {
              // If parsing fails, treat as raw string
              return `╭───────── Entry ${index + 1} ──────────╮\n${item}\n╰─────────────────────────────────────╯\n`;
            }
          }

          // If it's a YAML/formatted string with id/prompt structure, format it nicely
          if (typeof item === "string") {
            // Check if it's the YAML trace format
            if (
              item.includes("id:") &&
              item.includes("prompt:") &&
              item.includes("trace:")
            ) {
              return `╭───────── Execution Trace ${index + 1} ─────────╮\n${item}\n╰──────────────────────────────────────────╯\n`;
            }
            // If it's already formatted with separators, keep it
            if (item.includes("---") || item.includes("╭")) {
              return item;
            }
            // Otherwise format as a simple entry
            return `╭───────── Entry ${index + 1} ──────────╮\n${item}\n╰─────────────────────────────────────╯\n`;
          }

          // If it's an object, format it
          if (typeof item === "object") {
            return `╭───────── Entry ${index + 1} ──────────╮\n${JSON.stringify(item, null, 2)}\n╰─────────────────────────────────────╯\n`;
          }

          return `╭───────── Entry ${index + 1} ──────────╮\n${String(item)}\n╰─────────────────────────────────────╯\n`;
        })
        .join("\n");
    }

    // If it's a single JSON string, try to parse it
    if (typeof logData === "string" && logData.startsWith("{")) {
      try {
        const parsed = JSON.parse(logData);
        if (parsed.final_result && parsed.final_result.trace) {
          return `╭───────── Execution Trace ─────────╮\n${parsed.final_result.trace}\n╰──────────────────────────────────╯\n`;
        }
        return `╭───────── Parsed Data ─────────────╮\n${JSON.stringify(parsed, null, 2)}\n╰─────────────────────────────────────╯\n`;
      } catch {
        return `╭───────── Raw Data ───────────────╮\n${logData}\n╰─────────────────────────────────────╯\n`;
      }
    }

    // If it's a plain string, format it nicely
    if (typeof logData === "string") {
      // Check if it's already formatted
      if (logData.includes("╭") && logData.includes("╰")) {
        return logData;
      }
      return `╭───────── Transaction Log ─────────╮\n${logData}\n╰─────────────────────────────────────╯\n`;
    }

    // Fallback to JSON format with nice formatting
    return `╭───────── Formatted Data ───────────╮\n${JSON.stringify(logData, null, 2)}\n╰─────────────────────────────────────╯\n`;
  };

  if (!benchmarkId) {
    return (
      <div className="p-4 bg-white border rounded-lg">
        <h3 className="text-lg font-semibold mb-3">Transaction Log</h3>
        <div className="text-gray-500 text-center py-8">
          <svg
            class="w-12 h-12 mx-auto mb-3 text-gray-300"
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
    <div className="p-4 bg-white border rounded-lg">
      {/* Header */}
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold">Transaction Log</h3>
        <div className="flex items-center space-x-2">
          {isRunning && (
            <div className="flex items-center text-xs text-green-600">
              <div className="w-2 h-2 bg-green-500 rounded-full mr-1 animate-pulse"></div>
              Live
            </div>
          )}
          <button
            onClick={() => setAutoRefresh(!autoRefresh)}
            className={`px-2 py-1 text-xs rounded ${
              autoRefresh
                ? "bg-green-100 text-green-700 border border-green-200"
                : "bg-gray-100 text-gray-700 border border-gray-200"
            }`}
          >
            Auto-refresh
          </button>
        </div>
      </div>

      {/* Controls */}
      <div className="flex items-center space-x-2 mb-4">
        <div className="flex-1 text-sm text-gray-600">
          Flow Log YAML/ASCII Tree
        </div>
        <button
          onClick={clearLogs}
          className="px-3 py-1 text-sm bg-red-100 text-red-700 rounded hover:bg-red-200 transition-colors"
        >
          Clear
        </button>
        <button
          onClick={exportLogs}
          disabled={!flowLog}
          className="px-3 py-1 text-sm bg-blue-100 text-blue-700 rounded hover:bg-blue-200 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Export
        </button>
      </div>

      {/* Transaction List */}
      <div className="border rounded-lg overflow-hidden">
        {loading ? (
          <div className="flex items-center justify-center py-8">
            <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-blue-500 mr-2"></div>
            <span className="text-sm text-gray-600">
              Loading transactions...
            </span>
          </div>
        ) : error ? (
          <div className="p-4 text-center">
            <div className="text-red-500 mb-2">
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
                ></path>
              </svg>
            </div>
            <p className="text-sm text-red-600 mb-2">{error}</p>
            <button
              onClick={loadFlowLog}
              className="px-3 py-1 text-sm bg-red-600 text-white rounded hover:bg-red-700 transition-colors"
            >
              Retry
            </button>
          </div>
        ) : !flowLog ? (
          <div className="p-4 text-center text-gray-500">
            No flow log data available for this execution
          </div>
        ) : (
          <div className="p-4">
            <div className="text-xs font-medium text-gray-700 mb-2">
              Transaction Log (Real-time):
              {isRunning && (
                <span className="ml-2 text-green-600 animate-pulse">
                  ● Live
                </span>
              )}
            </div>
            <div className="bg-gray-900 rounded border overflow-hidden">
              <div className="text-xs font-medium text-gray-400 px-4 py-2 border-b border-gray-700">
                Transaction Log Output
                {isRunning && (
                  <span className="ml-2 text-green-400 animate-pulse">
                    ● Live
                  </span>
                )}
              </div>
              <pre className="text-xs text-green-400 p-4 overflow-x-auto font-mono leading-relaxed max-h-96 overflow-y-auto">
                {formatFlowLog(flowLog)}
              </pre>
            </div>
            {isRunning && (
              <div className="mt-2 text-xs text-blue-400 text-center">
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
