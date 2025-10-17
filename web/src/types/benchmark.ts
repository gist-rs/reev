// Benchmark data types for the Reev web interface

export enum ExecutionStatus {
  PENDING = "Pending",
  RUNNING = "Running",
  COMPLETED = "Completed",
  FAILED = "Failed",
  UNKNOWN = "Unknown",
}

export interface BenchmarkResult {
  id: string;
  benchmark_id: string;
  agent_type: string;
  score: number;
  final_status: ExecutionStatus;
  execution_time_ms: number;
  timestamp: string;
  color_class: "green" | "yellow" | "red" | "gray";
}

export interface AgentPerformanceSummary {
  agent_type: string;
  total_benchmarks: number;
  average_score: number;
  success_rate: number;
  best_benchmarks: string[];
  worst_benchmarks: string[];
  results: BenchmarkResult[];
}

export interface FlowLogResponse {
  session_id: string;
  benchmark_id: string;
  agent_type: string;
  events: any[];
  final_result: any;
  performance_metrics: PerformanceMetrics;
}

// Array type for flow logs response
export type FlowLogsResponse = FlowLogResponse[];

export interface PerformanceMetrics {
  total_execution_time_ms: number;
  total_llm_calls: number;
  total_tool_calls: number;
  total_tokens: number;
  max_depth: number;
}

export interface Pagination {
  page: number;
  total_pages: number;
  total_items: number;
  limit: number;
}

export interface PaginatedResponse<T> {
  data: T[];
  pagination: Pagination;
}

export interface ResultsQuery {
  agent?: string;
  min_score?: number;
  max_score?: number;
  start_date?: string;
  end_date?: string;
  page?: number;
  limit?: number;
}

export interface HealthResponse {
  status: string;
  timestamp: string;
  version: string;
}

export interface BenchmarkDetails {
  id: string;
  description: string;
  tags: string[];
}

export interface ErrorResponse {
  error: string;
  message: string;
  timestamp: string;
}

export interface BenchmarkInfo {
  id: string;
  description: string;
  tags: string[];
  prompt: string;
}
