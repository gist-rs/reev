use anyhow::Result;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};
use spl_associated_token_account::instruction as ata_instruction;
use spl_token;

/// Creates the instructions to create an Associated Token Account (ATA) and optionally mint an initial supply of tokens to it.
///
/// This function is pure and returns a list of instructions. The responsibility
/// of building and signing the transaction lies with the caller (e.g., the `SolanaEnv`).
///
/// # Arguments
/// * `funder_pubkey`: The public key of the account that will pay for the transaction fees. This is also assumed to be the Mint Authority for the mint.
/// * `owner_pubkey`: The public key of the wallet that will own the new ATA.
/// * `mint_pubkey`: The public key of the SPL Token Mint for the new account.
/// * `amount`: The initial amount of tokens to mint into the new account. If 0, only the account is created.
///
/// # Returns
/// A `Vec<Instruction>` containing the necessary instructions.
pub fn create_instructions(
    funder_pubkey: &Pubkey,
    owner_pubkey: &Pubkey,
    mint_pubkey: &Pubkey,
    amount: u64,
) -> Result<Vec<Instruction>> {
    // 1. Derive the Associated Token Account (ATA) address for the given owner and mint.
    let ata_pubkey =
        spl_associated_token_account::get_associated_token_address(owner_pubkey, mint_pubkey);

    // 2. Create the instruction to create the ATA.
    // This instruction is idempotent, meaning it will succeed without error if the account already exists.
    let create_ata_ix = ata_instruction::create_associated_token_account(
        funder_pubkey,
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
            funder_pubkey, // The funder is assumed to be the mint authority.
            &[],           // No multisig signers.
            amount,
        )?;
        instructions.push(mint_to_ix);
    }

    Ok(instructions)
}
