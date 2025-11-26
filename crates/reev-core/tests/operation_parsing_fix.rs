use reev_core::yml_generator::operation_parser::{Operation, OperationParser};

#[tokio::test]
async fn test_swap_operation_parsing() {
    let parser = OperationParser::new();

    // Test case 1: Direct swap operation
    let operations = parser.parse_operations("swap 0.1 SOL for USDC").unwrap();
    assert_eq!(operations.len(), 1);

    match &operations[0] {
        Operation::Swap { from, to, amount } => {
            assert_eq!(from, "SOL");
            assert_eq!(to, "USDC");
            assert_eq!(*amount, 0.1);
        }
        _ => panic!("Expected swap operation"),
    }

    // Test case 2: Swap operation with different tokens
    let operations = parser.parse_operations("swap 1 USDT for SOL").unwrap();
    assert_eq!(operations.len(), 1);

    match &operations[0] {
        Operation::Swap { from, to, amount } => {
            assert_eq!(from, "USDT");
            assert_eq!(to, "SOL");
            assert_eq!(*amount, 1.0);
        }
        _ => panic!("Expected swap operation"),
    }

    // Test case 3: Swap operation with "to" keyword
    let operations = parser.parse_operations("swap 2 SOL to USDC").unwrap();
    assert_eq!(operations.len(), 1);

    match &operations[0] {
        Operation::Swap { from, to, amount } => {
            assert_eq!(from, "SOL");
            assert_eq!(to, "USDC");
            assert_eq!(*amount, 2.0);
        }
        _ => panic!("Expected swap operation"),
    }

    // Test case 4: Verify transfer operations are correctly identified
    let operations = parser
        .parse_operations("transfer 0.5 SOL to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq")
        .unwrap();
    assert_eq!(operations.len(), 1);

    match &operations[0] {
        Operation::Transfer { mint, to, amount } => {
            assert_eq!(mint, "SOL");
            assert_eq!(to, "gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq");
            assert_eq!(*amount, 0.5);
        }
        _ => panic!("Expected transfer operation"),
    }

    // Test case 5: Verify that "send SOL for USDC" is correctly identified as a swap, not a transfer
    let operations = parser.parse_operations("send 1 SOL for USDC").unwrap();
    assert_eq!(operations.len(), 1);

    match &operations[0] {
        Operation::Swap { from, to, amount } => {
            assert_eq!(from, "SOL");
            assert_eq!(to, "USDC");
            assert_eq!(*amount, 1.0);
        }
        _ => panic!("Expected swap operation, not transfer"),
    }

    // Test case 6: Verify that "transfer SOL to USDC" is correctly identified as a swap, not a transfer
    let operations = parser.parse_operations("transfer 2 SOL to USDC").unwrap();
    assert_eq!(operations.len(), 1);

    match &operations[0] {
        Operation::Swap { from, to, amount } => {
            assert_eq!(from, "SOL");
            assert_eq!(to, "USDC");
            assert_eq!(*amount, 2.0);
        }
        _ => panic!("Expected swap operation, not transfer"),
    }
}
