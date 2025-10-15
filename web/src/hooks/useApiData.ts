// Custom hook for API data fetching with loading and error states

import { useState, useEffect, useCallback, useMemo } from "preact/hooks";
import { apiClient } from "../services/api";

interface ApiState<T> {
  data: T | null;
  loading: boolean;
  error: string | null;
  refetch: () => Promise<void>;
}

interface PaginatedResponse<T> {
  data: T[];
  pagination: {
    page: number;
    total_pages: number;
    total_items: number;
    limit: number;
  };
}

interface ResultsQuery {
  agent?: string;
  min_score?: number;
  max_score?: number;
  start_date?: string;
  end_date?: string;
  page?: number;
  limit?: number;
}

// Generic hook for API data fetching
export function useApiData<T>(
  fetcher: () => Promise<T>,
  dependencies: any[] = [],
): ApiState<T> {
  const [data, setData] = useState<T | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);

  const fetchData = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const result = await fetcher();
      setData(result);
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : "Unknown error occurred";
      setError(errorMessage);
      console.error("API fetch error:", err);
    } finally {
      setLoading(false);
    }
  }, dependencies);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

  return { data, loading, error, refetch: fetchData };
}

// Hook for paginated results with filtering
export function useResults(initialQuery: ResultsQuery = {}) {
  const [query, setQuery] = useState<ResultsQuery>(initialQuery);
  const [currentPage, setCurrentPage] = useState(1);

  const fetchResults = useCallback(async () => {
    return apiClient.listResults({
      ...query,
      page: currentPage,
      limit: 50,
    });
  }, [query, currentPage]);

  const { data, loading, error, refetch } = useApiData<PaginatedResponse<any>>(
    fetchResults,
    [JSON.stringify(query), currentPage],
  );

  const updateQuery = useCallback((newQuery: Partial<ResultsQuery>) => {
    setQuery((prev) => ({ ...prev, ...newQuery }));
    setCurrentPage(1); // Reset to first page when filters change
  }, []);

  const nextPage = useCallback(() => {
    if (data && currentPage < data.pagination.total_pages) {
      setCurrentPage((prev) => prev + 1);
    }
  }, [data, currentPage]);

  const prevPage = useCallback(() => {
    if (currentPage > 1) {
      setCurrentPage((prev) => prev - 1);
    }
  }, [currentPage]);

  return {
    data,
    loading,
    error,
    refetch,
    query,
    updateQuery,
    currentPage,
    nextPage,
    prevPage,
    hasNextPage: data ? currentPage < data.pagination.total_pages : false,
    hasPrevPage: currentPage > 1,
    pagination: data?.pagination,
  };
}

// Hook for agent performance data
export function useAgentPerformance() {
  const { data, loading, error, refetch } = useApiData(
    apiClient.getAgentPerformance,
    [],
  );

  // Transform API data to match BenchmarkResult interface
  const transformedData = useMemo(() => {
    if (!data) {
      return null;
    }

    const transformed = data.map((agent: any) => {
      console.log(`🔍 Transforming agent: ${agent.agent_type}`, {
        total_benchmarks: agent.total_benchmarks,
        average_score: agent.average_score,
        success_rate: agent.success_rate,
        results_count: agent.results?.length,
      });

      return {
        ...agent,
        results: agent.results.map((result: any) => {
          return {
            id: result.id.toString(),
            benchmark_id: result.benchmark_id,
            agent_type: agent.agent_type,
            score: result.score,
            final_status: result.final_status?.toLowerCase() || "unknown",
            execution_time_ms: 1000, // Default value since API doesn't provide this
            timestamp: result.timestamp,
            color_class:
              result.score >= 1.0
                ? "green"
                : result.score >= 0.25
                  ? "yellow"
                  : ("red" as const),
          };
        }),
      };
    });

    return transformed;
  }, [data]);

  // Mock data fallback when backend is not available
  const mockData = useMemo(
    () => [
      {
        agent_type: "deterministic",
        total_benchmarks: 5,
        average_score: 0.85,
        success_rate: 0.8,
        best_benchmarks: ["benchmark-1", "benchmark-2"],
        worst_benchmarks: ["benchmark-3"],
        results: [
          {
            id: "result-1",
            benchmark_id: "benchmark-1",
            agent_type: "deterministic",
            score: 0.95,
            final_status: "success",
            execution_time_ms: 1250,
            timestamp: new Date().toISOString(),
            color_class: "green" as const,
          },
          {
            id: "result-2",
            benchmark_id: "benchmark-2",
            agent_type: "deterministic",
            score: 0.75,
            final_status: "partial",
            execution_time_ms: 980,
            timestamp: new Date().toISOString(),
            color_class: "yellow" as const,
          },
        ],
      },
      {
        agent_type: "local",
        total_benchmarks: 3,
        average_score: 0.72,
        success_rate: 0.67,
        best_benchmarks: ["benchmark-1"],
        worst_benchmarks: ["benchmark-2"],
        results: [
          {
            id: "result-3",
            benchmark_id: "benchmark-1",
            agent_type: "local",
            score: 0.88,
            final_status: "success",
            execution_time_ms: 2100,
            timestamp: new Date().toISOString(),
            color_class: "green" as const,
          },
        ],
      },
      {
        agent_type: "gemini-2.5-flash-lite",
        total_benchmarks: 2,
        average_score: 0.91,
        success_rate: 1.0,
        best_benchmarks: ["benchmark-1"],
        worst_benchmarks: [],
        results: [
          {
            id: "result-4",
            benchmark_id: "benchmark-1",
            agent_type: "gemini-2.5-flash-lite",
            score: 0.91,
            final_status: "success",
            execution_time_ms: 3400,
            timestamp: new Date().toISOString(),
            color_class: "green" as const,
          },
        ],
      },
      {
        agent_type: "glm-4.6",
        total_benchmarks: 1,
        average_score: 0.65,
        success_rate: 1.0,
        best_benchmarks: [],
        worst_benchmarks: ["benchmark-1"],
        results: [
          {
            id: "result-5",
            benchmark_id: "benchmark-1",
            agent_type: "glm-4.6",
            score: 0.65,
            final_status: "partial",
            execution_time_ms: 2800,
            timestamp: new Date().toISOString(),
            color_class: "yellow" as const,
          },
        ],
      },
    ],
    [],
  );

  const stats = useMemo(() => {
    const effectiveData =
      error || !transformedData ? mockData : transformedData;

    if (!effectiveData) {
      return { totalResults: 0, testedAgents: 0, totalAgents: 4 };
    }
    const totalResults = effectiveData.reduce(
      (sum, agent) => sum + agent.results.length,
      0,
    );
    const testedAgents = effectiveData.length;
    const totalAgents = 4; // Total available agent types
    return { totalResults, testedAgents, totalAgents };
  }, [transformedData, error, mockData]);

  return {
    data: transformedData, // Use real data only, no mock fallback
    loading: loading && !error,
    error,
    hasData: !!transformedData,
    refetch,
    ...stats,
  };
}

// Hook for benchmarks list
export function useBenchmarks() {
  const { data, loading, error, refetch } = useApiData(
    apiClient.listBenchmarks,
    [],
  );

  // Mock benchmarks data fallback
  const mockBenchmarks = {
    benchmarks: [
      {
        id: "benchmark-1",
        name: "Basic Transfer",
        description: "Test basic SOL transfer functionality",
        category: "transfer",
        difficulty: "easy",
        estimated_time: 30,
      },
      {
        id: "benchmark-2",
        name: "Token Swap",
        description: "Test token swapping capabilities",
        category: "defi",
        difficulty: "medium",
        estimated_time: 60,
      },
      {
        id: "benchmark-3",
        name: "Complex DeFi Strategy",
        description: "Test complex DeFi operation execution",
        category: "defi",
        difficulty: "hard",
        estimated_time: 120,
      },
      {
        id: "benchmark-4",
        name: "NFT Mint",
        description: "Test NFT minting functionality",
        category: "nft",
        difficulty: "medium",
        estimated_time: 45,
      },
      {
        id: "benchmark-5",
        name: "Multi-step Transaction",
        description: "Test execution of multiple related transactions",
        category: "complex",
        difficulty: "hard",
        estimated_time: 90,
      },
    ],
  };

  return {
    data: error || !data ? mockBenchmarks : data,
    loading: loading && !error,
    error,
    refetch,
  };
}

// Hook for agents list
export function useAgents() {
  return useApiData(apiClient.listAgents, []);
}

// Removed health check for now to focus on basic functionality
