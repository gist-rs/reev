// BenchmarkList component for interactive benchmark navigation and execution

import { useState, useCallback, useEffect } from "preact/hooks";
import { apiClient } from "../services/api";
import { BenchmarkItem } from "../types/configuration";
import { ExecutionStatus } from "../types/benchmark";
import { AgentConfig } from "./AgentConfig";
import { Tooltip } from "./ui/Tooltip";
import { getBenchmarkStatusColor } from "../utils/benchmarkColors";

interface BenchmarkListProps {
  selectedAgent: string;
  selectedBenchmark: string | null;
  selectedDate?: string | null;
  onBenchmarkSelect: (benchmark: string) => void;
  isRunning: boolean;
  onExecutionStart: (executionId: string) => void;
  onExecutionComplete: (benchmarkId: string, execution: any) => void;
  executions: Map<string, any>;
  updateExecution: (benchmarkId: string, execution: any) => void;
  isRunningAll: boolean;
  setIsRunningAll: (running: boolean) => void;
  setCompletionCallback: (
    callback: (benchmarkId: string, execution: any) => void,
  ) => void;
  runAllCompletionCallback: (benchmarkId: string, execution: any) => void;
  runAllQueue: { current: BenchmarkItem[] };
  currentRunAllIndex: { current: number };
  benchmarks: any;
  loading: boolean;
  error: string | null;
  refetch: () => Promise<void>;
  agentPerformanceData?: any;
  agentPerformanceLoading?: boolean;
  agentPerformanceError?: string | null;
  refreshTrigger?: number;
}

export function BenchmarkList({
  selectedAgent,
  selectedBenchmark,
  selectedDate,
  onBenchmarkSelect,
  isRunning,
  onExecutionStart,
  onExecutionComplete,
  executions,
  updateExecution,
  isRunningAll,
  setIsRunningAll,
  setCompletionCallback,
  runAllCompletionCallback,
  runAllQueue,
  currentRunAllIndex,
  benchmarks,
  loading,
  error,
  refetch,
  agentPerformanceData,
  agentPerformanceLoading,
  agentPerformanceError,
  refreshTrigger = 0,
}: BenchmarkListProps) {
  // Track running benchmarks and their execution IDs for polling
  const [runningBenchmarks, setRunningBenchmarks] = useState<
    Map<string, string>
  >(new Map());
  const [historicalResults, setHistoricalResults] = useState<Map<string, any>>(
    new Map(),
  );

  // State for expand/collapse agent configuration
  const [showAgentConfig, setShowAgentConfig] = useState(false);

  // State for expand/collapse benchmark items
  const [expandedBenchmark, setExpandedBenchmark] = useState<string | null>(
    null,
  );

  // Handle benchmark expansion
  const handleBenchmarkExpand = useCallback(
    (benchmarkId: string) => {
      setExpandedBenchmark(
        expandedBenchmark === benchmarkId ? null : benchmarkId,
      );
    },
    [expandedBenchmark],
  );

  // Auto-expand when benchmark starts running
  useEffect(() => {
    const runningBenchmark = Array.from(runningBenchmarks.keys()).find(
      (benchmarkId) => {
        const execution = executions.get(benchmarkId);
        return execution?.status === ExecutionStatus.RUNNING;
      },
    );

    if (runningBenchmark) {
      setExpandedBenchmark(runningBenchmark);
    }
  }, [executions, runningBenchmarks]);

  // Auto-expand when selectedBenchmark changes from external source (e.g., grid click)
  useEffect(() => {
    if (selectedBenchmark) {
      // Check if any benchmark is currently running
      const runningBenchmark = Array.from(executions.keys()).find(
        (benchmarkId) => {
          const execution = executions.get(benchmarkId);
          return execution?.status === ExecutionStatus.RUNNING;
        },
      );

      // Only expand if no benchmark is running (to avoid interfering with auto-expand during execution)
      if (!runningBenchmark) {
        setExpandedBenchmark(selectedBenchmark);
      }
    }
  }, [selectedBenchmark, executions]);

  // Clean up completed benchmarks from runningBenchmarks
  useEffect(() => {
    const completedBenchmarks = Array.from(runningBenchmarks.keys()).filter(
      (benchmarkId) => {
        const execution = executions.get(benchmarkId);
        return (
          execution?.status === ExecutionStatus.COMPLETED ||
          execution?.status === ExecutionStatus.FAILED
        );
      },
    );

    if (completedBenchmarks.length > 0) {
      setRunningBenchmarks((prev) => {
        const updated = new Map(prev);
        completedBenchmarks.forEach((benchmarkId) => {
          updated.delete(benchmarkId);
        });
        return updated;
      });
    }
  }, [executions, runningBenchmarks]);

  // Handle focus change - collapse when other benchmark is selected
  const handleBenchmarkClick = useCallback(
    (benchmarkId: string) => {
      onBenchmarkSelect(benchmarkId);
      setExpandedBenchmark(
        expandedBenchmark === benchmarkId ? null : benchmarkId,
      );
    },
    [expandedBenchmark, onBenchmarkSelect, selectedAgent],
  );

  // Use agent performance data passed as props instead of duplicate API call

  // Process shared data into results map with date grouping
  useEffect(() => {
    console.log(
      `🔍 [BenchmarkList] Processing data for selectedAgent: ${selectedAgent}`,
    );
    console.log(
      `🔍 [BenchmarkList] Available agents:`,
      agentPerformanceData?.map((a) => a.agent_type),
    );

    // Clear historical results when selectedAgent changes
    setHistoricalResults(new Map());

    if (agentPerformanceData) {
      const resultsMap = new Map();
      let resultsCount = 0;

      // Group results by benchmarkId and date for the selected agent
      agentPerformanceData.forEach((agentSummary) => {
        if (agentSummary.agent_type === selectedAgent) {
          console.log(
            `🔍 [BenchmarkList] Found agent ${selectedAgent} with ${agentSummary.results.length} results`,
          );
          agentSummary.results.forEach((result) => {
            const date = result.timestamp.substring(0, 10); // YYYY-MM-DD format
            const key = `${result.benchmark_id}|${date}`; // Include date in key

            resultsMap.set(key, {
              ...result,
              status: result.final_status,
              progress: 100,
              execution_id: result.id,
              agent_type: result.agent_type,
              benchmarkId: result.benchmark_id,
              timestamp: result.timestamp, // Ensure timestamp is preserved
              date: date, // Add date field for easy access
            });
            resultsCount++;
          });
        }
      });

      console.log(
        `🔍 [BenchmarkList] Set ${resultsCount} historical results for ${selectedAgent}`,
      );
      setHistoricalResults(resultsMap);
    } else {
      console.log(`🔍 [BenchmarkList] No agent performance data available`);
    }
  }, [agentPerformanceData, selectedAgent]);

  // Force re-render when refreshTrigger changes to update box colors
  useEffect(() => {
    // This effect will trigger a re-render of the component
    // The color calculation in getBenchmarkStatusColor will use updated execution data
  }, [refreshTrigger]);

  // Polling is now handled by useBenchmarkExecution hook - removed duplicate polling
  // This prevents state inconsistency issues between hook state and component state

  const handleRunBenchmark = useCallback(
    async (benchmark: BenchmarkItem, isRunAll: boolean = false) => {
      if (isRunning) {
        return;
      }

      try {
        // Get agent configuration if needed
        let config;
        if (selectedAgent !== "deterministic") {
          try {
            config = await apiClient.getAgentConfig(selectedAgent);
          } catch {
            // No config found, that's okay for now
          }
        }

        const response = await apiClient.runBenchmark(benchmark.id, {
          agent: selectedAgent,
          config,
        });

        // Update shared execution state for parent components

        // Select the benchmark for Execution Details display
        onBenchmarkSelect(benchmark.id);

        updateExecution(benchmark.id, {
          id: response.execution_id,
          benchmark_id: benchmark.id,
          agent: selectedAgent,
          status: "Pending",
          progress: 0,
          start_time: new Date().toISOString(),
          trace: "",
          logs: "",
        });

        setRunningBenchmarks((prev) =>
          new Map(prev).set(benchmark.id, response.execution_id),
        );
        onExecutionStart(response.execution_id);

        // Only set completion callback for individual benchmark runs (not Run All)
        if (!isRunAll) {
          setCompletionCallback((benchmarkId: string, execution: any) => {
            console.log(
              "🔍 Individual benchmark completed:",
              benchmarkId,
              execution.status,
            );
            onExecutionComplete(benchmarkId, execution);
            // Clear the completion callback
            setCompletionCallback(() => () => {});
          });
        }

        // Return the response for Run All to use
        return response;
      } catch (error) {
        console.error("❌ Failed to run benchmark:", error);
        alert(
          `Failed to run benchmark: ${error instanceof Error ? error.message : "Unknown error"}`,
        );
        throw error; // Re-throw so Run All can handle it
      }
    },
    [selectedAgent, isRunning, onExecutionStart, refetch],
  );

  const handleRunAllBenchmarks = useCallback(async () => {
    if (isRunning || !benchmarks) return;

    setIsRunningAll(true);

    // Set up completion callback (from App component)
    setCompletionCallback(runAllCompletionCallback);

    // Auto-select the first benchmark for Execution Details display
    if (benchmarks.benchmarks.length > 0 && !selectedBenchmark) {
      const firstBenchmark = benchmarks.benchmarks[0];
      onBenchmarkSelect(firstBenchmark.id);
    }

    // Initialize queue (managed by App component) - filter out failure test benchmarks
    runAllQueue.current = [...benchmarks.benchmarks].filter(
      (benchmark) =>
        !benchmark.id.includes("003") && !benchmark.id.includes("004"),
    );
    currentRunAllIndex.current = 0;

    console.log(
      "🔍 Run All - Queue:",
      runAllQueue.current.map((b) => b.id),
    );

    // Start first benchmark
    const firstBenchmark = runAllQueue.current[0];

    try {
      await handleRunBenchmark(firstBenchmark, true); // Pass isRunAll=true
    } catch (error) {
      console.error(`Failed to start benchmark ${firstBenchmark.id}:`, error);
      // Continue to next one even on failure
      runAllCompletionCallback(firstBenchmark.id, { status: "Failed", error });
    }
  }, [
    isRunning,
    benchmarks,
    selectedBenchmark,
    onBenchmarkSelect,
    handleRunBenchmark,
    setCompletionCallback,
    runAllCompletionCallback,
  ]);

  const handleRunAllBelow = useCallback(async () => {
    if (isRunning || !benchmarks || !selectedBenchmark) return;

    setIsRunningAll(true);

    // Set up completion callback (from App component)
    setCompletionCallback(runAllCompletionCallback);

    // Find the index of the currently selected benchmark
    const selectedIndex = benchmarks.benchmarks.findIndex(
      (benchmark) => benchmark.id === selectedBenchmark,
    );

    if (selectedIndex === -1) {
      console.error("Selected benchmark not found in benchmarks list");
      setIsRunningAll(false);
      return;
    }

    // Initialize queue starting from selected benchmark - filter out failure test benchmarks
    const allBenchmarks = [...benchmarks.benchmarks];
    const filteredBenchmarks = allBenchmarks.filter(
      (benchmark) =>
        !benchmark.id.includes("003") && !benchmark.id.includes("004"),
    );

    // Find the selected benchmark in the filtered list
    const filteredSelectedIndex = filteredBenchmarks.findIndex(
      (benchmark) => benchmark.id === selectedBenchmark,
    );

    // If selected benchmark is filtered out, start from the next available one
    const startIndex =
      filteredSelectedIndex >= 0
        ? filteredSelectedIndex
        : filteredBenchmarks.findIndex(
            (benchmark) => allBenchmarks.indexOf(benchmark) > selectedIndex,
          );

    if (startIndex === -1 || startIndex >= filteredBenchmarks.length) {
      console.error("No valid benchmarks found to run from selected position");
      setIsRunningAll(false);
      return;
    }

    runAllQueue.current = filteredBenchmarks.slice(startIndex);
    currentRunAllIndex.current = 0;

    console.log(
      "🔍 Run Current & Below - Queue:",
      runAllQueue.current.map((b) => b.id),
    );
    console.log("🔍 Starting from index:", startIndex);

    // Start first benchmark in the filtered queue
    const firstBenchmark = runAllQueue.current[0];

    try {
      console.log("🚀 Starting benchmark:", firstBenchmark.id);
      await handleRunBenchmark(firstBenchmark, true); // Pass isRunAll=true
    } catch (error) {
      console.error(`Failed to start benchmark ${firstBenchmark.id}:`, error);
      // Continue to next one even on failure
      runAllCompletionCallback(firstBenchmark.id, { status: "Failed", error });
    }
  }, [
    isRunning,
    benchmarks,
    selectedBenchmark,
    onBenchmarkSelect,
    handleRunBenchmark,
    setCompletionCallback,
    runAllCompletionCallback,
  ]);

  const getBenchmarkStatus = useCallback(
    (benchmarkId: string, date?: string | null): any => {
      // First check current executions for the selected agent
      const execution = Array.from(executions.values()).find(
        (exec) =>
          exec.benchmark_id === benchmarkId &&
          (!selectedAgent || exec.agent === selectedAgent),
      );

      // Return current execution for selected agent immediately
      if (execution) {
        // Handle case where execution has score but wrong status (race condition handling)
        if (execution.score >= 1.0 && execution.status === "Failed") {
          return {
            ...execution,
            status: "Completed",
          };
        }
        return execution;
      }

      // If no current execution, check historical results for selected agent and date
      if (date) {
        const dateKey = `${benchmarkId}|${date}`;
        const historicalResult = historicalResults.get(dateKey);
        if (historicalResult) {
          // Map historical result status to execution status format
          return {
            ...historicalResult,
            status: historicalResult.final_status,
            progress: 100,
            timestamp: historicalResult.timestamp, // Ensure timestamp is preserved
          };
        }
      } else {
        // Fallback to latest result if no date specified (for backward compatibility)
        let latestResult = null;
        let latestTimestamp = "";

        // Find the latest result for this benchmark
        for (const [key, result] of historicalResults.entries()) {
          if (
            key.startsWith(`${benchmarkId}|`) &&
            (!selectedAgent || result.agent_type === selectedAgent)
          ) {
            if (
              !latestResult ||
              new Date(result.timestamp) > new Date(latestTimestamp)
            ) {
              latestResult = result;
              latestTimestamp = result.timestamp;
            }
          }
        }

        if (latestResult) {
          return {
            ...latestResult,
            status: latestResult.final_status,
            progress: 100,
            timestamp: latestResult.timestamp,
          };
        }
      }

      // No execution found for this benchmark with selected agent
      return null;
    },
    [executions, historicalResults, selectedAgent],
  );

  const getBenchmarkScore = useCallback(
    (benchmarkId: string, date?: string | null): number => {
      const execution = getBenchmarkStatus(benchmarkId, date);
      if (execution?.status === ExecutionStatus.COMPLETED) {
        return execution.score || 1.0;
      }
      return 0;
    },
    [getBenchmarkStatus],
  );

  const getStatusIcon = useCallback((status: ExecutionStatus) => {
    switch (status) {
      case ExecutionStatus.PENDING:
        return "[ ]";
      case ExecutionStatus.RUNNING:
        return "[…]";
      case ExecutionStatus.COMPLETED:
        return "[✓]";
      case ExecutionStatus.FAILED:
        return "[✗]";
      default:
        return "[?]";
    }
  }, []);

  const getStatusColor = useCallback((status: ExecutionStatus) => {
    switch (status) {
      case ExecutionStatus.PENDING:
        return "text-gray-500";
      case ExecutionStatus.RUNNING:
        return "text-yellow-500";
      case ExecutionStatus.COMPLETED:
        return "text-green-500";
      case ExecutionStatus.FAILED:
        return "text-red-500";
      default:
        return "text-gray-500";
    }
  }, []);

  const getScoreColor = useCallback((score: number) => {
    if (score >= 1.0) return "text-green-600 dark:text-green-400";
    if (score >= 0.25) return "text-yellow-600 dark:text-yellow-400";
    return "text-red-600 dark:text-red-400";
  }, []);

  const formatScore = useCallback((score: number) => {
    return `${(score * 100).toFixed(0)}%`;
  }, []);

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-red-500 dark:text-red-400 text-center">
          <p className="font-semibold">Failed to load benchmarks</p>
          <p className="text-sm">{error}</p>
          <button
            onClick={refetch}
            className="mt-2 px-3 py-1 bg-red-500 text-white rounded hover:bg-red-600"
          >
            Retry
          </button>
        </div>
      </div>
    );
  }

  if (!benchmarks || benchmarks.benchmarks.length === 0) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-gray-500 dark:text-gray-400 text-center">
          <p>No benchmarks found</p>
        </div>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
        <div className="flex items-center space-x-3">
          <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
            Benchmarks
          </h2>
          <span className="text-sm text-gray-500 dark:text-gray-400">
            {(() => {
              if (!selectedBenchmark) {
                return "(no selection)";
              }

              // Show selected date if available, otherwise show latest execution date
              if (selectedDate) {
                return `(date: ${selectedDate})`;
              }

              const selectedExecution = getBenchmarkStatus(
                selectedBenchmark,
                selectedDate,
              );
              const selectedTimestamp = selectedExecution?.timestamp;

              if (selectedTimestamp) {
                const latestDate = new Date(selectedTimestamp);
                const formattedDate = latestDate.toISOString().split("T")[0]; // yyyy-mm-dd format
                return `(latest: ${formattedDate})`;
              } else {
                return "(no execution)";
              }
            })()}
          </span>
        </div>
        <div className="flex space-x-2">
          <button
            onClick={handleRunAllBenchmarks}
            disabled={isRunning || isRunningAll}
            className="px-3 py-1 bg-green-600 text-white text-sm rounded hover:bg-green-700 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors"
          >
            {isRunningAll ? "Running All..." : "Run All"}
          </button>
          <button
            onClick={handleRunAllBelow}
            disabled={isRunning || isRunningAll || !selectedBenchmark}
            className="px-3 py-1 bg-blue-600 text-white text-sm rounded hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors"
          >
            {isRunningAll ? "Running..." : "Run Current & Below"}
          </button>
        </div>
      </div>

      {/* Selected Agent Header */}
      <div className="p-4 border-b border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-900/50">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-2">
            <div className="w-2 h-2 bg-blue-600 rounded-full"></div>
            <h3 className="font-medium text-gray-900 dark:text-gray-100 capitalize">
              {selectedAgent} Agent
            </h3>
            <span className="text-sm text-gray-500 dark:text-gray-400">
              ({benchmarks?.benchmarks.length || 0} benchmarks)
            </span>
          </div>
          <button
            onClick={() => setShowAgentConfig(!showAgentConfig)}
            className="p-1 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-full transition-colors"
            title="Agent Configuration"
          >
            {showAgentConfig ? (
              <svg
                className="w-5 h-5 text-gray-600 dark:text-gray-400"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M6 18L18 6M6 6l12 12"
                />
              </svg>
            ) : (
              <svg
                className="w-5 h-5 text-gray-600 dark:text-gray-400"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
                />
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                />
              </svg>
            )}
          </button>
        </div>
      </div>

      {/* Expandable Agent Configuration */}
      {showAgentConfig && (
        <div className="border-b border-gray-200 dark:border-gray-700">
          <AgentConfig
            selectedAgent={selectedAgent}
            isRunning={isRunning}
            onConfigSaved={() => {
              // Optionally refetch data when config is saved
              refetch();
            }}
          />
        </div>
      )}

      {/* Benchmark List */}
      <div className="flex-1 overflow-y-auto">
        <div className="divide-y divide-gray-200 dark:divide-gray-700">
          {[...benchmarks.benchmarks]
            .filter((benchmark) => {
              // Filter out failure test benchmarks (003, 004) from web interface
              // Keep only happy path benchmarks for web testing
              return (
                !benchmark.id.includes("003") && !benchmark.id.includes("004")
              );
            })
            .sort((a, b) => {
              // Sort by benchmark ID
              return a.id.localeCompare(b.id);
            })
            .map((benchmark) => {
              const execution = getBenchmarkStatus(benchmark.id, selectedDate);
              const status = execution?.status || null;
              const score =
                execution?.status === ExecutionStatus.COMPLETED
                  ? getBenchmarkScore(benchmark.id, selectedDate)
                  : execution?.status === ExecutionStatus.COMPLETED
                    ? execution.score || 1.0
                    : 0;
              const isSelected = selectedBenchmark === benchmark.id;

              // Find if any benchmark is currently running
              const runningBenchmark = Array.from(executions.keys()).find(
                (benchmarkId) => {
                  const execution = executions.get(benchmarkId);
                  return execution?.status === ExecutionStatus.RUNNING;
                },
              );

              const isExpanded =
                status === ExecutionStatus.RUNNING
                  ? true // Always expand running benchmark
                  : runningBenchmark
                    ? false // Collapse all others when something is running
                    : expandedBenchmark === benchmark.id; // Normal expansion logic when nothing is running

              return (
                <div
                  key={benchmark.id}
                  className={`p-3 hover:bg-gray-50 dark:hover:bg-gray-700/50 cursor-pointer transition-colors ${
                    isSelected
                      ? "bg-blue-50 dark:bg-blue-900/20 border-l-4 border-blue-500 dark:border-blue-400"
                      : ""
                  }`}
                  onClick={() => handleBenchmarkClick(benchmark.id)}
                >
                  {/* Collapsed view - only prompt and run button */}
                  {!isExpanded && (
                    <div className="flex items-center justify-between">
                      <div className="flex items-center flex-1">
                        {/* Status box and expand icon */}
                        <div
                          className="flex flex-col items-center mr-3"
                          style="height: -webkit-fill-available; justify-content: space-around;"
                        >
                          {/* Status indicator box */}
                          <div
                            key={`${benchmark.id}-${status}-${execution?.timestamp || Date.now()}-${selectedDate || "latest"}`}
                            className={`w-4 h-4 rounded-sm ${getBenchmarkStatusColor(
                              status,
                              execution,
                            )}`}
                          />
                        </div>
                        <div className="flex-1">
                          <div className="font-medium text-gray-900 dark:text-gray-100 break-words">
                            {benchmark.prompt || benchmark.name}
                          </div>
                        </div>
                      </div>
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          handleRunBenchmark(benchmark);
                        }}
                        disabled={
                          isRunning ||
                          isRunningAll ||
                          status === ExecutionStatus.RUNNING
                        }
                        className="ml-3 px-3 py-1 bg-blue-600 text-white text-sm rounded hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors"
                      >
                        {status === ExecutionStatus.RUNNING
                          ? "Running..."
                          : "Run"}
                      </button>
                    </div>
                  )}

                  {/* Expanded view - full details */}
                  {isExpanded && (
                    <div>
                      <div className="flex items-center justify-between">
                        <div className="flex items-center flex-1">
                          {/* Status box and expand icon */}
                          <div
                            className="flex flex-col items-center mr-3"
                            style="height: -webkit-fill-available; justify-content: space-around;"
                          >
                            {/* Status indicator box */}
                            <div
                              key={`${benchmark.id}-${status}-${execution?.timestamp || Date.now()}-${selectedDate || "latest"}`}
                              className={`w-4 h-4 rounded-sm ${getBenchmarkStatusColor(
                                status,
                                execution,
                              )}`}
                            />
                          </div>
                          {/* Benchmark Name */}
                          <div>
                            <Tooltip
                              content={
                                <div className="text-sm">
                                  <div className="font-medium mb-1">
                                    {benchmark.name}
                                  </div>
                                  <div className="text-gray-600 dark:text-gray-300">
                                    {benchmark.id}
                                  </div>
                                </div>
                              }
                              position="top"
                              className="max-w-md"
                            >
                              <div className="font-medium text-gray-900 dark:text-gray-100 break-words">
                                {benchmark.prompt || benchmark.name}
                              </div>
                            </Tooltip>
                          </div>
                        </div>

                        {/* Run/Running Button */}
                        <button
                          onClick={(e) => {
                            e.stopPropagation();
                            if (status === ExecutionStatus.RUNNING) {
                              // Do nothing when running - just show Running...
                              return;
                            } else {
                              handleRunBenchmark(benchmark);
                            }
                          }}
                          disabled={
                            status === ExecutionStatus.RUNNING ||
                            isRunning ||
                            isRunningAll
                          }
                          className={`px-3 py-1 text-white text-sm rounded transition-colors ${
                            status === ExecutionStatus.RUNNING
                              ? "bg-gray-500"
                              : "bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed"
                          }`}
                        >
                          {status === ExecutionStatus.RUNNING
                            ? "Running..."
                            : "Run"}
                        </button>
                      </div>

                      <hr className=" border-gray-200 dark:border-gray-700 mt-2" />

                      {/* Progress Bar for Running and Completed Benchmarks */}
                      {(status === ExecutionStatus.RUNNING ||
                        status === ExecutionStatus.COMPLETED ||
                        status === ExecutionStatus.FAILED) && (
                        <div className="mt-2">
                          <div className="flex items-center space-x-2 text-sm text-gray-500 dark:text-gray-400">
                            {/* Status Icon */}
                            <span
                              className={`font-mono text-sm ${getStatusColor(status)}`}
                            >
                              {getStatusIcon(status)}
                            </span>

                            {/* Score */}
                            <span
                              className={`font-mono text-xs font-medium ${getScoreColor(score)}`}
                            >
                              {status === ExecutionStatus.COMPLETED ||
                              status === ExecutionStatus.FAILED
                                ? formatScore(score)
                                : "000%"}
                            </span>
                            <span className="text-gray-400"></span>
                            <span className="inline-flex items-center px-2 py-0.5 rounded-md bg-gray-100 dark:bg-gray-800 text-xs font-mono text-gray-700 dark:text-gray-300">
                              {benchmark.id}
                            </span>
                          </div>
                          <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
                            <div
                              className={`h-2 rounded-full transition-all duration-300 ${
                                status === ExecutionStatus.COMPLETED
                                  ? "bg-green-600"
                                  : status === ExecutionStatus.FAILED
                                    ? "bg-red-600"
                                    : "bg-blue-600"
                              }`}
                              style={{
                                width: `${getBenchmarkStatus(benchmark.id, selectedDate)?.progress || 0}%`,
                              }}
                            ></div>
                          </div>
                          {status === ExecutionStatus.COMPLETED && (
                            <div className="text-xs text-green-600 dark:text-green-400 mt-1 font-medium">
                              ✓ Completed successfully
                            </div>
                          )}
                          {status === ExecutionStatus.FAILED && (
                            <div className="text-xs text-red-600 dark:text-red-400 mt-1 font-medium">
                              ✗ Failed
                            </div>
                          )}
                        </div>
                      )}
                    </div>
                  )}
                </div>
              );
            })}
        </div>
      </div>
    </div>
  );
}
