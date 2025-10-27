import { BenchmarkResult } from "../types/benchmark";

/**
 * Gets the appropriate color class for a benchmark result
 * @param result The benchmark result to evaluate
 * @param isRunning Whether the benchmark is currently running
 * @returns Tailwind CSS class name for the background color
 */
export function getBenchmarkColorClass(
  result: BenchmarkResult,
  isRunning: boolean = false
): string {
  // If running, don't apply static background color - animation will handle it
  if (isRunning) return "";

  // Use color_class if specified, otherwise fall back to score-based logic
  if (result.color_class === "gray") return "bg-gray-400";
  if (result.color_class === "green") return "bg-green-500";
  if (result.color_class === "yellow") return "bg-yellow-500";
  if (result.color_class === "red") return "bg-red-500";

  // Fallback to score-based logic
  if (result.score >= 1.0) return "bg-green-500"; // 100%
  if (result.score >= 0.25) return "bg-yellow-500"; // <100% but >=25%
  return "bg-red-500"; // <25%
}

/**
 * Gets the appropriate color class for a benchmark status
 * @param status The execution status
 * @param result The benchmark result (optional, for more accurate coloring)
 * @returns Tailwind CSS class name for the background color
 */
export function getBenchmarkStatusColor(
  status: string,
  result?: BenchmarkResult
): string {
  // If we have a result with color_class, use it for more accurate coloring
  if (result) {
    return getBenchmarkColorClass(result);
  }

  // Fallback to status-based logic
  switch (status) {
    case "Completed":
      return "bg-green-500";
    case "Failed":
      return "bg-red-500";
    case "Running":
      return "bg-blue-500";
    default:
      return "bg-gray-300 dark:bg-gray-600";
  }
}
