// Hook for managing benchmark execution state and data fetching

import { useState, useEffect, useCallback, useRef } from "preact/hooks";
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
  const [executions, setExecutions] = useState<Map<string, ExecutionState>>(
    new Map(),
  );
  const pollingIntervals = useRef<Map<string, NodeJS.Timeout>>(new Map());

  const fetchBenchmarks = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await apiClient.getBenchmarkList();
      setBenchmarks(data);
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : "Unknown error occurred";
      setError(errorMessage);
      console.error("Failed to fetch benchmarks:", err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchBenchmarks();
  }, [fetchBenchmarks]);

  const updateExecution = useCallback(
    (benchmarkId: string, execution: ExecutionState) => {
      console.log("useBenchmarkExecution.updateExecution called:", {
        benchmarkId,
        executionId: execution.id,
        status: execution.status,
      });
      setExecutions((prev) => {
        const updated = new Map(prev);
        updated.set(benchmarkId, execution);
        console.log(
          "Executions map after update:",
          Array.from(updated.entries()),
        );
        return updated;
      });

      // Start polling if execution is running
      if (execution.status === "Running" || execution.status === "Pending") {
        startPolling(benchmarkId, execution.id);
      } else {
        // Stop polling if execution is completed or failed
        stopPolling(benchmarkId);
      }
    },
    [],
  );

  const startPolling = useCallback(
    (benchmarkId: string, executionId: string) => {
      // Clear any existing polling for this benchmark
      stopPolling(benchmarkId);

      console.log(`Starting polling for ${benchmarkId} (${executionId})`);

      const pollExecution = async () => {
        try {
          // Get execution status from backend
          const updatedExecution = await apiClient.getExecutionStatus(
            benchmarkId,
            executionId,
          );

          if (updatedExecution) {
            console.log(
              `Polled update for ${benchmarkId}:`,
              updatedExecution.status,
            );
            setExecutions((prev) => {
              const updated = new Map(prev);
              updated.set(benchmarkId, updatedExecution);
              return updated;
            });

            // Stop polling if execution is completed or failed
            if (
              updatedExecution.status === "Completed" ||
              updatedExecution.status === "Failed"
            ) {
              console.log(
                `Stopping polling for ${benchmarkId} - execution completed`,
              );
              stopPolling(benchmarkId);
            }
          }
        } catch (error) {
          console.error(`Failed to poll execution ${executionId}:`, error);
        }
      };

      // Start polling every 2 seconds
      const intervalId = setInterval(pollExecution, 2000);
      pollingIntervals.current.set(benchmarkId, intervalId);

      // Poll immediately once
      pollExecution();
    },
    [],
  );

  const stopPolling = useCallback((benchmarkId: string) => {
    const intervalId = pollingIntervals.current.get(benchmarkId);
    if (intervalId) {
      clearInterval(intervalId);
      pollingIntervals.current.delete(benchmarkId);
      console.log(`Stopped polling for ${benchmarkId}`);
    }
  }, []);

  // Cleanup polling on unmount
  useEffect(() => {
    return () => {
      pollingIntervals.current.forEach((intervalId) => {
        clearInterval(intervalId);
      });
      pollingIntervals.current.clear();
    };
  }, []);

  const clearExecutions = useCallback(() => {
    // Stop all polling
    pollingIntervals.current.forEach((intervalId) => {
      clearInterval(intervalId);
    });
    pollingIntervals.current.clear();

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
  const { executions, updateExecution, clearExecutions } =
    useBenchmarkExecution();

  return {
    executions,
    updateExecution,
    clearExecutions,
  };
}
