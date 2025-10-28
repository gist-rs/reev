import { BenchmarkResult } from "../types/benchmark";

/**
 * Validates and normalizes a result object to BenchmarkResult format
 * @param result The result object to validate
 * @returns Validated BenchmarkResult
 * @throws Error if the result doesn't match expected structure
 */
function validateAndNormalizeResult(result: any): BenchmarkResult {
  if (!result || typeof result !== "object") {
    throw new Error(`Invalid result: expected object, got ${typeof result}`);
  }

  // Required fields
  const required = [
    "id",
    "benchmark_id",
    "agent_type",
    "score",
    "final_status",
    "execution_time_ms",
    "timestamp",
    "color_class",
  ];
  for (const field of required) {
    if (!(field in result)) {
      throw new Error(
        `Missing required field: ${field} in result: ${JSON.stringify(result)}`,
      );
    }
  }

  // Validate color_class
  const validColors = ["green", "yellow", "red", "gray"];
  if (!validColors.includes(result.color_class)) {
    throw new Error(
      `Invalid color_class: ${result.color_class}, must be one of: ${validColors.join(", ")}`,
    );
  }

  // Validate score
  if (typeof result.score !== "number" || result.score < 0) {
    throw new Error(
      `Invalid score: ${result.score}, must be a non-negative number`,
    );
  }

  return result as BenchmarkResult;
}

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

  // Validate result structure
  const validatedResult = validateAndNormalizeResult(result);

  // Use color_class if specified, otherwise fall back to score-based logic
  if (validatedResult.color_class === "gray") return "bg-gray-400";
  if (validatedResult.color_class === "green") return "bg-green-500";
  if (validatedResult.color_class === "yellow") return "bg-yellow-500";
  if (validatedResult.color_class === "red") return "bg-red-500";

  // Fallback to score-based logic
  if (validatedResult.score >= 1.0) return "bg-green-500"; // 100%
  if (validatedResult.score >= 0.25) return "bg-yellow-500"; // <100% but >=25%
  return "bg-red-500"; // <25%
}
