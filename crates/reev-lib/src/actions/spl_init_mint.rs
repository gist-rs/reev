use anyhow::Result;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey, system_instruction};
use spl_token;

/// Creates the instructions to create and initialize a new SPL Token Mint.
///
/// This function is pure and returns a list of instructions. The responsibility
/// of fetching rent, building, and signing the transaction lies with the caller
/// (e.g., the `SolanaEnv`).
///
/// # Arguments
/// * `funder_pubkey`: The public key of the account that will pay for the transaction fees and rent.
/// * `mint_pubkey`: The public key for the new mint account.
/// * `mint_authority_pubkey`: The pubkey that will have authority over the new mint.
/// * `rent_lamports`: The rent-exempt lamports required for the mint account. This must be fetched by the caller.
/// * `decimals`: The number of decimal places for the token.
///
/// # Returns
/// A `Vec<Instruction>` containing the necessary instructions.
pub fn create_instructions(
    funder_pubkey: &Pubkey,
    mint_pubkey: &Pubkey,
    mint_authority_pubkey: &Pubkey,
    rent_lamports: u64,
    decimals: u8,
) -> Result<Vec<Instruction>> {
    // 1. Create the instruction to create a new account with the required space for a mint.
    let create_account_ix = system_instruction::create_account(
        funder_pubkey,
        mint_pubkey,
        rent_lamports,
        spl_token::state::Mint::LEN as u64,
        &spl_token::id(), // The owner of the new account must be the SPL Token Program
    );

    // 2. Create the instruction to initialize the new account as a mint.
    let initialize_mint_ix = spl_token::instruction::initialize_mint(
        &spl_token::id(),
        mint_pubkey,
        mint_authority_pubkey,
        None, // No freeze authority
        decimals,
    )?;

    Ok(vec![create_account_ix, initialize_mint_ix])
}
