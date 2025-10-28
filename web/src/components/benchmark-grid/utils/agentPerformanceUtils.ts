import { useMemo, useCallback } from "preact/hooks";
import { BenchmarkResult, ExecutionStatus } from "../../../types/benchmark";
import { AgentPerformanceSummary } from "../types";
import { ExecutionState } from "../../../types/configuration";

export function useFilteredAgentData(
  agentData: AgentPerformanceSummary | undefined,
  agentType: string,
) {
  return useMemo(() => {
    const baseData = agentData || {
      agent_type: agentType,
      total_benchmarks: 0,
      average_score: 0,
      success_rate: 0,
      best_benchmarks: [],
      worst_benchmarks: [],
      results: [],
    };

    // Filter results to only include those for this specific agent type
    const filteredResults = (baseData.results || []).filter(
      (result) => result.agent_type === agentType,
    );

    console.log(`🔍 [AgentPerformanceCard] ${agentType}: Filtering results`, {
      originalCount: baseData.results?.length || 0,
      filteredCount: filteredResults.length,
      agentType,
    });

    return {
      ...baseData,
      results: filteredResults,
    };
  }, [agentData, agentType]);
}

export function useFilteredBenchmarks(allBenchmarks: any[]) {
  return useMemo(() => {
    return Array.isArray(allBenchmarks)
      ? allBenchmarks.filter((benchmark) => {
          if (!benchmark?.id) {
            console.warn("Invalid benchmark:", benchmark);
            return false;
          }
          return !benchmark.id.includes("003") && !benchmark.id.includes("004");
        })
      : [];
  }, [allBenchmarks]);
}

export function createPlaceholderResult(
  benchmarkId: string,
  timestamp: string,
  agentType: string,
  date?: string,
): BenchmarkResult {
  return {
    id: `placeholder-${agentType}-${benchmarkId}`,
    benchmark_id: benchmarkId,
    agent_type: agentType,
    score: 0,
    final_status: ExecutionStatus.UNKNOWN,
    execution_time_ms: 0,
    timestamp,
    color_class: "gray" as const,
    date: date,
    isEmpty: true,
  };
}

export function useOverallPercentage(
  results: BenchmarkResult[] | undefined,
  filteredBenchmarks: any[],
) {
  const calculateDayPercentage = useCallback(
    (dayResults: BenchmarkResult[]) => {
      if (filteredBenchmarks.length === 0) return 0;

      // Calculate total score including untested benchmarks (score 0)
      let totalScore = 0;
      filteredBenchmarks.forEach((benchmark) => {
        const result = dayResults.find((r) => r.benchmark_id === benchmark.id);
        totalScore += result?.score || 0;
      });

      return totalScore / filteredBenchmarks.length;
    },
    [filteredBenchmarks],
  );

  return useMemo(() => {
    if (!results?.length) return 0;

    // Calculate average of daily percentages
    const testRuns = results.reduce(
      (runs, result) => {
        const date = result.timestamp.substring(0, 10);
        if (!runs[date]) {
          runs[date] = {};
        }
        const existing = runs[date][result.benchmark_id];
        if (!existing || result.timestamp > existing.timestamp) {
          runs[date][result.benchmark_id] = result;
        }
        return runs;
      },
      {} as Record<string, Record<string, BenchmarkResult>>,
    );

    const dailyPercentages = Object.values(testRuns).map((dayResults) =>
      calculateDayPercentage(Object.values(dayResults)),
    );

    if (dailyPercentages.length === 0) return 0;
    return (
      dailyPercentages.reduce((sum, pct) => sum + pct, 0) /
      dailyPercentages.length
    );
  }, [results?.length, filteredBenchmarks, calculateDayPercentage]);
}

export function hasRunningBenchmark(
  runningBenchmarks: ExecutionState[],
  runningBenchmarkExecutions:
    | Map<string, { agent: string; status: string; progress: number }>
    | undefined,
  agentType: string,
): boolean {
  return runningBenchmarks.filter((e) => e.agent === agentType).length > 0;
  // return Array.from(runningBenchmarks.keys()).some((benchmarkId) => {
  //   return runningBenchmarkExecutions?.get(benchmarkId)?.agent === agentType;
  // });
}
