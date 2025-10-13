// BenchmarkList component for interactive benchmark navigation and execution

import { useState, useCallback, useEffect } from "preact/hooks";
import { useBenchmarkList } from "../hooks/useBenchmarkExecution";
import { apiClient } from "../services/api";
import { BenchmarkItem, ExecutionStatus } from "../types/configuration";

interface BenchmarkListProps {
  selectedAgent: string;
  selectedBenchmark: string | null;
  onBenchmarkSelect: (benchmark: string) => void;
  isRunning: boolean;
  onExecutionStart: (executionId: string) => void;
}

export function BenchmarkList({
  selectedAgent,
  selectedBenchmark,
  onBenchmarkSelect,
  isRunning,
  onExecutionStart,
}: BenchmarkListProps) {
  const { benchmarks, loading, error, refetch } = useBenchmarkList();
  const [executions, setExecutions] = useState<Map<string, any>>(new Map());
  const [runningBenchmarks, setRunningBenchmarks] = useState<Set<string>>(
    new Set(),
  );

  // Poll for execution status updates
  useEffect(() => {
    if (runningBenchmarks.size === 0) return;

    const interval = setInterval(async () => {
      const updates = new Map<string, any>();
      const stillRunning = new Set<string>();

      for (const benchmarkId of runningBenchmarks) {
        const executionId = executions.get(benchmarkId)?.execution_id;
        if (!executionId) continue;

        try {
          const status = await apiClient.getExecutionStatus(
            benchmarkId,
            executionId,
          );
          updates.set(benchmarkId, status);

          if (status.status === "Running") {
            stillRunning.add(benchmarkId);
          } else {
            // Execution completed or failed
            onExecutionStart(executionId); // Notify parent of completion
          }
        } catch (error) {
          console.error(`Failed to get status for ${benchmarkId}:`, error);
        }
      }

      setExecutions((prev) => {
        const updated = new Map(prev);
        updates.forEach((status, benchmarkId) => {
          updated.set(benchmarkId, status);
        });
        return updated;
      });

      setRunningBenchmarks(stillRunning);
    }, 2000); // Poll every 2 seconds

    return () => clearInterval(interval);
  }, [runningBenchmarks, executions, onExecutionStart]);

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

        // Track this execution
        setExecutions((prev) => {
          const updated = new Map(prev);
          updated.set(benchmark.id, {
            execution_id: response.execution_id,
            status: "Pending",
            progress: 0,
          });
          return updated;
        });

        setRunningBenchmarks((prev) => new Set(prev).add(benchmark.id));
        onExecutionStart(response.execution_id);

        // Refresh benchmark list to update status
        setTimeout(refetch, 500);
      } catch (error) {
        console.error("Failed to run benchmark:", error);
        alert(
          `Failed to run benchmark: ${error instanceof Error ? error.message : "Unknown error"}`,
        );
      }
    },
    [selectedAgent, isRunning, onExecutionStart, refetch],
  );

  const handleRunAllBenchmarks = useCallback(async () => {
    if (isRunning || !benchmarks) return;

    for (const benchmark of benchmarks.benchmarks) {
      await handleRunBenchmark(benchmark);
      // Small delay between starting benchmarks
      await new Promise((resolve) => setTimeout(resolve, 500));
    }
  }, [benchmarks, isRunning, handleRunBenchmark]);

  const getBenchmarkStatus = useCallback(
    (benchmark: BenchmarkItem): ExecutionStatus => {
      const execution = executions.get(benchmark.id);
      if (execution) {
        return execution.status as ExecutionStatus;
      }
      return benchmark.status;
    },
    [executions],
  );

  const getBenchmarkScore = useCallback(
    (benchmark: BenchmarkItem): number => {
      const execution = executions.get(benchmark.id);
      if (execution && execution.status === "Completed") {
        // For now, return a mock score
        return 1.0;
      }
      return 0;
    },
    [executions],
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

      {/* Benchmark List */}
      <div className="flex-1 overflow-y-auto">
        <div className="divide-y">
          {benchmarks.benchmarks.map((benchmark) => {
            const status = getBenchmarkStatus(benchmark);
            const score = getBenchmarkScore(benchmark);
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
                      {status === "Completed" ? formatScore(score) : "000%"}
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

                {/* Progress Bar for Running Benchmarks */}
                {status === "Running" && (
                  <div className="mt-2">
                    <div className="w-full bg-gray-200 rounded-full h-2">
                      <div
                        className="bg-blue-600 h-2 rounded-full transition-all duration-300"
                        style={{
                          width: `${executions.get(benchmark.id)?.progress || 0}%`,
                        }}
                      ></div>
                    </div>
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
