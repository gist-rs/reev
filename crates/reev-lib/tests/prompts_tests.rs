//! Tests for prompts module

use reev_lib::prompts::PromptManager;
use std::path::Path;

#[test]
fn test_prompt_manager_creation() {
    let manager = PromptManager::new("./templates".to_string());
    assert_eq!(manager.templates_dir(), "./templates");
}

#[test]
fn test_template_path_construction() {
    let manager = PromptManager::new("./templates".to_string());
    let template_path = Path::new(&manager.templates_dir()).join("test.yml");
    assert_eq!(template_path.to_str().unwrap(), "./templates/test.yml");
}
