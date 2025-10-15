import { useState, useCallback, useEffect } from "preact/hooks";
import {
  BenchmarkDetailsModal,
  LoadingStates,
  AgentPerformanceCard,
  type BenchmarkGridProps,
} from "./benchmark-grid";
import { BenchmarkResult } from "../types/benchmark";

export function BenchmarkGrid({
  className = "",
  refreshTrigger = 0,
  onBenchmarkSelect,
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
  const [runningBenchmarks, setRunningBenchmarks] = useState<Set<string>>(
    new Set(runningBenchmarkIds),
  );

  const allBenchmarks = benchmarks || [];

  useEffect(() => {
    setRunningBenchmarks(new Set(runningBenchmarkIds));
  }, [runningBenchmarkIds]);

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

  const handleRunBenchmark = useCallback(
    (benchmarkId: string, agentType?: string) => {
      if (onRunBenchmark) {
        setRunningBenchmarks((prev) => new Set(prev).add(benchmarkId));
        onRunBenchmark(benchmarkId, agentType);

        setTimeout(() => {
          setRunningBenchmarks((prev) => {
            const newSet = new Set(prev);
            newSet.delete(benchmarkId);
            return newSet;
          });
        }, 60000);
      }
    },
    [onRunBenchmark],
  );

  const handleBenchmarkClick = useCallback(
    (result: BenchmarkResult) => {
      setSelectedResult(result);
      console.log("Benchmark clicked:", result);

      if (onBenchmarkSelect) {
        onBenchmarkSelect(result.benchmark_id);
      }
    },
    [onBenchmarkSelect],
  );

  const handleCloseModal = useCallback(() => {
    setSelectedResult(null);
  }, []);

  const ALL_AGENT_TYPES = [
    "deterministic",
    "local",
    "gemini-2.5-flash-lite",
    "glm-4.6",
  ];

  const loadingStates = (
    <LoadingStates
      className={className}
      benchmarksLoading={benchmarksLoading}
      agentPerformanceLoading={agentPerformanceLoading}
      benchmarksError={benchmarksError}
      agentPerformanceError={agentPerformanceError}
      agentPerformanceData={agentPerformanceData}
      benchmarks={allBenchmarks}
    />
  );

  if (loadingStates) {
    return loadingStates;
  }

  return (
    <div className={`bg-gray-50 dark:bg-gray-900/50 ${className}`}>
      <main className="max-w-7xl mx-auto p-4 overflow-x-auto">
        <div className="flex justify-center">
          <div className="flex flex-wrap" style={{ width: "fit-content" }}>
            {ALL_AGENT_TYPES.map((agentType) => {
              const agentData = agentPerformanceData?.find(
                (a) => a.agent_type === agentType,
              );

              return (
                <AgentPerformanceCard
                  key={agentType}
                  agentType={agentType}
                  agentData={agentData}
                  allBenchmarks={allBenchmarks}
                  runningBenchmarks={runningBenchmarks}
                  onBenchmarkClick={handleBenchmarkClick}
                />
              );
            })}
          </div>
        </div>
      </main>

      <BenchmarkDetailsModal
        selectedResult={selectedResult}
        onClose={handleCloseModal}
        onRunBenchmark={onRunBenchmark}
        isRunning={isRunning}
        handleRunBenchmark={handleRunBenchmark}
      />
    </div>
  );
}
