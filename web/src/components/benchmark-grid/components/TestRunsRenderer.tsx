import { useMemo, useCallback } from "preact/hooks";
import { BenchmarkResult, ExecutionStatus } from "../../../types/benchmark";
import { BenchmarkBox } from "../../BenchmarkBox";
import {
  useFilteredBenchmarks,
  createPlaceholderResult,
} from "../utils/agentPerformanceUtils";

interface TestRunsRendererProps {
  finalAgentData: any;
  agentType: string;
  allBenchmarks: any[];
  runningBenchmarks: Set<string>;
  runningBenchmarkExecutions?: Map<
    string,
    { agent: string; status: string; progress: number }
  >;
  executions?: Map<string, any>;
  selectedBenchmark?: string | null;
  selectedAgent?: string;
  selectedDate?: string | null;
  onBenchmarkClick: (
    result: BenchmarkResult,
    agentType: string,
    date?: string,
  ) => void;
  onCardClick?: (agentType: string) => void;
  isAnyRunning?: boolean;
}

export function TestRunsRenderer({
  finalAgentData,
  agentType,
  allBenchmarks,
  runningBenchmarks,
  runningBenchmarkExecutions,
  executions,
  selectedBenchmark,
  selectedAgent,
  selectedDate,
  onBenchmarkClick,
  onCardClick,
  isAnyRunning = false,
}: TestRunsRendererProps) {
  const filteredBenchmarks = useFilteredBenchmarks(allBenchmarks);

  const renderBenchmarkBox = (
    benchmark: any,
    benchmarkResult?: BenchmarkResult,
    isRunning = false,
    isSelected = false,
    showDate = false,
    date?: string,
  ) => {
    const result =
      benchmarkResult ||
      createPlaceholderResult(
        benchmark.id,
        new Date().toISOString(),
        agentType,
      );

    return (
      <BenchmarkBox
        key={`${benchmark.id}-${date || "no-date"}`}
        result={result}
        onClick={() => {
          // Don't allow clicks when any benchmark is running (except the running one)
          if (isAnyRunning && !isRunning) return;

          // Create result with the correct benchmark_id from the clicked benchmark
          const clickResult = {
            ...result,
            benchmark_id: benchmark.id,
            agent_type: agentType,
          };

          // Click handling for date-aware benchmark selection
          const resultDate = date !== null ? date : null;

          onBenchmarkClick(clickResult, agentType, resultDate);
          // Also trigger card click to change tab focus
          if (onCardClick) {
            onCardClick(agentType);
          }
        }}
        isRunning={isRunning}
        isSelected={isSelected}
        disabled={false}
        showDate={showDate}
      />
    );
  };

  const renderTestRuns = useCallback(() => {
    // Get all unique dates from agent results, sorted descending
    const allDates = new Set<string>();
    (finalAgentData.results || []).forEach((result: BenchmarkResult) => {
      allDates.add(result.timestamp.substring(0, 10));
    });

    const sortedDates = Array.from(allDates)
      .sort((a, b) => b.localeCompare(a))
      .slice(0, 3); // Only keep top 3 most recent dates

    // Create complete data structure with empty placeholders filled by real data
    const dateData: [string, BenchmarkResult[]][] = sortedDates.map((date) => {
      const dateResults: BenchmarkResult[] = [];

      // Create placeholder results for all benchmarks
      filteredBenchmarks.forEach((benchmark) => {
        // Find real result for this benchmark and date
        const realResult = (finalAgentData.results || []).find(
          (result: BenchmarkResult) =>
            result.benchmark_id === benchmark.id &&
            result.timestamp.substring(0, 10) === date &&
            result.agent_type === agentType,
        );

        if (realResult) {
          dateResults.push(realResult);
        } else {
          // Create empty placeholder
          dateResults.push(
            createPlaceholderResult(
              benchmark.id,
              `${date}T00:00:00.000Z`,
              agentType,
              date,
            ),
          );
        }
      });

      return [date, dateResults];
    });

    // If we have fewer than 3 rows, add placeholder rows to show empty state
    const rowsWithPlaceholders = [...dateData];
    if (rowsWithPlaceholders.length < 3) {
      for (let i = rowsWithPlaceholders.length; i < 3; i++) {
        rowsWithPlaceholders.push([
          `placeholder-${i}`,
          // Create empty placeholder results for all benchmarks
          filteredBenchmarks.map((benchmark) =>
            createPlaceholderResult(
              benchmark.id,
              new Date().toISOString(),
              agentType,
              undefined, // no date for placeholder rows
            ),
          ),
        ]);
      }
    }

    return rowsWithPlaceholders.map((run, index) => {
      // Check if this is a placeholder row
      const isPlaceholderRow =
        typeof run[0] === "string" && run[0].startsWith("placeholder-");

      if (isPlaceholderRow) {
        return (
          <div
            key={`placeholder-${index}`}
            className="flex items-center space-x-2 text-sm"
          >
            <span className="text-gray-400 dark:text-gray-500 font-mono text-xs whitespace-nowrap">
              XXXX-XX-XX
            </span>
            <div className="flex flex-wrap gap-1">
              {run[1].map((placeholderResult: BenchmarkResult) => {
                const isSelected = false; // Never show selection in placeholder rows
                // Check if this benchmark is running even in placeholder rows
                const isRunning =
                  index === 0 && // Only check running state in first placeholder row
                  runningBenchmarks.has(placeholderResult.benchmark_id) &&
                  runningBenchmarkExecutions?.get(
                    placeholderResult.benchmark_id,
                  )?.agent === agentType;
                return renderBenchmarkBox(
                  placeholderResult, // Use placeholder result as both benchmark and result
                  placeholderResult,
                  isRunning,
                  isSelected,
                  true, // showDate for grouped date view
                  null, // pass null date for placeholder rows
                );
              })}
            </div>
          </div>
        );
      }

      // Handle regular date rows
      const [date, results] = run;
      const runDate = results[0]?.timestamp || `${date}T00:00:00.000Z`;
      // Only apply running animation to the most recent run (index 0)
      const isMostRecentRun = index === 0;

      return (
        <div key={date} className="flex items-center space-x-2 text-sm">
          <span className="text-gray-500 dark:text-gray-400 font-mono text-xs whitespace-nowrap">
            {date}
          </span>
          <div className="flex flex-wrap gap-1">
            {filteredBenchmarks.map((benchmark) => {
              // First check if there's live execution data for this benchmark
              const liveExecution = executions?.get(benchmark.id);
              let benchmarkResult = results.find(
                (r) =>
                  r.benchmark_id === benchmark.id && r.agent_type === agentType,
              );

              // Prioritize live execution data over historical results
              if (liveExecution && liveExecution.agent === agentType) {
                // Create a result object from live execution data
                benchmarkResult = {
                  id: `live-${liveExecution.id}`,
                  benchmark_id: benchmark.id,
                  agent_type: agentType,
                  score: liveExecution.score || 0,
                  final_status:
                    liveExecution.status === "Completed"
                      ? ExecutionStatus.COMPLETED
                      : liveExecution.status === "Failed"
                        ? ExecutionStatus.FAILED
                        : ExecutionStatus.UNKNOWN,
                  execution_time_ms: liveExecution.execution_time_ms || 0,
                  timestamp:
                    liveExecution.timestamp || new Date().toISOString(),
                  color_class:
                    liveExecution.score && liveExecution.score >= 1.0
                      ? "green"
                      : liveExecution.score && liveExecution.score >= 0.25
                        ? "yellow"
                        : "red",
                };
              }

              const isRunning =
                isMostRecentRun &&
                runningBenchmarks.has(benchmark.id) &&
                runningBenchmarkExecutions?.get(benchmark.id)?.agent ===
                  agentType;

              const isSelected =
                selectedBenchmark === benchmark.id &&
                selectedAgent === agentType &&
                selectedDate === date;

              if (benchmarkResult) {
                return renderBenchmarkBox(
                  benchmark,
                  benchmarkResult,
                  isRunning,
                  isSelected,
                  true, // showDate for grouped date view
                  date, // pass the date for click handling
                );
              } else {
                // Use empty result from the pre-populated array
                const emptyResult = results.find(
                  (r) => r.benchmark_id === benchmark.id && r.isEmpty,
                );
                const isSelected =
                  selectedBenchmark === benchmark.id &&
                  selectedAgent === agentType &&
                  selectedDate === date;

                return renderBenchmarkBox(
                  benchmark,
                  emptyResult,
                  isRunning,
                  isSelected,
                  true, // showDate for grouped date view
                  date, // pass the date for click handling
                );
              }
            })}
          </div>
        </div>
      );
    });
  }, [
    finalAgentData.results,
    agentType,
    filteredBenchmarks,
    runningBenchmarks,
    runningBenchmarkExecutions,
    selectedBenchmark,
    selectedAgent,
    onBenchmarkClick,
    onCardClick,
    isAnyRunning,
  ]);

  return <div className="space-y-2">{renderTestRuns()}</div>;
}
