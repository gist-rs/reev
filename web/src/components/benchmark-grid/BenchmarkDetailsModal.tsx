import { BenchmarkResult } from "../../types/benchmark";

interface BenchmarkDetailsModalProps {
  selectedResult: BenchmarkResult | null;
  onClose: () => void;
  onRunBenchmark?: (benchmarkId: string, agentType?: string) => void;
  isRunning?: boolean;
  handleRunBenchmark?: (benchmarkId: string, agentType?: string) => void;
}

export function BenchmarkDetailsModal({
  selectedResult,
  onClose,
  onRunBenchmark,
  isRunning = false,
  handleRunBenchmark,
}: BenchmarkDetailsModalProps) {
  if (!selectedResult) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
      <div className="bg-white dark:bg-gray-800 rounded-lg max-w-md w-full max-h-[80vh] overflow-y-auto">
        <div className="p-6">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
              Benchmark Details
            </h3>
            <button
              onClick={onClose}
              className="text-gray-400 dark:text-gray-500 hover:text-gray-600 dark:hover:text-gray-300"
            >
              <svg
                className="w-6 h-6"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth="2"
                  d="M6 18L18 6M6 6l12 12"
                />
              </svg>
            </button>
          </div>

          <div className="space-y-3">
            <div>
              <span className="font-medium text-gray-900 dark:text-gray-100">
                Benchmark:
              </span>
              <span className="ml-2 text-gray-800 dark:text-gray-200">
                {selectedResult.benchmark_id}
              </span>
            </div>
            <div>
              <span className="font-medium text-gray-900 dark:text-gray-100">
                Agent:
              </span>
              <span className="ml-2 text-gray-800 dark:text-gray-200">
                {selectedResult.agent_type}
              </span>
            </div>
            <div>
              <span className="font-medium text-gray-900 dark:text-gray-100">
                Score:
              </span>
              <span
                className={`ml-2 font-semibold ${
                  selectedResult.color_class === "green"
                    ? "text-green-600 dark:text-green-400"
                    : selectedResult.color_class === "yellow"
                      ? "text-yellow-600 dark:text-yellow-400"
                      : "text-red-600 dark:text-red-400"
                }`}
              >
                {(selectedResult.score * 100).toFixed(1)}%
              </span>
            </div>
            <div>
              <span className="font-medium text-gray-900 dark:text-gray-100">
                Status:
              </span>
              <span
                className={`ml-2 ${
                  selectedResult.final_status === "Succeeded"
                    ? "text-green-600 dark:text-green-400"
                    : "text-red-600 dark:text-red-400"
                }`}
              >
                {selectedResult.final_status}
              </span>
            </div>
            <div>
              <span className="font-medium text-gray-900 dark:text-gray-100">
                Execution Time:
              </span>
              <span className="ml-2 text-gray-800 dark:text-gray-200">
                {selectedResult.execution_time_ms}ms
              </span>
            </div>
            <div>
              <span className="font-medium text-gray-900 dark:text-gray-100">
                Timestamp:
              </span>
              <span className="ml-2 text-gray-800 dark:text-gray-200">
                {new Date(selectedResult.timestamp).toLocaleString()}
              </span>
            </div>
          </div>

          <div className="mt-6 pt-4 border-t border-gray-200 dark:border-gray-700">
            <div className="flex space-x-3">
              <button
                onClick={() => {
                  if (onRunBenchmark && !isRunning && handleRunBenchmark) {
                    handleRunBenchmark(
                      selectedResult.benchmark_id,
                      selectedResult.agent_type,
                    );
                    onClose();
                  }
                }}
                disabled={isRunning || !onRunBenchmark}
                className={`flex-1 px-4 py-2 rounded transition-colors ${
                  isRunning || !onRunBenchmark
                    ? "bg-gray-300 text-gray-500 cursor-not-allowed"
                    : "bg-green-600 text-white hover:bg-green-700"
                }`}
              >
                {isRunning ? "Running..." : "Run Benchmark"}
              </button>
              <button
                onClick={onClose}
                className="flex-1 px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 transition-colors"
              >
                Close
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
