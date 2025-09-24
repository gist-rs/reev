use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
use spl_token;

/// Builds the transaction to create and initialize a new SPL Token Mint.
///
/// This is required to set up the on-chain state for benchmarks that use SPL tokens.
///
/// # Arguments
/// * `rpc_client`: The RPC client to query for rent exemption.
/// * `funder_keypair`: The keypair that will pay for the transaction fees and rent.
/// * `mint_keypair`: The keypair for the new mint account.
/// * `mint_authority_pubkey`: The pubkey that will have authority over the new mint.
/// * `decimals`: The number of decimal places for the token.
///
/// # Returns
/// A `Transaction` object ready to be sent.
pub fn build_transaction(
    rpc_client: &RpcClient,
    funder_keypair: &Keypair,
    mint_keypair: &Keypair,
    mint_authority_pubkey: &Pubkey,
    decimals: u8,
) -> Result<Transaction> {
    // 1. Get the minimum balance required for a mint account to be rent-exempt.
    let rent_lamports =
        rpc_client.get_minimum_balance_for_rent_exemption(spl_token::state::Mint::LEN)?;

    // 2. Create the instruction to create a new account with the required space for a mint.
    let create_account_ix = system_instruction::create_account(
        &funder_keypair.pubkey(),
        &mint_keypair.pubkey(),
        rent_lamports,
        spl_token::state::Mint::LEN as u64,
        &spl_token::id(), // The owner of the new account must be the SPL Token Program
    );

    // 3. Create the instruction to initialize the new account as a mint.
    let initialize_mint_ix = spl_token::instruction::initialize_mint(
        &spl_token::id(),
        &mint_keypair.pubkey(),
        mint_authority_pubkey,
        None, // No freeze authority
        decimals,
    )?;

    // 4. Build the transaction.
    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    let transaction = Transaction::new_signed_with_payer(
        &[create_account_ix, initialize_mint_ix],
        Some(&funder_keypair.pubkey()),
        // The funder signs for the creation, the mint keypair signs because it's a new account being created.
        &[funder_keypair, mint_keypair],
        recent_blockhash,
    );

    Ok(transaction)
}
