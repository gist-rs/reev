import { BenchmarkResult, ExecutionStatus } from "../../types/benchmark";
import { ExecutionState } from "../../types/configuration";

export interface AgentPerformanceSummary {
  agent_type: string;
  total_benchmarks: number;
  average_score: number;
  success_rate: number;
  best_benchmarks: string[];
  worst_benchmarks: string[];
  results: BenchmarkResult[];
}

export interface BenchmarkBoxProps {
  result: BenchmarkResult;
  onClick: (result: BenchmarkResult) => void;
  isRunning?: boolean;
  isSelected?: boolean;
  disabled?: boolean;
  showDate?: boolean;
}

export interface AgentPerformanceCardProps {
  agentType: string;
  agentData?: AgentPerformanceSummary;
  allBenchmarks: any[];
  runningBenchmarks: ExecutionState[];
  onBenchmarkClick: (
    result: BenchmarkResult,
    agentType: string,
    date?: string,
  ) => void;
  onCardClick?: (agentType: string) => void;
  runningBenchmarkExecutions?: Map<
    string,
    { agent: string; status: string; progress: number }
  >;
  executions?: Map<string, any>;
  selectedBenchmark?: string | null;
  selectedAgent?: string;
  selectedDate?: string | null;
  isAnyRunning?: boolean;
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
  runningBenchmarkIds?: ExecutionState[];
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
