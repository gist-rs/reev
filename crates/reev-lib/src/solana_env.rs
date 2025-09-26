use crate::{
    agent::{AgentAction, AgentObservation},
    benchmark::GroundTruth,
    env::{GymEnv, Step},
};
use anyhow::{Context, Result};
use serde_json::{json, Value};
use solana_client::rpc_client::RpcClient;
use solana_program::program_pack::Pack;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use solana_system_interface::instruction as system_instruction;
use solana_transaction_status::UiTransactionEncoding;
use spl_token::{
    instruction as spl_instruction,
    state::{Account as SplTokenAccount, Mint},
};
use std::{collections::HashMap, thread, time::Duration};
use tracing::{debug, info, warn};

const LOCAL_SURFPOOL_RPC_URL: &str = "http://127.0.0.1:8899";

pub struct SolanaEnv {
    rpc_client: RpcClient,
    keypair_map: HashMap<String, Keypair>,
    fee_payer: Option<String>,
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
            fee_payer: None,
        })
    }

    pub fn fee_payer_placeholder(&self) -> &str {
        self.fee_payer.as_deref().unwrap_or_default()
    }

    fn get_fee_payer_keypair(&self) -> Result<&Keypair> {
        self.fee_payer
            .as_ref()
            .and_then(|p| self.keypair_map.get(p))
            .context("Fee payer keypair not found")
    }

    fn sign_and_send_transaction(
        &self,
        mut transaction: Transaction,
        signers: &[&Keypair],
    ) -> Result<solana_sdk::signature::Signature> {
        let recent_blockhash = self.rpc_client.get_latest_blockhash()?;
        transaction.sign(signers, recent_blockhash);
        self.rpc_client
            .send_and_confirm_transaction(&transaction)
            .context("Failed to send and confirm transaction")
    }

    fn get_observation(
        &self,
        last_tx_status: &str,
        last_tx_error: Option<String>,
        last_tx_logs: Vec<String>,
    ) -> Result<AgentObservation> {
        let mut account_states = HashMap::new();
        let mut key_map = HashMap::new();

        for (name, keypair) in &self.keypair_map {
            key_map.insert(name.clone(), keypair.pubkey().to_string());
            if let Ok(account) = self.rpc_client.get_account(&keypair.pubkey()) {
                let mut state = json!({
                    "lamports": account.lamports,
                    "owner": account.owner.to_string(),
                    "executable": account.executable,
                    "data_len": account.data.len(),
                });

                if account.owner == spl_token::ID
                    && account.data.len() == SplTokenAccount::LEN {
                        let token_account = SplTokenAccount::unpack(&account.data)?;
                        if let Some(obj) = state.as_object_mut() {
                            obj.insert("mint".to_string(), json!(token_account.mint.to_string()));
                            obj.insert(
                                "token_account_owner".to_string(),
                                json!(token_account.owner.to_string()),
                            );
                            obj.insert("amount".to_string(), json!(token_account.amount));
                        }
                    }
                account_states.insert(name.clone(), state);
            }
        }

        Ok(AgentObservation {
            account_states,
            key_map,
            last_transaction_status: last_tx_status.to_string(),
            last_transaction_error: last_tx_error,
            last_transaction_logs: last_tx_logs,
        })
    }
}

impl GymEnv for SolanaEnv {
    type Action = AgentAction;
    type Observation = AgentObservation;

    #[tracing::instrument(skip_all, name = "env.reset")]
    fn reset(&mut self, _seed: Option<u64>, options: Option<Value>) -> Result<Self::Observation> {
        info!("Resetting Solana environment...");
        info!("Checking for running `surfpool` validator...");
        for i in 0..10 {
            if self.rpc_client.get_health().is_ok() {
                break;
            }
            if i == 9 {
                anyhow::bail!(
                    "Could not connect to `surfpool` validator at {LOCAL_SURFPOOL_RPC_URL}"
                );
            }
            thread::sleep(Duration::from_secs(1));
        }
        info!("Validator is healthy.");

        self.keypair_map.clear();
        self.fee_payer = None;
        let initial_state_val = options
            .and_then(|v| v.get("initial_state").cloned())
            .context("Benchmark options must include 'initial_state'")?;
        let accounts: Vec<Value> = serde_json::from_value(initial_state_val)?;

        for account_config in &accounts {
            let pubkey_placeholder = account_config["pubkey"]
                .as_str()
                .context("Missing 'pubkey' placeholder in account config")?;

            if pubkey_placeholder == "USER_WALLET_PUBKEY" {
                self.fee_payer = Some(pubkey_placeholder.to_string());
            }

            self.keypair_map
                .insert(pubkey_placeholder.to_string(), Keypair::new());
        }

        let fee_payer_placeholder = self.fee_payer.as_ref().context("Fee payer not set")?;
        let fee_payer_config = accounts
            .iter()
            .find(|acc| acc["pubkey"].as_str() == Some(fee_payer_placeholder))
            .context("Fee payer config not found in initial state")?;

        let fee_payer_keypair = self.get_fee_payer_keypair()?;
        let initial_lamports = fee_payer_config["lamports"]
            .as_u64()
            .context("Fee payer 'lamports' not found or invalid in initial state")?;

        if initial_lamports > 0 {
            info!(
                "Funding fee payer ({}) with {} lamports...",
                fee_payer_keypair.pubkey(),
                initial_lamports
            );
            let sig = self
                .rpc_client
                .request_airdrop(&fee_payer_keypair.pubkey(), initial_lamports)?;
            self.rpc_client
                .confirm_transaction(&sig)
                .context("Failed to confirm fee payer airdrop")?;
            info!("Fee payer funded.");
        }

        let mut mint_configs = Vec::new();
        let mut token_configs = Vec::new();

        for account_config in &accounts {
            if let Some(owner) = account_config["owner"].as_str() {
                if owner == "11111111111111111111111111111111" {
                    let placeholder = account_config["pubkey"].as_str().unwrap();
                    if self.fee_payer.as_deref() == Some(placeholder) {
                        continue;
                    }

                    let keypair = self.keypair_map.get(placeholder).unwrap();
                    let lamports = account_config["lamports"].as_u64().unwrap_or(0);
                    if lamports > 0 {
                        info!(
                            "Airdropping {lamports} lamports to {placeholder} ({})...",
                            keypair.pubkey()
                        );
                        let sig = self
                            .rpc_client
                            .request_airdrop(&keypair.pubkey(), lamports)?;
                        self.rpc_client
                            .confirm_transaction(&sig)
                            .context("Failed to confirm airdrop")?;
                    }
                } else if owner == spl_token::ID.to_string() {
                    if account_config.get("mint_data").is_some() {
                        mint_configs.push(account_config.clone());
                    } else if account_config.get("data").is_some() {
                        token_configs.push(account_config.clone());
                    }
                }
            }
        }

        for config in &mint_configs {
            let placeholder = config["pubkey"].as_str().unwrap();
            let keypair = self.keypair_map.get(placeholder).unwrap();
            info!("Creating SPL Mint: {placeholder} ({})", keypair.pubkey());
            let mint_data = config.get("mint_data").unwrap();
            let decimals = mint_data["decimals"].as_u64().unwrap() as u8;
            let auth_placeholder = mint_data["mint_authority"]
                .as_str()
                .unwrap_or("USER_WALLET_PUBKEY");
            let authority = self
                .keypair_map
                .get(auth_placeholder)
                .context("Mint authority not found")?;

            let rent = self
                .rpc_client
                .get_minimum_balance_for_rent_exemption(Mint::LEN)?;
            let instructions = [
                system_instruction::create_account(
                    &fee_payer_keypair.pubkey(),
                    &keypair.pubkey(),
                    rent,
                    Mint::LEN as u64,
                    &spl_token::ID,
                ),
                spl_instruction::initialize_mint(
                    &spl_token::ID,
                    &keypair.pubkey(),
                    &authority.pubkey(),
                    None,
                    decimals,
                )?,
            ];
            let transaction =
                Transaction::new_with_payer(&instructions, Some(&fee_payer_keypair.pubkey()));
            self.sign_and_send_transaction(transaction, &[fee_payer_keypair, keypair])?;
        }

        for config in token_configs {
            let placeholder = config["pubkey"].as_str().unwrap();
            let keypair = self.keypair_map.get(placeholder).unwrap();
            info!(
                "Creating SPL Token Account: {placeholder} ({})",
                keypair.pubkey()
            );
            let data_str = config["data"]
                .as_str()
                .context("'data' must be a JSON string")?;
            let token_state: HashMap<String, Value> =
                serde_json::from_str(data_str).context("Failed to parse 'data' JSON")?;

            let mint_placeholder = token_state["mint"]
                .as_str()
                .context("Missing 'mint' in data")?;
            let owner_placeholder = token_state["owner"]
                .as_str()
                .context("Missing 'owner' in data")?;
            let amount = token_state
                .get("amount")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            let mint_pubkey = self
                .keypair_map
                .get(mint_placeholder)
                .context("Mint keypair not found")?
                .pubkey();
            let owner_pubkey = self
                .keypair_map
                .get(owner_placeholder)
                .context("Owner keypair not found")?
                .pubkey();

            let rent = self
                .rpc_client
                .get_minimum_balance_for_rent_exemption(SplTokenAccount::LEN)?;
            let mut instructions = vec![
                system_instruction::create_account(
                    &fee_payer_keypair.pubkey(),
                    &keypair.pubkey(),
                    rent,
                    SplTokenAccount::LEN as u64,
                    &spl_token::ID,
                ),
                spl_instruction::initialize_account(
                    &spl_token::ID,
                    &keypair.pubkey(),
                    &mint_pubkey,
                    &owner_pubkey,
                )?,
            ];

            let mut signers = vec![fee_payer_keypair, keypair];
            if amount > 0 {
                let mint_config = mint_configs
                    .iter()
                    .find(|mc| mc["pubkey"].as_str() == Some(mint_placeholder))
                    .context("Could not find mint config for token account")?;

                let mint_authority_placeholder = mint_config["mint_data"]
                    .get("mint_authority")
                    .and_then(|v| v.as_str())
                    .unwrap_or("USER_WALLET_PUBKEY");

                let mint_authority = self
                    .keypair_map
                    .get(mint_authority_placeholder)
                    .context("Mint authority keypair not found")?;

                signers.push(mint_authority);
                instructions.push(spl_instruction::mint_to(
                    &spl_token::ID,
                    &mint_pubkey,
                    &keypair.pubkey(),
                    &mint_authority.pubkey(),
                    &[],
                    amount,
                )?);
            }

            signers.sort_by_key(|k| k.pubkey());
            signers.dedup_by_key(|k| k.pubkey());

            let transaction =
                Transaction::new_with_payer(&instructions, Some(&fee_payer_keypair.pubkey()));
            self.sign_and_send_transaction(transaction, &signers)?;
        }

        info!("Environment reset complete.");
        self.get_observation("Success", None, vec!["Environment reset.".to_string()])
    }

    #[tracing::instrument(skip_all, name = "env.step")]
    fn step(
        &mut self,
        action: Self::Action,
        _ground_truth: &GroundTruth,
    ) -> Result<Step<Self::Observation>> {
        let instruction = action.0;
        let fee_payer_keypair = self.get_fee_payer_keypair()?;
        let mut signers = vec![fee_payer_keypair];

        for acc in &instruction.accounts {
            if acc.is_signer {
                if let Some(keypair) = self
                    .keypair_map
                    .values()
                    .find(|kp| kp.pubkey() == acc.pubkey)
                {
                    signers.push(keypair);
                } else {
                    warn!(
                        "Signer keypair for pubkey {} not found in keypair_map. Transaction may fail.",
                        acc.pubkey
                    );
                }
            }
        }
        signers.sort_by_key(|k| k.pubkey());
        signers.dedup_by_key(|k| k.pubkey());

        let transaction =
            Transaction::new_with_payer(&[instruction.clone()], Some(&fee_payer_keypair.pubkey()));

        info!(
            "Executing instruction for program: {}",
            instruction.program_id
        );

        match self.sign_and_send_transaction(transaction, &signers) {
            Ok(sig) => {
                let tx_info = self
                    .rpc_client
                    .get_transaction(&sig, UiTransactionEncoding::Json)?;
                let logs = tx_info
                    .transaction
                    .meta
                    .and_then(|meta| meta.log_messages.into())
                    .unwrap_or_default();
                let info = json!({ "signature": sig.to_string() });
                let obs = self.get_observation("Success", None, logs)?;
                Ok(Step {
                    observation: obs,
                    reward: 1.0,
                    terminated: true,
                    truncated: false,
                    info,
                })
            }
            Err(e) => {
                let error_string = format!("Transaction failed: {e}");
                warn!("{}", error_string);
                let obs = self.get_observation("Failure", Some(e.to_string()), vec![])?;
                Ok(Step {
                    observation: obs,
                    reward: 0.0,
                    terminated: true,
                    truncated: false,
                    info: json!({ "error": error_string }),
                })
            }
        }
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
