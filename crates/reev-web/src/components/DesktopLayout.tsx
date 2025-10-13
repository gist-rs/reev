// DesktopLayout component for horizontal agent arrangement

import { AgentPerformanceSummary, BenchmarkResult } from '../types/benchmark';
import { AgentSection } from './AgentSection';

interface DesktopLayoutProps {
  agents: AgentPerformanceSummary[];
  onBenchmarkClick?: (result: BenchmarkResult) => void;
}

export function DesktopLayout({ agents, onBenchmarkClick }: DesktopLayoutProps) {
  return (
    <div className="flex flex-col gap-4 p-4">
      {/* Agent Headers Row */}
      <div className="flex gap-4 items-center px-1">
        <span className="font-bold text-lg w-1/4">Deterministic</span>
        <span className="font-bold text-lg w-1/4">Local</span>
        <span className="font-bold text-lg w-1/4">GLM 4.6</span>
        <span className="font-bold text-lg w-1/4">Gemini</span>
      </div>

      {/* Agent Results Row */}
      <div className="flex gap-2">
        {agents.map((agent) => (
          <div key={agent.agent_type} className="flex flex-wrap gap-1" style={{ width: '25%' }}>
            <AgentSection
              agent={agent}
              onBenchmarkClick={onBenchmarkClick}
              showStats={false}
            />
          </div>
        ))}
      </div>

      {/* Detailed Agent Sections Below */}
      <div className="grid grid-cols-1 gap-6 mt-6">
        {agents.map((agent) => (
          <AgentSection
            key={`details-${agent.agent_type}`}
            agent={agent}
            onBenchmarkClick={onBenchmarkClick}
            showStats={true}
          />
        ))}
      </div>
    </div>
  );
}
