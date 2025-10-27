import { useMemo, useEffect, useCallback } from "preact/hooks";
import { BenchmarkBox } from "../BenchmarkBox";
import { BenchmarkResult, ExecutionStatus } from "../../types/benchmark";
import { AgentPerformanceSummary } from "./types";

interface AgentPerformanceCardProps {
  agentType: string;
  agentData?: AgentPerformanceSummary;
  allBenchmarks: any[];
  runningBenchmarks: Set<string>;
  onBenchmarkClick: (result: BenchmarkResult, agentType: string) => void;
  onCardClick?: (agentType: string) => void;
  runningBenchmarkExecutions?: Map<
    string,
    { agent: string; status: string; progress: number }
  >;
  selectedBenchmark?: string | null;
  selectedAgent?: string;
  isAnyRunning?: boolean;
}

export function AgentPerformanceCard({
  agentType,
  agentData,
  allBenchmarks,
  runningBenchmarks,
  onBenchmarkClick,
  onCardClick,
  runningBenchmarkExecutions,
  selectedBenchmark,
  selectedAgent,
  isAnyRunning = false,
}: AgentPerformanceCardProps) {
  const finalAgentData = useMemo(() => {
    const baseData = agentData || {
      agent_type: agentType,
      total_benchmarks: 0,
      average_score: 0,
      success_rate: 0,
      best_benchmarks: [],
      worst_benchmarks: [],
      results: [],
    };

    // Critical: Filter results to only include those for this specific agent type
    // This prevents cross-agent contamination when switching tabs
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

  const filteredBenchmarks = useMemo(() => {
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

  const createPlaceholderResult = (
    benchmarkId: string,
    timestamp: string,
  ): BenchmarkResult => {
    return {
      id: `placeholder-${agentType}-${benchmarkId}`,
      benchmark_id: benchmarkId,
      agent_type: agentType,
      score: 0,
      final_status: ExecutionStatus.UNKNOWN,
      execution_time_ms: 0,
      timestamp,
      color_class: "gray" as const,
    };
  };

  const renderBenchmarkBox = (
    benchmark: any,
    benchmarkResult?: BenchmarkResult,
    isRunning = false,
    isSelected = false,
  ) => {
    const result =
      benchmarkResult ||
      createPlaceholderResult(benchmark.id, new Date().toISOString());

    return (
      <BenchmarkBox
        key={benchmark.id}
        result={result}
        onClick={(result) => {
          // Don't allow clicks when any benchmark is running (except the running one)
          if (isAnyRunning && !isRunning) return;
          console.log(
            `🔍 [AgentPerformanceCard] CLICK: benchmark_id=${result.benchmark_id}, agentType=${agentType}`,
          );
          onBenchmarkClick(result, agentType);
          // Also trigger card click to change tab focus
          if (onCardClick) {
            onCardClick(agentType);
          }
        }}
        isRunning={isRunning}
        isSelected={isSelected}
        disabled={false} // Let the card handle the disabled state
      />
    );
  };

  // Check if this card has any running benchmarks
  const hasRunningBenchmark = useMemo(() => {
    return Array.from(runningBenchmarks.keys()).some((benchmarkId) => {
      return runningBenchmarkExecutions?.get(benchmarkId)?.agent === agentType;
    });
  }, [runningBenchmarks, runningBenchmarkExecutions, agentType]);

  const calculateDayPercentage = useCallback(
    (dayResults: BenchmarkResult[]) => {
      if (filteredBenchmarks.length === 0) return 0;

      // Calculate total score including untested benchmarks (score 0)
      let totalScore = 0;
      filteredBenchmarks.forEach((benchmark) => {
        const result = dayResults.find((r) => r.benchmark_id === benchmark.id);
        totalScore += result?.score || 0; // Add score if tested, 0 if not tested
      });

      return totalScore / filteredBenchmarks.length;
    },
    [filteredBenchmarks],
  );

  const overallPercentage = useMemo(() => {
    // Calculate average of daily percentages
    const testRuns = (finalAgentData.results || []).reduce(
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
  }, [
    finalAgentData.results?.length,
    filteredBenchmarks,
    calculateDayPercentage,
  ]);

  const renderTestRuns = useCallback(() => {
    const testRuns = (finalAgentData.results || []).reduce(
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

    const testRunsArray = Object.entries(testRuns).map(([date, benchmarks]) => [
      date,
      Object.values(benchmarks),
    ]);

    const lastThreeRuns = testRunsArray
      .sort(([a], [b]) => (b as string).localeCompare(a as string))
      .slice(0, 3);

    return [...lastThreeRuns, ...Array(3).fill(null)]
      .slice(0, 3)
      .map((run, index) => {
        if (run) {
          const [date, results] = run;
          const runDate = results[0].timestamp;
          // Only apply running animation to the most recent run (index 0)
          const isMostRecentRun = index === 0;

          return (
            <div key={index} className="flex items-center space-x-2 text-sm">
              <span className="text-gray-500 dark:text-gray-400 font-mono text-xs whitespace-nowrap">
                {date}
              </span>
              <div className="flex flex-wrap gap-1">
                {filteredBenchmarks.map((benchmark) => {
                  const benchmarkResult = results.find(
                    (r) =>
                      r.benchmark_id === benchmark.id &&
                      r.agent_type === agentType,
                  );

                  // Debug logging for result finding
                  if (benchmarkResult) {
                    console.log(
                      `✅ [AgentPerformanceCard] ${agentType}: Found result for ${benchmark.id}`,
                      {
                        resultAgent: benchmarkResult.agent_type,
                        expectedAgent: agentType,
                        benchmarkId: benchmark.id,
                      },
                    );
                  }

                  const isRunning =
                    isMostRecentRun &&
                    runningBenchmarks.has(benchmark.id) &&
                    runningBenchmarkExecutions?.get(benchmark.id)?.agent ===
                      agentType;

                  const isSelected =
                    isMostRecentRun &&
                    selectedBenchmark === benchmark.id &&
                    selectedAgent === agentType;

                  if (benchmarkResult) {
                    return renderBenchmarkBox(
                      benchmark,
                      benchmarkResult,
                      isRunning,
                      isSelected,
                    );
                  } else {
                    const placeholderResult = createPlaceholderResult(
                      benchmark.id,
                      runDate,
                    );
                    const isSelected =
                      isMostRecentRun &&
                      selectedBenchmark === benchmark.id &&
                      selectedAgent === agentType;
                    return renderBenchmarkBox(
                      benchmark,
                      placeholderResult,
                      isRunning,
                      isSelected,
                    );
                  }
                })}
              </div>
            </div>
          );
        } else {
          return (
            <div key={index} className="flex items-center space-x-2 text-sm">
              <span className="text-gray-400 dark:text-gray-500 font-mono text-xs whitespace-nowrap">
                XXXX-XX-XX
              </span>
              <div className="flex flex-wrap gap-1">
                {filteredBenchmarks.map((benchmark) => {
                  const placeholderResult = createPlaceholderResult(
                    benchmark.id,
                    new Date().toISOString(),
                  );
                  const isSelected = false; // Never show selection in placeholder rows
                  // Check if this benchmark is running even in placeholder rows
                  const isRunning =
                    index === 0 && // Only check running state in first placeholder row
                    runningBenchmarks.has(benchmark.id) &&
                    runningBenchmarkExecutions?.get(benchmark.id)?.agent ===
                      agentType;
                  return renderBenchmarkBox(
                    benchmark,
                    placeholderResult,
                    isRunning,
                    isSelected,
                  );
                })}
              </div>
            </div>
          );
        }
      });
  }, [
    finalAgentData.results,
    agentType,
    filteredBenchmarks,
    runningBenchmarks,
    runningBenchmarkExecutions,
    selectedBenchmark,
    selectedAgent,
  ]);

  const handleCardClick = () => {
    if (onCardClick) {
      onCardClick(agentType);
    }
  };

  return (
    <div
      className={`bg-white dark:bg-gray-800 rounded-lg shadow-sm border dark:border-gray-700 p-4 max-w-md m-2 transition-shadow ${
        isAnyRunning && !hasRunningBenchmark
          ? "cursor-not-allowed opacity-50"
          : "cursor-pointer hover:shadow-md"
      }`}
      onClick={
        isAnyRunning && !hasRunningBenchmark ? undefined : handleCardClick
      }
    >
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-bold text-gray-900 dark:text-gray-100">
          {agentType}
        </h3>
        <div className="text-sm text-gray-600 dark:text-gray-400">
          <span
            className={
              overallPercentage >= 0.9
                ? "text-green-600 dark:text-green-400"
                : overallPercentage >= 0.7
                  ? "text-yellow-600 dark:text-yellow-400"
                  : overallPercentage == 0.0
                    ? "text-gray-400 dark:text-gray-500"
                    : "text-red-600 dark:text-red-400"
            }
          >
            {(overallPercentage * 100).toFixed(1)}%
          </span>
        </div>
      </div>

      <div className="space-y-2">{renderTestRuns()}</div>
    </div>
  );
}
