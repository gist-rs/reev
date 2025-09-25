use crate::{
    agent::{AgentAction, AgentObservation},
    benchmark::GroundTruth,
    env::{GymEnv, Step},
};
use anyhow::{Context, Result};
use serde_json::Value;
use solana_client::rpc_client::RpcClient;
use solana_program::program_pack::Pack;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use solana_system_interface::instruction as system_instruction;
use spl_token::{instruction as spl_instruction, state::Account as SplTokenAccount, state::Mint};
use std::{collections::HashMap, thread, time::Duration};

const LOCAL_SURFPOOL_RPC_URL: &str = "http://127.0.0.1:8899";

/// The main environment for interacting with a hermetic Solana test validator.
/// It connects to an externally managed `surfpool` instance.
pub struct SolanaEnv {
    /// The RPC client for communicating with the `surfpool` instance.
    rpc_client: RpcClient,
    /// A map of placeholder names (e.g., "USER_WALLET_PUBKEY") to their actual `Keypair`.
    keypair_map: HashMap<String, Keypair>,
    /// The placeholder name of the account designated to pay transaction fees.
    fee_payer: Option<String>,
}

impl SolanaEnv {
    /// Creates a new `SolanaEnv`.
    /// This assumes a `surfpool` validator is already running and accessible.
    pub fn new() -> Result<Self> {
        Ok(Self {
            rpc_client: RpcClient::new_with_commitment(
                LOCAL_SURFPOOL_RPC_URL.to_string(),
                CommitmentConfig::confirmed(),
            ),
            keypair_map: HashMap::new(),
            fee_payer: None,
        })
    }

    /// Returns the placeholder name of the designated fee payer, if one has been set.
    pub fn fee_payer_placeholder(&self) -> Option<&String> {
        self.fee_payer.as_ref()
    }

    /// Gets the keypair for the designated fee payer.
    fn get_fee_payer_keypair(&self) -> Result<&Keypair> {
        self.fee_payer
            .as_ref()
            .and_then(|name| self.keypair_map.get(name))
            .context("Fee payer is not set or not found in the environment's keypair map.")
    }

    /// Signs and sends a transaction, confirming it afterward.
    fn sign_and_send_transaction(
        &self,
        mut transaction: Transaction,
        signers: &[&Keypair],
    ) -> Result<()> {
        let recent_blockhash = self.rpc_client.get_latest_blockhash()?;
        transaction.sign(signers, recent_blockhash);
        self.rpc_client
            .send_and_confirm_transaction(&transaction)
            .context("Failed to send and confirm transaction")?;
        Ok(())
    }

    /// Gathers the current state of relevant accounts to form an observation.
    fn get_observation(
        &self,
        status: &str,
        error: Option<String>,
        logs: Vec<String>,
    ) -> Result<AgentObservation> {
        let mut account_states = HashMap::new();
        let mut key_map = HashMap::new();

        for (name, keypair) in &self.keypair_map {
            let pubkey = keypair.pubkey();
            // Always include the key in the key_map for full context.
            key_map.insert(name.clone(), pubkey.to_string());

            // Only include the state if the account actually exists on-chain.
            if let Ok(account) = self.rpc_client.get_account(&pubkey) {
                let mut account_map: serde_json::Map<String, Value> = serde_json::Map::new();
                account_map.insert("lamports".to_string(), account.lamports.into());
                account_map.insert("owner".to_string(), account.owner.to_string().into());
                account_map.insert("executable".to_string(), account.executable.into());
                account_map.insert("data_len".to_string(), account.data.len().into());

                // If this is a token account, unpack its data and add to the observation.
                if account.owner == spl_token::ID && account.data.len() == SplTokenAccount::LEN {
                    if let Ok(token_account) = SplTokenAccount::unpack(&account.data) {
                        account_map
                            .insert("mint".to_string(), token_account.mint.to_string().into());
                        account_map.insert(
                            "token_account_owner".to_string(),
                            token_account.owner.to_string().into(),
                        );
                        account_map.insert("amount".to_string(), token_account.amount.into());
                    }
                }

                account_states.insert(name.clone(), Value::Object(account_map));
            }
        }

        Ok(AgentObservation {
            last_transaction_status: status.to_string(),
            last_transaction_error: error,
            last_transaction_logs: logs,
            account_states,
            key_map,
        })
    }
}

impl GymEnv for SolanaEnv {
    type Action = AgentAction;
    type Observation = AgentObservation;

    #[tracing::instrument(skip_all, name = "env.reset")]
    fn reset(&mut self, _seed: Option<u64>, options: Option<Value>) -> Result<Self::Observation> {
        println!("[SolanaEnv] Resetting environment...");

        // Check for the RPC server to be ready.
        println!("[SolanaEnv] Checking for running `surfpool` validator...");
        for i in 0..10 {
            if self.rpc_client.get_health().is_ok() {
                break;
            }
            if i == 9 {
                anyhow::bail!("Could not connect to `surfpool` validator. Is it running at {LOCAL_SURFPOOL_RPC_URL}?");
            }
            thread::sleep(Duration::from_secs(1));
        }
        println!("[SolanaEnv] Validator is healthy.");

        self.keypair_map.clear();
        self.fee_payer = None; // Reset fee payer
        let initial_state_val = options
            .and_then(|v| v.get("initial_state").cloned())
            .context("Benchmark options must include 'initial_state'")?;
        let accounts: Vec<Value> = serde_json::from_value(initial_state_val)?;

        // First pass: create all keypairs so the key_map is complete from the start.
        for account_config in &accounts {
            let pubkey_placeholder = account_config["pubkey"]
                .as_str()
                .context("Missing 'pubkey' placeholder in account config")?;

            // Designate the user's main wallet as the default fee payer.
            if pubkey_placeholder == "USER_WALLET_PUBKEY" {
                self.fee_payer = Some(pubkey_placeholder.to_string());
            }

            let keypair = Keypair::new();
            self.keypair_map
                .insert(pubkey_placeholder.to_string(), keypair);
        }

        // Fund the fee payer first so it can pay for account creation.
        let fee_payer_keypair = self.get_fee_payer_keypair()?;
        println!(
            "[SolanaEnv] Funding fee payer ({}) with 10 SOL...",
            fee_payer_keypair.pubkey()
        );
        let sig = self
            .rpc_client
            .request_airdrop(&fee_payer_keypair.pubkey(), 10_000_000_000)?;
        self.rpc_client
            .confirm_transaction(&sig)
            .context("Failed to confirm fee payer airdrop")?;
        println!("[SolanaEnv] Fee payer funded.");

        // Process accounts in stages: system, then mints, then token accounts.
        let mut mint_configs = Vec::new();
        let mut token_configs = Vec::new();

        // Stage 1: Handle System Accounts (simple airdrops)
        for account_config in &accounts {
            let owner = account_config["owner"]
                .as_str()
                .context("Missing 'owner'")?;
            if owner == "11111111111111111111111111111111" {
                let placeholder = account_config["pubkey"].as_str().unwrap();
                // Don't re-fund the fee payer.
                if self.fee_payer.as_deref() == Some(placeholder) {
                    continue;
                }

                let keypair = self.keypair_map.get(placeholder).unwrap();
                let lamports = account_config["lamports"].as_u64().unwrap_or(0);
                if lamports > 0 {
                    println!(
                        "[SolanaEnv] Airdropping {lamports} lamports to {placeholder} ({})...",
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

        // Stage 2: Create SPL Mint Accounts
        for config in &mint_configs {
            let placeholder = config["pubkey"].as_str().unwrap();
            let keypair = self.keypair_map.get(placeholder).unwrap();
            println!(
                "[SolanaEnv] Creating SPL Mint: {placeholder} ({})",
                keypair.pubkey()
            );
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

        // Stage 3: Create SPL Token Accounts and mint initial supply
        for config in token_configs {
            let placeholder = config["pubkey"].as_str().unwrap();
            let keypair = self.keypair_map.get(placeholder).unwrap();
            println!(
                "[SolanaEnv] Creating SPL Token Account: {placeholder} ({})",
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
                // Find the config for the mint this token account belongs to.
                let mint_config = mint_configs
                    .iter()
                    .find(|mc| mc["pubkey"].as_str() == Some(mint_placeholder))
                    .context("Could not find mint config for token account")?;

                // Get the authority placeholder from that mint's config.
                let mint_authority_placeholder = mint_config["mint_data"]
                    .get("mint_authority")
                    .and_then(|v| v.as_str())
                    .unwrap_or("USER_WALLET_PUBKEY");

                // Get the authority's keypair.
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

            // Deduplicate signers
            signers.sort_by_key(|k| k.pubkey());
            signers.dedup_by_key(|k| k.pubkey());

            let transaction =
                Transaction::new_with_payer(&instructions, Some(&fee_payer_keypair.pubkey()));
            self.sign_and_send_transaction(transaction, &signers)?;
        }

        println!("[SolanaEnv] Environment reset complete.");
        self.get_observation("Success", None, vec!["Environment reset.".to_string()])
    }

    #[tracing::instrument(skip(self, action), name = "env.step")]
    fn step(
        &mut self,
        action: Self::Action,
        _ground_truth: &GroundTruth,
    ) -> Result<Step<Self::Observation>> {
        let instruction = action.0;
        println!(
            "[SolanaEnv] Executing instruction for program: {}",
            instruction.program_id
        );

        // The fee payer is always the primary signer and pays for the transaction.
        let fee_payer_keypair = self
            .fee_payer
            .as_ref()
            .and_then(|name| self.keypair_map.get(name))
            .context("Fee payer is not set or not found in the environment's keypair map.")?;

        // Start with the fee payer, as it must always sign.
        let mut signers: Vec<&Keypair> = vec![fee_payer_keypair];

        // Add any other keypairs required by the instruction.
        for signer_acc in instruction.accounts.iter().filter(|acc| acc.is_signer) {
            let signer_keypair = self
                .keypair_map
                .values()
                .find(|kp| kp.pubkey() == signer_acc.pubkey)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Instruction requires a signer ({}) that the environment does not control.",
                        signer_acc.pubkey
                    )
                })?;
            signers.push(signer_keypair);
        }

        // Deduplicate the signers list to handle cases where the fee payer is also an authority.
        signers.sort_by_key(|k| k.pubkey());
        signers.dedup_by_key(|k| k.pubkey());

        let mut transaction =
            Transaction::new_with_payer(&[instruction], Some(&fee_payer_keypair.pubkey()));

        let recent_blockhash = self.rpc_client.get_latest_blockhash()?;
        transaction.sign(&signers, recent_blockhash);

        let mut logs = Vec::new();
        let (status, error, terminated) =
            match self.rpc_client.send_and_confirm_transaction(&transaction) {
                Ok(sig) => {
                    let sig_str = sig.to_string();
                    logs.push(format!("Transaction confirmed: {sig_str}"));
                    ("Success", None, true)
                }
                Err(e) => {
                    let err_str = e.to_string();
                    ("Failure", Some(err_str), true)
                }
            };

        let observation = self.get_observation(status, error.clone(), logs)?;

        Ok(Step {
            reward: if error.is_none() { 1.0 } else { 0.0 },
            terminated,
            truncated: false,
            info: serde_json::json!({ "error": error }),
            observation,
        })
    }

    fn render(&self) {
        println!("\n--- Current On-Chain State ---");
        for (name, keypair) in &self.keypair_map {
            let pubkey = keypair.pubkey();
            match self.rpc_client.get_account(&pubkey) {
                Ok(account) => {
                    println!(
                        "  Pubkey: {} (Name: {})\n    Owner: {}\n    Lamports: {}",
                        pubkey, name, account.owner, account.lamports
                    );
                }
                Err(_) => {
                    println!("  Pubkey: {pubkey} (Name: {name})\n    Account not found on-chain.");
                }
            }
        }
        println!("--------------------------------\n");
    }

    fn close(&mut self) -> Result<()> {
        println!("[SolanaEnv] Environment closed. Validator process is left running.");
        Ok(())
    }
}
