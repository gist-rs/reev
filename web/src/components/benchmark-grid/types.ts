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
  onBenchmarkSelect?: (benchmarkId: string) => void;
  isRunning?: boolean;
  onRunBenchmark?: (benchmarkId: string, agentType?: string) => void;
  runningBenchmarkIds?: string[];
  agentPerformanceData?: any;
  agentPerformanceLoading?: boolean;
  agentPerformanceError?: string | null;
  refetchAgentPerformance?: () => Promise<void>;
  benchmarks?: any[] | null;
  benchmarksLoading?: boolean;
  benchmarksError?: string | null;
  refetchBenchmarks?: () => Promise<void>;
}
