// ExecutionTrace component for real-time execution monitoring and log display

import { useState, useEffect, useRef } from "preact/hooks";
import { ExecutionState } from "../types/configuration";

interface ExecutionTraceProps {
  execution: ExecutionState | null;
  isRunning: boolean;
  className?: string;
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

  const getTraceLines = (trace: string) => {
    return trace.split("\n").filter((line) => line.trim() !== "");
  };

  const getLineClass = (line: string) => {
    if (line.includes("ERROR") || line.includes("Failed")) {
      return "text-red-600";
    }
    if (line.includes("WARNING") || line.includes("Warning")) {
      return "text-yellow-600";
    }
    if (
      line.includes("SUCCESS") ||
      line.includes("completed") ||
      line.includes("✔")
    ) {
      return "text-green-600";
    }
    if (line.includes("Progress:")) {
      return "text-blue-600";
    }
    return "text-gray-700";
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
        <div className="flex items-center justify-between p-4 border-b">
          <h2 className="text-lg font-semibold">Execution Trace</h2>
          <div className="flex space-x-2">
            <button
              className="px-3 py-1 text-sm bg-gray-100 text-gray-600 rounded hover:bg-gray-200 disabled:opacity-50"
              disabled
            >
              Copy
            </button>
          </div>
        </div>
        <div className="flex-1 flex items-center justify-center text-gray-500">
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

  const traceLines = getTraceLines(execution.trace);

  return (
    <div className={`h-full flex flex-col ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b bg-white">
        <div className="flex items-center space-x-3">
          <h2 className="text-lg font-semibold">Execution Trace</h2>
          <div className="flex items-center space-x-2">
            {/* Status Badge */}
            <span
              className={`px-2 py-1 text-xs font-medium rounded-full ${
                execution.status === "Running"
                  ? "bg-yellow-100 text-yellow-800"
                  : execution.status === "Completed"
                    ? "bg-green-100 text-green-800"
                    : execution.status === "Failed"
                      ? "bg-red-100 text-red-800"
                      : "bg-gray-100 text-gray-800"
              }`}
            >
              {execution.status}
            </span>

            {/* Progress */}
            {execution.status === "Running" && (
              <div className="flex items-center space-x-2">
                <div className="w-24 bg-gray-200 rounded-full h-2">
                  <div
                    className="bg-blue-600 h-2 rounded-full transition-all duration-300"
                    style={{ width: `${execution.progress}%` }}
                  ></div>
                </div>
                <span className="text-sm text-gray-600">
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
            <label htmlFor="auto-scroll" className="text-sm text-gray-600">
              Auto-scroll
            </label>
          </div>

          {/* Action buttons */}
          <button
            onClick={handleCopyTrace}
            disabled={!execution.trace}
            className="px-3 py-1 text-sm bg-gray-100 text-gray-700 rounded hover:bg-gray-200 disabled:opacity-50 transition-colors"
          >
            Copy
          </button>

          {isRunning && (
            <button className="px-3 py-1 text-sm bg-red-100 text-red-700 rounded hover:bg-red-200 transition-colors">
              Stop
            </button>
          )}
        </div>
      </div>

      {/* Execution Info */}
      <div className="px-4 py-2 bg-gray-50 border-b text-sm text-gray-600">
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
      <div
        ref={traceRef}
        className="flex-1 overflow-y-auto p-4 bg-gray-900 font-mono text-sm"
        onScroll={handleScroll}
      >
        {traceLines.length === 0 ? (
          <div className="text-gray-500 text-center py-8">
            {execution.status === "Running"
              ? "Waiting for execution output..."
              : "No trace output available"}
          </div>
        ) : (
          <div className="space-y-1">
            {traceLines.map((line, index) => (
              <div
                key={index}
                className={`whitespace-pre-wrap ${getLineClass(line)}`}
              >
                {line}
              </div>
            ))}

            {/* Show loading indicator when running */}
            {execution.status === "Running" && (
              <div className="flex items-center space-x-2 text-blue-400 animate-pulse">
                <span>●</span>
                <span>Execution in progress...</span>
              </div>
            )}
          </div>
        )}
      </div>

      {/* Error Display */}
      {execution.error && (
        <div className="p-4 bg-red-50 border-t border-red-200">
          <div className="flex items-start space-x-2">
            <span className="text-red-600 font-semibold">Error:</span>
            <span className="text-red-700 text-sm">{execution.error}</span>
          </div>
        </div>
      )}
    </div>
  );
}
