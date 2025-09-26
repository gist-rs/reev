use anyhow::Result;
use solana_program::pubkey::Pubkey;
use solana_sdk::instruction::Instruction;

/// Creates an SPL token transfer instruction.
///
/// This is a pure function that returns the instruction, which can then be
/// embedded in a transaction.
///
/// # Arguments
/// * `from_pubkey`: The source token account pubkey.
/// * `to_pubkey`: The destination token account pubkey.
/// * `authority_pubkey`: The pubkey of the account authorized to sign.
/// * `amount`: The amount of tokens to transfer.
///
/// # Returns
/// A `Result<Instruction>` for the transfer.
pub fn create_instruction(
    from_pubkey: &Pubkey,
    to_pubkey: &Pubkey,
    authority_pubkey: &Pubkey,
    amount: u64,
) -> Result<Instruction> {
    let ix = spl_token::instruction::transfer(
        &spl_token::id(),
        from_pubkey,
        to_pubkey,
        authority_pubkey,
        &[authority_pubkey], // The authority is the only signer required.
        amount,
    )?;
    Ok(ix)
}
