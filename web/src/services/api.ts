// API client service for Reev web interface

import {
  AgentPerformanceSummary,
  BenchmarkResult,
  FlowLogResponse,
  FlowLogsResponse,
  PaginatedResponse,
  ResultsQuery,
  HealthResponse,
  ErrorResponse,
  BenchmarkInfo,
  ExecutionStatus,
} from "../types/benchmark";

import {
  AgentConfig,
  ConnectionTestResult,
  BenchmarkExecutionRequest,
  ExecutionResponse,
  ExecutionState,
  BenchmarkList,
  BenchmarkItem,
  RealtimeUpdate,
} from "../types/configuration";

const API_BASE_URL = import.meta.env.VITE_API_URL || "http://localhost:3001";

class ApiClient {
  private baseUrl: string;

  constructor(baseUrl: string = API_BASE_URL) {
    this.baseUrl = baseUrl;
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {},
  ): Promise<T> {
    const url = `${this.baseUrl}${endpoint}`;

    const config: RequestInit = {
      headers: {
        "Content-Type": "application/json",
        ...options.headers,
      },
      ...options,
    };

    try {
      const response = await fetch(url, config);

      if (!response.ok) {
        const errorData: ErrorResponse = await response.json().catch(() => ({
          error: response.statusText,
          message: `HTTP ${response.status}: ${response.statusText}`,
          timestamp: new Date().toISOString(),
        }));
        throw new Error(
          errorData.message || `HTTP error! status: ${response.status}`,
        );
      }

      return await response.json();
    } catch (error) {
      console.error(`API request failed for ${endpoint}:`, error);
      throw error;
    }
  }

  // Health check
  async getHealth(): Promise<HealthResponse> {
    return this.request<HealthResponse>("/api/v1/health");
  }

  // Benchmarks
  async listBenchmarks(): Promise<BenchmarkInfo[]> {
    return this.request<BenchmarkInfo[]>("/api/v1/benchmarks");
  }

  async getBenchmark(benchmarkId: string): Promise<Record<string, any>> {
    return this.request<Record<string, any>>(
      `/api/v1/benchmarks/${benchmarkId}`,
    );
  }

  // Agents
  async listAgents(): Promise<string[]> {
    return this.request<string[]>("/api/v1/agents");
  }

  // Results
  async listResults(
    query?: ResultsQuery,
  ): Promise<PaginatedResponse<BenchmarkResult>> {
    const params = new URLSearchParams();

    if (query?.agent) params.append("agent", query.agent);
    if (query?.min_score !== undefined)
      params.append("min_score", query.min_score.toString());
    if (query?.max_score !== undefined)
      params.append("max_score", query.max_score.toString());
    if (query?.start_date) params.append("start_date", query.start_date);
    if (query?.end_date) params.append("end_date", query.end_date);
    if (query?.page) params.append("page", query.page.toString());
    if (query?.limit) params.append("limit", query.limit.toString());

    const queryString = params.toString();
    const endpoint = queryString
      ? `/api/v1/results?${queryString}`
      : "/api/v1/results";

    return this.request<PaginatedResponse<BenchmarkResult>>(endpoint);
  }

  async getBenchmarkResults(benchmarkId: string): Promise<BenchmarkResult[]> {
    return this.request<BenchmarkResult[]>(`/api/v1/results/${benchmarkId}`);
  }

  // Flow logs
  async getFlowLog(benchmarkId: string): Promise<FlowLogsResponse> {
    return this.request<FlowLogsResponse>(`/api/v1/flow-logs/${benchmarkId}`);
  }

  // Transaction logs
  async getTransactionLogs(benchmarkId: string): Promise<any> {
    return this.request<any>(`/api/v1/transaction-logs/${benchmarkId}`);
  }

  // Execution trace
  async getExecutionTrace(executionId: string): Promise<any> {
    return this.request<any>(`/api/v1/executions/${executionId}/trace`);
  }

  // Agent performance
  async getAgentPerformance(): Promise<AgentPerformanceSummary[]> {
    return this.request<AgentPerformanceSummary[]>("/api/v1/agent-performance");
  }

  // Benchmark execution
  async runBenchmark(
    benchmarkId: string,
    request: BenchmarkExecutionRequest,
  ): Promise<ExecutionResponse> {
    return this.request<ExecutionResponse>(
      `/api/v1/benchmarks/${benchmarkId}/run`,
      {
        method: "POST",
        body: JSON.stringify(request),
      },
    );
  }

  async getExecutionStatus(
    benchmarkId: string,
    executionId: string,
  ): Promise<ExecutionState> {
    return this.request<ExecutionState>(
      `/api/v1/benchmarks/${benchmarkId}/status/${executionId}`,
    );
  }

  // Agent configuration
  async saveAgentConfig(config: AgentConfig): Promise<{ status: string }> {
    return this.request<{ status: string }>("/api/v1/agents/config", {
      method: "POST",
      body: JSON.stringify(config),
    });
  }

  async getAgentConfig(agentType: string): Promise<AgentConfig> {
    return this.request<AgentConfig>(`/api/v1/agents/config/${agentType}`);
  }

  async testAgentConnection(
    config: AgentConfig,
  ): Promise<ConnectionTestResult> {
    return this.request<ConnectionTestResult>("/api/v1/agents/test", {
      method: "POST",
      body: JSON.stringify(config),
    });
  }

  // Benchmark list
  async getBenchmarkList(): Promise<BenchmarkList> {
    const benchmarks = await this.listBenchmarks();

    // The benchmarks list already includes prompt data, so use it directly
    const benchmarksWithPrompts = benchmarks.map((benchmark) => ({
      id: benchmark.id,
      name:
        benchmark.prompt ||
        benchmark.id
          .replace(/-/g, " ")
          .replace(/\b\w/g, (l) => l.toUpperCase()),
      prompt: benchmark.prompt,
      file_path: `benchmarks/${benchmark.id}.yml`,
      status: "Pending" as ExecutionStatus,
    }));

    return {
      benchmarks: benchmarksWithPrompts,
      total: benchmarks.length,
    };
  }

  // Utility method to check API availability
  async isAvailable(): Promise<boolean> {
    try {
      await this.getHealth();
      return true;
    } catch {
      return false;
    }
  }
}

// Export singleton instance
const apiClientInstance = new ApiClient();

// Export methods bound to the instance to avoid 'this' context issues
export const apiClient = {
  getHealth: () => apiClientInstance.getHealth(),
  listBenchmarks: () => apiClientInstance.listBenchmarks(),
  getBenchmark: (benchmarkId: string) =>
    apiClientInstance.getBenchmark(benchmarkId),
  listAgents: () => apiClientInstance.listAgents(),
  listResults: (query?: ResultsQuery) => apiClientInstance.listResults(query),
  getBenchmarkResults: (benchmarkId: string) =>
    apiClientInstance.getBenchmarkResults(benchmarkId),
  getFlowLog: (benchmarkId: string) =>
    apiClientInstance.getFlowLog(benchmarkId),
  getTransactionLogs: (benchmarkId: string) =>
    apiClientInstance.getTransactionLogs(benchmarkId),
  getExecutionTrace: (executionId: string) =>
    apiClientInstance.getExecutionTrace(executionId),
  getAgentPerformance: () => apiClientInstance.getAgentPerformance(),
  // New methods
  runBenchmark: (benchmarkId: string, request: BenchmarkExecutionRequest) =>
    apiClientInstance.runBenchmark(benchmarkId, request),
  getExecutionStatus: (benchmarkId: string, executionId: string) =>
    apiClientInstance.getExecutionStatus(benchmarkId, executionId),

  saveAgentConfig: (config: AgentConfig) =>
    apiClientInstance.saveAgentConfig(config),
  getAgentConfig: (agentType: string) =>
    apiClientInstance.getAgentConfig(agentType),
  testAgentConnection: (config: AgentConfig) =>
    apiClientInstance.testAgentConnection(config),
  getBenchmarkList: () => apiClientInstance.getBenchmarkList(),
  isAvailable: () => apiClientInstance.isAvailable(),
};

// Export class for custom instances if needed
export { ApiClient };

// Export types for use in components
export type {
  AgentPerformanceSummary,
  BenchmarkResult,
  FlowLogResponse,
  FlowLogsResponse,
  PaginatedResponse,
  ResultsQuery,
  HealthResponse,
  ErrorResponse,
  AgentConfig,
  ConnectionTestResult,
  BenchmarkExecutionRequest,
  ExecutionResponse,
  ExecutionState,
  BenchmarkList,
  BenchmarkItem,
  RealtimeUpdate,
};
