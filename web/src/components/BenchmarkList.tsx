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

// All available agent types
const ALL_AGENT_TYPES = ["deterministic", "local", "gemini", "glm-4.6"];

export function BenchmarkList({
  selectedAgent,
  selectedBenchmark,
  onBenchmarkSelect,
  isRunning,
  onExecutionStart,
}: BenchmarkListProps) {
  const { benchmarks, loading, error, refetch } = useBenchmarkList();
  // Track executions by benchmarkId+agentType key to distinguish runs per agent
  const [executions, setExecutions] = useState<Map<string, any>>(new Map());
  const [runningKeys, setRunningKeys] = useState<Set<string>>(new Set());

  // Poll for execution status updates
  useEffect(() => {
    if (runningKeys.size === 0) return;

    const interval = setInterval(async () => {
      const updates = new Map<string, any>();
      const stillRunning = new Set<string>();

      for (const key of runningKeys) {
        const [benchmarkId] = key.split("|");
        const executionId = executions.get(key)?.execution_id;
        if (!executionId) continue;

        try {
          const status = await apiClient.getExecutionStatus(
            benchmarkId,
            executionId,
          );
          updates.set(key, status);

          if (status.status === "Running") {
            stillRunning.add(key);
          } else if (
            status.status === "Completed" ||
            status.status === "Failed"
          ) {
            // Execution completed or failed - update final status but keep polling a bit longer
            // Don't remove from running immediately to allow UI to show final state
            if (!executions.get(key)?.finalUpdateShown) {
              // Mark that we've shown the final update
              updates.set(key, { ...status, finalUpdateShown: true });
              setTimeout(() => {
                setRunningKeys((prev) => {
                  const updated = new Set(prev);
                  updated.delete(key);
                  return updated;
                });
              }, 2000); // Keep in "running" state for 2 more seconds to show final status
            }
          }
        } catch (error) {
          console.error(`Failed to get status for ${key}:`, error);
        }
      }

      setExecutions((prev) => {
        const updated = new Map(prev);
        updates.forEach((status, key) => {
          const current = updated.get(key) || {};
          updated.set(key, { ...current, ...status });
        });
        return updated;
      });

      setRunningKeys(stillRunning);
    }, 1000); // Poll every 1 second for more responsive updates

    return () => clearInterval(interval);
  }, [runningKeys, executions, onExecutionStart]);

  const handleRunBenchmark = useCallback(
    async (benchmark: BenchmarkItem, agentType: string) => {
      if (isRunning) return;

      const key = `${benchmark.id}|${agentType}`;

      try {
        // Get agent configuration if needed
        let config;
        if (agentType !== "deterministic") {
          try {
            config = await apiClient.getAgentConfig(agentType);
          } catch {
            // No config found, that's okay for now
          }
        }

        const response = await apiClient.runBenchmark(benchmark.id, {
          agent: agentType,
          config,
        });

        // Track this execution by benchmarkId+agentType
        setExecutions((prev) => {
          const updated = new Map(prev);
          updated.set(key, {
            execution_id: response.execution_id,
            status: "Pending",
            progress: 0,
            agentType,
            benchmarkId: benchmark.id,
          });
          return updated;
        });

        setRunningKeys((prev) => new Set(prev).add(key));
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
    [isRunning, onExecutionStart, refetch],
  );

  const handleRunAllBenchmarks = useCallback(async () => {
    if (isRunning || !benchmarks) return;

    for (const benchmark of benchmarks.benchmarks) {
      await handleRunBenchmark(benchmark);
      // Small delay between starting benchmarks
      await new Promise((resolve) => setTimeout(resolve, 500));
    }
  }, [benchmarks, isRunning, handleRunBenchmark]);

  const getExecutionStatus = useCallback(
    (benchmarkId: string, agentType: string): any => {
      const key = `${benchmarkId}|${agentType}`;
      return executions.get(key);
    },
    [executions],
  );

  const getBenchmarkScore = useCallback(
    (benchmarkId: string, agentType: string): number => {
      const execution = getExecutionStatus(benchmarkId, agentType);
      if (execution && execution.status === "Completed") {
        // For now, return a mock score
        return 1.0;
      }
      return 0;
    },
    [getExecutionStatus],
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

      {/* Benchmark List with Agent Grid */}
      <div className="flex-1 overflow-y-auto">
        <div className="divide-y">
          {benchmarks.benchmarks.map((benchmark) => {
            const isSelected = selectedBenchmark === benchmark.id;

            return (
              <div
                key={benchmark.id}
                className={`p-3 hover:bg-gray-50 cursor-pointer transition-colors ${
                  isSelected ? "bg-blue-50 border-l-4 border-blue-500" : ""
                }`}
                onClick={() => onBenchmarkSelect(benchmark.id)}
              >
                {/* Benchmark Header */}
                <div className="flex items-center justify-between mb-3">
                  <div>
                    <div className="font-medium text-gray-900">
                      {benchmark.name}
                    </div>
                    <div className="text-sm text-gray-500">{benchmark.id}</div>
                  </div>
                </div>

                {/* Agent Status Grid */}
                <div className="grid grid-cols-4 gap-2">
                  {ALL_AGENT_TYPES.map((agentType) => {
                    const execution = getExecutionStatus(
                      benchmark.id,
                      agentType,
                    );
                    const status = execution?.status || null;
                    const score = getBenchmarkScore(benchmark.id, agentType);
                    const isCurrentlyRunning = runningKeys.has(
                      `${benchmark.id}|${agentType}`,
                    );

                    return (
                      <div
                        key={agentType}
                        className="border rounded p-2 text-center"
                      >
                        {/* Agent Type Name */}
                        <div className="text-xs font-medium text-gray-600 mb-1 capitalize">
                          {agentType}
                        </div>

                        {/* Status Box */}
                        <div
                          className={`h-8 w-full rounded flex items-center justify-center text-xs font-mono font-medium ${
                            status === "Running"
                              ? "bg-yellow-100 text-yellow-800"
                              : status === "Completed"
                                ? "bg-green-100 text-green-800"
                                : status === "Failed"
                                  ? "bg-red-100 text-red-800"
                                  : status === "Pending"
                                    ? "bg-blue-100 text-blue-800"
                                    : "bg-gray-100 text-gray-400"
                          }`}
                        >
                          {status === "Running" && "[…]"}
                          {status === "Completed" && "[✔]"}
                          {status === "Failed" && "[✗]"}
                          {status === "Pending" && "[⏳]"}
                          {!status && "[ ]"}
                        </div>

                        {/* Score */}
                        <div className="text-xs text-gray-600 mt-1 font-mono">
                          {status === "Completed" ? formatScore(score) : "---"}
                        </div>

                        {/* Run Button */}
                        <button
                          onClick={(e) => {
                            e.stopPropagation();
                            handleRunBenchmark(benchmark, agentType);
                          }}
                          disabled={isRunning || isCurrentlyRunning}
                          className="mt-1 w-full px-1 py-0.5 bg-blue-600 text-white text-xs rounded hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors"
                        >
                          {isCurrentlyRunning ? "Run…" : "Run"}
                        </button>

                        {/* Progress Bar for Running */}
                        {status === "Running" && (
                          <div className="mt-1">
                            <div className="w-full bg-gray-200 rounded-full h-1">
                              <div
                                className="bg-blue-600 h-1 rounded-full transition-all duration-300"
                                style={{
                                  width: `${execution?.progress || 0}%`,
                                }}
                              ></div>
                            </div>
                          </div>
                        )}

                        {/* Completion Message */}
                        {status === "Completed" && (
                          <div className="text-xs text-green-600 mt-1">
                            ✓ Done
                          </div>
                        )}
                        {status === "Failed" && (
                          <div className="text-xs text-red-600 mt-1">
                            ✗ Failed
                          </div>
                        )}
                      </div>
                    );
                  })}
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
