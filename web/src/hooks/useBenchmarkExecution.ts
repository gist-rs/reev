// Hook for managing benchmark execution state and data fetching

import { useState, useEffect, useCallback } from "preact/hooks";
import { apiClient } from "../services/api";
import { BenchmarkList, ExecutionState } from "../types/configuration";

interface UseBenchmarkExecutionReturn {
  benchmarks: BenchmarkList | null;
  loading: boolean;
  error: string | null;
  refetch: () => Promise<void>;
  executions: Map<string, ExecutionState>;
  updateExecution: (benchmarkId: string, execution: ExecutionState) => void;
  clearExecutions: () => void;
}

export function useBenchmarkExecution(): UseBenchmarkExecutionReturn {
  const [benchmarks, setBenchmarks] = useState<BenchmarkList | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [executions, setExecutions] = useState<Map<string, ExecutionState>>(new Map());

  const fetchBenchmarks = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await apiClient.getBenchmarkList();
      setBenchmarks(data);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : "Unknown error occurred";
      setError(errorMessage);
      console.error("Failed to fetch benchmarks:", err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchBenchmarks();
  }, [fetchBenchmarks]);

  const updateExecution = useCallback((benchmarkId: string, execution: ExecutionState) => {
    setExecutions((prev) => {
      const updated = new Map(prev);
      updated.set(benchmarkId, execution);
      return updated;
    });
  }, []);

  const clearExecutions = useCallback(() => {
    setExecutions(new Map());
  }, []);

  return {
    benchmarks,
    loading,
    error,
    refetch: fetchBenchmarks,
    executions,
    updateExecution,
    clearExecutions,
  };
}

export function useBenchmarkList() {
  const { benchmarks, loading, error, refetch } = useBenchmarkExecution();

  return {
    benchmarks,
    loading,
    error,
    refetch,
  };
}

export function useExecutionState() {
  const { executions, updateExecution, clearExecutions } = useBenchmarkExecution();

  return {
    executions,
    updateExecution,
    clearExecutions,
  };
}
