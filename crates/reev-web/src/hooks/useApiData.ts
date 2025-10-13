// Custom hook for API data fetching with loading and error states

import { useState, useEffect, useCallback } from 'preact/hooks';
import { apiClient, PaginatedResponse, ResultsQuery, ErrorResponse } from '../services/api';

interface ApiState<T> {
  data: T | null;
  loading: boolean;
  error: string | null;
  refetch: () => Promise<void>;
}

// Generic hook for API data fetching
export function useApiData<T>(
  fetcher: () => Promise<T>,
  dependencies: any[] = []
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
      const errorMessage = err instanceof Error ? err.message : 'Unknown error occurred';
      setError(errorMessage);
      console.error('API fetch error:', err);
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

  const { data, loading, error, refetch } = useApiData<PaginatedResponse<any>>(fetchResults, [
    JSON.stringify(query),
    currentPage,
  ]);

  const updateQuery = useCallback((newQuery: Partial<ResultsQuery>) => {
    setQuery(prev => ({ ...prev, ...newQuery }));
    setCurrentPage(1); // Reset to first page when filters change
  }, []);

  const nextPage = useCallback(() => {
    if (data && currentPage < data.pagination.total_pages) {
      setCurrentPage(prev => prev + 1);
    }
  }, [data, currentPage]);

  const prevPage = useCallback(() => {
    if (currentPage > 1) {
      setCurrentPage(prev => prev - 1);
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
  return useApiData(apiClient.getAgentPerformance, []);
}

// Hook for benchmarks list
export function useBenchmarks() {
  return useApiData(apiClient.listBenchmarks, []);
}

// Hook for agents list
export function useAgents() {
  return useApiData(apiClient.listAgents, []);
}

// Hook for API health status
export function useApiHealth() {
  const [isHealthy, setIsHealthy] = useState<boolean | null>(null);
  const [checking, setChecking] = useState(true);

  useEffect(() => {
    const checkHealth = async () => {
      try {
        setChecking(true);
        const available = await apiClient.isAvailable();
        setIsHealthy(available);
      } catch {
        setIsHealthy(false);
      } finally {
        setChecking(false);
      }
    };

    checkHealth();

    // Check health every 30 seconds
    const interval = setInterval(checkHealth, 30000);
    return () => clearInterval(interval);
  }, []);

  return { isHealthy, checking };
}
