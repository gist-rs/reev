// MobileLayout component for vertical agent arrangement

import { AgentPerformanceSummary, BenchmarkResult } from '../types/benchmark';
import { AgentSection } from './AgentSection';

interface MobileLayoutProps {
  agents: AgentPerformanceSummary[];
  onBenchmarkClick?: (result: BenchmarkResult) => void;
}

export function MobileLayout({ agents, onBenchmarkClick }: MobileLayoutProps) {
  return (
    <div className="flex flex-col gap-6 p-4">
      {agents.map((agent) => (
        <AgentSection
          key={agent.agent_type}
          agent={agent}
          onBenchmarkClick={onBenchmarkClick}
          showStats={true}
        />
      ))}
    </div>
  );
}
