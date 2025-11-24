//! Solana transaction utilities for reev-core

use crate::agent::RawInstruction;
use anyhow::{anyhow, Result};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Signer,
    transaction::Transaction,
};
use std::str::FromStr;
use tracing::{debug, info, instrument};

/// Build a Solana transaction from raw instructions
pub fn build_transaction_from_instructions(
    instructions: Vec<RawInstruction>,
    payer: Pubkey,
) -> Result<solana_sdk::transaction::Transaction> {
    // Convert RawInstruction to Solana SDK Instruction
    let mut sol_instructions = Vec::with_capacity(instructions.len());

    for raw_inst in instructions {
        // Parse program ID
        let program_id = Pubkey::from_str(&raw_inst.program_id)
            .map_err(|e| anyhow!("Invalid program ID: {}: {}", raw_inst.program_id, e))?;

        // Convert account metas
        let mut accounts = Vec::with_capacity(raw_inst.accounts.len());
        for acc in raw_inst.accounts {
            let pubkey = Pubkey::from_str(&acc.pubkey)
                .map_err(|e| anyhow!("Invalid account pubkey: {}: {}", acc.pubkey, e))?;

            accounts.push(solana_sdk::instruction::AccountMeta {
                pubkey,
                is_signer: acc.is_signer,
                is_writable: acc.is_writable,
            });
        }

        // Decode instruction data
        let data = bs58::decode(&raw_inst.data)
            .into_vec()
            .map_err(|e| anyhow!("Failed to decode instruction data: {e}"))?;

        // Create instruction
        let instruction = solana_sdk::instruction::Instruction {
            program_id,
            accounts,
            data,
        };

        sol_instructions.push(instruction);
    }

    // Get recent blockhash from client
    let client = RpcClient::new_with_commitment(
        "http://localhost:8899".to_string(), // Default SURFPOOL URL
        CommitmentConfig::confirmed(),
    );

    // Create transaction
    let transaction = Transaction::new_with_payer(&sol_instructions, Some(&payer));

    Ok(transaction)
}

/// Sign a transaction with the user's keypair
pub fn sign_transaction(mut transaction: Transaction, signer: &impl Signer) -> Result<Transaction> {
    // Get the latest blockhash for signing
    let client = RpcClient::new_with_commitment(
        "http://localhost:8899".to_string(), // Default SURFPOOL URL
        CommitmentConfig::confirmed(),
    );

    let blockhash = tokio::runtime::Handle::current().block_on(client.get_latest_blockhash())?;

    // Sign the transaction with the blockhash
    transaction.try_sign(&[signer], blockhash)?;
    Ok(transaction)
}

/// Send a signed transaction to SURFPOOL
#[instrument(skip(transaction))]
pub async fn send_transaction_to_surfpool(transaction: Transaction) -> Result<String> {
    info!("Sending transaction to SURFPOOL");

    let client = RpcClient::new_with_commitment(
        "http://localhost:8899".to_string(), // Default SURFPOOL URL
        CommitmentConfig::confirmed(),
    );

    let signature = client.send_transaction(&transaction).await?;

    info!(
        "Transaction sent successfully with signature: {}",
        signature
    );
    Ok(signature.to_string())
}

/// Complete transaction flow: build, sign, and send
#[instrument(skip(instructions, signer))]
pub async fn execute_transaction(
    instructions: Vec<RawInstruction>,
    payer: Pubkey,
    signer: &impl Signer,
) -> Result<String> {
    debug!(
        "Building transaction from {} instructions",
        instructions.len()
    );
    let transaction = build_transaction_from_instructions(instructions, payer)?;

    debug!("Signing transaction");
    let signed_tx = sign_transaction(transaction, signer)?;

    debug!("Sending transaction to SURFPOOL");
    let signature = send_transaction_to_surfpool(signed_tx).await?;

    Ok(signature)
}
