import { render } from "preact";
import { useState, useCallback, useEffect, useRef } from "preact/hooks";
import { AgentSelector } from "./components/AgentSelector";

import { BenchmarkList } from "./components/BenchmarkList";
import { ExecutionTrace } from "./components/ExecutionTrace";
import { TransactionLog } from "./components/TransactionLog";
import { BenchmarkGrid } from "./components/BenchmarkGrid";
import { useAgentPerformance } from "./hooks/useApiData";
import { useBenchmarkExecution } from "./hooks/useBenchmarkExecution";
import { apiClient } from "./services/api";
import { BenchmarkItem } from "./types/configuration";
import "./style.css";

export function App() {
  const [selectedAgent, setSelectedAgent] = useState("deterministic");
  const [selectedBenchmark, setSelectedBenchmark] = useState<string | null>(
    null,
  );
  const [isRunning, setIsRunning] = useState(false);
  const [currentExecution, setCurrentExecution] = useState<any>(null);
  const [isRunningAll, setIsRunningAll] = useState(false);
  const runAllQueue = useRef<BenchmarkItem[]>([]);
  const currentRunAllIndex = useRef(0);

  const [showTransactionLog, setShowTransactionLog] = useState(false);
  const {
    benchmarks,
    executions,
    updateExecution,
    setCompletionCallback,
    loading,
    error,
    refetch,
  } = useBenchmarkExecution();

  // State for performance overview refresh
  const [performanceOverviewRefresh, setPerformanceOverviewRefresh] =
    useState(0);

  // Get performance data for header stats
  const {
    refetch: refetchAgentPerformance,
    totalResults,
    testedAgents,
    totalAgents,
  } = useAgentPerformance();

  // Refresh performance data when a benchmark completes
  useEffect(() => {
    if (performanceOverviewRefresh > 0) {
      refetchAgentPerformance();
    }
  }, [performanceOverviewRefresh, refetchAgentPerformance]);

  // Keep currentExecution in sync with executions map
  useEffect(() => {
    console.log("=== EXECUTION SYNC CHECK ===");
    console.log("selectedBenchmark:", selectedBenchmark);
    console.log(
      "executions.has(selectedBenchmark):",
      executions.has(selectedBenchmark),
    );
    console.log("executions map keys:", Array.from(executions.keys()));
    console.log(
      "executions map values:",
      Array.from(executions.values()).map((e) => ({
        id: e.id,
        benchmark_id: e.benchmark_id,
        status: e.status,
      })),
    );

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
      console.log("=== BENCHMARK SELECTED ===");
      console.log("benchmarkId:", benchmarkId);
      console.log("Previous selectedBenchmark:", selectedBenchmark);
      setSelectedBenchmark(benchmarkId);
      console.log("Set selectedBenchmark to:", benchmarkId);

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
            // Get ASCII tree directly from database (single API call)
            console.log("Loading ASCII tree from database...");
            const response = await fetch(
              `/api/v1/ascii-tree/${benchmarkId}/deterministic`,
            );

            if (response.ok) {
              const asciiTree = await response.text();
              console.log("ASCII tree loaded:", asciiTree);

              // Create execution from historical data
              const historicalExecution = {
                id: `historical-${benchmarkId}-${Date.now()}`,
                benchmark_id: benchmarkId,
                agent: "deterministic",
                status: "Completed",
                progress: 100,
                start_time: new Date().toISOString(),
                end_time: new Date().toISOString(),
                trace: asciiTree,
                logs: "",
                score: 0, // We don't have score in this simple approach
              };

              console.log("Created historical execution:", historicalExecution);
              setCurrentExecution(historicalExecution);
            } else if (response.status === 404) {
              console.log("No ASCII tree found for benchmark:", benchmarkId);
              setCurrentExecution(null);
            } else {
              console.error("Failed to get ASCII tree:", response.statusText);
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
          const errorText = await response.text();
          console.error("Failed to parse YML:", response.status, errorText);
          throw new Error(
            `Failed to parse YML: ${response.statusText} - ${errorText}`,
          );
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
        const errorText = await renderResponse.text();
        console.error(
          "Failed to render ASCII tree:",
          renderResponse.status,
          errorText,
        );
        throw new Error(
          `Failed to render ASCII tree: ${renderResponse.statusText} - ${errorText}`,
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
      console.log("=== EXECUTION START ===");
      console.log("executionId:", executionId);
      console.log("selectedBenchmark:", selectedBenchmark);
      console.log("executions map before:", Array.from(executions.entries()));
      setIsRunning(true);

      // Trigger performance overview refresh for execution start
      setPerformanceOverviewRefresh((prev) => prev + 1);

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
    [
      executions,
      updateExecution,
      currentExecution,
      setPerformanceOverviewRefresh,
    ],
  );

  // Focused polling now handles getting the ASCII tree, no need for verification here

  const handleExecutionComplete = useCallback(
    (benchmarkId: string, execution: any) => {
      console.log("=== App.handleExecutionComplete ===");
      console.log("benchmarkId:", benchmarkId);
      console.log("execution:", execution);
      console.log("selectedBenchmark:", selectedBenchmark);

      setIsRunning(false);

      // Trigger performance overview refresh for individual benchmark completion
      setPerformanceOverviewRefresh((prev) => prev + 1);

      // If this is the currently selected benchmark, update currentExecution immediately
      if (selectedBenchmark === benchmarkId) {
        console.log("Updating currentExecution for completed benchmark");
        setCurrentExecution(execution);
      }
    },
    [selectedBenchmark, setPerformanceOverviewRefresh],
  );

  const handleStopExecution = useCallback(() => {
    setIsRunning(false);
    // TODO: Add actual stop execution logic
  }, []);

  // Run All completion callback - simplified approach
  const runAllCompletionCallback = useCallback(
    async (benchmarkId: string, execution: any) => {
      console.log(
        `🎯 App: Run All completion callback triggered for ${benchmarkId}`,
      );

      // Notify BenchmarkList component
      handleExecutionComplete(benchmarkId, execution);

      // Trigger performance overview refresh
      setPerformanceOverviewRefresh((prev) => prev + 1);

      // Continue to next benchmark in queue
      currentRunAllIndex.current++;

      if (currentRunAllIndex.current < runAllQueue.current.length) {
        const nextBenchmark = runAllQueue.current[currentRunAllIndex.current];
        console.log(
          `🚀 App: Starting next benchmark ${currentRunAllIndex.current + 1}/${runAllQueue.current.length}: ${nextBenchmark.id}`,
        );

        // Auto-select the next benchmark for Execution Details display
        handleBenchmarkSelect(nextBenchmark.id);

        // Start next benchmark directly via API
        try {
          const response = await apiClient.runBenchmark(nextBenchmark.id, {
            agent: selectedAgent,
          });

          console.log(
            `🚀 App: Started next benchmark ${nextBenchmark.id} with execution ID: ${response.execution_id}`,
          );

          // Update execution state
          updateExecution(nextBenchmark.id, {
            id: response.execution_id,
            benchmark_id: nextBenchmark.id,
            agent: selectedAgent,
            status: "Pending",
            progress: 0,
            start_time: new Date().toISOString(),
            trace: "",
            logs: "",
          });

          handleExecutionStart(response.execution_id);
        } catch (error) {
          console.error(
            `Failed to start benchmark ${nextBenchmark.id}:`,
            error,
          );
          // Continue to next one even on failure
          runAllCompletionCallback(nextBenchmark.id, {
            status: "Failed",
            error,
          });
        }
      } else {
        // All benchmarks completed
        console.log("✅ App: All benchmarks completed, refreshing overview");
        setIsRunningAll(false);
        runAllQueue.current = [];
        currentRunAllIndex.current = 0;

        // Clear the completion callback after a delay
        setTimeout(() => {
          console.log(
            "🧹 App: Clearing completion callback after Run All completion",
          );
          setCompletionCallback(() => () => {});
        }, 1000);
      }
    },
    [
      handleExecutionComplete,
      handleBenchmarkSelect,
      setCompletionCallback,
      selectedAgent,
      updateExecution,
      handleExecutionStart,
    ],
  );

  return (
    <div className="h-screen flex flex-col bg-gray-50">
      {/* Performance Overview - Top Section (shows all agents) */}
      <div className="h-96 border-b bg-white">
        {/* Overview Header */}
        <div className="p-4 border-b bg-white">
          <div className="flex items-center justify-between">
            <h2 className="text-lg font-semibold">Performance Overview</h2>
            <div className="flex items-center space-x-4">
              <span className="text-sm text-gray-600">
                {totalResults || 0} total results
              </span>
              <span className="text-sm text-gray-600">
                {testedAgents || 0}/{totalAgents || 0} agents
              </span>

              {/* Legend */}
              <div className="flex items-center space-x-4 text-xs text-gray-600 p-2 bg-gray-50 rounded border">
                <div className="flex items-center">
                  <div className="w-3 h-3 bg-green-500 rounded mr-1"></div>
                  <span>Perfect (100%)</span>
                </div>
                <div className="flex items-center">
                  <div className="w-3 h-3 bg-yellow-500 rounded mr-1"></div>
                  <span>Partial (25-99%)</span>
                </div>
                <div className="flex items-center">
                  <div className="w-3 h-3 bg-red-500 rounded mr-1"></div>
                  <span>Poor (&lt;25%)</span>
                </div>
                <div className="flex items-center">
                  <div className="w-3 h-3 bg-gray-400 rounded mr-1"></div>
                  <span>Not Tested</span>
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Overview Content */}
        <div className="flex-1 overflow-auto">
          <BenchmarkGrid
            refreshTrigger={performanceOverviewRefresh}
            onBenchmarkSelect={handleBenchmarkSelect}
          />
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
              isRunning={isRunning || isRunningAll}
              onExecutionStart={handleExecutionStart}
              onExecutionComplete={handleExecutionComplete}
              executions={executions}
              updateExecution={updateExecution}
              isRunningAll={isRunningAll}
              setIsRunningAll={setIsRunningAll}
              setCompletionCallback={setCompletionCallback}
              runAllCompletionCallback={runAllCompletionCallback}
              runAllQueue={runAllQueue}
              currentRunAllIndex={currentRunAllIndex}
              benchmarks={benchmarks}
              loading={loading}
              error={error}
              refetch={refetch}
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
