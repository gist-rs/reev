//! Extract User Prompt From YAML Module
//!
//! This module provides utility function for extracting user prompts from YAML log content.

/// Extract user prompt from YAML log content
pub fn extract_user_prompt_from_yaml(log_content: &str) -> Option<String> {
    // Look for user_prompt pattern in text
    for line in log_content.lines() {
        if line.trim().starts_with("user_prompt:") {
            let prompt_start = line.find(':').unwrap_or(0) + 1;
            let prompt = line[prompt_start..].trim().trim_matches('"');
            return Some(prompt.to_string());
        }
    }
    None
}
