import { useMemo, useCallback } from "preact/hooks";
import { BenchmarkResult } from "../../types/benchmark";
import { AgentPerformanceSummary, AgentPerformanceCardProps } from "./types";
import { TestRunsRenderer } from "./components/TestRunsRenderer";
import { AgentCardHeader } from "./components/AgentCardHeader";
import {
  useFilteredAgentData,
  useFilteredBenchmarks,
  hasRunningBenchmark,
  useOverallPercentage,
} from "./utils/agentPerformanceUtils";

export function AgentPerformanceCard({
  agentType,
  agentData,
  allBenchmarks,
  runningBenchmarks,
  onBenchmarkClick,
  onCardClick,
  runningBenchmarkExecutions,
  executions,
  selectedBenchmark,
  selectedAgent,
  selectedDate,
  isAnyRunning = false,
}: AgentPerformanceCardProps) {
  const finalAgentData = useFilteredAgentData(agentData, agentType);
  const filteredBenchmarks = useFilteredBenchmarks(allBenchmarks);
  const overallPercentage = useOverallPercentage(
    finalAgentData.results,
    filteredBenchmarks,
  );

  const hasRunning = useMemo(() => {
    const result = hasRunningBenchmark(
      runningBenchmarks,
      runningBenchmarkExecutions,
      agentType,
    );

    // console.log(`🏃 [AgentPerformanceCard] ${agentType} running check:`, {
    //   hasRunning: result,
    //   runningBenchmarks: Array.from(runningBenchmarks),
    //   runningBenchmarkExecutions: Array.from(
    //     runningBenchmarkExecutions?.entries() || [],
    //   ),
    //   agentType,
    // });

    return result;
  }, [runningBenchmarks, runningBenchmarkExecutions, agentType]);

  const handleCardClick = useCallback(() => {
    if (onCardClick) {
      onCardClick(agentType);
    }
  }, [onCardClick, agentType]);

  // console.log(`🔍 [AgentPerformanceCard] ${agentType} props:`, {
  //   agentType,
  //   hasRunning,
  //   isAnyRunning,
  //   runningBenchmarksCount: runningBenchmarks.size,
  //   executionsCount: runningBenchmarkExecutions?.size || 0,
  // });

  return (
    <div
      className={`bg-white dark:bg-gray-800 rounded-lg shadow-sm border dark:border-gray-700 p-4 max-w-md m-2 transition-shadow ${
        isAnyRunning && !hasRunning
          ? "cursor-not-allowed opacity-50"
          : "cursor-pointer hover:shadow-md"
      }`}
      onClick={isAnyRunning && !hasRunning ? undefined : handleCardClick}
    >
      <AgentCardHeader
        agentType={agentType}
        overallPercentage={overallPercentage}
      />

      <TestRunsRenderer
        finalAgentData={finalAgentData}
        agentType={agentType}
        allBenchmarks={allBenchmarks}
        runningBenchmarks={runningBenchmarks}
        runningBenchmarkExecutions={runningBenchmarkExecutions}
        executions={executions}
        selectedBenchmark={selectedBenchmark}
        selectedAgent={selectedAgent}
        selectedDate={selectedDate}
        onBenchmarkClick={onBenchmarkClick}
        onCardClick={onCardClick}
        isAnyRunning={isAnyRunning}
      />
    </div>
  );
}
