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

  const stats = useMemo(() => {
    if (!data) {
      return { totalResults: 0, totalAgents: 0 };
    }
    const totalResults = data.reduce(
      (sum, agent) => sum + agent.results.length,
      0,
    );
    const totalAgents = data.length;
    return { totalResults, totalAgents };
  }, [data]);

  return { data, loading, error, refetch, ...stats };
}

// Hook for benchmarks list
export function useBenchmarks() {
  return useApiData(apiClient.listBenchmarks, []);
}

// Hook for agents list
export function useAgents() {
  return useApiData(apiClient.listAgents, []);
}

// Removed health check for now to focus on basic functionality
