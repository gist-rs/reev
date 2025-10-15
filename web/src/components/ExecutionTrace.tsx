// ExecutionTrace component for real-time execution monitoring and log display

import { useState, useEffect, useRef } from "preact/hooks";
import { ExecutionState } from "../types/configuration";

interface ExecutionTraceProps {
  execution: ExecutionState | null;
  isRunning: boolean;
  className?: string;
}

// Helper function to split trace into lines for display
function getTraceLines(trace: string): string[] {
  if (!trace || trace.trim() === "") {
    return [];
  }
  return trace.split("\n");
}

// Helper function to get CSS class for trace lines based on content
function getLineClass(line: string): string {
  const trimmed = line.trim();

  if (trimmed.includes("✅") || trimmed.includes("Succeeded")) {
    return "text-green-400";
  } else if (
    trimmed.includes("❌") ||
    trimmed.includes("Failed") ||
    trimmed.includes("Error:")
  ) {
    return "text-red-400";
  } else if (trimmed.includes("⚠️") || trimmed.includes("Warning")) {
    return "text-yellow-400";
  } else if (trimmed.includes("Step")) {
    return "text-blue-400 font-semibold";
  } else if (trimmed.includes("ACTION:")) {
    return "text-cyan-400";
  } else if (trimmed.includes("OBSERVATION:")) {
    return "text-purple-400";
  } else if (trimmed.includes("Program ID:")) {
    return "text-gray-300";
  } else if (trimmed.includes("Accounts:") || trimmed.includes("Data")) {
    return "text-gray-400";
  } else if (
    trimmed.includes("🖋️") ||
    trimmed.includes("🖍️") ||
    trimmed.includes("➕") ||
    trimmed.includes("➖")
  ) {
    return "text-gray-300";
  } else if (trimmed.startsWith("     ")) {
    return "text-gray-300";
  } else {
    return "text-gray-200";
  }
}

export function ExecutionTrace({
  execution,
  isRunning,
  className = "",
}: ExecutionTraceProps) {
  const [autoScroll, setAutoScroll] = useState(true);
  const traceRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to bottom when new content is added
  useEffect(() => {
    if (autoScroll && traceRef.current) {
      traceRef.current.scrollTop = traceRef.current.scrollHeight;
    }
  }, [execution?.trace, autoScroll]);

  // Also auto-scroll when execution completes - more aggressive
  useEffect(() => {
    if (autoScroll && traceRef.current && execution?.status === "Completed") {
      // Multiple attempts to ensure scroll works
      const scrollToBottom = () => {
        if (traceRef.current) {
          traceRef.current.scrollTop = traceRef.current.scrollHeight;
        }
      };

      // Immediate scroll
      scrollToBottom();

      // Delayed scroll to ensure content is rendered
      setTimeout(scrollToBottom, 100);

      // Another delayed scroll for safety
      setTimeout(scrollToBottom, 500);
    }
  }, [execution?.status, autoScroll]);

  const handleScroll = () => {
    if (traceRef.current) {
      const { scrollTop, scrollHeight, clientHeight } = traceRef.current;
      const isAtBottom = scrollTop + clientHeight >= scrollHeight - 10;
      setAutoScroll(isAtBottom);
    }
  };

  const formatTimestamp = (timestamp: string) => {
    try {
      const date = new Date(timestamp);
      return date.toLocaleTimeString();
    } catch {
      return timestamp;
    }
  };

  const handleCopyTrace = () => {
    if (execution?.trace) {
      navigator.clipboard.writeText(execution.trace);
    }
  };

  const handleClearTrace = () => {
    // This would need to be handled by parent component
    console.log("Clear trace requested");
  };

  if (!execution) {
    return (
      <div className={`h-full flex flex-col ${className}`}>
        <div className="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
          <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
            Execution Trace
          </h2>
          <div className="flex space-x-2">
            <button
              className="px-3 py-1 text-sm bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300 rounded hover:bg-gray-200 dark:hover:bg-gray-600 disabled:opacity-50"
              disabled
            >
              Copy
            </button>
          </div>
        </div>
        <div className="flex-1 flex items-center justify-center text-gray-500 dark:text-gray-400">
          <div className="text-center">
            <div className="text-6xl mb-4">📋</div>
            <p>No execution selected</p>
            <p className="text-sm">
              Select a benchmark to see execution details
            </p>
          </div>
        </div>
      </div>
    );
  }

  const traceLines = getTraceLines(execution?.trace || "");

  return (
    <div className={`h-full flex flex-col ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800">
        <div className="flex items-center space-x-3">
          <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
            Execution Trace
          </h2>
          <div className="flex items-center space-x-2">
            {/* Status Badge */}
            <span
              className={`px-2 py-1 text-xs font-medium rounded-full ${
                execution.status === "Running"
                  ? "bg-yellow-100 dark:bg-yellow-900/20 text-yellow-800 dark:text-yellow-400"
                  : execution.status === "Completed"
                    ? "bg-green-100 dark:bg-green-900/20 text-green-800 dark:text-green-400"
                    : execution.status === "Failed"
                      ? "bg-red-100 dark:bg-red-900/20 text-red-800 dark:text-red-400"
                      : "bg-gray-100 dark:bg-gray-700 text-gray-800 dark:text-gray-300"
              }`}
            >
              {execution.status}
            </span>

            {/* Progress */}
            {execution.status === "Running" && (
              <div className="flex items-center space-x-2">
                <div className="w-24 bg-gray-200 dark:bg-gray-700 rounded-full h-2">
                  <div
                    className="bg-blue-600 h-2 rounded-full transition-all duration-300"
                    style={{ width: `${execution.progress}%` }}
                  ></div>
                </div>
                <span className="text-sm text-gray-600 dark:text-gray-400">
                  {execution.progress}%
                </span>
              </div>
            )}
          </div>
        </div>

        <div className="flex items-center space-x-2">
          {/* Auto-scroll indicator */}
          <div className="flex items-center space-x-1">
            <input
              type="checkbox"
              id="auto-scroll"
              checked={autoScroll}
              onChange={(e) => setAutoScroll(e.currentTarget.checked)}
              className="rounded text-blue-600 focus:ring-blue-500"
            />
            <label
              htmlFor="auto-scroll"
              className="text-sm text-gray-600 dark:text-gray-400"
            >
              Auto-scroll
            </label>
          </div>

          {/* Action buttons */}
          <button
            onClick={handleCopyTrace}
            disabled={!execution?.trace}
            className="px-3 py-1 text-sm bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded hover:bg-gray-200 dark:hover:bg-gray-600 disabled:opacity-50 transition-colors"
          >
            Copy
          </button>

          {isRunning && (
            <button className="px-3 py-1 text-sm bg-red-100 dark:bg-red-900/20 text-red-700 dark:text-red-400 rounded hover:bg-red-200 dark:hover:bg-red-900/30 transition-colors">
              Stop
            </button>
          )}
        </div>
      </div>

      {/* Execution Info */}
      <div className="px-4 py-2 bg-gray-50 dark:bg-gray-900/50 border-b border-gray-200 dark:border-gray-700 text-sm text-gray-600 dark:text-gray-400">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-4">
            <span>
              <strong>Benchmark:</strong> {execution.benchmark_id}
            </span>
            <span>
              <strong>Agent:</strong> {execution.agent}
            </span>
          </div>
          <div className="flex items-center space-x-4">
            <span>
              <strong>Started:</strong> {formatTimestamp(execution.start_time)}
            </span>
            {execution.end_time && (
              <span>
                <strong>Ended:</strong> {formatTimestamp(execution.end_time)}
              </span>
            )}
          </div>
        </div>
      </div>

      {/* Trace Content */}
      {/* Trace Display */}
      <div
        ref={traceRef}
        className="flex-1 bg-gray-900 dark:bg-black font-mono text-sm p-4 overflow-auto border border-gray-700 dark:border-gray-600"
        style={{
          minHeight: "200px",
          maxHeight: "calc(100vh - 400px)",
        }}
      >
        {traceLines.length === 0 ? (
          <div className="text-gray-500 dark:text-gray-400 text-center py-8">
            {execution?.status === "Running"
              ? "Waiting for execution output..."
              : "No trace output available"}
          </div>
        ) : (
          <div key={execution?.trace?.length || 0} className="space-y-1">
            {console.log("=== RENDERING TRACE LINES ===")}
            {console.log("Total lines:", traceLines.length)}
            {console.log("Execution status:", execution?.status)}
            {console.log(
              "First line preview:",
              traceLines[0]?.substring(0, 50),
            )}
            {traceLines.map((line, index) => (
              <div
                key={`${index}-${execution?.trace?.length || 0}`}
                className={`whitespace-pre-wrap ${getLineClass(line)}`}
              >
                {line}
              </div>
            ))}

            {/* Show loading indicator when running */}
            {execution?.status === "Running" && (
              <div className="flex items-center space-x-2 text-blue-400 dark:text-blue-300 animate-pulse">
                <span>●</span>
                <span>Execution in progress...</span>
              </div>
            )}

            {/* Show completion indicator */}
            {execution?.status === "Completed" && (
              <div className="flex items-center space-x-2 text-green-400 dark:text-green-300 font-semibold">
                <span>✓</span>
                <span>Execution completed - Full trace displayed above</span>
              </div>
            )}
          </div>
        )}
      </div>

      {/* Error Display */}
      {execution.error && (
        <div className="p-4 bg-red-50 dark:bg-red-900/20 border-t border-red-200 dark:border-red-700">
          <div className="flex items-start space-x-2">
            <span className="text-red-600 dark:text-red-400 font-semibold">
              Error:
            </span>
            <span className="text-red-700 dark:text-red-300 text-sm">
              {execution.error}
            </span>
          </div>
        </div>
      )}
    </div>
  );
}
