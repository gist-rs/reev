interface AgentCardHeaderProps {
  agentType: string;
  overallPercentage: number;
}

export function AgentCardHeader({ agentType, overallPercentage }: AgentCardHeaderProps) {
  return (
    <div className="flex items-center justify-between mb-4">
      <h3 className="text-lg font-bold text-gray-900 dark:text-gray-100">
        {agentType}
      </h3>
      <div className="text-sm text-gray-600 dark:text-gray-400">
        <span
          className={
            overallPercentage >= 0.9
              ? "text-green-600 dark:text-green-400"
              : overallPercentage >= 0.7
                ? "text-yellow-600 dark:text-yellow-400"
                : overallPercentage == 0.0
                  ? "text-gray-400 dark:text-gray-500"
                  : "text-red-600 dark:text-red-400"
          }
        >
          {(overallPercentage * 100).toFixed(1)}%
        </span>
      </div>
    </div>
  );
}
