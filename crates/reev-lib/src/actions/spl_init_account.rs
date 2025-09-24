use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use spl_associated_token_account::instruction as ata_instruction;
use spl_token;

/// Builds the transaction to create an Associated Token Account (ATA) and optionally mint an initial supply of tokens to it.
///
/// This is required to set up the on-chain state for benchmarks that use SPL tokens.
///
/// # Arguments
/// * `rpc_client`: The RPC client to query for the recent blockhash.
/// * `funder_keypair`: The keypair that will pay for the transaction fees. This is also assumed to be the Mint Authority for the mint.
/// * `owner_pubkey`: The public key of the wallet that will own the new ATA.
/// * `mint_pubkey`: The public key of the SPL Token Mint for the new account.
/// * `amount`: The initial amount of tokens to mint into the new account. If 0, only the account is created.
///
/// # Returns
/// A `Transaction` object ready to be sent.
pub fn build_transaction(
    rpc_client: &RpcClient,
    funder_keypair: &Keypair,
    owner_pubkey: &Pubkey,
    mint_pubkey: &Pubkey,
    amount: u64,
) -> Result<Transaction> {
    // 1. Derive the Associated Token Account (ATA) address for the given owner and mint.
    let ata_pubkey =
        spl_associated_token_account::get_associated_token_address(owner_pubkey, mint_pubkey);

    // 2. Create the instruction to create the ATA.
    // This instruction is idempotent, meaning it will succeed without error if the account already exists.
    let create_ata_ix = ata_instruction::create_associated_token_account(
        &funder_keypair.pubkey(),
        owner_pubkey,
        mint_pubkey,
        &spl_token::id(),
    );

    let mut instructions = vec![create_ata_ix];

    // 3. If a non-zero amount is specified, create and add the instruction to mint tokens to the new ATA.
    if amount > 0 {
        let mint_to_ix = spl_token::instruction::mint_to(
            &spl_token::id(),
            mint_pubkey,
            &ata_pubkey,
            &funder_keypair.pubkey(), // The funder is assumed to be the mint authority.
            &[],                      // No multisig signers.
            amount,
        )?;
        instructions.push(mint_to_ix);
    }

    // 4. Build and sign the transaction.
    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    let transaction = Transaction::new_signed_with_payer(
        &instructions,
        Some(&funder_keypair.pubkey()),
        // Only the funder/mint authority needs to sign. The ATA is derived, not a keypair.
        &[funder_keypair],
        recent_blockhash,
    );

    Ok(transaction)
}
