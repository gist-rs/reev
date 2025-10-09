katopz@m3 reev % ./test_local_agent.sh --local
Discovering benchmark files...
Found 13 benchmark files
  - 001-sol-transfer.yml
  - 002-spl-transfer.yml
  - 003-spl-transfer-fail.yml
  - 004-partial-score-spl-transfer.yml
  - 100-jup-swap-sol-usdc.yml
  - 110-jup-lend-deposit-sol.yml
  - 111-jup-lend-deposit-usdc.yml
  - 112-jup-lend-withdraw-sol.yml
  - 113-jup-lend-withdraw-usdc.yml
  - 114-jup-positions-and-earnings.yml
  - 115-jup-lend-mint-usdc.yml
  - 116-jup-lend-redeem-usdc.yml
  - 200-jup-swap-then-lend-deposit.yml
Testing 13 benchmark(s) with local agents (flag: --agent local)
All benchmarks
===========================================================================
Testing benchmarks/001-sol-transfer.yml... ✅ Score: 100.0%
Testing benchmarks/002-spl-transfer.yml... ✅ Score: 100.0%
Testing benchmarks/003-spl-transfer-fail.yml... ✅ Score: 75.0%
Testing benchmarks/004-partial-score-spl-transfer.yml... ✅ Score: 78.6%
Testing benchmarks/100-jup-swap-sol-usdc.yml... ✅ Score: 100.0%
Testing benchmarks/110-jup-lend-deposit-sol.yml... ✅ Score: 75.0%
Testing benchmarks/111-jup-lend-deposit-usdc.yml... ❌ ERROR - Last 10 lines of output:
  "id": "111-jup-lend-deposit-usdc",
  "model_name": "local",
  "prompt": "Lend 50 USDC using Jupiter."
}
2025-10-09T03:17:18.263249Z  INFO reev_runner: Dependency manager dropped - processes will be cleaned up on next startup
2025-10-09T03:17:18.263325Z DEBUG reev_runner::dependency::manager::dependency_manager: DependencyManager dropped
Error: Evaluation loop failed for benchmark: 111-jup-lend-deposit-usdc

Caused by:
    LLM API request failed with status 500 Internal Server Error: {"error":"Internal agent error: MaxDepthError: (reached limit: 3)"}
---
❌ ERROR
Testing benchmarks/112-jup-lend-withdraw-sol.yml... ❌ ERROR - Last 10 lines of output:
  "id": "112-jup-lend-withdraw-sol",
  "model_name": "local",
  "prompt": "Withdraw 0.1 SOL using Jupiter."
}
2025-10-09T03:19:18.854263Z  INFO reev_runner: Dependency manager dropped - processes will be cleaned up on next startup
2025-10-09T03:19:18.854354Z DEBUG reev_runner::dependency::manager::dependency_manager: DependencyManager dropped
Error: Evaluation loop failed for benchmark: 112-jup-lend-withdraw-sol

Caused by:
    LLM API request failed with status 500 Internal Server Error: {"error":"Internal agent error: MaxDepthError: (reached limit: 3)"}
---
❌ ERROR
Testing benchmarks/113-jup-lend-withdraw-usdc.yml... ❌ ERROR - Last 10 lines of output:
  "id": "113-jup-lend-withdraw-usdc",
  "model_name": "local",
  "prompt": "Withdraw 50 USDC from your Solend lending position. You have L-USDC tokens (mint: 9BEcn9aPEmhSPbPQeFGjidRiEKki46fVQDyPpSQXPA2D) that can be redeemed for USDC."
}
2025-10-09T03:21:16.641451Z  INFO reev_runner: Dependency manager dropped - processes will be cleaned up on next startup
2025-10-09T03:21:16.641507Z DEBUG reev_runner::dependency::manager::dependency_manager: DependencyManager dropped
Error: Evaluation loop failed for benchmark: 113-jup-lend-withdraw-usdc

Caused by:
    LLM API request failed with status 500 Internal Server Error: {"error":"Internal agent error: MaxDepthError: (reached limit: 3)"}
---
❌ ERROR
Testing benchmarks/114-jup-positions-and-earnings.yml... ✅ Score: 100.0%
Testing benchmarks/115-jup-lend-mint-usdc.yml... ✅ Score: 85.0%
Testing benchmarks/116-jup-lend-redeem-usdc.yml... ✅ Score: 75.0%
Testing benchmarks/200-jup-swap-then-lend-deposit.yml... ✅ Score: 100.0%

Summary:
========
benchmarks/001-sol-transfer.yml: SUCCESS (Score: 100.0%)
benchmarks/002-spl-transfer.yml: SUCCESS (Score: 100.0%)
benchmarks/003-spl-transfer-fail.yml: SUCCESS (Score: 75.0%)
benchmarks/004-partial-score-spl-transfer.yml: SUCCESS (Score: 78.6%)
benchmarks/100-jup-swap-sol-usdc.yml: SUCCESS (Score: 100.0%)
benchmarks/110-jup-lend-deposit-sol.yml: SUCCESS (Score: 75.0%)
benchmarks/111-jup-lend-deposit-usdc.yml: ERROR
benchmarks/112-jup-lend-withdraw-sol.yml: ERROR
benchmarks/113-jup-lend-withdraw-usdc.yml: ERROR
benchmarks/114-jup-positions-and-earnings.yml: SUCCESS (Score: 100.0%)
benchmarks/115-jup-lend-mint-usdc.yml: SUCCESS (Score: 85.0%)
benchmarks/116-jup-lend-redeem-usdc.yml: SUCCESS (Score: 75.0%)
benchmarks/200-jup-swap-then-lend-deposit.yml: SUCCESS (Score: 100.0%)
