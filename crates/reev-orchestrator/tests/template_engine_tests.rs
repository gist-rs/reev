//! Tests for template engine module

use reev_orchestrator::templates::engine::TemplateEngine;
use reev_orchestrator::templates::{TemplateMetadata, TemplateType};
use reev_types::flow::WalletContext;
use serde_json::json;
use tempfile::TempDir;

#[tokio::test]
async fn test_template_engine_creation() {
    let temp_dir = TempDir::new().unwrap();
    let engine = TemplateEngine::new(temp_dir.path());
    assert!(engine.is_ok());
}

#[tokio::test]
async fn test_template_registration() {
    let temp_dir = TempDir::new().unwrap();

    // Create a simple template file
    let template_path = temp_dir.path().join("test.hbs");
    tokio::fs::write(&template_path, "Hello {{name}}!")
        .await
        .unwrap();

    let metadata = TemplateMetadata::new(
        "test".to_string(),
        TemplateType::Base,
        "Test template".to_string(),
        vec!["name".to_string()],
        vec![],
    );

    let engine = TemplateEngine::new(temp_dir.path()).unwrap();
    let registration = engine
        .register_template_file(&template_path, metadata)
        .await
        .unwrap();

    assert_eq!(registration.name, "test");
}

#[tokio::test]
async fn test_template_rendering() {
    let temp_dir = TempDir::new().unwrap();

    // Create and register a simple template
    let template_path = temp_dir.path().join("test.hbs");
    tokio::fs::write(&template_path, "Amount: {{amount}}")
        .await
        .unwrap();

    let metadata = TemplateMetadata::new(
        "test".to_string(),
        TemplateType::Base,
        "Test template".to_string(),
        vec!["amount".to_string()],
        vec![],
    );

    let engine = TemplateEngine::new(temp_dir.path()).unwrap();
    engine
        .register_template_file(&template_path, metadata)
        .await
        .unwrap();

    // Render template
    let mut variables = std::collections::HashMap::new();
    variables.insert("amount".to_string(), json!(100));

    let context = WalletContext::new("test".to_string());
    let result = engine
        .render_template("test", &context, &variables)
        .await
        .unwrap();

    assert_eq!(result.rendered, "Amount: 100");
    assert_eq!(result.template_name, "test");
}
