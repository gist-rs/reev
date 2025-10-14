// AgentSelector component for selecting and configuring different agent types

import { useState, useCallback } from "preact/hooks";

import { apiClient } from "../services/api";

interface AgentSelectorProps {
  selectedAgent: string;
  onAgentChange: (agent: string) => void;
  isRunning: boolean;
}

interface AgentTab {
  id: string;
  displayName: string;
  disabled?: boolean;
  requiresConfig?: boolean;
}

const AGENT_TABS: AgentTab[] = [
  { id: "deterministic", displayName: "Deterministic" },
  { id: "local", displayName: "Local (Qwen3)", requiresConfig: true },
  { id: "glm-4.6", displayName: "GLM 4.6", requiresConfig: true },
  {
    id: "gemini-2.5-flash-lite",
    displayName: "Gemini 2.5 Flash Lite",
    requiresConfig: true,
  },
];

export function AgentSelector({
  selectedAgent,
  onAgentChange,
  isRunning,
}: AgentSelectorProps) {
  const handleAgentTabClick = useCallback(
    (agent: AgentTab) => {
      if (agent.disabled || isRunning) return;

      onAgentChange(agent.id);
    },
    [onAgentChange, isRunning],
  );

  const getTabStyle = (agent: AgentTab) => {
    const baseClasses =
      "px-4 py-2 text-sm font-medium transition-colors border-b-2";
    const isSelected = selectedAgent === agent.id;
    const isDisabled = agent.disabled || isRunning;

    if (isDisabled) {
      return `${baseClasses} text-gray-400 dark:text-gray-500 border-gray-300 dark:border-gray-700 cursor-not-allowed`;
    }

    if (isSelected) {
      return `${baseClasses} text-blue-600 dark:text-blue-400 border-blue-600 dark:border-blue-400 bg-blue-50 dark:bg-blue-900/20`;
    }

    return `${baseClasses} text-gray-600 dark:text-gray-300 border-gray-300 dark:border-gray-700 hover:text-gray-800 dark:hover:text-gray-100 hover:border-gray-400 dark:hover:border-gray-600 cursor-pointer`;
  };

  return (
    <div className="bg-white dark:bg-gray-800 shadow-sm border-b dark:border-gray-700">
      <div className="mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex items-center justify-between h-16">
          {/* Agent Tabs */}
          <div className="flex items-center space-x-1">
            {AGENT_TABS.map((agent) => (
              <button
                key={agent.id}
                className={getTabStyle(agent)}
                onClick={() => handleAgentTabClick(agent)}
                disabled={agent.disabled || isRunning}
                title={
                  isRunning
                    ? "Cannot change agent while benchmark is running"
                    : undefined
                }
              >
                {agent.displayName}
              </button>
            ))}
          </div>

          {/* Status */}
          <div className="flex items-center space-x-4">
            <span className="text-sm text-gray-600 dark:text-gray-400">
              {isRunning ? "Running..." : "Ready"}
            </span>
          </div>
        </div>
      </div>
    </div>
  );
}
