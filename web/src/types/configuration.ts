// Configuration types for agent management and API communication

import { ExecutionStatus } from "./benchmark";

export interface AgentConfig {
  agent_type: string;
  api_url?: string;
  api_key?: string;
}

export interface ConnectionTestResult {
  status: "success" | "error";
  message: string;
}

export interface BenchmarkExecutionRequest {
  agent: string;
  config?: AgentConfig;
}

export interface ExecutionResponse {
  execution_id: string;
  status: string;
}

export interface ExecutionState {
  id: string;
  benchmark_id: string;
  agent: string;
  status: ExecutionStatus;
  progress: number;
  start_time: string;
  end_time?: string;
  trace: string;
  logs: string;
  error?: string;
  score?: number;
  execution_time_ms?: number;
  timestamp?: string;
  color_class?: string;
}

export interface BenchmarkList {
  benchmarks: BenchmarkItem[];
  total: number;
}

export interface BenchmarkItem {
  id: string;
  name: string;
  file_path: string;
  description?: string;
  status: ExecutionStatus;
  result?: BenchmarkResult;
  prompt?: string;
}

export interface BenchmarkResult {
  id: string;
  benchmark_id: string;
  agent_type: string;
  score: number;
  final_status: string;
  execution_time_ms: number;
  timestamp: string;
  color_class: "green" | "yellow" | "red" | "gray";
}

export interface RealtimeUpdate {
  type: "status" | "trace" | "log" | "progress";
  execution_id: string;
  data: any;
  timestamp: string;
}

export interface WebSocketMessage {
  event: string;
  data: any;
  timestamp: string;
}

export interface ApiResponse<T = any> {
  data?: T;
  error?: string;
  message?: string;
  timestamp: string;
}
