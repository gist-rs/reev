import {
  AgentPerformanceCard,
  type BenchmarkGridProps,
} from "./benchmark-grid";

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
  const ALL_AGENT_TYPES = [
    "deterministic",
    "local",
    "gemini-2.5-flash-lite",
    "glm-4.6",
  ];
  const allBenchmarks = benchmarks || [];
  const runningBenchmarks = new Set<string>(runningBenchmarkIds);

  // Trace the real data before rendering (only when not loading)
  if (!benchmarksLoading && !agentPerformanceLoading) {
    console.log("🔍 BenchmarkGrid - Real Data Trace:");
    console.log("  - benchmarks:", benchmarks);
    console.log("  - benchmarks length:", benchmarks?.length);
    console.log("  - agentPerformanceData:", agentPerformanceData);
    console.log(
      "  - agentPerformanceData length:",
      agentPerformanceData?.length,
    );
    console.log("  - benchmarksLoading:", benchmarksLoading);
    console.log("  - agentPerformanceLoading:", agentPerformanceLoading);
    console.log("  - runningBenchmarkIds:", runningBenchmarkIds);
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
                  onBenchmarkClick={(result) => {
                    console.log("Benchmark clicked:", result);
                    if (onBenchmarkSelect) {
                      onBenchmarkSelect(result.benchmark_id);
                    }
                  }}
                />
              );
            })}
          </div>
        </div>
      </main>
    </div>
  );
}
