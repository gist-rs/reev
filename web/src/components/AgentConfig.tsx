// AgentConfig component for managing LLM agent configurations

import { useState, useCallback, useEffect } from "preact/hooks";
import { apiClient } from "../services/api";
import {
  AgentConfig as AgentConfigType,
  ConnectionTestResult,
} from "../types/configuration";

interface AgentConfigProps {
  selectedAgent: string;
  isRunning: boolean;
  onConfigSaved?: (agentType: string) => void;
}

// Default API URLs for each agent type
const DEFAULT_API_URLS: Record<string, string> = {
  "gemini-2.5-flash-lite": "https://generativelanguage.googleapis.com/v1beta",
  "glm-4.6": "https://open.bigmodel.cn/api/paas/v4",
  local: "http://localhost:8080/api",
  deterministic: "",
};

function getDefaultApiUrl(agentType: string): string {
  return DEFAULT_API_URLS[agentType] || "";
}

export function AgentConfig({
  selectedAgent,
  isRunning,
  onConfigSaved,
}: AgentConfigProps) {
  const [config, setConfig] = useState<AgentConfigType>({
    agent_type: selectedAgent,
    api_url: getDefaultApiUrl(selectedAgent),
    api_key: "",
  });
  const [loading, setLoading] = useState(false);
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<ConnectionTestResult | null>(
    null,
  );
  const [saved, setSaved] = useState(false);

  // Load saved configuration when agent changes
  useEffect(() => {
    if (selectedAgent === "deterministic") {
      // Deterministic agent doesn't need configuration
      setConfig({
        agent_type: selectedAgent,
        api_url: "",
        api_key: "",
      });
      setTestResult(null);
      setSaved(false);
      return;
    }

    loadConfig();
  }, [selectedAgent]);

  const loadConfig = useCallback(async () => {
    try {
      const savedConfig = await apiClient.getAgentConfig(selectedAgent);
      setConfig(savedConfig);
      setSaved(true);
    } catch (error) {
      // No saved configuration found
      setConfig({
        agent_type: selectedAgent,
        api_url: "",
        api_key: "",
      });
      setSaved(false);
    }
    setTestResult(null);
  }, [selectedAgent]);

  const handleInputChange = useCallback(
    (field: keyof AgentConfigType, value: string) => {
      setConfig((prev) => ({
        ...prev,
        [field]: value,
      }));
      setTestResult(null);
      setSaved(false);
    },
    [],
  );

  const handleSave = useCallback(async () => {
    if (isRunning) return;

    setLoading(true);
    try {
      await apiClient.saveAgentConfig(config);
      setSaved(true);
      onConfigSaved?.(selectedAgent);
    } catch (error) {
      console.error("Failed to save configuration:", error);
      alert(
        `Failed to save configuration: ${error instanceof Error ? error.message : "Unknown error"}`,
      );
    } finally {
      setLoading(false);
    }
  }, [config, isRunning, selectedAgent, onConfigSaved]);

  const handleTest = useCallback(async () => {
    if (isRunning) return;

    setTesting(true);
    try {
      const result = await apiClient.testAgentConnection(config);
      setTestResult(result);
    } catch (error) {
      console.error("Failed to test connection:", error);
      setTestResult({
        status: "error",
        message:
          error instanceof Error ? error.message : "Connection test failed",
      });
    } finally {
      setTesting(false);
    }
  }, [config, isRunning]);

  const handleReset = useCallback(() => {
    setConfig({
      agent_type: selectedAgent,
      api_url: getDefaultApiUrl(selectedAgent),
      api_key: "",
    });
    setTestResult(null);
    setSaved(false);
  }, [selectedAgent]);

  // Deterministic agent doesn't need configuration
  if (selectedAgent === "deterministic") {
    return (
      <div className="p-4 bg-white dark:bg-gray-800 border dark:border-gray-700 rounded-lg">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-3">
          Agent Configuration
        </h3>
        <div className="text-gray-600 dark:text-gray-400 bg-gray-50 dark:bg-gray-900/50 p-3 rounded border dark:border-gray-700">
          <div className="flex items-center">
            <svg
              class="w-5 h-5 mr-2 text-green-600 dark:text-green-400"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
              ></path>
            </svg>
            <span>Deterministic agent requires no configuration</span>
          </div>
          <p className="text-sm mt-2 text-gray-500 dark:text-gray-400">
            This agent uses predefined logic and doesn't need API keys or
            external services.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="p-4 bg-white dark:bg-gray-800 border dark:border-gray-700 rounded-lg">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
          Agent Configuration
        </h3>
        {saved && (
          <span className="text-xs text-green-600 dark:text-green-400 bg-green-50 dark:bg-green-900/20 px-2 py-1 rounded border border-green-200 dark:border-green-700">
            ✓ Saved
          </span>
        )}
      </div>

      <div className="space-y-4">
        {/* API URL */}
        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
            API URL
          </label>
          <input
            type="text"
            value={config.api_url || ""}
            onChange={(e) =>
              handleInputChange("api_url", e.currentTarget.value)
            }
            placeholder={
              getDefaultApiUrl(selectedAgent) || "https://api.example.com"
            }
            className="w-full px-3 py-2 border border-gray-300 dark:border-gray-700 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100"
            disabled={isRunning}
          />
          <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
            Enter the API endpoint for the {selectedAgent} service
            {getDefaultApiUrl(selectedAgent) && (
              <span className="block mt-1">
                Default:{" "}
                <code className="bg-gray-100 dark:bg-gray-700 px-1 py-0.5 rounded text-xs">
                  {getDefaultApiUrl(selectedAgent)}
                </code>
              </span>
            )}
          </p>
        </div>

        {/* API Key */}
        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
            API Key
          </label>
          <input
            type="password"
            value={config.api_key || ""}
            onChange={(e) =>
              handleInputChange("api_key", e.currentTarget.value)
            }
            placeholder="Enter your API key"
            className="w-full px-3 py-2 border border-gray-300 dark:border-gray-700 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100"
            disabled={isRunning}
          />
          <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
            Your API key will be stored securely and used for authentication
          </p>
        </div>

        {/* Test Result */}
        {testResult && (
          <div
            className={`p-3 rounded-md border ${
              testResult.status === "success"
                ? "bg-green-50 dark:bg-green-900/20 border-green-200 dark:border-green-700 text-green-800 dark:text-green-400"
                : "bg-red-50 dark:bg-red-900/20 border-red-200 dark:border-red-700 text-red-800 dark:text-red-400"
            }`}
          >
            <div className="flex items-center">
              {testResult.status === "success" ? (
                <svg
                  class="w-5 h-5 mr-2"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
                  ></path>
                </svg>
              ) : (
                <svg
                  class="w-5 h-5 mr-2"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                  ></path>
                </svg>
              )}
              <span className="text-sm font-medium">
                {testResult.status === "success"
                  ? "Connection Successful"
                  : "Connection Failed"}
              </span>
            </div>
            <p className="text-sm mt-1">{testResult.message}</p>
          </div>
        )}

        {/* Action Buttons */}
        <div className="flex space-x-3 pt-2">
          <button
            onClick={handleTest}
            disabled={
              isRunning || testing || !config.api_url || !config.api_key
            }
            className="px-4 py-2 bg-blue-600 text-white text-sm rounded-md hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors"
          >
            {testing ? "Testing..." : "Test Connection"}
          </button>

          <button
            onClick={handleSave}
            disabled={isRunning || loading || saved}
            className="px-4 py-2 bg-green-600 text-white text-sm rounded-md hover:bg-green-700 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors"
          >
            {loading ? "Saving..." : saved ? "Saved" : "Save Configuration"}
          </button>

          <button
            onClick={handleReset}
            disabled={isRunning}
            className="px-4 py-2 bg-gray-600 text-white text-sm rounded-md hover:bg-gray-700 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors"
          >
            Reset
          </button>
        </div>

        {/* Help Text */}
        <div className="text-xs text-gray-500 dark:text-gray-400 bg-gray-50 dark:bg-gray-900/50 p-3 rounded border dark:border-gray-700">
          <p className="font-medium mb-1">Configuration Help:</p>
          <ul className="space-y-1">
            <li>• Ensure your API URL is accessible from this browser</li>
            <li>• Use a valid API key with appropriate permissions</li>
            <li>• Test the connection before saving the configuration</li>
            <li>• Configuration is saved locally for this session</li>
          </ul>
        </div>
      </div>
    </div>
  );
}
