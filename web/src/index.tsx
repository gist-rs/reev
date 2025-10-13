import { render } from "preact";
import { useState, useCallback } from "preact/hooks";
import { AgentSelector } from "./components/AgentSelector";
import { AgentConfig } from "./components/AgentConfig";
import { BenchmarkList } from "./components/BenchmarkList";
import { ExecutionTrace } from "./components/ExecutionTrace";
import { TransactionLog } from "./components/TransactionLog";
import { BenchmarkGrid } from "./components/BenchmarkGrid";
import { useExecutionState } from "./hooks/useBenchmarkExecution";
import "./style.css";

export function App() {
  const [selectedAgent, setSelectedAgent] = useState("deterministic");
  const [selectedBenchmark, setSelectedBenchmark] = useState<string | null>(
    null,
  );
  const [isRunning, setIsRunning] = useState(false);
  const [currentExecution, setCurrentExecution] = useState<any>(null);
  const [showOverview, setShowOverview] = useState(true);
  const [showTransactionLog, setShowTransactionLog] = useState(false);
  const { executions, updateExecution } = useExecutionState();

  const handleBenchmarkSelect = useCallback(
    (benchmarkId: string) => {
      setSelectedBenchmark(benchmarkId);
      setShowOverview(false);

      // Update current execution if we have one for this benchmark
      const execution = Array.from(executions.values()).find(
        (exec) => exec.benchmark_id === benchmarkId,
      );
      setCurrentExecution(execution || null);
    },
    [executions],
  );

  const handleExecutionStart = useCallback(
    (executionId: string) => {
      setIsRunning(true);

      // Find the execution and update current
      const execution = Array.from(executions.values()).find(
        (exec) => exec.id === executionId,
      );
      if (execution) {
        setCurrentExecution(execution);
        updateExecution(execution.benchmark_id, execution);
      }
    },
    [executions, updateExecution],
  );

  const handleExecutionComplete = useCallback(() => {
    setIsRunning(false);
  }, []);

  const handleToggleOverview = useCallback(() => {
    setShowOverview(!showOverview);
  }, [showOverview]);

  return (
    <div className="h-screen flex flex-col bg-gray-50">
      {/* Agent Selector */}
      <AgentSelector
        selectedAgent={selectedAgent}
        onAgentChange={setSelectedAgent}
        isRunning={isRunning}
      />

      {/* Performance Overview - Top Section */}
      {showOverview && (
        <div className="h-96 border-b bg-white">
          {/* Overview Header */}
          <div className="p-4 border-b bg-white">
            <div className="flex items-center justify-between">
              <h2 className="text-lg font-semibold">Performance Overview</h2>
              <button
                onClick={handleToggleOverview}
                className="px-3 py-1 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors"
              >
                Show Details
              </button>
            </div>
          </div>

          {/* Overview Content */}
          <div className="flex-1 overflow-auto">
            <BenchmarkGrid />
          </div>
        </div>
      )}

      {/* Main Content */}
      <div className="flex-1 flex overflow-hidden">
        {/* Left Panel - Benchmark List and Config */}
        <div className="w-1/3 border-r bg-white flex flex-col">
          {/* Benchmark List */}
          <div className="flex-1 overflow-hidden">
            <BenchmarkList
              selectedAgent={selectedAgent}
              selectedBenchmark={selectedBenchmark}
              onBenchmarkSelect={handleBenchmarkSelect}
              isRunning={isRunning}
              onExecutionStart={handleExecutionStart}
            />
          </div>

          {/* Agent Configuration */}
          <div className="border-t">
            <AgentConfig
              selectedAgent={selectedAgent}
              isRunning={isRunning}
              onConfigSaved={() => {
                // Refresh or notify as needed
              }}
            />
          </div>
        </div>

        {/* Right Panel - Execution Trace */}
        <div className="flex-1 flex flex-col">
          {/* Details Header */}
          <div className="p-4 border-b bg-white">
            <div className="flex items-center justify-between">
              <h2 className="text-lg font-semibold">
                {selectedBenchmark
                  ? `Benchmark: ${selectedBenchmark}`
                  : "Execution Details"}
              </h2>
              <div className="flex space-x-2">
                {!showOverview && (
                  <button
                    onClick={handleToggleOverview}
                    className="px-3 py-1 text-sm bg-gray-600 text-white rounded hover:bg-gray-700 transition-colors"
                  >
                    Show Overview
                  </button>
                )}
                {currentExecution && currentExecution.status === "Running" && (
                  <button
                    onClick={handleExecutionComplete}
                    className="px-3 py-1 text-sm bg-red-600 text-white rounded hover:bg-red-700 transition-colors"
                  >
                    Stop Execution
                  </button>
                )}
              </div>
            </div>
          </div>

          {/* Right Panel Content */}
          <div className="flex-1 flex flex-col">
            {/* Tab Navigation */}
            <div className="flex border-b bg-white">
              <button
                onClick={() => setShowTransactionLog(false)}
                className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
                  !showTransactionLog
                    ? "border-blue-500 text-blue-600 bg-blue-50"
                    : "border-transparent text-gray-500 hover:text-gray-700"
                }`}
              >
                Execution Trace
              </button>
              <button
                onClick={() => setShowTransactionLog(true)}
                className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
                  showTransactionLog
                    ? "border-blue-500 text-blue-600 bg-blue-50"
                    : "border-transparent text-gray-500 hover:text-gray-700"
                }`}
              >
                Transaction Log
              </button>
            </div>

            {/* Tab Content */}
            <div className="flex-1">
              {showTransactionLog ? (
                <TransactionLog
                  benchmarkId={selectedBenchmark}
                  executionId={currentExecution?.id || null}
                  isRunning={isRunning}
                />
              ) : (
                <ExecutionTrace
                  execution={currentExecution}
                  isRunning={isRunning}
                />
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

render(<App />, document.getElementById("app"));
