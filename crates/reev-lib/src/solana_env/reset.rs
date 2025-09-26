use crate::{
    agent::AgentObservation,
    solana_env::{observation, SolanaEnv},
};
use anyhow::{Context, Result};
use serde_json::Value;
use solana_program::program_pack::Pack;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use solana_system_interface::instruction as system_instruction;
use spl_associated_token_account::{get_associated_token_address, instruction as ata_instruction};
use spl_token::{
    instruction as spl_instruction, native_mint,
    state::{Account as SplTokenAccount, Mint},
};
use std::{collections::HashMap, str::FromStr, thread, time::Duration};
use tracing::{info, instrument};

#[instrument(skip_all, name = "env.reset")]
pub(crate) fn handle_reset(
    env: &mut SolanaEnv,
    options: Option<Value>,
) -> Result<AgentObservation> {
    info!("Resetting Solana environment...");
    info!("Checking for running `surfpool` validator...");
    for i in 0..10 {
        if env.rpc_client.get_health().is_ok() {
            break;
        }
        if i == 9 {
            anyhow::bail!(
                "Could not connect to `surfpool` validator at {}",
                "http://127.0.0.1:8899"
            );
        }
        thread::sleep(Duration::from_secs(1));
    }
    info!("Validator is healthy.");

    env.keypair_map.clear();
    env.fee_payer = None;

    let options = options.context("Benchmark options are required")?;
    let initial_state_val = options
        .get("initial_state")
        .cloned()
        .context("Benchmark options must include 'initial_state'")?;
    let benchmark_id = options
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or_default();

    let accounts: Vec<Value> = serde_json::from_value(initial_state_val)?;

    for account_config in &accounts {
        let pubkey_placeholder = account_config["pubkey"]
            .as_str()
            .context("Missing 'pubkey' placeholder in account config")?;

        if pubkey_placeholder == "USER_WALLET_PUBKEY" {
            env.fee_payer = Some(pubkey_placeholder.to_string());
        }

        env.keypair_map
            .insert(pubkey_placeholder.to_string(), Keypair::new());
    }

    let fee_payer_placeholder = env.fee_payer.as_ref().context("Fee payer not set")?;
    let fee_payer_config = accounts
        .iter()
        .find(|acc| acc["pubkey"].as_str() == Some(fee_payer_placeholder))
        .context("Fee payer config not found in initial state")?;

    let fee_payer_keypair = env.get_fee_payer_keypair()?;
    let initial_lamports = fee_payer_config["lamports"]
        .as_u64()
        .context("Fee payer 'lamports' not found or invalid in initial state")?;

    if initial_lamports > 0 {
        info!(
            "Funding fee payer ({}) with {} lamports...",
            fee_payer_keypair.pubkey(),
            initial_lamports
        );
        let sig = env
            .rpc_client
            .request_airdrop(&fee_payer_keypair.pubkey(), initial_lamports)?;
        env.rpc_client
            .confirm_transaction(&sig)
            .context("Failed to confirm fee payer airdrop")?;
        info!("Fee payer funded.");
    }

    // --- SPECIAL LOGIC FOR JUPITER SWAP ---
    // Pre-create and fund all necessary accounts to satisfy the mainnet transaction.
    if benchmark_id.contains("JUP-SWAP") {
        info!("[JUP-SWAP] Pre-creating and funding required ATAs for test...");
        let user_pubkey = fee_payer_keypair.pubkey();
        let mainnet_usdc_mint =
            Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap();
        // Fund with 1 SOL to provide a large buffer for the 0.1 SOL swap and fees.
        let wsol_funding_amount = 1_000_000_000;
        let wsol_ata = get_associated_token_address(&user_pubkey, &native_mint::ID);

        let setup_instructions = vec![
            // 1. Create the user's ATA for WSOL (idempotent)
            ata_instruction::create_associated_token_account_idempotent(
                &user_pubkey,
                &user_pubkey,
                &native_mint::ID,
                &spl_token::id(),
            ),
            // 2. Create the user's ATA for mainnet USDC (idempotent)
            ata_instruction::create_associated_token_account_idempotent(
                &user_pubkey,
                &user_pubkey,
                &mainnet_usdc_mint,
                &spl_token::id(),
            ),
            // 3. Fund the WSOL ATA by transferring native SOL to it.
            system_instruction::transfer(&user_pubkey, &wsol_ata, wsol_funding_amount),
            // 4. Sync the native mint to update the WSOL ATA's balance.
            spl_token::instruction::sync_native(&spl_token::id(), &wsol_ata)?,
        ];

        let transaction = Transaction::new_with_payer(&setup_instructions, Some(&user_pubkey));
        env.sign_and_send_transaction(transaction, &[fee_payer_keypair])
            .context("Failed to pre-create/fund ATAs for Jupiter swap test")?;
        info!("[JUP-SWAP] Setup transaction sent successfully.");

        // --- VERIFICATION AND ASSERTION ---
        info!("[JUP-SWAP] Verifying on-chain state after setup...");
        let wsol_balance = env.rpc_client.get_token_account_balance(&wsol_ata)?;
        let user_sol_balance = env.rpc_client.get_balance(&user_pubkey)?;
        info!("[JUP-SWAP] User SOL balance: {}", user_sol_balance);
        info!(
            "[JUP-SWAP] User WSOL balance: {}",
            wsol_balance.ui_amount_string
        );

        assert_eq!(
            wsol_balance.amount.parse::<u64>()?,
            wsol_funding_amount,
            "WSOL ATA was not funded correctly!"
        );
        info!("[JUP-SWAP] On-chain state verified. WSOL account is correctly funded.");
    }
    // --- END SPECIAL LOGIC ---

    let mut mint_configs = Vec::new();
    let mut token_configs = Vec::new();

    for account_config in &accounts {
        if let Some(owner) = account_config["owner"].as_str() {
            if owner == "11111111111111111111111111111111" {
                let placeholder = account_config["pubkey"].as_str().unwrap();
                if env.fee_payer.as_deref() == Some(placeholder) {
                    continue;
                }

                let keypair = env.keypair_map.get(placeholder).unwrap();
                let lamports = account_config["lamports"].as_u64().unwrap_or(0);
                if lamports > 0 {
                    info!(
                        "Airdropping {lamports} lamports to {placeholder} ({})...",
                        keypair.pubkey()
                    );
                    let sig = env
                        .rpc_client
                        .request_airdrop(&keypair.pubkey(), lamports)?;
                    env.rpc_client
                        .confirm_transaction(&sig)
                        .context("Failed to confirm airdrop")?;
                }
            } else if owner == spl_token::id().to_string() {
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
        let keypair = env.keypair_map.get(placeholder).unwrap();
        info!("Creating SPL Mint: {placeholder} ({})", keypair.pubkey());
        let mint_data = config.get("mint_data").unwrap();
        let decimals = mint_data["decimals"].as_u64().unwrap() as u8;
        let auth_placeholder = mint_data["mint_authority"]
            .as_str()
            .unwrap_or("USER_WALLET_PUBKEY");
        let authority = env
            .keypair_map
            .get(auth_placeholder)
            .context("Mint authority not found")?;

        let rent = env
            .rpc_client
            .get_minimum_balance_for_rent_exemption(Mint::LEN)?;
        let instructions = [
            system_instruction::create_account(
                &fee_payer_keypair.pubkey(),
                &keypair.pubkey(),
                rent,
                Mint::LEN as u64,
                &spl_token::id(),
            ),
            spl_instruction::initialize_mint(
                &spl_token::id(),
                &keypair.pubkey(),
                &authority.pubkey(),
                None,
                decimals,
            )?,
        ];
        let transaction =
            Transaction::new_with_payer(&instructions, Some(&fee_payer_keypair.pubkey()));
        env.sign_and_send_transaction(transaction, &[fee_payer_keypair, keypair])?;
    }

    for config in token_configs {
        let placeholder = config["pubkey"].as_str().unwrap();
        let keypair = env.keypair_map.get(placeholder).unwrap();
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

        let mint_pubkey = env
            .keypair_map
            .get(mint_placeholder)
            .context("Mint keypair not found")?
            .pubkey();
        let owner_pubkey = env
            .keypair_map
            .get(owner_placeholder)
            .context("Owner keypair not found")?
            .pubkey();

        let rent = env
            .rpc_client
            .get_minimum_balance_for_rent_exemption(SplTokenAccount::LEN)?;
        let mut instructions = vec![
            system_instruction::create_account(
                &fee_payer_keypair.pubkey(),
                &keypair.pubkey(),
                rent,
                SplTokenAccount::LEN as u64,
                &spl_token::id(),
            ),
            spl_instruction::initialize_account(
                &spl_token::id(),
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

            let mint_authority = env
                .keypair_map
                .get(mint_authority_placeholder)
                .context("Mint authority keypair not found")?;

            signers.push(mint_authority);
            instructions.push(spl_instruction::mint_to(
                &spl_token::id(),
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
        env.sign_and_send_transaction(transaction, &signers)?;
    }

    info!("Environment reset complete.");
    observation::get_observation(env, "Success", None, vec!["Environment reset.".to_string()])
}
