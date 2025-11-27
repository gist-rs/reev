//! Tests for template metadata module

use reev_orchestrator::templates::{TemplateMetadata, TemplateType};
use reev_types::flow::WalletContext;
use serde_json::json;

#[test]
fn test_template_metadata_validation() {
    let metadata = TemplateMetadata::new(
        "test_swap".to_string(),
        TemplateType::Base,
        "Test swap template".to_string(),
        vec!["amount".to_string(), "from_token".to_string()],
        vec!["slippage".to_string()],
    );

    let context = WalletContext::new("test".to_string());
    let mut variables = std::collections::HashMap::new();
    variables.insert("amount".to_string(), json!(100));
    variables.insert("from_token".to_string(), json!("SOL"));

    assert!(metadata.validate_variables(&context, &variables).is_ok());

    // Test missing required variable
    variables.remove("from_token");
    assert!(metadata.validate_variables(&context, &variables).is_err());
}
