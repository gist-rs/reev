// BenchmarkList component for interactive benchmark navigation and execution

import { useState, useCallback, useEffect, useRef } from "preact/hooks";
import { apiClient } from "../services/api";
import { BenchmarkItem, ExecutionStatus } from "../types/configuration";
import { AgentConfig } from "./AgentConfig";

interface BenchmarkListProps {
  selectedAgent: string;
  selectedBenchmark: string | null;
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
}

export function BenchmarkList({
  selectedAgent,
  selectedBenchmark,
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

  // Load historical benchmark results on component mount
  useEffect(() => {
    const loadHistoricalResults = async () => {
      try {
        const results = await apiClient.getAgentPerformance();
        const resultsMap = new Map();

        // Group results by benchmarkId for the selected agent
        results.forEach((agentSummary) => {
          if (agentSummary.agent_type === selectedAgent) {
            agentSummary.results.forEach((result) => {
              const key = `${result.benchmark_id}`;
              if (
                !resultsMap.has(key) ||
                new Date(result.timestamp) >
                  new Date(resultsMap.get(key).timestamp)
              ) {
                resultsMap.set(key, {
                  ...result,
                  status:
                    result.final_status === "Succeeded"
                      ? "Completed"
                      : "Failed",
                  progress: 100,
                  execution_id: result.id,
                  agentType: result.agent_type,
                  benchmarkId: result.benchmark_id,
                });
              }
            });
          }
        });

        setHistoricalResults(resultsMap);
      } catch (error) {
        console.error("Failed to load historical results:", error);
      }
    };

    loadHistoricalResults();
  }, [selectedAgent]);

  // Polling is now handled by useBenchmarkExecution hook - removed duplicate polling
  // This prevents state inconsistency issues between hook state and component state

  const handleRunBenchmark = useCallback(
    async (benchmark: BenchmarkItem, isRunAll: boolean = false) => {
      if (isRunning) return;

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
        console.log(
          "Starting benchmark execution:",
          benchmark.id,
          response.execution_id,
        );

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
              "🎯 Individual benchmark completion callback:",
              benchmarkId,
            );
            onExecutionComplete(benchmarkId, execution);
            // Clear the completion callback
            setCompletionCallback(() => () => {});
          });
        }

        // Return the response for Run All to use
        return response;
      } catch (error) {
        console.error("Failed to run benchmark:", error);
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

    console.log("🎯 Starting Run All Benchmarks");
    setIsRunningAll(true);

    // Set up completion callback (from App component)
    console.log("🔧 BenchmarkList: Setting up completion callback for Run All");
    setCompletionCallback(runAllCompletionCallback);

    // Auto-select the first benchmark for Execution Details display
    if (benchmarks.benchmarks.length > 0 && !selectedBenchmark) {
      const firstBenchmark = benchmarks.benchmarks[0];
      console.log(
        "Auto-selecting first benchmark for Run All:",
        firstBenchmark.id,
      );
      onBenchmarkSelect(firstBenchmark.id);
    }

    // Initialize queue (managed by App component) - filter out failure test benchmarks
    runAllQueue.current = [...benchmarks.benchmarks].filter(
      (benchmark) =>
        !benchmark.id.includes("003") && !benchmark.id.includes("004"),
    );
    currentRunAllIndex.current = 0;

    // Start first benchmark
    const firstBenchmark = runAllQueue.current[0];
    console.log(
      `🚀 Starting benchmark 1/${runAllQueue.current.length}: ${firstBenchmark.id}`,
    );

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

  const getBenchmarkStatus = useCallback(
    (benchmarkId: string): any => {
      return Array.from(executions.values()).find(
        (exec) => exec.benchmark_id === benchmarkId,
      );
    },
    [executions],
  );

  const getBenchmarkScore = useCallback(
    (benchmarkId: string): number => {
      const execution = getBenchmarkStatus(benchmarkId);
      if (execution && execution.status === "Completed") {
        // For now, return a mock score
        return 1.0;
      }
      return 0;
    },
    [getBenchmarkStatus],
  );

  const getStatusIcon = useCallback((status: ExecutionStatus) => {
    switch (status) {
      case "Pending":
        return "[ ]";
      case "Running":
        return "[…]";
      case "Completed":
        return "[✔]";
      case "Failed":
        return "[✗]";
      default:
        return "[?]";
    }
  }, []);

  const getStatusColor = useCallback((status: ExecutionStatus) => {
    switch (status) {
      case "Pending":
        return "text-gray-500";
      case "Running":
        return "text-yellow-500";
      case "Completed":
        return "text-green-500";
      case "Failed":
        return "text-red-500";
      default:
        return "text-gray-500";
    }
  }, []);

  const getScoreColor = useCallback((score: number) => {
    if (score >= 1.0) return "text-green-600";
    if (score >= 0.25) return "text-yellow-600";
    return "text-red-600";
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
        <div className="text-red-500 text-center">
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
        <div className="text-gray-500 text-center">
          <p>No benchmarks found</p>
        </div>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b">
        <h2 className="text-lg font-semibold">Benchmarks</h2>
        <div className="flex space-x-2">
          <button
            onClick={handleRunAllBenchmarks}
            disabled={isRunning || isRunningAll}
            className="px-3 py-1 bg-green-600 text-white text-sm rounded hover:bg-green-700 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors"
          >
            {isRunningAll ? "Running All..." : "Run All"}
          </button>
        </div>
      </div>

      {/* Selected Agent Header */}
      <div className="p-4 border-b bg-gray-50">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-2">
            <div className="w-2 h-2 bg-blue-600 rounded-full"></div>
            <h3 className="font-medium text-gray-900 capitalize">
              {selectedAgent} Agent
            </h3>
            <span className="text-sm text-gray-500">
              ({benchmarks?.benchmarks.length || 0} benchmarks)
            </span>
          </div>
          <button
            onClick={() => setShowAgentConfig(!showAgentConfig)}
            className="p-1 hover:bg-gray-100 rounded-full transition-colors"
            title="Agent Configuration"
          >
            {showAgentConfig ? (
              <svg
                className="w-5 h-5 text-gray-600"
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
                className="w-5 h-5 text-gray-600"
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
        <div className="border-b border-gray-200">
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
        <div className="divide-y">
          {benchmarks.benchmarks
            .filter((benchmark) => {
              // Filter out failure test benchmarks (003, 004) from web interface
              // Keep only happy path benchmarks for web testing
              return (
                !benchmark.id.includes("003") && !benchmark.id.includes("004")
              );
            })
            .map((benchmark) => {
              const execution = getBenchmarkStatus(benchmark.id);
              const historicalResult = historicalResults.get(benchmark.id);
              const status =
                execution?.status || historicalResult?.status || null;
              const score =
                execution?.status === "Completed"
                  ? getBenchmarkScore(benchmark.id)
                  : historicalResult?.final_status === "Succeeded"
                    ? historicalResult.score || 1.0
                    : 0;
              const isSelected = selectedBenchmark === benchmark.id;

              return (
                <div
                  key={benchmark.id}
                  className={`p-3 hover:bg-gray-50 cursor-pointer transition-colors ${
                    isSelected ? "bg-blue-50 border-l-4 border-blue-500" : ""
                  }`}
                  onClick={() => onBenchmarkSelect(benchmark.id)}
                >
                  <div className="flex items-center justify-between">
                    <div className="flex items-center space-x-3">
                      {/* Status Icon */}
                      <span
                        className={`font-mono text-sm ${getStatusColor(status)}`}
                      >
                        {getStatusIcon(status)}
                      </span>

                      {/* Score */}
                      <span
                        className={`font-mono text-sm font-medium ${getScoreColor(score)} min-w-[3rem]`}
                      >
                        {status === "Completed" || status === "Failed"
                          ? formatScore(score)
                          : "000%"}
                      </span>

                      {/* Benchmark Name */}
                      <div>
                        <div className="font-medium text-gray-900">
                          {benchmark.name}
                        </div>
                        <div className="text-sm text-gray-500">
                          {benchmark.id}
                        </div>
                      </div>
                    </div>

                    {/* Run Button */}
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        handleRunBenchmark(benchmark);
                      }}
                      disabled={
                        isRunning || isRunningAll || status === "Running"
                      }
                      className="px-3 py-1 bg-blue-600 text-white text-sm rounded hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors"
                    >
                      {status === "Running" ? "Running..." : "Run"}
                    </button>
                  </div>

                  {/* Progress Bar for Running and Completed Benchmarks */}
                  {(status === "Running" || status === "Completed") && (
                    <div className="mt-2">
                      <div className="w-full bg-gray-200 rounded-full h-2">
                        <div
                          className={`h-2 rounded-full transition-all duration-300 ${
                            status === "Completed"
                              ? "bg-green-600"
                              : "bg-blue-600"
                          }`}
                          style={{
                            width: `${getBenchmarkStatus(benchmark.id)?.progress || 0}%`,
                          }}
                        ></div>
                      </div>
                      {status === "Completed" && (
                        <div className="text-xs text-green-600 mt-1 font-medium">
                          ✓ Completed successfully
                        </div>
                      )}
                      {status === "Failed" && (
                        <div className="text-xs text-red-600 mt-1 font-medium">
                          ✗ Failed
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
