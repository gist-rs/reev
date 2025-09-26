use reev_lib::actions::{sol_transfer, spl_transfer};
use solana_sdk::{
    bs58,
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use std::str::FromStr;
use tracing::info;

/// A helper function to pretty-print instruction details as JSON.
/// This is useful for debugging and creating test cases.
fn print_instruction_as_json(name: &str, ix: &Instruction) {
    // Serde JSON Value for easier construction
    let ix_json = serde_json::json!({
        "program_id": ix.program_id.to_string(),
        "accounts": ix.accounts.iter().map(|acc| {
            serde_json::json!({
                "pubkey": acc.pubkey.to_string(),
                "is_signer": acc.is_signer,
                "is_writable": acc.is_writable,
            })
        }).collect::<Vec<_>>(),
        // Encode binary data as Base58 for readability
        "data": bs58::encode(&ix.data).into_string(),
    });
    info!("\n--- {name} Instruction ---");
    info!("{}", serde_json::to_string_pretty(&ix_json).unwrap());
    info!("-------------------------\n");
}

#[test]
fn test_print_instructions() {
    // --- Setup ---
    // A dummy sender keypair.
    let sender = Keypair::new();
    // A known recipient public key.
    let recipient = Pubkey::from_str("CNdJxMoD8L8C6RxydLakcgEjQb5nUTsi1p3JyEKEmsZC").unwrap();
    // The public key for the USDC mint on Solana Mainnet.
    let usdc_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap();

    // --- 1. Native SOL Transfer ---
    let lamports = 1_000_000; // 0.001 SOL
    let sol_transfer_ix = sol_transfer::create_instruction(&sender.pubkey(), &recipient, lamports);

    info!("✅ Generated Native SOL Transfer Instruction:");
    print_instruction_as_json("Native SOL Transfer", &sol_transfer_ix);

    // --- 2. SPL Token Transfer ---
    // Derive the Associated Token Account (ATA) addresses for the sender and recipient.
    let sender_ata =
        spl_associated_token_account::get_associated_token_address(&sender.pubkey(), &usdc_mint);
    let recipient_ata =
        spl_associated_token_account::get_associated_token_address(&recipient, &usdc_mint);
    // 1 USDC, assuming the token has 6 decimal places.
    let amount = 1_000_000;

    let spl_transfer_ix =
        spl_transfer::create_instruction(&sender_ata, &recipient_ata, &sender.pubkey(), amount)
            .unwrap();

    info!("✅ Generated SPL Token Transfer Instruction:");
    print_instruction_as_json("SPL Token Transfer", &spl_transfer_ix);
}
