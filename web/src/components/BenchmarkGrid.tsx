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
}

export function BenchmarkGrid({ className = "" }: BenchmarkGridProps) {
  const [selectedResult, setSelectedResult] = useState<BenchmarkResult | null>(
    null,
  );

  // Get benchmark list and agent performance data
  const { benchmarks } = useBenchmarkList();
  const { data, loading, error } = useAgentPerformance();
  const [allBenchmarks, setAllBenchmarks] = useState<string[]>([]);
  const [isLoading, setIsLoading] = useState(true);

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

  const handleBenchmarkClick = useCallback((result: BenchmarkResult) => {
    setSelectedResult(result);
    console.log("Benchmark clicked:", result);
  }, []);

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
    <div className={`min-h-screen bg-gray-50 ${className}`}>
      {/* Header */}
      <header className="bg-white shadow-sm border-b">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex items-center justify-between h-16">
            <div className="flex items-center">
              <h1 className="text-2xl font-bold text-gray-900">
                Reev Benchmark Dashboard
              </h1>
            </div>
            <div className="flex items-center space-x-4">
              <span className="text-sm text-gray-600">
                {data.reduce((sum, agent) => sum + agent.results.length, 0)}{" "}
                total results
              </span>
              <span className="text-sm text-gray-600">
                {data.length} agents
              </span>
            </div>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="max-w-7xl mx-auto p-4">
        {/* Legend - Updated to include untested */}
        <div className="mb-4 p-2 bg-gray-50 rounded border">
          <div className="flex justify-center">
            <div className="flex items-center space-x-4 text-xs text-gray-600">
              <div className="flex items-center">
                <div className="w-3 h-3 bg-green-500 rounded mr-1"></div>
                <span>Perfect (100%)</span>
              </div>
              <div className="flex items-center">
                <div className="w-3 h-3 bg-yellow-500 rounded mr-1"></div>
                <span>Partial (25-99%)</span>
              </div>
              <div className="flex items-center">
                <div className="w-3 h-3 bg-red-500 rounded mr-1"></div>
                <span>Poor (&lt;25%)</span>
              </div>
              <div className="flex items-center">
                <div className="w-3 h-3 bg-gray-400 rounded mr-1"></div>
                <span>Not Tested</span>
              </div>
            </div>
          </div>
        </div>

        {/* Agent Sections */}
        <div className="space-y-6">
          {data.map((agent) => (
            <div
              key={agent.agent_type}
              className="bg-white rounded-lg shadow-sm border p-4"
            >
              <div className="flex items-center justify-between mb-4">
                <h3 className="text-lg font-bold">{agent.agent_type}</h3>
                <div className="text-sm text-gray-600">
                  <span className="mr-4">
                    Avg:{" "}
                    <span
                      className={
                        agent.average_score >= 1.0
                          ? "text-green-600"
                          : agent.average_score >= 0.25
                            ? "text-yellow-600"
                            : "text-red-600"
                      }
                    >
                      {(agent.average_score * 100).toFixed(1)}%
                    </span>
                  </span>
                  <span>
                    Success:{" "}
                    <span
                      className={
                        agent.success_rate >= 0.9
                          ? "text-green-600"
                          : agent.success_rate >= 0.7
                            ? "text-yellow-600"
                            : "text-red-600"
                      }
                    >
                      {(agent.success_rate * 100).toFixed(1)}%
                    </span>
                  </span>
                </div>
              </div>

              {/* Benchmark Grid - Shows all benchmarks (tested + untested) */}
              <div className="flex flex-wrap gap-1">
                {(() => {
                  // Create a map of all benchmark results for this agent
                  const resultsMap = new Map();
                  agent.results.forEach((result) => {
                    const benchmarkId = result.benchmark_id;
                    const existing = resultsMap.get(benchmarkId);
                    if (
                      !existing ||
                      new Date(result.timestamp) > new Date(existing.timestamp)
                    ) {
                      resultsMap.set(benchmarkId, result);
                    }
                  });

                  // Show all benchmarks, creating grey boxes for untested ones
                  return allBenchmarks.map((benchmarkId) => {
                    const result = resultsMap.get(benchmarkId);

                    if (result) {
                      // Tested benchmark - show actual result
                      return (
                        <BenchmarkBox
                          key={`${agent.agent_type}-${benchmarkId}`}
                          result={result}
                          onClick={handleBenchmarkClick}
                        />
                      );
                    } else {
                      // Untested benchmark - create grey placeholder result
                      const placeholderResult: BenchmarkResult = {
                        id: `placeholder-${agent.agent_type}-${benchmarkId}`,
                        benchmark_id: benchmarkId,
                        agent_type: agent.agent_type,
                        score: 0,
                        final_status: "Not Tested",
                        execution_time_ms: 0,
                        timestamp: new Date().toISOString(),
                        color_class: "gray",
                      };

                      return (
                        <BenchmarkBox
                          key={`${agent.agent_type}-${benchmarkId}`}
                          result={placeholderResult}
                          onClick={handleBenchmarkClick}
                        />
                      );
                    }
                  });
                })()}
              </div>
            </div>
          ))}
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
                  <span className="font-medium">Benchmark:</span>
                  <span className="ml-2">{selectedResult.benchmark_id}</span>
                </div>
                <div>
                  <span className="font-medium">Agent:</span>
                  <span className="ml-2">{selectedResult.agent_type}</span>
                </div>
                <div>
                  <span className="font-medium">Score:</span>
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
                  <span className="font-medium">Status:</span>
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
                  <span className="font-medium">Execution Time:</span>
                  <span className="ml-2">
                    {selectedResult.execution_time_ms}ms
                  </span>
                </div>
                <div>
                  <span className="font-medium">Timestamp:</span>
                  <span className="ml-2">
                    {new Date(selectedResult.timestamp).toLocaleString()}
                  </span>
                </div>
              </div>

              <div className="mt-6 pt-4 border-t">
                <button
                  onClick={handleCloseModal}
                  className="w-full px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 transition-colors"
                >
                  Close
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
