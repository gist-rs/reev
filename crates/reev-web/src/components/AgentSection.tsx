// AgentSection component for grouping benchmarks by agent type

import { AgentPerformanceSummary, BenchmarkResult } from '../types/benchmark';
import { BenchmarkBox } from './BenchmarkBox';
import { useResponsiveLayout } from '../hooks/useResponsiveLayout';

interface AgentSectionProps {
  agent: AgentPerformanceSummary;
  onBenchmarkClick?: (result: BenchmarkResult) => void;
  showStats?: boolean;
}

export function AgentSection({ agent, onBenchmarkClick, showStats = true }: AgentSectionProps) {
  const { isMobile } = useResponsiveLayout();

  const getScoreColor = (score: number): string => {
    if (score >= 1.0) return 'text-green-600';
    if (score >= 0.25) return 'text-yellow-600';
    return 'text-red-600';
  };

  const getSuccessRateColor = (rate: number): string => {
    if (rate >= 0.9) return 'text-green-600';
    if (rate >= 0.7) return 'text-yellow-600';
    return 'text-red-600';
  };

  return (
    <div className={`${isMobile ? 'mb-6' : 'mb-4'}`}>
      {/* Agent Header */}
      <div className="flex items-center justify-between mb-3">
        <h3 className={`${isMobile ? 'text-lg' : 'text-xl'} font-bold border-b pb-2 flex-1`}>
          {agent.agent_type}
        </h3>

        {showStats && (
          <div className={`${isMobile ? 'text-sm' : 'text-base'} text-gray-600 ml-4`}>
            <span className="mr-4">
              Avg: <span className={getScoreColor(agent.average_score)}>
                {(agent.average_score * 100).toFixed(1)}%
              </span>
            </span>
            <span>
              Success: <span className={getSuccessRateColor(agent.success_rate)}>
                {(agent.success_rate * 100).toFixed(1)}%
              </span>
            </span>
          </div>
        )}
      </div>

      {/* Benchmark Grid */}
      <div className={`${isMobile ? 'flex flex-wrap gap-1' : 'flex flex-wrap gap-1'}`}
           style={isMobile ? {} : { width: '100%' }}>
        {agent.results.map((result) => (
          <BenchmarkBox
            key={`${agent.agent_type}-${result.benchmark_id}`}
            result={result}
            onClick={onBenchmarkClick}
          />
        ))}
      </div>

      {/* Detailed Stats (shown on mobile or when expanded) */}
      {showStats && isMobile && (
        <div className="mt-4 p-3 bg-gray-50 rounded-lg text-sm">
          <div className="grid grid-cols-2 gap-2">
            <div>
              <span className="font-semibold">Total:</span> {agent.total_benchmarks}
            </div>
            <div>
              <span className="font-semibold">Avg Score:</span>{' '}
              <span className={getScoreColor(agent.average_score)}>
                {(agent.average_score * 100).toFixed(1)}%
              </span>
            </div>
            <div>
              <span className="font-semibold">Success Rate:</span>{' '}
              <span className={getSuccessRateColor(agent.success_rate)}>
                {(agent.success_rate * 100).toFixed(1)}%
              </span>
            </div>
            <div>
              <span className="font-semibold">Benchmarks:</span> {agent.results.length}
            </div>
          </div>

          {agent.best_benchmarks.length > 0 && (
            <div className="mt-2">
              <span className="font-semibold">Best:</span>{' '}
              <span className="text-green-600">
                {agent.best_benchmarks.slice(0, 3).join(', ')}
                {agent.best_benchmarks.length > 3 && '...'}
              </span>
            </div>
          )}

          {agent.worst_benchmarks.length > 0 && (
            <div className="mt-1">
              <span className="font-semibold">Needs Work:</span>{' '}
              <span className="text-red-600">
                {agent.worst_benchmarks.slice(0, 3).join(', ')}
                {agent.worst_benchmarks.length > 3 && '...'}
              </span>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
