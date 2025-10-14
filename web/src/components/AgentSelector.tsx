// AgentSelector component for selecting and configuring different agent types

import { useState, useCallback } from "preact/hooks";
import { AgentConfig } from "../types/configuration";
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
  const [showConfig, setShowConfig] = useState(false);
  const [config, setConfig] = useState<AgentConfig>({
    agent_type: "",
    api_url: "",
    api_key: "",
  });
  const [configLoading, setConfigLoading] = useState(false);
  const [configError, setConfigError] = useState<string | null>(null);
  const [testingConnection, setTestingConnection] = useState(false);
  const [testResult, setTestResult] = useState<{
    status: string;
    message: string;
  } | null>(null);

  const handleAgentTabClick = useCallback(
    async (agent: AgentTab) => {
      if (agent.disabled || isRunning) return;

      onAgentChange(agent.id);

      // Load existing configuration for this agent
      if (agent.requiresConfig) {
        await loadAgentConfig(agent.id);
      }

      setTestResult(null);
      setConfigError(null);
    },
    [onAgentChange, isRunning],
  );

  const loadAgentConfig = async (agentType: string) => {
    setConfigLoading(true);
    setConfigError(null);

    try {
      const existingConfig = await apiClient.getAgentConfig(agentType);
      setConfig({
        agent_type: agentType,
        api_url: existingConfig.api_url || "",
        api_key: existingConfig.api_key || "",
      });
    } catch (error) {
      // Config not found is okay, start with empty config
      setConfig({
        agent_type: agentType,
        api_url: "",
        api_key: "",
      });
    } finally {
      setConfigLoading(false);
    }
  };

  const handleConfigChange = useCallback(
    (field: keyof AgentConfig, value: string) => {
      setConfig((prev) => ({
        ...prev,
        [field]: value,
      }));
      setTestResult(null);
      setConfigError(null);
    },
    [],
  );

  const handleSaveConfig = async () => {
    if (!config.agent_type) return;

    setConfigLoading(true);
    setConfigError(null);

    try {
      await apiClient.saveAgentConfig(config);
      setTestResult({
        status: "success",
        message: "Configuration saved successfully",
      });
    } catch (error) {
      setConfigError(
        error instanceof Error ? error.message : "Failed to save configuration",
      );
    } finally {
      setConfigLoading(false);
    }
  };

  const handleTestConnection = async () => {
    if (!config.agent_type) return;

    setTestingConnection(true);
    setTestResult(null);

    try {
      const result = await apiClient.testAgentConnection(config);
      setTestResult(result);
    } catch (error) {
      setTestResult({
        status: "error",
        message:
          error instanceof Error ? error.message : "Connection test failed",
      });
    } finally {
      setTestingConnection(false);
    }
  };

  const handleResetConfig = () => {
    setConfig({
      agent_type: config.agent_type,
      api_url: "",
      api_key: "",
    });
    setTestResult(null);
    setConfigError(null);
  };

  const getTabStyle = (agent: AgentTab) => {
    const baseClasses =
      "px-4 py-2 text-sm font-medium transition-colors border-b-2";
    const isSelected = selectedAgent === agent.id;
    const isDisabled = agent.disabled || isRunning;

    if (isDisabled) {
      return `${baseClasses} text-gray-400 border-gray-300 cursor-not-allowed`;
    }

    if (isSelected) {
      return `${baseClasses} text-blue-600 border-blue-600 bg-blue-50`;
    }

    return `${baseClasses} text-gray-600 border-gray-300 hover:text-gray-800 hover:border-gray-400 cursor-pointer`;
  };

  return (
    <div className="bg-white shadow-sm border-b">
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

          {/* Configuration Button */}
          <div className="flex items-center space-x-4">
            {AGENT_TABS.find((tab) => tab.id === selectedAgent)
              ?.requiresConfig && (
              <button
                onClick={() => setShowConfig(!showConfig)}
                className="px-3 py-1 text-sm bg-gray-100 text-gray-700 rounded hover:bg-gray-200 transition-colors"
                disabled={isRunning}
              >
                {showConfig ? "Hide Config" : "Configure Agent"}
              </button>
            )}
            <span className="text-sm text-gray-600">
              {isRunning ? "Running..." : "Ready"}
            </span>
          </div>
        </div>

        {/* Configuration Panel */}
        {showConfig &&
          AGENT_TABS.find((tab) => tab.id === selectedAgent)
            ?.requiresConfig && (
            <div className="border-t border-gray-200 bg-gray-50 px-4 py-4">
              <div className="max-w-2xl">
                <h3 className="text-lg font-medium text-gray-900 mb-4">
                  Configure{" "}
                  {
                    AGENT_TABS.find((tab) => tab.id === selectedAgent)
                      ?.displayName
                  }
                </h3>

                {/* API URL */}
                <div className="mb-4">
                  <label
                    htmlFor="api-url"
                    className="block text-sm font-medium text-gray-700 mb-1"
                  >
                    API URL
                  </label>
                  <input
                    id="api-url"
                    type="text"
                    value={config.api_url || ""}
                    onChange={(e) =>
                      handleConfigChange("api_url", e.currentTarget.value)
                    }
                    placeholder="https://api.example.com/v1"
                    className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                    disabled={configLoading}
                  />
                </div>

                {/* API Key */}
                <div className="mb-4">
                  <label
                    htmlFor="api-key"
                    className="block text-sm font-medium text-gray-700 mb-1"
                  >
                    API Key
                  </label>
                  <input
                    id="api-key"
                    type="password"
                    value={config.api_key || ""}
                    onChange={(e) =>
                      handleConfigChange("api_key", e.currentTarget.value)
                    }
                    placeholder="Enter your API key"
                    className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                    disabled={configLoading}
                  />
                </div>

                {/* Error Message */}
                {configError && (
                  <div className="mb-4 p-3 bg-red-100 border border-red-300 text-red-700 rounded-md">
                    {configError}
                  </div>
                )}

                {/* Test Result */}
                {testResult && (
                  <div
                    className={`mb-4 p-3 rounded-md border ${
                      testResult.status === "success"
                        ? "bg-green-100 border-green-300 text-green-700"
                        : "bg-red-100 border-red-300 text-red-700"
                    }`}
                  >
                    {testResult.message}
                  </div>
                )}

                {/* Action Buttons */}
                <div className="flex space-x-3">
                  <button
                    onClick={handleSaveConfig}
                    disabled={
                      configLoading || !config.api_url || !config.api_key
                    }
                    className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors"
                  >
                    {configLoading ? "Saving..." : "Save Configuration"}
                  </button>

                  <button
                    onClick={handleTestConnection}
                    disabled={
                      testingConnection || !config.api_url || !config.api_key
                    }
                    className="px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors"
                  >
                    {testingConnection ? "Testing..." : "Test Connection"}
                  </button>

                  <button
                    onClick={handleResetConfig}
                    disabled={configLoading}
                    className="px-4 py-2 bg-gray-600 text-white rounded-md hover:bg-gray-700 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors"
                  >
                    Reset
                  </button>
                </div>
              </div>
            </div>
          )}
      </div>
    </div>
  );
}
