// Standalone test for Run All logic
// Run with: node test-run-all.js

// Mock the API and execution state
class MockApiClient {
  constructor() {
    this.callCount = 0;
    this.calls = [];
  }

  async runBenchmark(benchmarkId, options) {
    this.callCount++;
    this.calls.push({ benchmarkId, options });
    console.log(
      `API: Starting benchmark ${benchmarkId} (call #${this.callCount})`,
    );

    return {
      execution_id: `exec-${benchmarkId}-${Date.now()}`,
      status: "started",
    };
  }
}

// Test the Run All logic
async function testRunAllLogic() {
  console.log("=== Testing Run All Logic ===\n");

  const apiClient = new MockApiClient();
  const executions = new Map();
  const updateExecution = () => {};
  const onExecutionStart = () => {};
  const refetch = { mock: { calls: [] } };

  const benchmarks = [
    { id: "benchmark-1", name: "Benchmark 1" },
    { id: "benchmark-2", name: "Benchmark 2" },
    { id: "benchmark-3", name: "Benchmark 3" },
  ];

  const handleRunBenchmark = async (benchmark) => {
    console.log(`Running benchmark: ${benchmark.id}`);

    const response = await apiClient.runBenchmark(benchmark.id, {
      agent: "deterministic",
    });

    // Simulate updating execution state
    const execution = {
      id: response.execution_id,
      benchmark_id: benchmark.id,
      agent: "deterministic",
      status: "Pending",
      progress: 0,
      start_time: new Date().toISOString(),
      trace: "",
      logs: "",
    };

    executions.set(response.execution_id, execution);
    updateExecution(benchmark.id, execution);
    onExecutionStart(response.execution_id);

    console.log(`Execution created: ${response.execution_id}`);
    return response;
  };

  // The actual Run All logic we want to test
  const handleRunAllBenchmarks = async () => {
    console.log(`Starting Run All for ${benchmarks.length} benchmarks`);

    for (let i = 0; i < benchmarks.length; i++) {
      const benchmark = benchmarks[i];
      console.log(
        `Starting benchmark ${i + 1}/${benchmarks.length}: ${benchmark.id}`,
      );

      // Start the benchmark
      const response = await handleRunBenchmark(benchmark);

      // Wait for the benchmark to complete before starting the next one
      await new Promise((resolve) => {
        const checkCompletion = () => {
          const execution = Array.from(executions.values()).find(
            (exec) => exec.benchmark_id === benchmark.id,
          );

          console.log(
            `Checking completion for ${benchmark.id}, status: ${execution?.status}`,
          );

          if (
            execution &&
            (execution.status === "Completed" || execution.status === "Failed")
          ) {
            console.log(
              `Benchmark ${benchmark.id} completed with status: ${execution.status}`,
            );
            resolve();
          } else {
            // Simulate completion after 2 seconds for testing
            setTimeout(() => {
              // Update execution to completed
              if (execution) {
                execution.status = "Completed";
                execution.progress = 100;
                execution.trace = "Mock trace completed";
                execution.logs = "Mock logs completed";
                console.log(`Simulated completion for ${benchmark.id}`);
              }
              setTimeout(checkCompletion, 100);
            }, 2000);
          }
        };

        // Start checking after a short delay
        setTimeout(checkCompletion, 1000);
      });
    }

    console.log("All benchmarks completed, refreshing overview");
    refetch.mock.calls.push("refetch called");
  };

  // Run the test
  try {
    await handleRunAllBenchmarks();

    console.log("\n=== Test Results ===");
    console.log(`✅ All benchmarks completed successfully`);
    console.log(`✅ API calls made: ${apiClient.callCount}`);
    console.log(
      `✅ Benchmarks called:`,
      apiClient.calls.map((c) => c.benchmarkId),
    );
    console.log(`✅ Refetch called: ${refetch.mock.calls.length} time(s)`);
    console.log(`✅ Executions created: ${executions.size}`);

    // Verify all benchmarks were called
    const expectedBenchmarks = ["benchmark-1", "benchmark-2", "benchmark-3"];
    const actualBenchmarks = apiClient.calls.map((c) => c.benchmarkId);

    if (
      JSON.stringify(expectedBenchmarks) === JSON.stringify(actualBenchmarks)
    ) {
      console.log(`✅ All expected benchmarks were called in order`);
    } else {
      console.log(`❌ Expected ${expectedBenchmarks}, got ${actualBenchmarks}`);
    }

    if (refetch.mock.calls.length === 1) {
      console.log(`✅ Overview refresh called exactly once at the end`);
    } else {
      console.log(
        `❌ Expected 1 refetch call, got ${refetch.mock.calls.length}`,
      );
    }
  } catch (error) {
    console.error("❌ Test failed:", error);
  }
}

// Mock functions for testing

// Run the test
testRunAllLogic()
  .then(() => {
    console.log("\n=== Test Complete ===");
  })
  .catch((error) => {
    console.error("Test failed:", error);
  });
