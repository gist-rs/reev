// TransactionLog component for detailed transaction log viewing

import { useState, useCallback, useEffect } from "preact/hooks";
import { apiClient } from "../services/api";

interface TransactionLogProps {
  benchmarkId: string | null;
  executionId: string | null;
  isRunning: boolean;
}

export function TransactionLog({
  benchmarkId,
  executionId,
  isRunning,
}: TransactionLogProps) {
  const [flowLog, setFlowLog] = useState<any>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [autoRefresh, setAutoRefresh] = useState(true);

  // Load flow logs from database
  const loadFlowLog = useCallback(async () => {
    if (!benchmarkId) return;

    setLoading(true);
    setError(null);

    try {
      const logData = await apiClient.getFlowLog(benchmarkId);
      setFlowLog(logData);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load flow log");
    } finally {
      setLoading(false);
    }
  }, [benchmarkId]);

  // Auto-refresh for running executions
  useEffect(() => {
    if (!autoRefresh || !isRunning || !benchmarkId) return;

    const interval = setInterval(loadFlowLog, 2000);
    return () => clearInterval(interval);
  }, [autoRefresh, isRunning, benchmarkId, loadFlowLog]);

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
    a.download = `flow-log-${benchmarkId}.json`;
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

    // If the flow log contains ASCII tree format, return it as-is
    if (logData.events && Array.isArray(logData.events)) {
      return logData.events
        .map((event: any) =>
          typeof event === "string" ? event : JSON.stringify(event, null, 2),
        )
        .join("\n");
    }

    // If it's a YAML/ASCII tree string, return it
    if (typeof logData === "string") {
      return logData;
    }

    // Fallback to JSON format
    return JSON.stringify(logData, null, 2);
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
              Flow Log (YAML/ASCII Tree):
            </div>
            <pre className="text-xs bg-gray-900 text-green-400 p-4 rounded border overflow-x-auto font-mono leading-relaxed">
              {formatFlowLog(flowLog)}
            </pre>
          </div>
        )}
      </div>

      {/* Footer Info */}
      <div className="mt-3 text-xs text-gray-500 flex items-center justify-between">
        <span>Flow Log Data</span>
        <span>Benchmark: {benchmarkId}</span>
      </div>
    </div>
  );
}
