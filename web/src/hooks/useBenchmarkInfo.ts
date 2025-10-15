import { useState, useEffect, useCallback } from "react";
import { BenchmarkInfo } from "../types/benchmark";
import { apiClient } from "../services/api";

export function useBenchmarkInfo() {
  const [benchmarkInfo, setBenchmarkInfo] = useState<
    Map<string, BenchmarkInfo>
  >(new Map());
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Load all benchmark info on mount
  const loadBenchmarkInfo = useCallback(async () => {
    setLoading(true);
    setError(null);

    try {
      const benchmarks = await apiClient.listBenchmarks();
      const infoMap = new Map<string, BenchmarkInfo>();

      benchmarks.forEach((benchmark) => {
        // API now returns full BenchmarkInfo objects from YAML files
        infoMap.set(benchmark.id, benchmark);
      });

      setBenchmarkInfo(infoMap);
      console.log(`Loaded ${benchmarks.length} benchmark information entries`);
    } catch (err) {
      setError(
        err instanceof Error
          ? err.message
          : "Failed to load benchmark information",
      );
      console.error("Failed to load benchmark info:", err);
    } finally {
      setLoading(false);
    }
  }, []);

  // Get benchmark info by ID
  const getBenchmarkInfo = useCallback(
    (benchmarkId: string): BenchmarkInfo | null => {
      return benchmarkInfo.get(benchmarkId) || null;
    },
    [benchmarkInfo],
  );

  // Get all benchmark info
  const getAllBenchmarkInfo = useCallback((): BenchmarkInfo[] => {
    return Array.from(benchmarkInfo.values());
  }, [benchmarkInfo]);

  // Refresh benchmark info
  const refresh = useCallback(() => {
    loadBenchmarkInfo();
  }, [loadBenchmarkInfo]);

  // Auto-load on mount
  useEffect(() => {
    loadBenchmarkInfo();
  }, [loadBenchmarkInfo]);

  return {
    benchmarkInfo,
    loading,
    error,
    getBenchmarkInfo,
    getAllBenchmarkInfo,
    refresh,
  };
}
