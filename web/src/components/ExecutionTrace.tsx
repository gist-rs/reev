// ExecutionTrace component for real-time execution monitoring and log display

import { useState, useEffect, useRef, useMemo } from "preact/hooks";
import { ExecutionState } from "../types/configuration";

// Configuration for trace display limits
const MAX_TRACE_LINES = 1000; // Maximum lines to display
const VIRTUAL_SCROLL_THRESHOLD = 200; // Lines above which to use virtual scrolling
const VIRTUAL_ITEM_HEIGHT = 20; // Height of each line in pixels
const VIRTUAL_BUFFER_SIZE = 10; // Extra items to render above/below viewport

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

      // Clear any existing timeouts before setting new ones
      const timeout1 = setTimeout(scrollToBottom, 100);
      const timeout2 = setTimeout(scrollToBottom, 500);

      // Cleanup function to clear timeouts
      return () => {
        clearTimeout(timeout1);
        clearTimeout(timeout2);
      };
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
          <TraceDisplay
            traceLines={traceLines}
            execution={execution}
            isRunning={isRunning}
          />
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

// Virtualized trace display component
function TraceDisplay({
  traceLines,
  execution,
  isRunning,
}: {
  traceLines: string[];
  execution: ExecutionState | null;
  isRunning: boolean;
}) {
  const [showFullTrace, setShowFullTrace] = useState(false);
  const [viewportHeight, setViewportHeight] = useState(400);
  const containerRef = useRef<HTMLDivElement>(null);
  const scrollElementRef = useRef<HTMLDivElement>(null);

  // Limit trace lines to prevent memory issues
  const limitedTraceLines = useMemo(() => {
    if (traceLines.length <= MAX_TRACE_LINES) {
      return traceLines;
    }
    return traceLines.slice(0, MAX_TRACE_LINES);
  }, [traceLines]);

  // Determine if we should use virtual scrolling
  const useVirtualScroll = limitedTraceLines.length > VIRTUAL_SCROLL_THRESHOLD;

  // Update viewport height
  useEffect(() => {
    const updateHeight = () => {
      if (containerRef.current) {
        setViewportHeight(containerRef.current.clientHeight);
      }
    };
    updateHeight();
    window.addEventListener("resize", updateHeight);
    return () => window.removeEventListener("resize", updateHeight);
  }, []);

  // Simple virtual scrolling implementation
  const visibleRange = useMemo(() => {
    if (!useVirtualScroll || !scrollElementRef.current) {
      return { start: 0, end: limitedTraceLines.length };
    }

    const scrollTop = scrollElementRef.current.scrollTop;
    const start = Math.max(
      0,
      Math.floor(scrollTop / VIRTUAL_ITEM_HEIGHT) - VIRTUAL_BUFFER_SIZE,
    );
    const visibleCount = Math.ceil(viewportHeight / VIRTUAL_ITEM_HEIGHT);
    const end = Math.min(
      limitedTraceLines.length,
      start + visibleCount + VIRTUAL_BUFFER_SIZE * 2,
    );

    return { start, end };
  }, [useVirtualScroll, limitedTraceLines.length, viewportHeight]);

  const visibleLines = useMemo(() => {
    if (!useVirtualScroll) {
      return limitedTraceLines;
    }
    return limitedTraceLines.slice(visibleRange.start, visibleRange.end);
  }, [useVirtualScroll, limitedTraceLines, visibleRange]);

  if (traceLines.length > MAX_TRACE_LINES && !showFullTrace) {
    return (
      <div className="p-4 text-center">
        <div className="text-yellow-600 dark:text-yellow-400 mb-4">
          <div className="text-4xl mb-2">⚠️</div>
          <h3 className="text-lg font-semibold mb-2">Large Trace Detected</h3>
          <p className="mb-4">
            This trace contains {traceLines.length.toLocaleString()} lines.
            Showing first {MAX_TRACE_LINES.toLocaleString()} lines for
            performance.
          </p>
          <button
            onClick={() => setShowFullTrace(true)}
            className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors"
          >
            Show Full Trace (May Impact Performance)
          </button>
        </div>
      </div>
    );
  }

  if (useVirtualScroll) {
    return (
      <div
        ref={containerRef}
        className="relative overflow-auto"
        style={{ height: "400px" }}
      >
        <div
          ref={scrollElementRef}
          style={{
            height: `${limitedTraceLines.length * VIRTUAL_ITEM_HEIGHT}px`,
          }}
        >
          <div
            style={{
              transform: `translateY(${visibleRange.start * VIRTUAL_ITEM_HEIGHT}px)`,
              position: "absolute",
              top: 0,
              left: 0,
              right: 0,
            }}
          >
            {visibleLines.map((line, index) => {
              const actualIndex = visibleRange.start + index;
              return (
                <div
                  key={`${actualIndex}-${execution?.trace?.length || 0}`}
                  className={`whitespace-pre-wrap ${getLineClass(line)}`}
                  style={{ height: `${VIRTUAL_ITEM_HEIGHT}px` }}
                >
                  {line}
                </div>
              );
            })}
          </div>
        </div>

        {/* Status indicators */}
        {isRunning && (
          <div className="absolute bottom-4 left-4 flex items-center space-x-2 text-blue-400 dark:text-blue-300 animate-pulse bg-black/80 px-3 py-2 rounded">
            <span>●</span>
            <span>Execution in progress...</span>
          </div>
        )}

        {execution?.status === "Completed" && (
          <div className="absolute bottom-4 left-4 flex items-center space-x-2 text-green-400 dark:text-green-300 font-semibold bg-black/80 px-3 py-2 rounded">
            <span>✓</span>
            <span>Execution completed</span>
          </div>
        )}

        {/* Line counter */}
        <div className="absolute top-4 right-4 text-xs text-gray-400 bg-black/80 px-2 py-1 rounded">
          Showing {visibleRange.start + 1}-
          {Math.min(visibleRange.end, limitedTraceLines.length)} of{" "}
          {limitedTraceLines.length} lines
        </div>
      </div>
    );
  }

  // Non-virtualized rendering for smaller traces
  return (
    <div className="space-y-1">
      {visibleLines.map((line, index) => (
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
  );
}
