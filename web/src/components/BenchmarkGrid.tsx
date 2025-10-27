import {
  AgentPerformanceCard,
  type BenchmarkGridProps,
} from "./benchmark-grid";
import { useEffect } from "preact/hooks";

export function BenchmarkGrid({
  className = "",
  refreshTrigger = 0,
  onBenchmarkSelect,
  onCardClick,
  isRunning = false,
  onRunBenchmark,
  runningBenchmarkIds = [],
  executions,
  agentPerformanceData,
  agentPerformanceLoading,
  agentPerformanceError,
  refetchAgentPerformance,
  benchmarks,
  benchmarksLoading,
  benchmarksError,
  refetchBenchmarks,
  selectedBenchmark,
  selectedAgent,
}: BenchmarkGridProps) {
  const ALL_AGENT_TYPES = [
    "deterministic",
    "local",
    "gemini-2.5-flash-lite",
    "glm-4.6",
    "glm-4.6-coding",
  ];
  const allBenchmarks = benchmarks || [];
  const runningBenchmarks = new Set<string>(runningBenchmarkIds);

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
                  runningBenchmarkExecutions={executions}
                  selectedBenchmark={selectedBenchmark}
                  selectedAgent={selectedAgent}
                  isAnyRunning={runningBenchmarkIds.length > 0}
                  onBenchmarkClick={(result) => {
                    if (onBenchmarkSelect) {
                      onBenchmarkSelect(result.benchmark_id, agentType);
                    }
                  }}
                  onCardClick={onCardClick}
                />
              );
            })}
          </div>
        </div>
      </main>
    </div>
  );
}
