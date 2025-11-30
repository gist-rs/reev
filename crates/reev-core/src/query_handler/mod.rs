//! Query Handler for Processing User Prompts
//!
//! This module provides a high-level interface for processing user queries
//! from natural language to transaction execution. It handles the complete
//! flow from prompt refinement to transaction execution, including special
//! handling for "all" keyword in transfer requests.
//!
//! This module is intended to be used by:
//! 1. API endpoints that handle user queries
//! 2. Test suites that need to verify end-to-end functionality
//! 3. CLI tools that interact with the system

use anyhow::Result;
use solana_sdk::pubkey::Pubkey;
use tracing::{debug, info, instrument};

use crate::context::{ContextResolver, SolanaEnvironment};
use crate::executor::Executor;
use crate::planner::Planner;
use crate::yml_schema::YmlFlow;
use reev_types::flow::WalletContext;

/// Result of processing a user query
#[derive(Debug, Clone)]
pub struct QueryResult {
    /// Whether the query was processed successfully
    pub success: bool,
    /// Transaction signature if applicable
    pub transaction_signature: Option<String>,
    /// Error message if processing failed
    pub error_message: Option<String>,
}

impl QueryResult {
    /// Create a successful result with a transaction signature
    pub fn success_with_signature(signature: String) -> Self {
        Self {
            success: true,
            transaction_signature: Some(signature),
            error_message: None,
        }
    }

    /// Create a successful result without a transaction signature
    pub fn success() -> Self {
        Self {
            success: true,
            transaction_signature: None,
            error_message: None,
        }
    }

    /// Create a failed result with an error message
    pub fn failure(error: String) -> Self {
        Self {
            success: false,
            transaction_signature: None,
            error_message: Some(error),
        }
    }
}

/// Handler for processing user queries end-to-end
///
/// This struct provides a high-level interface for processing user prompts,
/// handling all the steps from language refinement to transaction execution.
/// It properly handles special cases like "all" keyword in transfer requests.
pub struct QueryHandler {
    /// Context resolver for wallet information
    context_resolver: ContextResolver,
    /// Planner for refining and planning the query
    planner: Planner,
    /// Executor for executing the generated flow
    executor: Executor,
}

impl QueryHandler {
    /// Create a new query handler with default configuration
    ///
    /// # Returns
    /// A new QueryHandler instance
    ///
    /// # Errors
    /// Returns an error if initialization fails
    pub async fn new() -> Result<Self> {
        // Initialize context resolver with SURFPOOL environment
        let context_resolver = ContextResolver::new(SolanaEnvironment {
            rpc_url: Some("http://localhost:8899".to_string()),
        });

        // Create planner with GLM client
        let planner = Planner::new_with_glm(context_resolver.clone())?;

        // Create executor with rig
        let executor = Executor::new_with_rig().await?;

        Ok(Self {
            context_resolver,
            planner,
            executor,
        })
    }

    /// Create a new query handler with custom components
    ///
    /// # Arguments
    /// * `context_resolver` - Custom context resolver
    /// * `planner` - Custom planner
    /// * `executor` - Custom executor
    ///
    /// # Returns
    /// A new QueryHandler instance with the provided components
    pub fn with_components(
        context_resolver: ContextResolver,
        planner: Planner,
        executor: Executor,
    ) -> Self {
        Self {
            context_resolver,
            planner,
            executor,
        }
    }

    /// Process a user query from natural language to execution
    ///
    /// This method handles the complete flow of processing a user query:
    /// 1. Resolves the wallet context
    /// 2. Refines the prompt (handling "all" keyword for transfers)
    /// 3. Generates a structured flow
    /// 4. Executes the flow
    /// 5. Returns the result
    ///
    /// # Arguments
    /// * `prompt` - Natural language prompt from the user
    /// * `wallet_pubkey` - Public key of the wallet to use
    ///
    /// # Returns
    /// A QueryResult containing the outcome of the operation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use reev_core::query_handler::QueryHandler;
    ///
    /// let handler = QueryHandler::new().await?;
    ///
    /// // Process a transfer with specific amount
    /// let result = handler.process_query(
    ///     "send 1.5 sol to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq",
    ///     &user_pubkey
    /// ).await?;
    ///
    /// // Process a transfer with "all" keyword
    /// let result = handler.process_query(
    ///     "send all sol to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq",
    ///     &user_pubkey
    /// ).await?;
    /// ```
    #[instrument(skip(self))]
    pub async fn process_query(
        &mut self,
        prompt: &str,
        wallet_pubkey: &Pubkey,
    ) -> Result<QueryResult> {
        info!("Processing user query: {}", prompt);

        // Step 1: Resolve wallet context
        debug!("Resolving wallet context for {}", wallet_pubkey);
        let wallet_context = self
            .context_resolver
            .resolve_wallet_context(&wallet_pubkey.to_string())
            .await?;

        // Step 2: Refine and plan the query
        debug!("Refining and planning query");
        let flow = self
            .planner
            .refine_and_plan(prompt, &wallet_pubkey.to_string())
            .await?;

        // Step 3: Execute the flow
        debug!("Executing flow: {}", flow.flow_id);
        let result = self.executor.execute_flow(&flow, &wallet_context).await?;

        // Step 4: Extract transaction signature
        debug!("Extracting transaction signature");
        let signature = crate::utils::result_utils::extract_transaction_signature(&result)?;

        // Step 5: Create and return result
        info!("Query processed successfully with signature: {}", signature);
        Ok(QueryResult::success_with_signature(signature))
    }

    /// Process a user query without execution (planning only)
    ///
    /// This method only performs the planning phase and returns the generated flow
    /// without executing it. Useful for previewing what would be executed.
    ///
    /// # Arguments
    /// * `prompt` - Natural language prompt from the user
    /// * `wallet_pubkey` - Public key of the wallet to use
    ///
    /// # Returns
    /// A tuple containing the generated flow and wallet context
    #[instrument(skip(self))]
    pub async fn plan_query_only(
        &mut self,
        prompt: &str,
        wallet_pubkey: &Pubkey,
    ) -> Result<(YmlFlow, WalletContext)> {
        info!("Planning query: {}", prompt);

        // Resolve wallet context
        debug!("Resolving wallet context for {}", wallet_pubkey);
        let wallet_context = self
            .context_resolver
            .resolve_wallet_context(&wallet_pubkey.to_string())
            .await?;

        // Refine and plan the query
        debug!("Refining and planning query");
        let flow = self
            .planner
            .refine_and_plan(prompt, &wallet_pubkey.to_string())
            .await?;

        Ok((flow, wallet_context))
    }

    /// Get the current balance for a wallet
    ///
    /// # Arguments
    /// * `wallet_pubkey` - Public key of the wallet to check
    ///
    /// # Returns
    /// The current SOL balance in lamports
    #[instrument(skip(self))]
    pub async fn get_wallet_balance(&self, wallet_pubkey: &Pubkey) -> Result<u64> {
        debug!("Getting wallet balance for {}", wallet_pubkey);

        let wallet_context = self
            .context_resolver
            .resolve_wallet_context(&wallet_pubkey.to_string())
            .await?;

        Ok(wallet_context.sol_balance)
    }

    /// Calculate the maximum transferable amount for a wallet
    ///
    /// This is a convenience method that uses the calculate_max_transferable_amount
    /// utility function with the current wallet balance.
    ///
    /// # Arguments
    /// * `wallet_pubkey` - Public key of the wallet to check
    /// * `gas_reserve` - Amount of lamports to reserve for gas (default: 1,000,000)
    ///
    /// # Returns
    /// The maximum amount in lamports that can be transferred
    #[instrument(skip(self))]
    pub async fn calculate_max_transferable(
        &self,
        wallet_pubkey: &Pubkey,
        gas_reserve: Option<u64>,
    ) -> Result<u64> {
        debug!("Calculating max transferable amount for {}", wallet_pubkey);

        let wallet_context = self
            .context_resolver
            .resolve_wallet_context(&wallet_pubkey.to_string())
            .await?;

        let gas_reserve = gas_reserve.unwrap_or(1_000_000); // Default 0.001 SOL

        Ok(
            crate::utils::transfer_utils::calculate_max_transferable_amount(
                "", // SOL
                wallet_context.sol_balance,
                gas_reserve,
            ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_query_handler_creation() {
        // Test that we can create a query handler
        let result = QueryHandler::new().await;
        assert!(result.is_ok(), "Failed to create query handler");
    }

    #[test]
    fn test_query_result_creation() {
        // Test successful result with signature
        let success_result = QueryResult::success_with_signature("test_signature".to_string());
        assert!(success_result.success);
        assert_eq!(
            success_result.transaction_signature,
            Some("test_signature".to_string())
        );
        assert!(success_result.error_message.is_none());

        // Test successful result without signature
        let success_result = QueryResult::success();
        assert!(success_result.success);
        assert!(success_result.transaction_signature.is_none());
        assert!(success_result.error_message.is_none());

        // Test failed result
        let failure_result = QueryResult::failure("test error".to_string());
        assert!(!failure_result.success);
        assert!(failure_result.transaction_signature.is_none());
        assert_eq!(failure_result.error_message, Some("test error".to_string()));
    }
}
