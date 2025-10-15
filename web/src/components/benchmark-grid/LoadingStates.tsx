interface LoadingStatesProps {
  className?: string;
  benchmarksLoading?: boolean;
  agentPerformanceLoading?: boolean;
  benchmarksError?: string | null;
  agentPerformanceError?: string | null;
  agentPerformanceData?: any;
  benchmarks?: any[] | null;
}

export function LoadingStates({
  className = "",
  benchmarksLoading = false,
  agentPerformanceLoading = false,
  benchmarksError = null,
  agentPerformanceError = null,
  agentPerformanceData,
  benchmarks,
}: LoadingStatesProps) {
  const isLoading = benchmarksLoading || agentPerformanceLoading;
  const hasError = benchmarksError || agentPerformanceError;
  const hasNoData =
    (!agentPerformanceData || agentPerformanceData.length === 0) &&
    (!benchmarks || benchmarks.length === 0);

  // Loading state
  if (isLoading) {
    return (
      <div className={`flex items-center justify-center min-h-96 ${className}`}>
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500 mx-auto mb-4"></div>
          <p className="text-gray-600 dark:text-gray-400">
            Loading benchmark results...
          </p>
        </div>
      </div>
    );
  }

  // Error state
  if (hasError) {
    return (
      <div className={`flex items-center justify-center min-h-96 ${className}`}>
        <div className="text-center max-w-md">
          <div className="text-red-500 dark:text-red-400 mb-4">
            <svg
              className="w-16 h-16 mx-auto"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth="2"
                d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
          </div>
          <h3 className="text-lg font-semibold text-red-600 dark:text-red-400 mb-2">
            Failed to load data
          </h3>
          <p className="text-red-500 dark:text-red-400 mb-4">
            {agentPerformanceError || benchmarksError}
          </p>
          <button
            onClick={() => window.location.reload()}
            className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 transition-colors"
          >
            Retry
          </button>
        </div>
      </div>
    );
  }

  // No data state
  if (hasNoData) {
    return (
      <div className={`flex items-center justify-center min-h-96 ${className}`}>
        <div className="text-center">
          <div className="text-gray-400 dark:text-gray-500 mb-4">
            <svg
              className="w-16 h-16 mx-auto"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth="2"
                d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
              />
            </svg>
          </div>
          <h3 className="text-lg font-semibold text-gray-700 dark:text-gray-300 mb-2">
            No benchmark data available
          </h3>
          <p className="text-gray-600 dark:text-gray-400">
            Run some benchmarks to see results here.
          </p>
        </div>
      </div>
    );
  }

  return null;
}
