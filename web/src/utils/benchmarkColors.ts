import { BenchmarkResult, ExecutionStatus } from "../types/benchmark";

/**
 * Gets the appropriate color class for a benchmark result
 * @param result The benchmark result to evaluate
 * @param isRunning Whether the benchmark is currently running
 * @returns Tailwind CSS class name for the background color
 */
export function getBenchmarkColorClass(
  result: BenchmarkResult,
  isRunning: boolean = false,
): string {
  // If running, don't apply static background color - animation will handle it
  if (isRunning) return "";

  // Handle missing or invalid color_class
  const validColors = ["green", "yellow", "red", "gray"];

  // Use color_class if it's valid, otherwise fall back to score-based logic
  if (result.color_class && validColors.includes(result.color_class)) {
    const colorMap: Record<string, string> = {
      gray: "bg-gray-400",
      green: "bg-green-500",
      yellow: "bg-yellow-500",
      red: "bg-red-500",
    };
    return colorMap[result.color_class];
  }

  // Fallback to score-based logic
  if (result.score >= 1.0) return "bg-green-500"; // 100%
  if (result.score >= 0.25) return "bg-yellow-500"; // <100% but >=25%
  return "bg-red-500"; // <25%
}
