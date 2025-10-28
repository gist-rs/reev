import { BenchmarkResult } from "../../types/benchmark";

export interface AgentPerformanceSummary {
  agent_type: string;
  total_benchmarks: number;
  average_score: number;
  success_rate: number;
  best_benchmarks: string[];
  worst_benchmarks: string[];
  results: BenchmarkResult[];
}

export interface BenchmarkGridProps {
  className?: string;
  refreshTrigger?: number;
  onBenchmarkSelect?: (
    benchmarkId: string,
    agentType?: string,
    date?: string,
  ) => void;
  onCardClick?: (agentType: string) => void;
  isRunning?: boolean;
  onRunBenchmark?: (benchmarkId: string, agentType?: string) => void;
  runningBenchmarkIds?: string[];
  runningBenchmarkExecutions?: Map<
    string,
    { agent: string; status: string; progress: number }
  >;
  selectedBenchmark?: string | null;
  selectedAgent?: string;
  selectedDate?: string | null;
  executions?: Map<string, any>;
  agentPerformanceData?: any;
  agentPerformanceLoading?: boolean;
  agentPerformanceError?: string | null;
  refetchAgentPerformance?: () => Promise<void>;
  benchmarks?: any[];
  benchmarksLoading?: boolean;
  benchmarksError?: string | null;
  refetchBenchmarks?: () => Promise<void>;
}
