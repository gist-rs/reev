import { render } from "preact";
import {
  useState,
  useCallback,
  useEffect,
  useRef,
  useMemo,
} from "preact/hooks";
import { AgentSelector } from "./components/AgentSelector";
import { DarkModeToggle } from "./components/DarkModeToggle";
import { ThemeProvider } from "./contexts/ThemeContext";

import { BenchmarkList } from "./components/BenchmarkList";
import { ExecutionTrace } from "./components/ExecutionTrace";
import { TransactionLog } from "./components/TransactionLog";
import { BenchmarkGrid } from "./components/BenchmarkGrid";
import { useAgentPerformance, useBenchmarks } from "./hooks/useApiData";
import { useBenchmarkExecution } from "./hooks/useBenchmarkExecution";
import { apiClient } from "./services/api";
import { BenchmarkItem } from "./types/configuration";
import "./style.css";
import { ExecutionStatus } from "./types/benchmark";

export function App() {
  const [selectedAgent, setSelectedAgent] = useState("deterministic");
  const [selectedBenchmark, setSelectedBenchmark] = useState<string | null>(
    null,
  );
  const [selectedDate, setSelectedDate] = useState<string | null>(null);
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
  const {
    data: benchmarkData,
    loading: benchmarksLoading,
    error: benchmarksError,
    refetch: refetchBenchmarks,
  } = useBenchmarks();

  // State for performance overview refresh
  const [performanceOverviewRefresh, setPerformanceOverviewRefresh] =
    useState(0);

  // Get performance data for header stats and shared with components
  const {
    refetch: refetchAgentPerformance,
    totalResults,
    testedAgents,
    totalAgents,
    data: agentPerformanceData,
    loading: agentPerformanceLoading,
    error: agentPerformanceError,
  } = useAgentPerformance();

  // Auto-select most recent benchmark when agent changes
  useEffect(() => {
    if (agentPerformanceData && selectedAgent) {
      // Find agent data for selected agent
      const agentData = agentPerformanceData.find(
        (agent) => agent.agent_type === selectedAgent,
      );

      // Only auto-select if there's no current benchmark selection
      if (!selectedBenchmark) {
        if (agentData && agentData.results && agentData.results.length > 0) {
          // Find most recent result (sorted by timestamp descending)
          const mostRecentResult = agentData.results.sort(
            (a, b) =>
              new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime(),
          )[0];

          if (mostRecentResult) {
            const latestDate = mostRecentResult.timestamp.substring(0, 10);
            setSelectedBenchmark(mostRecentResult.benchmark_id);
            setSelectedDate(latestDate);

            // Update current execution if found in executions
            const execution = Array.from(executions.values()).find(
              (exec) =>
                exec.benchmark_id === mostRecentResult.benchmark_id &&
                exec.agent === selectedAgent,
            );
            setCurrentExecution(execution || null);
          }
        } else {
          // No results for this agent - clear selection
          setSelectedBenchmark(null);
          setSelectedDate(null);
          setCurrentExecution(null);
        }
      }
    }
  }, [selectedAgent, agentPerformanceData, executions]);

  // Refresh performance data when a benchmark completes
  // Refetch agent performance data when refresh trigger changes
  useEffect(() => {
    if (performanceOverviewRefresh > 0) {
      refetchAgentPerformance();

      // Add retry mechanism for database locks - retry after 2 seconds if initial fetch fails
      setTimeout(() => {
        if (agentPerformanceError) {
          console.log(
            "🔄 [App] Retrying performance data fetch due to potential database lock",
          );
          refetchAgentPerformance();
        }
      }, 2000);
    }
  }, [
    performanceOverviewRefresh,
    refetchAgentPerformance,
    agentPerformanceError,
  ]);

  // Derive running benchmarks by benchmark ID and agent type
  const runningBenchmarkIds = useMemo(() => {
    const runningEntries = Array.from(executions.entries()).filter(
      ([_, execution]) =>
        execution.status === "Running" ||
        execution.status === "Pending" ||
        (execution.progress !== undefined && execution.progress < 100),
    );

    // Return just benchmark IDs for now - we'll handle agent-specific logic in the components
    return runningEntries.map(([benchmarkId, _]) => benchmarkId);
  }, [executions]);

  // Keep currentExecution in sync with executions map
  useEffect(() => {
    if (selectedBenchmark && selectedAgent) {
      // Find the execution that matches both benchmark and agent
      const matchingExecution = Array.from(executions.values()).find(
        (exec) =>
          exec.benchmark_id === selectedBenchmark &&
          exec.agent === selectedAgent,
      );

      // Only update if the execution is different or has new data
      if (
        !currentExecution ||
        currentExecution.id !== matchingExecution?.id ||
        currentExecution.trace !== matchingExecution?.trace ||
        currentExecution.status !== matchingExecution?.status ||
        currentExecution.progress !== matchingExecution?.progress
      ) {
        setCurrentExecution(matchingExecution || null);
      }
    }
  }, [executions, selectedBenchmark, selectedAgent]);

  const handleBenchmarkSelect = useCallback(
    async (benchmarkId: string, agentType?: string, date?: string) => {
      // Benchmark selection triggered
      setSelectedBenchmark(benchmarkId);
      setSelectedDate(date || null);

      // Update selected agent if provided
      if (agentType) {
        console.log("Setting selectedAgent to:", agentType);
        setSelectedAgent(agentType);
      }

      // Keep current tab selection (don't auto-switch to Transaction Log)

      // Update current execution if we have one for this benchmark and agent
      const execution = Array.from(executions.values()).find(
        (exec) => exec.benchmark_id === benchmarkId && exec.agent === agentType,
      );

      // Set current execution directly, no history loading
      setCurrentExecution(execution || null);
    },
    [executions],
  );

  const handleExecutionStart = useCallback(
    (executionId: string) => {
      if (import.meta.env.DEV) {
        console.log("=== EXECUTION START ===");
        console.log("executionId:", executionId);
        console.log("selectedBenchmark:", selectedBenchmark);
      }
      setIsRunning(true);

      // Trigger performance overview refresh for execution start
      setPerformanceOverviewRefresh((prev) => prev + 1);

      // Find the execution and update current
      const execution = Array.from(executions.values()).find(
        (exec) => exec.id === executionId,
      );

      if (import.meta.env.DEV) {
        console.log("=== App.handleExecutionStart ===");
        console.log("Execution started with ID:", executionId);
        console.log("Found execution:", execution);
      }

      if (execution) {
        setCurrentExecution(execution);
        updateExecution(execution.id, execution);
        if (import.meta.env.DEV) {
          console.log("Set current execution to:", execution);
        }
      } else {
        if (import.meta.env.DEV) {
          console.log("No execution found for ID:", executionId);
        }
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

  const handleRunBenchmark = useCallback(
    async (benchmarkId: string, agentType?: string) => {
      if (isRunning) return;

      // Use provided agent type or fall back to global selectedAgent
      const agentToUse = agentType || selectedAgent;

      // Update global selectedAgent if different agent type is being used
      if (agentType && agentType !== selectedAgent) {
        setSelectedAgent(agentType);
      }

      try {
        // Get agent configuration if needed
        let config;
        if (agentToUse !== "deterministic") {
          try {
            config = await apiClient.getAgentConfig(agentToUse);
          } catch {
            // No config found, that's okay for now
          }
        }

        const response = await apiClient.runBenchmark(benchmarkId, {
          agent: agentToUse,
          config,
        });

        console.log(
          "Starting benchmark execution from modal:",
          benchmarkId,
          response.execution_id,
        );

        // Select the benchmark for Execution Details display
        handleBenchmarkSelect(benchmarkId);

        updateExecution(response.execution_id, {
          id: response.execution_id,
          benchmark_id: benchmarkId,
          agent: agentToUse,
          status: ExecutionStatus.PENDING,
          progress: 0,
          start_time: new Date().toISOString(),
          trace: "",
          logs: "",
          timestamp: "",
        });

        setIsRunning(true);
        handleExecutionStart(response.execution_id);

        // Set completion callback for modal execution
        setCompletionCallback((benchmarkId: string, execution: any) => {
          console.log("🎯 Modal execution completion callback:", benchmarkId);
          handleExecutionComplete(benchmarkId, execution);
          // Clear the completion callback
          setCompletionCallback(() => () => {});
        });
      } catch (error) {
        console.error("Failed to start benchmark:", error);
      }
    },
    [
      isRunning,
      selectedAgent,
      handleBenchmarkSelect,
      updateExecution,
      handleExecutionStart,
      handleExecutionComplete,
      setCompletionCallback,
    ],
  );

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
          updateExecution(response.execution_id, {
            id: response.execution_id,
            benchmark_id: nextBenchmark.id,
            agent: selectedAgent,
            status: ExecutionStatus.PENDING,
            progress: 0,
            start_time: new Date().toISOString(),
            trace: "",
            logs: "",
            timestamp: "",
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
        if (import.meta.env.DEV) {
          console.log("✅ App: All benchmarks completed, refreshing overview");
        }
        setIsRunningAll(false);
        runAllQueue.current = [];
        currentRunAllIndex.current = 0;

        // Clear the completion callback after a delay
        setTimeout(() => {
          if (import.meta.env.DEV) {
            console.log(
              "🧹 App: Clearing completion callback after Run All completion",
            );
          }
          setCompletionCallback(null);
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
    <div className="h-screen flex flex-col bg-gray-50 dark:bg-gray-900">
      {/* Performance Overview - Top Section (shows all agents) */}
      <div className="border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800">
        {/* Overview Header */}
        <div className="p-4 border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-3">
              <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
                Performance Overview
              </h2>
              <DarkModeToggle />
            </div>
            <div className="flex items-center space-x-4">
              <span className="text-sm text-gray-600 dark:text-gray-400">
                {totalResults || 0} total results
              </span>
              <span className="text-sm text-gray-600 dark:text-gray-400">
                {testedAgents || 0}/{totalAgents || 0} agents
              </span>

              {/* Legend */}
              <div className="flex items-center space-x-4 text-xs text-gray-600 dark:text-gray-400 p-2 bg-gray-50 dark:bg-gray-700 rounded border dark:border-gray-700">
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
            onCardClick={(agentType) => {
              // Keep current tab selection (don't auto-switch to Transaction Log)
              // Also update selected agent to match the clicked card
              setSelectedAgent(agentType);
            }}
            isRunning={isRunning}
            onRunBenchmark={handleRunBenchmark}
            runningBenchmarkIds={runningBenchmarkIds}
            executions={executions}
            agentPerformanceData={agentPerformanceData}
            agentPerformanceLoading={agentPerformanceLoading}
            agentPerformanceError={agentPerformanceError}
            refetchAgentPerformance={refetchAgentPerformance}
            benchmarks={Array.isArray(benchmarkData) ? benchmarkData : []}
            benchmarksLoading={benchmarksLoading}
            benchmarksError={benchmarksError}
            refetchBenchmarks={refetchBenchmarks}
            selectedBenchmark={selectedBenchmark}
            selectedAgent={selectedAgent}
            selectedDate={selectedDate}
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
      <div className="flex-1 flex">
        {/* Left Panel - Benchmark List and Config */}
        <div
          className="w-1/3 border-r bg-white dark:bg-gray-800 flex flex-col"
          style="min-width: 490px;"
        >
          {/* Benchmark List */}
          <div className="flex-1 overflow-hidden">
            <BenchmarkList
              selectedAgent={selectedAgent}
              selectedBenchmark={selectedBenchmark}
              selectedDate={selectedDate}
              onBenchmarkSelect={handleBenchmarkSelect}
              isRunning={isRunning || isRunningAll}
              onExecutionStart={handleExecutionStart}
              onExecutionComplete={handleExecutionComplete}
              executions={executions}
              updateExecution={updateExecution}
              isRunningAll={isRunningAll}
              agentPerformanceData={agentPerformanceData}
              agentPerformanceLoading={agentPerformanceLoading}
              agentPerformanceError={agentPerformanceError}
              refreshTrigger={performanceOverviewRefresh}
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
          <div className="p-4 border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800">
            <div className="flex items-center justify-between">
              <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
                {selectedBenchmark
                  ? `Benchmark: ${selectedBenchmark}`
                  : "Execution Details"}
              </h2>
            </div>
          </div>

          {/* Right Panel Content */}
          <div className="flex-1 flex flex-col">
            {/* Tab Navigation */}
            <div className="flex border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800">
              <button
                onClick={() => setShowTransactionLog(false)}
                className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
                  !showTransactionLog
                    ? "border-blue-500 text-blue-600 bg-blue-50 dark:bg-blue-900/20"
                    : "border-transparent text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200"
                }`}
              >
                Execution Trace
              </button>
              <button
                onClick={() => setShowTransactionLog(true)}
                className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
                  showTransactionLog
                    ? "border-blue-500 text-blue-600 bg-blue-50 dark:bg-blue-900/20"
                    : "border-transparent text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200"
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
                  <ExecutionTrace
                    benchmarkId={selectedBenchmark}
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

render(
  <ThemeProvider>
    <App />
  </ThemeProvider>,
  document.getElementById("app"),
);
