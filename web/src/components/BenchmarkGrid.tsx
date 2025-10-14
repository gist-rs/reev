// BenchmarkGrid component for main dashboard display

import { useState, useCallback, useEffect } from "preact/hooks";
import { useAgentPerformance } from "../hooks/useApiData";
import { useBenchmarkList } from "../hooks/useBenchmarkExecution";
import { apiClient } from "../services/api";
import { BenchmarkBox } from "./BenchmarkBox";

// Temporary types to avoid import issues
interface BenchmarkResult {
  id: string;
  benchmark_id: string;
  agent_type: string;
  score: number;
  final_status: string;
  execution_time_ms: number;
  timestamp: string;
  color_class: "green" | "yellow" | "red" | "gray";
}

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
}

export function BenchmarkGrid({
  className = "",
  refreshTrigger = 0,
  onBenchmarkSelect,
  selectedAgent = "deterministic",
  isRunning = false,
  onRunBenchmark,
}: BenchmarkGridProps) {
  const [selectedResult, setSelectedResult] = useState<BenchmarkResult | null>(
    null,
  );

  // Get benchmark list and agent performance data
  const { benchmarks } = useBenchmarkList();
  const { data, loading, error, refetch } = useAgentPerformance();
  const [allBenchmarks, setAllBenchmarks] = useState<string[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  // All agent types that should be displayed
  const ALL_AGENT_TYPES = ["deterministic", "local", "gemini", "glm-4.6"];

  // Load all benchmarks from API
  useEffect(() => {
    const loadBenchmarks = async () => {
      try {
        const benchmarkList = await apiClient.listBenchmarks();
        setAllBenchmarks(benchmarkList);
      } catch (error) {
        console.error("Failed to load benchmarks:", error);
      } finally {
        setIsLoading(false);
      }
    };

    loadBenchmarks();
  }, []);

  // Refetch performance data when refreshTrigger changes
  useEffect(() => {
    if (refreshTrigger > 0) {
      console.log(
        "🔄 Refreshing performance overview due to benchmark completion",
      );
      refetch();
    }
  }, [refreshTrigger, refetch]);

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
  if (isLoading) {
    return (
      <div className={`flex items-center justify-center min-h-96 ${className}`}>
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500 mx-auto mb-4"></div>
          <p className="text-gray-600">Loading benchmark results...</p>
        </div>
      </div>
    );
  }

  // Error state
  if (error) {
    return (
      <div className={`flex items-center justify-center min-h-96 ${className}`}>
        <div className="text-center max-w-md">
          <div className="text-red-500 mb-4">
            <svg
              class="w-16 h-16 mx-auto"
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
          <h3 className="text-lg font-semibold text-gray-900 mb-2">
            Failed to load data
          </h3>
          <p className="text-gray-600 mb-4">{error}</p>
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
  if (!data || data.length === 0) {
    return (
      <div className={`flex items-center justify-center min-h-96 ${className}`}>
        <div className="text-center">
          <div className="text-gray-400 mb-4">
            <svg
              class="w-16 h-16 mx-auto"
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
          <h3 className="text-lg font-semibold text-gray-900 mb-2">
            No benchmark data available
          </h3>
          <p className="text-gray-600">
            Run some benchmarks to see results here.
          </p>
        </div>
      </div>
    );
  }

  // Remove health check for now to focus on basic functionality

  return (
    <div className={`bg-gray-50 ${className}`}>
      {/* Main Content */}
      <main className="max-w-7xl mx-auto p-4">
        {/* Agent Sections */}
        <div className="flex justify-center">
          <div className="flex flex-wrap" style={{ width: "fit-content" }}>
            {ALL_AGENT_TYPES.map((agentType) => {
              // Find the agent data from the API results, or create placeholder
              const agentData = data.find(
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

              return (
                <div
                  key={agentType}
                  className="bg-white rounded-lg shadow-sm border p-4 w-80 max-w-sm m-2"
                >
                  <div className="flex items-center justify-between mb-4">
                    <h3 className="text-lg font-bold">{agentType}</h3>
                    <div className="text-sm text-gray-600">
                      <span
                        className={
                          agentData.success_rate >= 0.9
                            ? "text-green-600"
                            : agentData.success_rate >= 0.7
                              ? "text-yellow-600"
                              : agentData.success_rate == 0.0
                                ? "text-gray-400"
                                : "text-red-600"
                        }
                      >
                        {(agentData.success_rate * 100).toFixed(1)}%
                      </span>
                    </div>
                  </div>

                  {/* Benchmark Grid - Shows all benchmarks (tested + untested) */}
                  <div className="flex flex-wrap gap-1">
                    {(() => {
                      // Create a map of all benchmark results for this agent
                      const resultsMap = new Map();
                      agentData.results.forEach((result) => {
                        const benchmarkId = result.benchmark_id;
                        const existing = resultsMap.get(benchmarkId);
                        const resultDate = new Date(result.timestamp);
                        const existingDate = existing
                          ? new Date(existing.timestamp)
                          : null;

                        if (!existing || resultDate > existingDate) {
                          resultsMap.set(benchmarkId, result);
                        }
                      });

                      // Show all benchmarks except failure test ones (003, 004), creating grey boxes for untested ones
                      return allBenchmarks
                        .filter((benchmarkId) => {
                          // Filter out failure test benchmarks (003, 004) from web interface
                          // Keep only happy path benchmarks for web testing
                          return (
                            !benchmarkId.includes("003") &&
                            !benchmarkId.includes("004")
                          );
                        })
                        .map((benchmarkId) => {
                          const result = resultsMap.get(benchmarkId);

                          if (result) {
                            // Tested benchmark - show actual result
                            return (
                              <BenchmarkBox
                                key={`${agentType}-${benchmarkId}`}
                                result={result}
                                onClick={handleBenchmarkClick}
                              />
                            );
                          } else {
                            // Untested benchmark - create grey placeholder result
                            const placeholderResult: BenchmarkResult = {
                              id: `placeholder-${agentType}-${benchmarkId}`,
                              benchmark_id: benchmarkId,
                              agent_type: agentType,
                              score: 0,
                              final_status: "Not Tested",
                              execution_time_ms: 0,
                              timestamp: new Date().toISOString(),
                              color_class: "gray",
                            };

                            return (
                              <BenchmarkBox
                                key={`${agentType}-${benchmarkId}`}
                                result={placeholderResult}
                                onClick={handleBenchmarkClick}
                              />
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
          <div className="bg-white rounded-lg max-w-md w-full max-h-96 overflow-y-auto">
            <div className="p-6">
              <div className="flex items-center justify-between mb-4">
                <h3 className="text-lg font-semibold">Benchmark Details</h3>
                <button
                  onClick={handleCloseModal}
                  className="text-gray-400 hover:text-gray-600"
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
                  <span className="font-medium text-gray-900">Benchmark:</span>
                  <span className="ml-2 text-gray-800">
                    {selectedResult.benchmark_id}
                  </span>
                </div>
                <div>
                  <span className="font-medium text-gray-900">Agent:</span>
                  <span className="ml-2 text-gray-800">
                    {selectedResult.agent_type}
                  </span>
                </div>
                <div>
                  <span className="font-medium text-gray-900">Score:</span>
                  <span
                    className={`ml-2 font-semibold ${
                      selectedResult.color_class === "green"
                        ? "text-green-600"
                        : selectedResult.color_class === "yellow"
                          ? "text-yellow-600"
                          : "text-red-600"
                    }`}
                  >
                    {(selectedResult.score * 100).toFixed(1)}%
                  </span>
                </div>
                <div>
                  <span className="font-medium text-gray-900">Status:</span>
                  <span
                    className={`ml-2 ${
                      selectedResult.final_status === "Succeeded"
                        ? "text-green-600"
                        : "text-red-600"
                    }`}
                  >
                    {selectedResult.final_status}
                  </span>
                </div>
                <div>
                  <span className="font-medium text-gray-900">
                    Execution Time:
                  </span>
                  <span className="ml-2 text-gray-800">
                    {selectedResult.execution_time_ms}ms
                  </span>
                </div>
                <div>
                  <span className="font-medium text-gray-900">Timestamp:</span>
                  <span className="ml-2 text-gray-800">
                    {new Date(selectedResult.timestamp).toLocaleString()}
                  </span>
                </div>
              </div>

              <div className="mt-6 pt-4 border-t">
                <div className="flex space-x-3">
                  <button
                    onClick={() => {
                      if (onRunBenchmark && !isRunning) {
                        onRunBenchmark(
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
