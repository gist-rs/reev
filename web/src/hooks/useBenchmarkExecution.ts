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
  updateExecution: (executionId: string, execution: ExecutionState) => void;
  clearExecutions: () => void;
  setCompletionCallback: (
    callback: (benchmarkId: string, execution: ExecutionState) => void,
  ) => void;
  getExecutionTraceWithLatestId: (
    benchmarkId: string,
    isRunning?: boolean,
  ) => Promise<any>;
  getTransactionLogsWithLatestId: (
    benchmarkId: string,
    isRunning?: boolean,
  ) => Promise<any>;
}

export function useBenchmarkExecution(): UseBenchmarkExecutionReturn {
  const [benchmarks, setBenchmarks] = useState<BenchmarkList | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [executions, setExecutions] = useState<Map<string, ExecutionState>>(
    new Map(),
  );
  const pollingIntervals = useRef<Map<string, number>>(new Map());
  const completionCallback = useRef<
    ((benchmarkId: string, execution: ExecutionState) => void) | null
  >(null);

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
    (executionId: string, execution: ExecutionState) => {
      setExecutions((prev) => {
        const updated = new Map(prev);
        // Use execution ID (session ID) as the key, not benchmark ID
        updated.set(execution.id, execution);
        return updated;
      });

      // Start polling if execution is running
      if (execution.status === "Running" || execution.status === "Pending") {
        startPolling(execution.benchmark_id, execution.id);
      } else {
        // Stop polling if execution is completed or failed
        stopPolling(execution.id);
      }
    },
    [],
  );

  const startPolling = useCallback(
    (benchmarkId: string, executionId: string) => {
      // Clear any existing polling for this execution
      stopPolling(executionId);

      const pollExecution = async () => {
        try {
          // Get execution status from backend
          const updatedExecution = await apiClient.getExecutionStatus(
            benchmarkId,
            executionId,
          );

          if (updatedExecution) {
            setExecutions((prev) => {
              const updated = new Map(prev);
              // Use execution ID as the key
              updated.set(updatedExecution.id, updatedExecution);
              return updated;
            });

            // Stop polling if execution is completed
            if (
              updatedExecution.status === "Completed" ||
              updatedExecution.status === "Failed"
            ) {
              stopPolling(executionId);

              // Call completion callback if set
              if (completionCallback.current) {
                completionCallback.current(benchmarkId, updatedExecution);
              }
            }
          }
        } catch (error) {
          console.error(`Failed to poll execution ${executionId}:`, error);
        }
      };

      // Clear any existing polling for this execution first
      stopPolling(executionId);

      // Start polling every 2 seconds
      const intervalId = setInterval(pollExecution, 2000);
      pollingIntervals.current.set(executionId, intervalId);

      // Poll immediately once
      pollExecution();
    },
    [],
  );

  // New function to get execution trace with proper execution_id
  // Updated to accept isRunning parameter for context-aware execution tracing
  const getExecutionTraceWithLatestId = useCallback(
    async (benchmarkId: string, isRunning: boolean = false): Promise<any> => {
      try {
        // PRIORITY 1: When actively running, use current execution from executions Map
        if (isRunning) {
          // Find current running execution for this benchmark
          const currentExecution = Array.from(executions.values()).find(
            (exec) => exec.benchmark_id === benchmarkId,
          );

          if (currentExecution) {
            console.log(
              `🔄 [EXECUTION_TRACE] Using current running execution: ${currentExecution.id}`,
            );
            return await apiClient.getExecutionTrace(
              benchmarkId,
              currentExecution.id,
            );
          } else {
            console.warn(
              `⚠️ [EXECUTION_TRACE] isRunning=true but no current execution found for ${benchmarkId}`,
            );
          }
        }

        // PRIORITY 2: Fall back to database historical data (for info viewing)
        // First, get benchmark with recent executions to find latest execution_id
        const benchmarkData =
          await apiClient.getBenchmarkWithExecutions(benchmarkId);

        // Use the latest execution_id if available
        const latestExecutionId = benchmarkData.latest_execution_id;

        if (latestExecutionId) {
          console.log(
            `📚 [EXECUTION_TRACE] Using historical execution: ${latestExecutionId}`,
          );
          return await apiClient.getExecutionTrace(
            benchmarkId,
            latestExecutionId,
          );
        } else {
          // Return empty result if no execution_id found - prevents stale cache
          return {
            benchmark_id: benchmarkId,
            execution_id: null,
            error: "No execution found",
            message: "No executions available for this benchmark",
            trace: "",
            is_running: false,
            progress: 0.0,
          };
        }
      } catch (error) {
        console.error(
          `Failed to get execution trace for benchmark ${benchmarkId}:`,
          error,
        );
        throw error;
      }
    },
    [executions],
  );

  // New function to get transaction logs with proper execution_id
  // Updated to accept isRunning parameter for context-aware transaction log fetching
  const getTransactionLogsWithLatestId = useCallback(
    async (benchmarkId: string, isRunning: boolean = false): Promise<any> => {
      try {
        // PRIORITY 1: When actively running, use current execution from executions Map
        if (isRunning) {
          // Find current running execution for this benchmark
          const currentExecution = Array.from(executions.values()).find(
            (exec) => exec.benchmark_id === benchmarkId,
          );

          if (currentExecution) {
            console.log(
              `🔄 [TRANSACTION_LOGS] Using current running execution: ${currentExecution.id}`,
            );
            return await apiClient.getTransactionLogs(
              benchmarkId,
              currentExecution.id,
            );
          } else {
            console.warn(
              `⚠️ [TRANSACTION_LOGS] isRunning=true but no current execution found for ${benchmarkId}`,
            );
          }
        }

        // PRIORITY 2: Fall back to database historical data (for info viewing)
        // First, get benchmark with recent executions to find latest execution_id
        const benchmarkData =
          await apiClient.getBenchmarkWithExecutions(benchmarkId);

        // Use the latest execution_id if available
        const latestExecutionId = benchmarkData.latest_execution_id;

        if (latestExecutionId) {
          console.log(
            `📚 [TRANSACTION_LOGS] Using historical execution: ${latestExecutionId}`,
          );
          return await apiClient.getTransactionLogs(
            benchmarkId,
            latestExecutionId,
          );
        } else {
          // Return empty result if no execution_id found - prevents stale cache
          return {
            benchmark_id: benchmarkId,
            execution_id: null,
            error: "No execution found",
            message: "No executions available for this benchmark",
            trace: "📝 No execution trace available",
            is_running: false,
            progress: 0.0,
          };
        }
      } catch (error) {
        console.error(
          `Failed to get transaction logs for benchmark ${benchmarkId}:`,
          error,
        );
        throw error;
      }
    },
    [executions],
  );

  const stopPolling = useCallback((executionId: string) => {
    const intervalId = pollingIntervals.current.get(executionId);
    if (intervalId) {
      clearInterval(intervalId);
      pollingIntervals.current.delete(executionId);
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

  // Additional cleanup when benchmarks change
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

  const setCompletionCallback = useCallback(
    (callback: (executionId: string, execution: ExecutionState) => void) => {
      completionCallback.current = callback;
    },
    [],
  );

  return {
    benchmarks,
    loading,
    error,
    refetch: fetchBenchmarks,
    executions,
    updateExecution,
    clearExecutions,
    setCompletionCallback,
    getExecutionTraceWithLatestId,
    getTransactionLogsWithLatestId,
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

export function useExecutionTrace() {
  const { getExecutionTraceWithLatestId } = useBenchmarkExecution();

  return {
    getExecutionTraceWithLatestId,
  };
}

export function useTransactionLogs() {
  const { getTransactionLogsWithLatestId } = useBenchmarkExecution();

  return {
    getTransactionLogsWithLatestId,
  };
}
