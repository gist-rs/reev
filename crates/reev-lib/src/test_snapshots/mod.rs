//! Snapshot-based testing infrastructure for reliable testing

use crate::types::*;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tokio::fs as async_fs;
use tracing::{debug, info, warn};

/// Snapshot data for deterministic testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSnapshot {
    /// Token prices snapshot
    pub prices: TokenPrices,
    /// Wallet states by address
    pub wallet_states: HashMap<String, WalletState>,
    /// Jupiter transaction data
    pub jupiter_responses: HashMap<String, serde_json::Value>,
    /// Mock tool responses
    pub tool_responses: HashMap<String, serde_json::Value>,
    /// Snapshot creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl ApiSnapshot {
    /// Create a new snapshot with default data
    pub fn new() -> Self {
        let mut prices = HashMap::new();
        prices.insert(SOL_MINT.to_string(), 150.0);
        prices.insert(USDC_MINT.to_string(), 1.0);

        let mut wallet_states = HashMap::new();

        // Add test wallet states
        let test_wallet1 = WalletState {
            sol_amount: 2_000_000_000, // 2 SOL
            usdc_amount: 100_000_000,  // 100 USDC
            sol_usd_value: 300.0,
            usdc_usd_value: 100.0,
            total_usd_value: 400.0,
        };
        wallet_states.insert("test_wallet_1".to_string(), test_wallet1);

        let test_wallet2 = WalletState {
            sol_amount: 5_000_000_000, // 5 SOL
            usdc_amount: 0,
            sol_usd_value: 750.0,
            usdc_usd_value: 0.0,
            total_usd_value: 750.0,
        };
        wallet_states.insert("test_wallet_2".to_string(), test_wallet2);

        // Mock Jupiter responses
        let mut jupiter_responses = HashMap::new();
        let mock_swap_response = serde_json::json!({
            "inputMint": SOL_MINT,
            "outputMint": USDC_MINT,
            "inputAmount": 1000000000,
            "outputAmount": 150000000,
            "slippageBps": 100,
            "priceImpactPct": 0.1
        });
        jupiter_responses.insert("sol_to_usdc_swap".to_string(), mock_swap_response);

        // Mock tool responses
        let mut tool_responses = HashMap::new();
        // For proper 70/30 allocation from 5 SOL ($750 total):
        // Need 525 USD in SOL (3.5 SOL) and 225 USD in USDC
        // So swap 1.5 SOL for 225 USDC
        let mock_swap_result = serde_json::json!({
            "success": true,
            "signature": "mock_signature_abc123",
            "input_amount": "1500000000",
            "output_amount": "225000000"
        });
        tool_responses.insert("jupiter_swap".to_string(), mock_swap_result);

        Self {
            prices,
            wallet_states,
            jupiter_responses,
            tool_responses,
            created_at: chrono::Utc::now(),
        }
    }

    /// Load snapshot from file
    pub async fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = async_fs::read_to_string(path)
            .await
            .map_err(|e| anyhow!("Failed to read snapshot file: {e}"))?;

        serde_json::from_str(&content).map_err(|e| anyhow!("Failed to parse snapshot JSON: {e}"))
    }

    /// Save snapshot to file
    pub async fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| anyhow!("Failed to serialize snapshot: {e}"))?;

        async_fs::write(path, content)
            .await
            .map_err(|e| anyhow!("Failed to write snapshot file: {e}"))
    }

    /// Get token price from snapshot
    pub fn get_price(&self, mint: &str) -> Option<f64> {
        self.prices.get(mint).copied()
    }

    /// Get wallet state from snapshot
    pub fn get_wallet_state(&self, address: &str) -> Option<&WalletState> {
        self.wallet_states.get(address)
    }

    /// Get Jupiter response from snapshot
    pub fn get_jupiter_response(&self, key: &str) -> Option<&serde_json::Value> {
        self.jupiter_responses.get(key)
    }

    /// Get tool response from snapshot
    pub fn get_tool_response(&self, tool_name: &str) -> Option<&serde_json::Value> {
        self.tool_responses.get(tool_name)
    }
}

impl Default for ApiSnapshot {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot manager for handling test data
pub struct SnapshotManager {
    /// Default snapshot instance
    snapshot: ApiSnapshot,
    /// Cache directory for snapshots
    cache_dir: String,
}

impl SnapshotManager {
    /// Create a new snapshot manager
    pub fn new(cache_dir: String) -> Self {
        Self {
            snapshot: ApiSnapshot::new(),
            cache_dir,
        }
    }

    /// Load or create snapshot
    pub async fn load_or_create_snapshot(&mut self) -> Result<&ApiSnapshot> {
        let snapshot_path = Path::new(&self.cache_dir).join("test_snapshot.json");

        if snapshot_path.exists() {
            debug!("Loading existing snapshot from: {:?}", snapshot_path);
            match ApiSnapshot::load_from_file(&snapshot_path).await {
                Ok(snapshot) => {
                    self.snapshot = snapshot;
                    info!(
                        "Successfully loaded snapshot with {} wallet states",
                        self.snapshot.wallet_states.len()
                    );
                }
                Err(e) => {
                    warn!("Failed to load snapshot, creating new one: {}", e);
                    self.create_default_snapshot().await?;
                }
            }
        } else {
            debug!("No snapshot found, creating default snapshot");
            self.create_default_snapshot().await?;
        }

        Ok(&self.snapshot)
    }

    /// Create and save default snapshot
    async fn create_default_snapshot(&mut self) -> Result<()> {
        // Ensure cache directory exists
        fs::create_dir_all(&self.cache_dir)
            .map_err(|e| anyhow!("Failed to create cache directory: {e}"))?;

        self.snapshot = ApiSnapshot::new();

        let snapshot_path = Path::new(&self.cache_dir).join("test_snapshot.json");
        self.snapshot.save_to_file(&snapshot_path).await?;

        info!("Created default snapshot at: {:?}", snapshot_path);
        Ok(())
    }

    /// Get current snapshot
    pub fn get_snapshot(&self) -> &ApiSnapshot {
        &self.snapshot
    }

    /// Update snapshot and save
    pub async fn update_snapshot(&mut self, snapshot: ApiSnapshot) -> Result<()> {
        self.snapshot = snapshot;

        let snapshot_path = Path::new(&self.cache_dir).join("test_snapshot.json");
        self.snapshot.save_to_file(&snapshot_path).await?;

        info!("Updated and saved snapshot");
        Ok(())
    }
}

/// Mock Jupiter client using snapshots
pub struct MockJupiterClient {
    snapshot: ApiSnapshot,
}

impl MockJupiterClient {
    pub fn new(snapshot: ApiSnapshot) -> Self {
        Self { snapshot }
    }

    pub async fn get_swap_quote(
        &self,
        input_mint: &str,
        output_mint: &str,
        _amount: u64,
    ) -> Result<serde_json::Value> {
        let key = format!("{input_mint}_to_{output_mint}_swap");
        self.snapshot
            .get_jupiter_response(&key)
            .cloned()
            .ok_or_else(|| anyhow!("No swap quote data for key: {key}"))
    }
}

#[async_trait::async_trait]
impl crate::core::JupiterClient for MockJupiterClient {
    async fn get_token_price(&self, mint: &str) -> Result<f64> {
        self.snapshot
            .get_price(mint)
            .ok_or_else(|| anyhow!("No price data for mint: {mint}"))
    }
}

/// Mock wallet manager using snapshots
pub struct MockWalletManager {
    snapshot: ApiSnapshot,
}

impl MockWalletManager {
    pub fn new(snapshot: ApiSnapshot) -> Self {
        Self { snapshot }
    }

    pub async fn get_wallet_state_internal(&self, address: &str) -> Result<WalletState> {
        self.snapshot
            .get_wallet_state(address)
            .cloned()
            .ok_or_else(|| anyhow!("No wallet state for address: {address}"))
    }
}

#[async_trait::async_trait]
impl crate::core::WalletManager for MockWalletManager {
    async fn get_wallet_state(&self, address: &str) -> Result<WalletState> {
        self.get_wallet_state_internal(address).await
    }
}

/// Mock tool executor using snapshots
pub struct MockToolExecutor {
    snapshot: ApiSnapshot,
}

impl MockToolExecutor {
    pub fn new(snapshot: ApiSnapshot) -> Self {
        Self { snapshot }
    }

    pub async fn execute_tool_internal(
        &self,
        request: &ToolExecutionRequest,
    ) -> Result<WalletState> {
        debug!(
            "Executing tool: {} with parameters: {:?}",
            request.tool_name, request.parameters
        );

        // Get mock response from snapshot
        let response = self
            .snapshot
            .get_tool_response(&request.tool_name)
            .ok_or_else(|| anyhow!("No mock response for tool: {}", request.tool_name))?;

        // Check if tool execution was successful
        if let Some(success) = response.get("success").and_then(|s| s.as_bool()) {
            if success {
                // Return updated wallet state (mock implementation)
                let mut updated_state = request.context.clone();

                // Simulate state changes based on tool type
                match request.tool_name.as_str() {
                    "jupiter_swap" => {
                        // Use actual parameters from request instead of hardcoded response
                        if let Some(input_amount_str) = request
                            .parameters
                            .get("input_amount")
                            .and_then(|v| v.as_str())
                        {
                            println!("Processing swap with input: {input_amount_str} SOL");
                            // Update SOL and USDC balances for swap
                            if let Ok(input_lamports) = input_amount_str.parse::<u64>() {
                                // Mock conversion: 1 SOL = 150 USDC (at $150/SOL)
                                // Convert input SOL lamports to SOL units, then to USD, then to USDC lamports
                                let input_sol = input_lamports as f64 / 1_000_000_000.0; // Convert lamports to SOL
                                let output_usd = input_sol * 150.0; // Convert SOL to USD at $150/SOL
                                let output_lamports = (output_usd * 1_000_000.0) as u64; // Convert USD to USDC lamports

                                println!("Calculated output: {} USDC", output_lamports / 1_000_000);

                                // Assume SOL -> USDC swap for mock
                                if request.context.sol_amount >= input_lamports {
                                    updated_state.sol_amount -= input_lamports;
                                    updated_state.usdc_amount += output_lamports;
                                    updated_state.sol_usd_value =
                                        updated_state.sol_balance_sol() * 150.0;
                                    updated_state.usdc_usd_value =
                                        updated_state.usdc_balance_usdc() * 1.0;
                                    updated_state.calculate_total_value();
                                }
                            }
                        }
                    }
                    _ => {
                        // No state change for other tools
                    }
                }

                Ok(updated_state)
            } else {
                Err(anyhow!("Tool execution failed"))
            }
        } else {
            Err(anyhow!("Invalid mock response format"))
        }
    }
}

#[async_trait::async_trait]
impl crate::core::ToolExecutor for MockToolExecutor {
    async fn execute_tool(&self, request: &ToolExecutionRequest) -> Result<WalletState> {
        self.execute_tool_internal(request).await
    }
}

/// Mock LLM client for testing
pub struct MockLLMClient {
    responses: HashMap<String, String>,
}

impl MockLLMClient {
    pub fn new() -> Self {
        let mut responses = HashMap::new();

        // Mock prompt refinement response
        let refinement_response = r#"
refined_prompt_series:
  step 1:
    prompt: "Swap 1.5 SOL to USDC using Jupiter aggregator for portfolio rebalancing"
    reasoning: "User needs 70/30 SOL/USDC allocation, convert 1.5 SOL to USDC to achieve target allocation"
  step 2:
    prompt: "Check transaction status after swap completion"
    reasoning: "Verify the swap was successful and funds are available"
"#;
        responses.insert("refine_prompt".to_string(), refinement_response.to_string());

        // Mock tool execution response
        let tool_execution_response = r#"
tool_call:
  tool_name: "jupiter_swap"
  parameters:
    input_mint: "So11111111111111111111111111111111111111112"
    output_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
    input_amount: "1500000000"
    slippage_bps: "100"
  reasoning: "Execute swap from SOL to USDC with 1% slippage tolerance for portfolio rebalancing"
"#;
        responses.insert(
            "tool_execution".to_string(),
            tool_execution_response.to_string(),
        );

        Self { responses }
    }

    pub async fn generate_response_internal(&self, prompt: &str) -> Result<String> {
        // Simple heuristic to determine which mock response to use
        if prompt.contains("refine") || prompt.contains("refinement") {
            self.responses
                .get("refine_prompt")
                .cloned()
                .ok_or_else(|| anyhow!("No mock refinement response available"))
        } else if prompt.contains("tool") || prompt.contains("execution") {
            self.responses
                .get("tool_execution")
                .cloned()
                .ok_or_else(|| anyhow!("No mock tool execution response available"))
        } else {
            Err(anyhow!("No mock response available for prompt"))
        }
    }
}

impl Default for MockLLMClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl crate::core::LLMClient for MockLLMClient {
    async fn generate_response(&self, prompt: &str) -> Result<String> {
        self.generate_response_internal(prompt).await
    }
}
