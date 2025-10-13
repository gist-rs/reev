import { render } from "preact";
import { useState, useCallback } from "preact/hooks";
import { AgentSelector } from "./components/AgentSelector";
import { AgentConfig } from "./components/AgentConfig";
import { BenchmarkList } from "./components/BenchmarkList";
import { ExecutionTrace } from "./components/ExecutionTrace";
import { TransactionLog } from "./components/TransactionLog";
import { BenchmarkGrid } from "./components/BenchmarkGrid";
import { useBenchmarkExecution } from "./hooks/useBenchmarkExecution";
import { apiClient } from "./services/api";
import "./style.css";

export function App() {
  const [selectedAgent, setSelectedAgent] = useState("deterministic");
  const [selectedBenchmark, setSelectedBenchmark] = useState<string | null>(
    null,
  );
  const [isRunning, setIsRunning] = useState(false);
  const [currentExecution, setCurrentExecution] = useState<any>(null);

  const [showTransactionLog, setShowTransactionLog] = useState(false);
  const { benchmarks, executions, updateExecution } = useBenchmarkExecution();

  const handleBenchmarkSelect = useCallback(
    async (benchmarkId: string) => {
      setSelectedBenchmark(benchmarkId);

      // Update current execution if we have one for this benchmark
      const execution = Array.from(executions.values()).find(
        (exec) => exec.benchmark_id === benchmarkId,
      );

      // Debug log to help with troubleshooting
      console.log("=== App.handleBenchmarkSelect ===");
      console.log("Benchmark selected:", benchmarkId);
      console.log("Available executions:", Array.from(executions.entries()));
      console.log(
        "Available execution values:",
        Array.from(executions.values()),
      );
      console.log("Found execution:", execution);
      console.log("Current currentExecution before update:", currentExecution);

      // If no current execution, try to load flow logs from database
      if (!execution) {
        try {
          console.log("No execution found, loading flow logs from database...");
          const flowLogs = await apiClient.getFlowLog(benchmarkId);
          console.log("Flow logs loaded:", flowLogs);

          if (flowLogs && Array.isArray(flowLogs) && flowLogs.length > 0) {
            // Get the most recent flow log
            const latestFlowLog = flowLogs[flowLogs.length - 1];
            console.log("Latest flow log:", latestFlowLog);

            // Extract trace data from flow log
            const traceData = extractTraceFromFlowLog(latestFlowLog);
            console.log("Extracted trace data:", traceData);

            // Create execution from flow log data
            const flowExecution = {
              id: latestFlowLog.session_id,
              benchmark_id: benchmarkId,
              agent: latestFlowLog.agent_type,
              status: latestFlowLog.final_result?.success
                ? "Completed"
                : "Failed",
              progress: 100,
              start_time: new Date(
                latestFlowLog.start_time.secs_since_epoch * 1000,
              ).toISOString(),
              end_time: latestFlowLog.end_time
                ? new Date(
                    latestFlowLog.end_time.secs_since_epoch * 1000,
                  ).toISOString()
                : undefined,
              trace: traceData,
              logs: extractTransactionLogsFromFlowLog(latestFlowLog),
              score: latestFlowLog.final_result?.score || 0,
            };

            console.log("Created execution from flow log:", flowExecution);
            setCurrentExecution(flowExecution);
          } else {
            console.log("No flow logs found for benchmark:", benchmarkId);
            setCurrentExecution(null);
          }
        } catch (error) {
          console.error("Failed to load flow logs:", error);
          setCurrentExecution(null);
        }
      } else {
        setCurrentExecution(execution);
      }

      // Log after state update (in next tick)
      setTimeout(() => {
        console.log("Current currentExecution after update:", currentExecution);
      }, 0);
    },
    [executions],
  );

  // Helper function to extract trace data from flow log
  function extractTraceFromFlowLog(flowLog: any): string {
    try {
      // Try to extract trace from the events data
      for (const event of flowLog.events || []) {
        if (event.content?.data?.trace) {
          const traceData = event.content.data.trace;

          // If trace is a string, return it directly
          if (typeof traceData === "string") {
            return traceData;
          }

          // If trace is an object with trace.steps, convert to ASCII tree
          if (traceData.trace?.steps) {
            const testResult = {
              id: flowLog.benchmark_id,
              final_status:
                traceData.final_status === "Succeeded" ? "Succeeded" : "Failed",
              score: traceData.score || 1.0,
              trace: traceData.trace,
            };

            // This would normally use the renderer, but for now return a formatted string
            return formatTraceAsText(testResult);
          }
        }
      }

      return "No trace data found in flow log";
    } catch (error) {
      console.error("Error extracting trace from flow log:", error);
      return "Error extracting trace data";
    }
  }

  // Helper function to extract transaction logs from flow log
  function extractTransactionLogsFromFlowLog(flowLog: any): string {
    try {
      const logs: string[] = [];

      for (const event of flowLog.events || []) {
        if (event.content?.data?.trace?.trace?.steps) {
          for (const step of event.content.data.trace.trace.steps) {
            if (step.observation?.last_transaction_logs) {
              logs.push(...step.observation.last_transaction_logs);
            }
          }
        }
      }

      return logs.join("\n");
    } catch (error) {
      console.error("Error extracting transaction logs from flow log:", error);
      return "Error extracting transaction logs";
    }
  }

  // Helper function to format trace as text (simplified version of the renderer)
  function formatTraceAsText(testResult: any): string {
    const statusIcon = testResult.final_status === "Succeeded" ? "✅" : "❌";
    const scorePercent = testResult.score * 100;

    let output = `${statusIcon} ${testResult.id} (Score: ${scorePercent.toFixed(1)}%): ${testResult.final_status}\n`;

    testResult.trace.steps.forEach((step: any, index: number) => {
      output += ` └─ Step ${index + 1}\n`;

      if (step.action && step.action.length > 0) {
        const action = step.action[0][0]; // First instruction
        output += `    ├─ ACTION:\n`;
        output += `     Program ID: ${action.program_id}\n`;
        output += `     Accounts:\n`;

        action.accounts.forEach((account: any, accIndex: number) => {
          const signerIcon = account.is_signer ? "🖋️" : "🖍️";
          const writableIcon = account.is_writable ? "➕" : "➖";
          output += `     [${accIndex.toString().padStart(2)}] ${signerIcon} ${writableIcon} ${account.pubkey}\n`;
        });

        output += `     Data (Base58): ${action.data}\n`;

        if (step.action.length > 1) {
          output += `     (+ ${step.action.length - 1} more instructions in this transaction)\n`;
        }
      }

      output += `    └─ OBSERVATION: ${step.observation?.last_transaction_status || "Unknown"}\n`;

      if (step.observation?.last_transaction_error) {
        output += `       Error: ${step.observation.last_transaction_error}\n`;
      }
    });

    return output;
  }

  const handleExecutionStart = useCallback(
    (executionId: string) => {
      setIsRunning(true);

      // Find the execution and update current
      const execution = Array.from(executions.values()).find(
        (exec) => exec.id === executionId,
      );

      console.log("=== App.handleExecutionStart ===");
      console.log("Execution started with ID:", executionId);
      console.log("Found execution:", execution);
      console.log("Available executions:", Array.from(executions.entries()));

      if (execution) {
        setCurrentExecution(execution);
        updateExecution(execution.benchmark_id, execution);
        console.log("Set current execution to:", execution);
      } else {
        console.log("No execution found for ID:", executionId);
      }
    },
    [executions, updateExecution, currentExecution],
  );

  const handleExecutionComplete = useCallback(() => {
    setIsRunning(false);
  }, []);

  return (
    <div className="h-screen flex flex-col bg-gray-50">
      {/* Performance Overview - Top Section (shows all agents) */}
      <div className="h-96 border-b bg-white">
        {/* Overview Header */}
        <div className="p-4 border-b bg-white">
          <div className="flex items-center justify-between">
            <h2 className="text-lg font-semibold">Performance Overview</h2>
          </div>
        </div>

        {/* Overview Content */}
        <div className="flex-1 overflow-auto">
          <BenchmarkGrid />
        </div>
      </div>

      {/* Agent Selector */}
      <AgentSelector
        selectedAgent={selectedAgent}
        onAgentChange={setSelectedAgent}
        isRunning={isRunning}
      />

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
              executions={executions}
              updateExecution={updateExecution}
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
                  execution={currentExecution}
                  isRunning={isRunning}
                />
              ) : (
                <>
                  {/* Debug info */}
                  {console.log("=== Rendering ExecutionTrace ===")}
                  {console.log("currentExecution:", currentExecution)}
                  {console.log("isRunning:", isRunning)}
                  <ExecutionTrace
                    execution={currentExecution}
                    isRunning={isRunning}
                  />
                </>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

render(<App />, document.getElementById("app"));
