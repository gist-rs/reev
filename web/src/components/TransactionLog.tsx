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

    // When running, use live execution data and clear previous logs
    if (isRunning && execution?.logs) {
      const newLogs = execution.logs
        .split("\n")
        .filter((line) => line.trim() !== "");

      // Clear previous logs and show current execution logs
      setFlowLog({
        events: newLogs,
        final_result: {
          status: execution.status,
          progress: execution.progress,
          trace: execution.trace,
        },
      });
      return;
    }

    // For completed runs or when no execution data, load from database
    setLoading(true);
    setError(null);

    try {
      // Get transaction logs from backend endpoint
      const transactionLogData =
        await apiClient.getTransactionLogs(benchmarkId);
      setFlowLog(transactionLogData);
    } catch (err) {
      setError(
        err instanceof Error ? err.message : "Failed to load transaction logs",
      );
    } finally {
      setLoading(false);
    }
  }, [benchmarkId, isRunning, execution]);

  // Clear logs when benchmark changes
  useEffect(() => {
    setFlowLog(null);
  }, [benchmarkId]);

  // Auto-refresh for running executions - use live data instead of database polling
  useEffect(() => {
    if (!autoRefresh || !isRunning || !benchmarkId) return;

    // During execution, update from live execution data instead of polling database
    const interval = setInterval(() => {
      if (execution?.logs) {
        const newLogs = execution.logs
          .split("\n")
          .filter((line) => line.trim() !== "");

        setFlowLog({
          events: newLogs,
          final_result: {
            status: execution.status,
            progress: execution.progress,
            trace: execution.trace,
          },
        });
      }
    }, 1000); // More frequent updates for real-time feel
    return () => {
      clearInterval(interval);
      if (import.meta.env.DEV) {
        console.log("Cleared TransactionLog real-time interval");
      }
    };
  }, [autoRefresh, isRunning, benchmarkId, execution]);

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

  const formatTransactionLogs = (logData: any) => {
    if (!logData) return "No transaction logs available";

    // Extract transaction logs from the response
    const logs = logData.transaction_logs || "";

    if (!logs || logs.trim() === "") {
      return "No transaction logs available";
    }

    return logs;
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
                {formatTransactionLogs(flowLog)}
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
