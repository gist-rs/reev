// BenchmarkGrid component for main dashboard display

import { useState, useCallback, useEffect, useMemo } from "preact/hooks";
import { useAgentPerformance } from "../hooks/useApiData";
import { apiClient } from "../services/api";
import { BenchmarkBox } from "./BenchmarkBox";
import { BenchmarkInfo, BenchmarkResult } from "../types/benchmark";
import { BenchmarkList } from "../types/configuration";

interface AgentPerformanceSummary {
  agent_type: string;
  total_benchmarks: number;
  average_score: number;
  success_rate: number;
  best_benchmarks: string[];
  worst_benchmarks: string[];
  results: BenchmarkResult[];
}

interface BenchmarkGridProps {
  className?: string;
  refreshTrigger?: number;
  onBenchmarkSelect?: (benchmarkId: string) => void;
  selectedAgent?: string;
  isRunning?: boolean;
  onRunBenchmark?: (benchmarkId: string, agentType?: string) => void;
  runningBenchmarkIds?: string[];
  agentPerformanceData?: any;
  agentPerformanceLoading?: boolean;
  agentPerformanceError?: string | null;
  refetchAgentPerformance?: () => Promise<void>;
  benchmarks?: BenchmarkList | null;
  benchmarksLoading?: boolean;
  benchmarksError?: string | null;
  refetchBenchmarks?: () => Promise<void>;
}

export function BenchmarkGrid({
  className = "",
  refreshTrigger = 0,
  onBenchmarkSelect,
  selectedAgent = "deterministic",
  isRunning = false,
  onRunBenchmark,
  runningBenchmarkIds = [],
  agentPerformanceData,
  agentPerformanceLoading,
  agentPerformanceError,
  refetchAgentPerformance,
  benchmarks,
  benchmarksLoading,
  benchmarksError,
  refetchBenchmarks,
}: BenchmarkGridProps) {
  const [selectedResult, setSelectedResult] = useState<BenchmarkResult | null>(
    null,
  );

  // Use shared benchmark data passed as props instead of duplicate API call
  // Use shared benchmark data passed as props instead of duplicate API call

  const [allBenchmarks, setAllBenchmarks] = useState<BenchmarkInfo[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [runningBenchmarks, setRunningBenchmarks] = useState<Set<string>>(
    new Set(runningBenchmarkIds),
  );

  // Update running benchmarks when prop changes
  useEffect(() => {
    setRunningBenchmarks(new Set(runningBenchmarkIds));
  }, [runningBenchmarkIds]);

  // Track running benchmarks
  const handleRunBenchmark = useCallback(
    (benchmarkId: string, agentType?: string) => {
      if (onRunBenchmark) {
        setRunningBenchmarks((prev) => new Set(prev).add(benchmarkId));
        onRunBenchmark(benchmarkId, agentType);

        // Remove from running after a timeout (in case completion event doesn't fire)
        setTimeout(() => {
          setRunningBenchmarks((prev) => {
            const newSet = new Set(prev);
            newSet.delete(benchmarkId);
            return newSet;
          });
        }, 60000); // 1 minute timeout
      }
    },
    [onRunBenchmark],
  );

  // All agent types that should be displayed
  const ALL_AGENT_TYPES = [
    "deterministic",
    "local",
    "gemini-2.5-flash-lite",
    "glm-4.6",
  ];

  // Refetch performance data when refreshTrigger changes
  useEffect(() => {
    if (refreshTrigger > 0) {
      if (import.meta.env.DEV) {
        console.log(
          "🔄 Refreshing performance overview due to benchmark completion",
        );
      }
      refetchAgentPerformance?.();
    }
  }, [refreshTrigger, refetchAgentPerformance]);

  const handleBenchmarkClick = useCallback(
    (result: BenchmarkResult) => {
      setSelectedResult(result);
      console.log("Benchmark clicked:", result);

      // Also select the benchmark for execution
      if (onBenchmarkSelect) {
        onBenchmarkSelect(result.benchmark_id);
      }
    },
    [onBenchmarkSelect],
  );

  const handleCloseModal = useCallback(() => {
    setSelectedResult(null);
  }, []);

  // Loading state
  if (benchmarksLoading || agentPerformanceLoading) {
    return (
      <div className={`flex items-center justify-center min-h-96 ${className}`}>
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500 mx-auto mb-4"></div>
          <p className="text-gray-600 dark:text-gray-400">
            Loading benchmark results...
          </p>
        </div>
      </div>
    );
  }

  // Error state
  if (benchmarksError || agentPerformanceError) {
    return (
      <div className={`flex items-center justify-center min-h-96 ${className}`}>
        <div className="text-center max-w-md">
          <div className="text-red-500 dark:text-red-400 mb-4">
            <svg
              className="w-16 h-16 mx-auto"
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
          <h3 className="text-lg font-semibold text-red-600 dark:text-red-400 mb-2">
            Failed to load data
          </h3>
          <p className="text-red-500 dark:text-red-400 mb-4">
            {agentPerformanceError}
          </p>
          <button
            onClick={() => window.location.reload()}
            className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 transition-colors"
          >
            Retry
          </button>
        </div>
      </div>
    );
  }

  // No data state
  if (!agentPerformanceData || agentPerformanceData.length === 0) {
    return (
      <div className={`flex items-center justify-center min-h-96 ${className}`}>
        <div className="text-center">
          <div className="text-gray-400 dark:text-gray-500 mb-4">
            <svg
              className="w-16 h-16 mx-auto"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
              />
            </svg>
          </div>
          <h3 className="text-lg font-semibold text-gray-700 dark:text-gray-300 mb-2">
            No benchmark data available
          </h3>
          <p className="text-gray-600 dark:text-gray-400">
            Run some benchmarks to see results here.
          </p>
        </div>
      </div>
    );
  }

  // Remove health check for now to focus on basic functionality

  return (
    <div className={`bg-gray-50 dark:bg-gray-900/50 ${className}`}>
      {/* Main Content */}
      <main className="max-w-7xl mx-auto p-4 overflow-x-auto">
        {/* Agent Sections */}
        <div className="flex justify-center">
          <div className="flex flex-wrap" style={{ width: "fit-content" }}>
            {ALL_AGENT_TYPES.map((agentType) => {
              // Find the agent data from the API results, or create placeholder
              const agentData = agentPerformanceData.find(
                (a) => a.agent_type === agentType,
              ) || {
                agent_type: agentType,
                total_benchmarks: 0,
                average_score: 0,
                success_rate: 0,
                best_benchmarks: [],
                worst_benchmarks: [],
                results: [],
              };

              // Calculate percentage from latest results per benchmark only
              const lastThreePercentage = useMemo(() => {
                if (!agentData.results || agentData.results.length === 0)
                  return 0;

                // Get latest result per benchmark
                const latestByBenchmark = new Map();
                agentData.results?.forEach((result) => {
                  const existing = latestByBenchmark.get(result.benchmark_id);
                  if (!existing || result.timestamp > existing.timestamp) {
                    latestByBenchmark.set(result.benchmark_id, result);
                  }
                });

                const latestResults = Array.from(latestByBenchmark.values());
                if (latestResults.length === 0) return 0;

                const totalScore = latestResults.reduce(
                  (sum, result) => sum + result.score,
                  0,
                );
                return totalScore / latestResults.length;
              }, [agentData.results]);

              return (
                <div
                  key={agentType}
                  className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border dark:border-gray-700 p-4 w-96 max-w-md m-2"
                >
                  <div className="flex items-center justify-between mb-4">
                    <h3 className="text-lg font-bold text-gray-900 dark:text-gray-100">
                      {agentType}
                    </h3>
                    <div className="text-sm text-gray-600 dark:text-gray-400">
                      <span
                        className={
                          lastThreePercentage >= 0.9
                            ? "text-green-600 dark:text-green-400"
                            : lastThreePercentage >= 0.7
                              ? "text-yellow-600 dark:text-yellow-400"
                              : lastThreePercentage == 0.0
                                ? "text-gray-400 dark:text-gray-500"
                                : "text-red-600 dark:text-red-400"
                        }
                      >
                        {(lastThreePercentage * 100).toFixed(1)}%
                      </span>
                    </div>
                  </div>

                  {/* Last 3 Test Results with Date */}
                  <div className="space-y-2">
                    {(() => {
                      // Group results by date, taking latest result per benchmark per day
                      const testRuns = (agentData.results || []).reduce(
                        (runs, result) => {
                          const date = result.timestamp.substring(0, 10); // Group by date YYYY-MM-DD
                          if (!runs[date]) {
                            runs[date] = {};
                          }
                          // Keep only the latest result for each benchmark on this day
                          const existing = runs[date][result.benchmark_id];
                          if (
                            !existing ||
                            result.timestamp > existing.timestamp
                          ) {
                            runs[date][result.benchmark_id] = result;
                          }
                          return runs;
                        },
                        {} as Record<string, Record<string, BenchmarkResult>>,
                      );

                      // Convert to array format for frontend
                      const testRunsArray = Object.entries(testRuns).map(
                        ([date, benchmarks]) => [
                          date,
                          Object.values(benchmarks),
                        ],
                      );

                      // Get last 3 daily runs by date
                      const lastThreeRuns = testRunsArray
                        .sort(([a], [b]) =>
                          (b as string).localeCompare(a as string),
                        )
                        .slice(0, 3);

                      return [...lastThreeRuns, ...Array(3).fill(null)]
                        .slice(0, 3)
                        .map((run, index) => {
                          if (run) {
                            const [date, results] = run;
                            const runDate = results[0].timestamp;

                            return (
                              <div
                                key={index}
                                className="flex items-center space-x-2 text-sm"
                              >
                                <span className="text-gray-500 dark:text-gray-400 font-mono text-xs whitespace-nowrap">
                                  {date}
                                </span>
                                <div className="flex flex-wrap gap-1">
                                  {allBenchmarks
                                    .filter((benchmark) => {
                                      // Safety check for undefined benchmark
                                      if (!benchmark?.id) {
                                        console.warn(
                                          "Invalid benchmark:",
                                          benchmark,
                                        );
                                        return false;
                                      }
                                      // Filter out failure test benchmarks (003, 004)
                                      return (
                                        !benchmark.id.includes("003") &&
                                        !benchmark.id.includes("004")
                                      );
                                    })
                                    .map((benchmark) => {
                                      const benchmarkResult = results.find(
                                        (r) => r.benchmark_id === benchmark.id,
                                      );
                                      if (benchmarkResult) {
                                        // Real result - use BenchmarkBox for clicking
                                        return (
                                          <BenchmarkBox
                                            key={benchmark.id}
                                            result={benchmarkResult}
                                            onClick={handleBenchmarkClick}
                                            isRunning={runningBenchmarks.has(
                                              benchmark.id,
                                            )}
                                          />
                                        );
                                      } else {
                                        // Missing result - gray placeholder
                                        const placeholderResult: BenchmarkResult =
                                          {
                                            id: `placeholder-${agentType}-${benchmark.id}`,
                                            benchmark_id: benchmark.id,
                                            agent_type: agentType,
                                            score: 0,
                                            final_status: "Not Tested",
                                            execution_time_ms: 0,
                                            timestamp: runDate,
                                            color_class: "gray" as const,
                                          };
                                        return (
                                          <BenchmarkBox
                                            key={benchmark.id}
                                            result={placeholderResult}
                                            onClick={handleBenchmarkClick}
                                            isRunning={runningBenchmarks.has(
                                              benchmark.id,
                                            )}
                                          />
                                        );
                                      }
                                    })}
                                </div>
                              </div>
                            );
                          } else {
                            // Placeholder for missing test
                            return (
                              <div
                                key={index}
                                className="flex items-center space-x-2 text-sm"
                              >
                                <span className="text-gray-400 dark:text-gray-500 font-mono text-xs whitespace-nowrap">
                                  XXXX-XX-XX
                                </span>
                                <div className="flex flex-wrap gap-1">
                                  {allBenchmarks
                                    .filter((benchmark) => {
                                      // Safety check for undefined benchmark
                                      if (!benchmark?.id) {
                                        console.warn(
                                          "Invalid benchmark:",
                                          benchmark,
                                        );
                                        return false;
                                      }
                                      // Filter out failure test benchmarks (003, 004)
                                      return (
                                        !benchmark.id.includes("003") &&
                                        !benchmark.id.includes("004")
                                      );
                                    })
                                    .map((benchmark) => {
                                      const placeholderResult: BenchmarkResult =
                                        {
                                          id: `placeholder-${agentType}-${benchmark.id}`,
                                          benchmark_id: benchmark.id,
                                          agent_type: agentType,
                                          score: 0,
                                          final_status: "Not Tested",
                                          execution_time_ms: 0,
                                          timestamp: new Date().toISOString(),
                                          color_class: "gray",
                                        };
                                      return (
                                        <BenchmarkBox
                                          key={benchmark.id}
                                          result={placeholderResult}
                                          onClick={handleBenchmarkClick}
                                          isRunning={runningBenchmarks.has(
                                            benchmark.id,
                                          )}
                                        />
                                      );
                                    })}
                                </div>
                              </div>
                            );
                          }
                        });
                    })()}
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      </main>

      {/* Result Detail Modal */}
      {selectedResult && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
          <div className="bg-white dark:bg-gray-800 rounded-lg max-w-md w-full max-h-[80vh] overflow-y-auto">
            <div className="p-6">
              <div className="flex items-center justify-between mb-4">
                <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
                  Benchmark Details
                </h3>
                <button
                  onClick={handleCloseModal}
                  className="text-gray-400 dark:text-gray-500 hover:text-gray-600 dark:hover:text-gray-300"
                >
                  <svg
                    class="w-6 h-6"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M6 18L18 6M6 6l12 12"
                    />
                  </svg>
                </button>
              </div>

              <div className="space-y-3">
                <div>
                  <span className="font-medium text-gray-900 dark:text-gray-100">
                    Benchmark:
                  </span>
                  <span className="ml-2 text-gray-800 dark:text-gray-200">
                    {selectedResult.benchmark_id}
                  </span>
                </div>
                <div>
                  <span className="font-medium text-gray-900 dark:text-gray-100">
                    Agent:
                  </span>
                  <span className="ml-2 text-gray-800 dark:text-gray-200">
                    {selectedResult.agent_type}
                  </span>
                </div>
                <div>
                  <span className="font-medium text-gray-900 dark:text-gray-100">
                    Score:
                  </span>
                  <span
                    className={`ml-2 font-semibold ${
                      selectedResult.color_class === "green"
                        ? "text-green-600 dark:text-green-400"
                        : selectedResult.color_class === "yellow"
                          ? "text-yellow-600 dark:text-yellow-400"
                          : "text-red-600 dark:text-red-400"
                    }`}
                  >
                    {(selectedResult.score * 100).toFixed(1)}%
                  </span>
                </div>
                <div>
                  <span className="font-medium text-gray-900 dark:text-gray-100">
                    Status:
                  </span>
                  <span
                    className={`ml-2 ${
                      selectedResult.final_status === "Succeeded"
                        ? "text-green-600 dark:text-green-400"
                        : "text-red-600 dark:text-red-400"
                    }`}
                  >
                    {selectedResult.final_status}
                  </span>
                </div>
                <div>
                  <span className="font-medium text-gray-900 dark:text-gray-100">
                    Execution Time:
                  </span>
                  <span className="ml-2 text-gray-800 dark:text-gray-200">
                    {selectedResult.execution_time_ms}ms
                  </span>
                </div>
                <div>
                  <span className="font-medium text-gray-900 dark:text-gray-100">
                    Timestamp:
                  </span>
                  <span className="ml-2 text-gray-800 dark:text-gray-200">
                    {new Date(selectedResult.timestamp).toLocaleString()}
                  </span>
                </div>
              </div>

              <div className="mt-6 pt-4 border-t border-gray-200 dark:border-gray-700">
                <div className="flex space-x-3">
                  <button
                    onClick={() => {
                      if (onRunBenchmark && !isRunning) {
                        handleRunBenchmark(
                          selectedResult.benchmark_id,
                          selectedResult.agent_type,
                        );
                        handleCloseModal();
                      }
                    }}
                    disabled={isRunning || !onRunBenchmark}
                    className={`flex-1 px-4 py-2 rounded transition-colors ${
                      isRunning || !onRunBenchmark
                        ? "bg-gray-300 text-gray-500 cursor-not-allowed"
                        : "bg-green-600 text-white hover:bg-green-700"
                    }`}
                  >
                    {isRunning ? "Running..." : "Run Benchmark"}
                  </button>
                  <button
                    onClick={handleCloseModal}
                    className="flex-1 px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 transition-colors"
                  >
                    Close
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
