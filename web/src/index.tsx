import { render } from "preact";
import { useState, useCallback, useEffect } from "preact/hooks";
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

  // Keep currentExecution in sync with executions map
  useEffect(() => {
    if (selectedBenchmark && executions.has(selectedBenchmark)) {
      const execution = executions.get(selectedBenchmark);
      console.log("=== Syncing currentExecution with executions map ===");
      console.log("selectedBenchmark:", selectedBenchmark);
      console.log("execution from map:", execution);
      console.log("current currentExecution:", currentExecution);

      // Only update if the execution is different or has new data
      if (
        !currentExecution ||
        currentExecution.id !== execution?.id ||
        currentExecution.trace !== execution?.trace ||
        currentExecution.status !== execution?.status ||
        currentExecution.progress !== execution?.progress
      ) {
        console.log("Updating currentExecution to match executions map");

        // Add debugging for trace data when execution completes
        if (execution?.status === "Completed" && execution.trace) {
          console.log("=== EXECUTION COMPLETED WITH TRACE ===");
          console.log("Trace length:", execution.trace.length);
          console.log(
            "First 200 chars of trace:",
            execution.trace.substring(0, 200),
          );
          console.log("Last 200 chars of trace:", execution.trace.slice(-200));
        }

        setCurrentExecution(execution);
      }
    }
  }, [executions, selectedBenchmark, currentExecution]);

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
        (async () => {
          try {
            console.log(
              "No execution found, loading flow logs from database...",
            );
            const flowLogs = await apiClient.getFlowLog(benchmarkId);
            console.log("Flow logs loaded:", flowLogs);

            if (flowLogs && Array.isArray(flowLogs) && flowLogs.length > 0) {
              // Get the most recent flow log
              const latestFlowLog = flowLogs[flowLogs.length - 1];
              console.log("Latest flow log:", latestFlowLog);

              // Extract trace data from flow log
              const traceData = await extractTraceFromFlowLog(latestFlowLog);
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
                start_time: (latestFlowLog as any).start_time?.secs_since_epoch
                  ? new Date(
                      (latestFlowLog as any).start_time.secs_since_epoch * 1000,
                    ).toISOString()
                  : new Date().toISOString(),
                end_time: (latestFlowLog as any).end_time?.secs_since_epoch
                  ? new Date(
                      (latestFlowLog as any).end_time.secs_since_epoch * 1000,
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
        })();
      } else {
        setCurrentExecution(execution);
      }

      // Log after state update (in next tick)
      setTimeout(() => {
        console.log("Current currentExecution after update:", currentExecution);
      }, 0);
    },
    [executions, currentExecution],
  );

  // Helper function to extract trace data from flow log
  async function extractTraceFromFlowLog(flowLog: any): Promise<string> {
    try {
      console.log("Extracting trace from flow log:", flowLog);

      // The flow log should contain YML TestResult data directly
      console.log("Raw YML content:", flowLog);

      // Parse YML to TestResult object
      let testResult;
      if (typeof flowLog === "string") {
        // Parse YML string to TestResult object using backend
        const response = await fetch(`/api/v1/parse-yml-to-testresult`, {
          method: "POST",
          headers: {
            "Content-Type": "text/plain",
          },
          body: flowLog,
        });

        if (response.ok) {
          testResult = await response.json();
          console.log("Parsed TestResult from YML:", testResult);
        } else {
          throw new Error(`Failed to parse YML: ${response.statusText}`);
        }
      } else {
        testResult = flowLog; // Already parsed
      }

      // Call backend to convert TestResult to ASCII tree
      const renderResponse = await fetch(`/api/v1/render-ascii-tree`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(testResult),
      });

      if (renderResponse.ok) {
        const asciiTree = await renderResponse.text();
        console.log("Got ASCII tree from backend:", asciiTree);
        return asciiTree;
      } else {
        throw new Error(
          `Failed to render ASCII tree: ${renderResponse.statusText}`,
        );
      }
    } catch (error) {
      console.error("Failed to extract trace:", error);
      return `Error extracting trace: ${error}`;
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

  // Focused polling now handles getting the ASCII tree, no need for verification here

  const handleExecutionComplete = useCallback(
    (benchmarkId: string, execution: any) => {
      console.log("=== App.handleExecutionComplete ===");
      console.log("benchmarkId:", benchmarkId);
      console.log("execution:", execution);
      console.log("selectedBenchmark:", selectedBenchmark);

      setIsRunning(false);

      // If this is the currently selected benchmark, update currentExecution immediately
      if (selectedBenchmark === benchmarkId) {
        console.log("Updating currentExecution for completed benchmark");
        setCurrentExecution(execution);
      }
    },
    [selectedBenchmark],
  );

  const handleStopExecution = useCallback(() => {
    setIsRunning(false);
    // TODO: Add actual stop execution logic
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
              onExecutionComplete={handleExecutionComplete}
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
                    onClick={handleStopExecution}
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
                  {currentExecution?.status === "Completed" &&
                    currentExecution?.trace &&
                    console.log("=== ABOUT TO RENDER COMPLETED EXECUTION ===")}
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
