use crate::{
    agent::{AgentAction, AgentObservation},
    benchmark::GroundTruth,
    env::{GymEnv, Step},
    solana_env::{reset, step},
};
use anyhow::{Context, Result};
use serde_json::Value;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use std::collections::HashMap;
use tracing::{debug, info};

const LOCAL_SURFPOOL_RPC_URL: &str = "http://127.0.0.1:8899";

pub struct SolanaEnv {
    pub rpc_client: RpcClient,
    pub keypair_map: HashMap<String, Keypair>,
    pub pubkey_map: HashMap<String, Pubkey>,
    pub fee_payer: Option<String>,
}

impl SolanaEnv {
    pub fn new() -> Result<Self> {
        let rpc_client = RpcClient::new_with_commitment(
            LOCAL_SURFPOOL_RPC_URL.to_string(),
            CommitmentConfig::confirmed(),
        );

        Ok(Self {
            rpc_client,
            keypair_map: HashMap::new(),
            pubkey_map: HashMap::new(),
            fee_payer: None,
        })
    }

    pub fn fee_payer_placeholder(&self) -> &str {
        self.fee_payer.as_deref().unwrap_or_default()
    }

    pub fn get_fee_payer_keypair(&self) -> Result<&Keypair> {
        self.fee_payer
            .as_ref()
            .and_then(|p| self.keypair_map.get(p))
            .context("Fee payer keypair not found")
    }

    pub(crate) fn sign_and_send_transaction(
        &self,
        mut transaction: Transaction,
        signers: &[&Keypair],
    ) -> Result<solana_sdk::signature::Signature> {
        let recent_blockhash = self.rpc_client.get_latest_blockhash()?;
        transaction.sign(signers, recent_blockhash);
        self.rpc_client
            .send_and_confirm_transaction_with_spinner(&transaction)
            .context("Failed to send and confirm transaction")
    }
}

impl GymEnv for SolanaEnv {
    type Action = AgentAction;
    type Observation = AgentObservation;

    #[tracing::instrument(skip_all, name = "env.reset")]
    async fn reset(
        &mut self,
        _seed: Option<u64>,
        options: Option<Value>,
    ) -> Result<Self::Observation> {
        reset::handle_reset(self, options).await
    }

    #[tracing::instrument(skip_all, name = "env.step")]
    fn step(
        &mut self,
        actions: Vec<Self::Action>,
        _ground_truth: &GroundTruth,
    ) -> Result<Step<Self::Observation>> {
        step::handle_step(self, actions)
    }

    fn render(&self) {
        debug!("--- Current On-Chain State ---");
        for (name, keypair) in &self.keypair_map {
            let pubkey = keypair.pubkey();
            match self.rpc_client.get_account(&pubkey) {
                Ok(account) => {
                    debug!(
                        "Pubkey: {} (Name: {}), Owner: {}, Lamports: {}",
                        pubkey, name, account.owner, account.lamports
                    );
                }
                Err(_) => {
                    debug!("Pubkey: {pubkey} (Name: {name}), Account not found on-chain.");
                }
            }
        }
        debug!("--------------------------------");
    }

    fn close(&mut self) -> Result<()> {
        info!("[SolanaEnv] Environment closed. Validator process is left running.");
        Ok(())
    }
}
