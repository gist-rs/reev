// FilterPanel component for filtering benchmark results

import { useState } from 'preact/hooks';
import { ResultsQuery } from '../types/benchmark';
import { useAgents } from '../hooks/useApiData';

interface FilterPanelProps {
  query: ResultsQuery;
  onQueryChange: (query: Partial<ResultsQuery>) => void;
  className?: string;
}

export function FilterPanel({ query, onQueryChange, className = "" }: FilterPanelProps) {
  const [isExpanded, setIsExpanded] = useState(false);
  const { data: agents } = useAgents();

  const handleAgentChange = (agent: string) => {
    if (query.agent === agent) {
      onQueryChange({ agent: undefined });
    } else {
      onQueryChange({ agent });
    }
  };

  const handleScoreRangeChange = (field: 'min_score' | 'max_score', value: string) => {
    const numValue = value === '' ? undefined : parseFloat(value);
    if (isNaN(numValue as number)) return;
    onQueryChange({ [field]: numValue });
  };

  const handleDateChange = (field: 'start_date' | 'end_date', value: string) => {
    onQueryChange({ [field]: value || undefined });
  };

  const clearFilters = () => {
    onQueryChange({
      agent: undefined,
      min_score: undefined,
      max_score: undefined,
      start_date: undefined,
      end_date: undefined,
    });
  };

  const hasActiveFilters = !!(
    query.agent ||
    query.min_score !== undefined ||
    query.max_score !== undefined ||
    query.start_date ||
    query.end_date
  );

  return (
    <div className={`bg-white border rounded-lg shadow-sm ${className}`}>
      {/* Filter Toggle Button */}
      <button
        onClick={() => setIsExpanded(!isExpanded)}
        className="w-full px-4 py-3 flex items-center justify-between hover:bg-gray-50 transition-colors"
      >
        <div className="flex items-center gap-2">
          <svg class="w-5 h-5 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z" />
          </svg>
          <span className="font-medium">Filters</span>
          {hasActiveFilters && (
            <span className="bg-blue-100 text-blue-800 text-xs px-2 py-1 rounded-full">
              Active
            </span>
          )}
        </div>
        <svg
          class={`w-5 h-5 text-gray-500 transform transition-transform ${isExpanded ? 'rotate-180' : ''}`}
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
        </svg>
      </button>

      {/* Filter Content */}
      {isExpanded && (
        <div className="border-t px-4 py-4 space-y-4">
          {/* Agent Filter */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">Agent Type</label>
            <div className="flex flex-wrap gap-2">
              {agents?.map((agent) => (
                <button
                  key={agent}
                  onClick={() => handleAgentChange(agent)}
                  className={`px-3 py-1 rounded-full text-sm transition-colors ${
                    query.agent === agent
                      ? 'bg-blue-500 text-white'
                      : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
                  }`}
                >
                  {agent}
                </button>
              ))}
            </div>
          </div>

          {/* Score Range Filter */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">Score Range (%)</label>
            <div className="flex items-center gap-2">
              <input
                type="number"
                min="0"
                max="100"
                step="0.1"
                placeholder="Min"
                value={query.min_score !== undefined ? (query.min_score * 100) : ''}
                onChange={(e) => handleScoreRangeChange('min_score', e.currentTarget.value)}
                className="w-20 px-2 py-1 border rounded text-sm"
              />
              <span className="text-gray-500">to</span>
              <input
                type="number"
                min="0"
                max="100"
                step="0.1"
                placeholder="Max"
                value={query.max_score !== undefined ? (query.max_score * 100) : ''}
                onChange={(e) => handleScoreRangeChange('max_score', e.currentTarget.value)}
                className="w-20 px-2 py-1 border rounded text-sm"
              />
              <span className="text-gray-500">%</span>
            </div>
          </div>

          {/* Date Range Filter */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">Date Range</label>
            <div className="flex flex-col gap-2">
              <input
                type="date"
                placeholder="Start date"
                value={query.start_date || ''}
                onChange={(e) => handleDateChange('start_date', e.currentTarget.value)}
                className="px-3 py-1 border rounded text-sm"
              />
              <input
                type="date"
                placeholder="End date"
                value={query.end_date || ''}
                onChange={(e) => handleDateChange('end_date', e.currentTarget.value)}
                className="px-3 py-1 border rounded text-sm"
              />
            </div>
          </div>

          {/* Clear Filters Button */}
          {hasActiveFilters && (
            <div className="pt-2 border-t">
              <button
                onClick={clearFilters}
                className="px-4 py-2 text-sm text-red-600 hover:text-red-700 hover:bg-red-50 rounded transition-colors"
              >
                Clear All Filters
              </button>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
