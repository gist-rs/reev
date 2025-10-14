import { useState, useEffect, useCallback } from "react";
import { BenchmarkDetails } from "../types/benchmark";
import { apiClient } from "../services/api";

// Global cache to share between components
const benchmarkDetailsCache = new Map<string, BenchmarkDetails>();
const loadingCache = new Set<string>();

export function useBenchmarkDetails() {
  const [localCache, setLocalCache] = useState<Map<string, BenchmarkDetails>>(
    new Map(),
  );

  const fetchBenchmarkDetails = useCallback(
    async (benchmarkId: string): Promise<BenchmarkDetails | null> => {
      // Check global cache first
      if (benchmarkDetailsCache.has(benchmarkId)) {
        return benchmarkDetailsCache.get(benchmarkId)!;
      }

      // Check if already loading
      if (loadingCache.has(benchmarkId)) {
        return null;
      }

      // Mark as loading
      loadingCache.add(benchmarkId);

      try {
        const details = await apiClient.getBenchmark(benchmarkId);
        console.log(`Raw API response for ${benchmarkId}:`, details);

        const benchmarkDetails: BenchmarkDetails = {
          id: details.id || benchmarkId,
          description: details.description || "No description available",
          tags: details.tags || [],
        };

        console.log(`Processed details for ${benchmarkId}:`, benchmarkDetails);

        // Cache the result
        benchmarkDetailsCache.set(benchmarkId, benchmarkDetails);
        setLocalCache(new Map(benchmarkDetailsCache));

        return benchmarkDetails;
      } catch (error) {
        console.error(
          `Failed to fetch benchmark details for ${benchmarkId}:`,
          error,
        );
        console.error(`Error details:`, {
          message: error.message,
          stack: error.stack,
          benchmarkId,
        });

        // Cache fallback details
        const fallbackDetails: BenchmarkDetails = {
          id: benchmarkId,
          description: "Failed to load description",
          tags: [],
        };

        benchmarkDetailsCache.set(benchmarkId, fallbackDetails);
        setLocalCache(new Map(benchmarkDetailsCache));

        return fallbackDetails;
      } finally {
        loadingCache.delete(benchmarkId);
      }
    },
    [],
  );

  const getCachedDetails = useCallback(
    (benchmarkId: string): BenchmarkDetails | null => {
      return benchmarkDetailsCache.get(benchmarkId) || null;
    },
    [],
  );

  const preloadBenchmarkDetails = useCallback(
    async (benchmarkIds: string[]) => {
      const uncachedIds = benchmarkIds.filter(
        (id) => !benchmarkDetailsCache.has(id),
      );

      if (uncachedIds.length === 0) return;

      // Fetch in parallel but limit concurrent requests
      const batchSize = 5;
      for (let i = 0; i < uncachedIds.length; i += batchSize) {
        const batch = uncachedIds.slice(i, i + batchSize);
        await Promise.all(
          batch.map((id) => fetchBenchmarkDetails(id).catch(() => null)),
        );
      }
    },
    [fetchBenchmarkDetails],
  );

  return {
    fetchBenchmarkDetails,
    getCachedDetails,
    preloadBenchmarkDetails,
    cache: localCache,
  };
}
