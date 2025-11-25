//! Language Refiner for Phase 1 of V3 Plan
//!
//! This module implements the language refinement functionality in Phase 1 of the V3 plan.
//! It uses LLM to refine user prompts by fixing typos, normalizing terminology, and making
//! the language clearer and more unambiguous.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, instrument, warn};

/// Language refiner for refining user prompts
pub struct LanguageRefiner {
    /// API key for LLM service
    api_key: Option<String>,
    /// Model name for LLM
    model_name: String,
}

impl Default for LanguageRefiner {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageRefiner {
    /// Create a new language refiner
    pub fn new() -> Self {
        let model_name =
            std::env::var("GLM_MODEL").unwrap_or_else(|_| "glm-4.6-coding".to_string());
        let api_key = std::env::var("ZAI_API_KEY").ok();

        Self {
            api_key,
            model_name,
        }
    }

    /// Create a language refiner with custom configuration
    pub fn with_config(api_key: Option<String>, model_name: String) -> Self {
        Self {
            api_key,
            model_name,
        }
    }

    /// Refine a user prompt using LLM
    #[instrument(skip(self))]
    pub async fn refine_prompt(&self, prompt: &str) -> Result<RefinedPrompt> {
        info!("Refining prompt: {}", prompt);

        // If no API key is configured, use a simple rule-based refiner
        if self.api_key.is_none() {
            warn!("No API key configured, using rule-based refiner");
            return Ok(self.rule_based_refine(prompt));
        }

        // Build LLM request for language refinement
        let request = LanguageRefineRequest {
            prompt: prompt.to_string(),
        };

        // Send request to LLM
        let response = self.send_refine_request(&request).await;

        // Handle LLM request failure
        let response = match response {
            Ok(response) => response,
            Err(e) => {
                warn!("LLM request failed: {}", e);
                warn!("Falling back to rule-based refiner");
                return Ok(self.rule_based_refine(prompt));
            }
        };

        // Parse response
        let refined = match serde_json::from_str::<LanguageRefineResponse>(&response) {
            Ok(response) => response,
            Err(e) => {
                warn!("Failed to parse LLM response: {}", e);
                warn!("Falling back to rule-based refiner");
                return Ok(self.rule_based_refine(prompt));
            }
        };

        info!("Refined prompt: {}", refined.refined_prompt);
        debug!(
            "Original: {} -> Refined: {}",
            prompt, refined.refined_prompt
        );

        Ok(RefinedPrompt {
            original: prompt.to_string(),
            refined: refined.refined_prompt,
            changes_detected: refined.changes_detected,
            confidence: refined.confidence,
        })
    }

    /// Send request to LLM for language refinement
    async fn send_refine_request(&self, request: &LanguageRefineRequest) -> Result<String> {
        let client = reqwest::Client::new();
        let url = "https://api.openai.com/v1/chat/completions"; // Placeholder URL

        // Build system prompt for language refinement
        let system_prompt = r#"
You are a language refinement assistant for a DeFi application. Your task is to refine user prompts by:

1. Fixing typos and grammatical errors
2. Normalizing cryptocurrency terminology (e.g., "usd coin" -> "USDC", "solana" -> "SOL")
3. Making the language clearer and more unambiguous
4. Preserving the original intent and meaning
5. Keeping the refined prompt concise and direct

Do NOT:
- Extract intent or determine tools
- Add information not present in the original prompt
- Change the meaning of the request
- Add explanations or additional context

Respond with a JSON object with the following fields:
- refined_prompt: The refined prompt
- changes_detected: Boolean indicating if changes were made
- confidence: Float from 0.0 to 1.0 indicating confidence in the refinement
"#;

        let body = serde_json::json!({
            "model": self.model_name,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": format!("Refine this prompt: {}", request.prompt)}
            ],
            "temperature": 0.1,
            "max_tokens": 200,
            "response_format": {"type": "json_object"}
        });

        let response = client
            .post(url)
            .header("Content-Type", "application/json")
            .header(
                "Authorization",
                format!("Bearer {}", self.api_key.as_ref().unwrap()),
            )
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send request to LLM: {e}"))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "LLM request failed with status: {}",
                response.status()
            ));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| anyhow!("Failed to read LLM response: {e}"))?;

        // Extract the content from the response
        let response_json: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| anyhow!("Failed to parse JSON: {e}"))?;

        if let Some(content) = response_json
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
        {
            Ok(content.to_string())
        } else {
            Err(anyhow!("Invalid LLM response format"))
        }
    }

    /// Rule-based language refinement as fallback
    fn rule_based_refine(&self, prompt: &str) -> RefinedPrompt {
        let mut refined = prompt.to_string();
        let mut changes_detected = false;

        // Common typo corrections
        let corrections: HashMap<&str, &str> = [
            ("sendd", "send"),
            ("tranfer", "transfer"),
            ("trasnfer", "transfer"),
            ("solana", "SOL"),
            ("sol", "SOL"),
            ("eth", "ETH"),
            ("ethereum", "ETH"),
            ("usd coin", "USDC"),
            ("usdc", "USDC"),
            ("usdt", "USDT"),
            ("tether", "USDT"),
        ]
        .iter()
        .cloned()
        .collect();

        // Apply corrections
        for (wrong, correct) in &corrections {
            if refined.contains(wrong) {
                refined = refined.replace(wrong, correct);
                changes_detected = true;
            }
        }

        // Normalize spacing
        let original = refined.clone();
        refined = refined.split_whitespace().collect::<Vec<&str>>().join(" ");
        if original != refined {
            changes_detected = true;
        }

        // Determine confidence based on changes
        let confidence = if changes_detected {
            0.8 // High confidence if changes were made
        } else {
            0.9 // Even higher confidence if no changes were needed
        };

        RefinedPrompt {
            original: prompt.to_string(),
            refined,
            changes_detected,
            confidence,
        }
    }
}

/// Result of language refinement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefinedPrompt {
    /// Original prompt
    pub original: String,
    /// Refined prompt
    pub refined: String,
    /// Whether changes were detected
    pub changes_detected: bool,
    /// Confidence in the refinement (0.0-1.0)
    pub confidence: f32,
}

/// Request for language refinement
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LanguageRefineRequest {
    /// Original prompt to refine
    prompt: String,
}

/// Response from language refinement
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LanguageRefineResponse {
    /// Refined prompt
    refined_prompt: String,
    /// Whether changes were detected
    changes_detected: bool,
    /// Confidence in the refinement
    confidence: f32,
}
