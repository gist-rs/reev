// BenchmarkGrid component for main dashboard display

import { useState, useCallback } from 'preact/hooks';
import { AgentPerformanceSummary, BenchmarkResult } from '../types/benchmark';
import { useAgentPerformance, useApiHealth } from '../hooks/useApiData';
import { useResponsiveLayout } from '../hooks/useResponsiveLayout';
import { DesktopLayout } from './DesktopLayout';
import { MobileLayout } from './MobileLayout';
import { FilterPanel } from './FilterPanel';
import { useResults } from '../hooks/useApiData';

interface BenchmarkGridProps {
  className?: string;
}

export function BenchmarkGrid({ className = "" }: BenchmarkGridProps) {
  const { isDesktop } = useResponsiveLayout();
  const { isHealthy, checking } = useApiHealth();
  const [selectedResult, setSelectedResult] = useState<BenchmarkResult | null>(null);

  // Try to get data from agent performance endpoint first, fallback to results
  const { data: agentData, loading: agentLoading, error: agentError } = useAgentPerformance();
  const {
    data: resultsData,
    loading: resultsLoading,
    error: resultsError,
    query,
    updateQuery
  } = useResults();

  const loading = agentLoading || resultsLoading;
  const error = agentError || resultsError;
  const data = agentData || [];

  const handleBenchmarkClick = useCallback((result: BenchmarkResult) => {
    setSelectedResult(result);
    console.log('Benchmark clicked:', result);
  }, []);

  const handleCloseModal = useCallback(() => {
    setSelectedResult(null);
  }, []);

  // Loading state
  if (loading) {
    return (
      <div className={`flex items-center justify-center min-h-96 ${className}`}>
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500 mx-auto mb-4"></div>
          <p className="text-gray-600">Loading benchmark results...</p>
        </div>
      </div>
    );
  }

  // Error state
  if (error) {
    return (
      <div className={`flex items-center justify-center min-h-96 ${className}`}>
        <div className="text-center max-w-md">
          <div className="text-red-500 mb-4">
            <svg class="w-16 h-16 mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
          </div>
          <h3 className="text-lg font-semibold text-gray-900 mb-2">Failed to load data</h3>
          <p className="text-gray-600 mb-4">{error}</p>
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
  if (!data || data.length === 0) {
    return (
      <div className={`flex items-center justify-center min-h-96 ${className}`}>
        <div className="text-center">
          <div className="text-gray-400 mb-4">
            <svg class="w-16 h-16 mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
            </svg>
          </div>
          <h3 className="text-lg font-semibold text-gray-900 mb-2">No benchmark data available</h3>
          <p className="text-gray-600">Run some benchmarks to see results here.</p>
        </div>
      </div>
    );
  }

  // Health warning
  if (!checking && !isHealthy) {
    return (
      <div className={`flex items-center justify-center min-h-96 ${className}`}>
        <div className="text-center max-w-md">
          <div className="text-yellow-500 mb-4">
            <svg class="w-16 h-16 mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
          </div>
          <h3 className="text-lg font-semibold text-gray-900 mb-2">API Unavailable</h3>
          <p className="text-gray-600 mb-4">Cannot connect to the API server. Please ensure it's running.</p>
          <button
            onClick={() => window.location.reload()}
            className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 transition-colors"
          >
            Retry Connection
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className={`min-h-screen bg-gray-50 ${className}`}>
      {/* Header */}
      <header className="bg-white shadow-sm border-b">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex items-center justify-between h-16">
            <div className="flex items-center">
              <h1 className="text-2xl font-bold text-gray-900">Reev Benchmark Dashboard</h1>
              {!checking && (
                <span className={`ml-3 px-2 py-1 text-xs rounded-full ${
                  isHealthy
                    ? 'bg-green-100 text-green-800'
                    : 'bg-red-100 text-red-800'
                }`}>
                  {isHealthy ? 'Connected' : 'Disconnected'}
                </span>
              )}
            </div>
            <div className="flex items-center space-x-4">
              <span className="text-sm text-gray-600">
                {data.reduce((sum, agent) => sum + agent.results.length, 0)} total results
              </span>
              <span className="text-sm text-gray-600">
                {data.length} agents
              </span>
            </div>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="max-w-7xl mx-auto">
        {/* Filters */}
        <div className="p-4">
          <FilterPanel
            query={query}
            onQueryChange={updateQuery}
          />
        </div>

        {/* Benchmark Grid */}
        <div className="bg-white border-t">
          {isDesktop ? (
            <DesktopLayout
              agents={data}
              onBenchmarkClick={handleBenchmarkClick}
            />
          ) : (
            <MobileLayout
              agents={data}
              onBenchmarkClick={handleBenchmarkClick}
            />
          )}
        </div>

        {/* Legend */}
        <div className="p-4 bg-white border-t">
          <div className="flex items-center justify-center space-x-6 text-sm text-gray-600">
            <div className="flex items-center">
              <div className="w-4 h-4 bg-green-500 rounded mr-2"></div>
              <span>Perfect (100%)</span>
            </div>
            <div className="flex items-center">
              <div className="w-4 h-4 bg-yellow-500 rounded mr-2"></div>
              <span>Partial (25-99%)</span>
            </div>
            <div className="flex items-center">
              <div className="w-4 h-4 bg-red-500 rounded mr-2"></div>
              <span>Poor (&lt;25%)</span>
            </div>
          </div>
        </div>
      </main>

      {/* Result Detail Modal */}
      {selectedResult && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
          <div className="bg-white rounded-lg max-w-md w-full max-h-96 overflow-y-auto">
            <div className="p-6">
              <div className="flex items-center justify-between mb-4">
                <h3 className="text-lg font-semibold">Benchmark Details</h3>
                <button
                  onClick={handleCloseModal}
                  className="text-gray-400 hover:text-gray-600"
                >
                  <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                  </svg>
                </button>
              </div>

              <div className="space-y-3">
                <div>
                  <span className="font-medium">Benchmark:</span>
                  <span className="ml-2">{selectedResult.benchmark_id}</span>
                </div>
                <div>
                  <span className="font-medium">Agent:</span>
                  <span className="ml-2">{selectedResult.agent_type}</span>
                </div>
                <div>
                  <span className="font-medium">Score:</span>
                  <span className={`ml-2 font-semibold ${
                    selectedResult.color_class === 'green' ? 'text-green-600' :
                    selectedResult.color_class === 'yellow' ? 'text-yellow-600' :
                    'text-red-600'
                  }`}>
                    {(selectedResult.score * 100).toFixed(1)}%
                  </span>
                </div>
                <div>
                  <span className="font-medium">Status:</span>
                  <span className={`ml-2 ${
                    selectedResult.final_status === 'Succeeded' ? 'text-green-600' : 'text-red-600'
                  }`}>
                    {selectedResult.final_status}
                  </span>
                </div>
                <div>
                  <span className="font-medium">Execution Time:</span>
                  <span className="ml-2">{selectedResult.execution_time_ms}ms</span>
                </div>
                <div>
                  <span className="font-medium">Timestamp:</span>
                  <span className="ml-2">{new Date(selectedResult.timestamp).toLocaleString()}</span>
                </div>
              </div>

              <div className="mt-6 pt-4 border-t">
                <button
                  onClick={handleCloseModal}
                  className="w-full px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 transition-colors"
                >
                  Close
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
