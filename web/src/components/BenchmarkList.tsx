// BenchmarkList component for interactive benchmark navigation and execution

import { useState, useCallback, useEffect } from "preact/hooks";
import {
  useBenchmarkList,
  useExecutionState,
} from "../hooks/useBenchmarkExecution";
import { apiClient } from "../services/api";
import { BenchmarkItem, ExecutionStatus } from "../types/configuration";

interface BenchmarkListProps {
  selectedAgent: string;
  selectedBenchmark: string | null;
  onBenchmarkSelect: (benchmark: string) => void;
  isRunning: boolean;
  onExecutionStart: (executionId: string) => void;
  onExecutionComplete: (benchmarkId: string, execution: any) => void;
  executions: Map<string, any>;
  updateExecution: (benchmarkId: string, execution: any) => void;
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
}: BenchmarkListProps) {
  const { benchmarks, loading, error, refetch } = useBenchmarkList();
  // Track running benchmarks and their execution IDs for polling
  const [runningBenchmarks, setRunningBenchmarks] = useState<
    Map<string, string>
  >(new Map());
  const [historicalResults, setHistoricalResults] = useState<Map<string, any>>(
    new Map(),
  );

  // Focused polling function to get ASCII tree after completion
  const startFocusedPolling = useCallback(
    async (benchmarkId: string, executionId: string, agent: string) => {
      console.log("=== STARTING FOCUSED POLLING FOR ASCII TREE ===");
      console.log("benchmarkId:", benchmarkId);
      console.log("executionId:", executionId);

      let attempts = 0;
      const maxAttempts = 3;
      const retryDelay = 3000; // 3 seconds

      const pollForAsciiTree = async () => {
        attempts++;
        console.log(`Focused polling attempt ${attempts}/${maxAttempts}`);

        try {
          const status = await apiClient.getExecutionStatus(
            benchmarkId,
            executionId,
          );

          console.log("=== FOCUSED POLLING RESULT ===");
          console.log("status:", status.status);
          console.log("progress:", status.progress);
          console.log("trace length:", status.trace?.length || 0);
          console.log("trace preview:", status.trace?.substring(0, 100));

          // Update execution state
          updateExecution(benchmarkId, {
            id: status.id,
            benchmark_id: benchmarkId,
            agent: agent,
            status: status.status,
            progress: status.progress,
            start_time: status.start_time,
            end_time: status.end_time,
            trace: status.trace,
            logs: status.logs,
            error: status.error,
          });

          // Notify parent component
          onExecutionComplete(benchmarkId, status);

          // Check if we got the ASCII tree
          if (
            status.status === "Completed" &&
            status.trace &&
            status.trace.length > 50 // ASCII tree should have substantial content
          ) {
            console.log("✅ ASCII TREE SUCCESSFULLY RETRIEVED!");
            console.log("Final trace length:", status.trace.length);

            // Remove from running benchmarks
            setRunningBenchmarks((prev) => {
              const updated = new Map(prev);
              updated.delete(benchmarkId);
              return updated;
            });
            return;
          }

          // If we haven't got the ASCII tree yet and have more attempts
          if (attempts < maxAttempts) {
            console.log(`ASCII tree not ready, retrying in ${retryDelay}ms...`);
            setTimeout(pollForAsciiTree, retryDelay);
          } else {
            console.log("❌ Focused polling failed - no ASCII tree retrieved");
            // Remove from running anyway after max attempts
            setRunningBenchmarks((prev) => {
              const updated = new Map(prev);
              updated.delete(benchmarkId);
              return updated;
            });
          }
        } catch (error) {
          console.error("Focused polling error:", error);
          if (attempts < maxAttempts) {
            setTimeout(pollForAsciiTree, retryDelay);
          } else {
            // Remove from running after max attempts
            setRunningBenchmarks((prev) => {
              const updated = new Map(prev);
              updated.delete(benchmarkId);
              return updated;
            });
          }
        }
      };

      // Start the focused polling
      pollForAsciiTree();
    },
    [updateExecution, onExecutionComplete],
  );

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

  // Poll for execution status updates
  useEffect(() => {
    if (runningBenchmarks.size === 0) return;

    const interval = setInterval(async () => {
      const stillRunning = new Set<string>();

      for (const [benchmarkId, executionId] of runningBenchmarks) {
        console.log("=== Polling for benchmark ===");
        console.log("benchmarkId:", benchmarkId);
        console.log("executionId:", executionId);
        console.log(
          "runningBenchmarks:",
          Array.from(runningBenchmarks.entries()),
        );

        if (!executionId) continue;

        try {
          console.log("Fetching status for:", benchmarkId, executionId);
          const status = await apiClient.getExecutionStatus(
            benchmarkId,
            executionId,
          );

          // Update the shared execution state for parent components
          console.log("Updating execution for benchmark:", benchmarkId, status);
          updateExecution(benchmarkId, {
            id: status.id,
            benchmark_id: benchmarkId,
            agent: selectedAgent,
            status: status.status,
            progress: status.progress,
            start_time: status.start_time,
            end_time: status.end_time,
            trace: status.trace,
            logs: status.logs,
            error: status.error,
          });

          if (status.status === "Running") {
            stillRunning.add(benchmarkId);
          } else if (
            status.status === "Completed" ||
            status.status === "Failed"
          ) {
            // Execution completed - trigger focused polling to get ASCII tree
            console.log(
              "=== Execution completed - starting focused polling ===",
            );
            console.log("benchmarkId:", benchmarkId);
            console.log("Final status:", status);
            console.log("Final trace length:", status.trace?.length || 0);

            // Start focused polling to get the ASCII tree
            startFocusedPolling(benchmarkId, executionId, selectedAgent);

            stillRunning.add(benchmarkId); // Keep in running to prevent removal
          }
        } catch (error) {
          console.error(`Failed to get status for ${benchmarkId}:`, error);
        }
      }

      // Convert stillRunning Set back to Map for next iteration
      const nextRunningBenchmarks = new Map<string, string>();
      stillRunning.forEach((benchmarkId) => {
        const executionId = runningBenchmarks.get(benchmarkId);
        if (executionId) {
          nextRunningBenchmarks.set(benchmarkId, executionId);
        }
      });
      setRunningBenchmarks(nextRunningBenchmarks);
    }, 1000); // Poll every 1 second for more responsive updates

    return () => clearInterval(interval);
  }, [runningBenchmarks, executions, selectedAgent, updateExecution]);

  const handleRunBenchmark = useCallback(
    async (benchmark: BenchmarkItem) => {
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

    console.log(
      "Starting Run All for",
      benchmarks.benchmarks.length,
      "benchmarks",
    );

    for (let i = 0; i < benchmarks.benchmarks.length; i++) {
      const benchmark = benchmarks.benchmarks[i];
      console.log(
        `Starting benchmark ${i + 1}/${benchmarks.benchmarks.length}:`,
        benchmark.id,
      );

      // Start the benchmark and get the execution ID
      let response;
      try {
        response = await handleRunBenchmark(benchmark);
      } catch (error) {
        console.error(`Failed to start benchmark ${benchmark.id}:`, error);
        continue; // Skip to next benchmark on failure
      }

      const executionId = response.execution_id;
      console.log(`Started ${benchmark.id} with execution ID: ${executionId}`);

      // Wait for the benchmark to complete before starting the next one
      await new Promise<void>((resolve) => {
        let checkCount = 0;
        const maxChecks = 60; // Max 2 minutes per benchmark (60 * 2 seconds)

        const checkCompletion = () => {
          checkCount++;

          // Look for execution using the proper key (benchmark_id)
          const execution = executions.get(benchmark.id);

          console.log(
            `Check ${checkCount}: ${benchmark.id} (${executionId}) status: ${execution?.status || "not found"}`,
          );

          if (
            execution &&
            (execution.status === "Completed" || execution.status === "Failed")
          ) {
            console.log(
              `✅ Benchmark ${benchmark.id} completed with status:`,
              execution.status,
            );
            resolve();
          } else if (checkCount >= maxChecks) {
            console.log(
              `⏰ Timeout waiting for ${benchmark.id}, continuing to next`,
            );
            resolve(); // Continue even if timeout
          } else {
            // Check again in 2 seconds
            setTimeout(checkCompletion, 2000);
          }
        };

        // Start checking after 3 seconds to allow execution to be created and updated
        setTimeout(checkCompletion, 3000);
      });
    }

    // Refresh overview when all benchmarks are complete
    console.log("All benchmarks completed, refreshing overview");
    refetch();
  }, [benchmarks, isRunning, handleRunBenchmark, executions, refetch]);

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
            disabled={isRunning}
            className="px-3 py-1 bg-green-600 text-white text-sm rounded hover:bg-green-700 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors"
          >
            Run All
          </button>
        </div>
      </div>

      {/* Selected Agent Header */}
      <div className="p-4 border-b bg-gray-50">
        <div className="flex items-center space-x-2">
          <div className="w-2 h-2 bg-blue-600 rounded-full"></div>
          <h3 className="font-medium text-gray-900 capitalize">
            {selectedAgent} Agent
          </h3>
          <span className="text-sm text-gray-500">
            ({benchmarks?.benchmarks.length || 0} benchmarks)
          </span>
        </div>
      </div>

      {/* Benchmark List */}
      <div className="flex-1 overflow-y-auto">
        <div className="divide-y">
          {benchmarks.benchmarks.map((benchmark) => {
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
                    disabled={isRunning || status === "Running"}
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
