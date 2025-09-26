use solana_sdk::{instruction::Instruction, pubkey::Pubkey};
use solana_system_interface::instruction as system_instruction;

/// Creates a native SOL transfer instruction.
///
/// This is a pure function that returns the instruction, which can then be
/// embedded in a transaction.
///
/// # Arguments
/// * `from_pubkey`: The public key of the account that will send the SOL.
/// * `to_pubkey`: The public key of the account that will receive the SOL.
/// * `lamports`: The amount of lamports to transfer.
///
/// # Returns
/// An `Instruction` object for the transfer.
pub fn create_instruction(from_pubkey: &Pubkey, to_pubkey: &Pubkey, lamports: u64) -> Instruction {
    system_instruction::transfer(from_pubkey, to_pubkey, lamports)
}
