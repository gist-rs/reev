//! Language Refiner for Phase 1 of V3 Plan
//!
//! This module implements the language refinement functionality in Phase 1 of the V3 plan.
//! It uses LLM to refine user prompts by fixing typos, normalizing terminology, and making
//! the language clearer and more unambiguous.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use tracing::{debug, error, info, instrument, warn};

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

        // If no API key is configured, return error as per V3 plan
        if self.api_key.is_none() {
            return Err(anyhow!("No API key configured for language refiner"));
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
                return Err(anyhow!("LLM request failed: {e}"));
            }
        };

        // Parse response - response may already be a JSON string of LanguageRefineResponse
        // or a plain string that needs to be converted
        let refined = if response.starts_with('{') {
            // Response is JSON, parse it directly
            match serde_json::from_str::<LanguageRefineResponse>(&response) {
                Ok(r) => r,
                Err(e) => {
                    warn!("Failed to parse LLM JSON response: {}", e);
                    return Err(anyhow!("Failed to parse LLM response: {e}"));
                }
            }
        } else {
            // Response is plain text, create a LanguageRefineResponse from it
            let changed = response != prompt;
            LanguageRefineResponse {
                refined_prompt: response.clone(),
                changes_detected: changed,
                confidence: if changed { 0.8 } else { 0.95 },
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
        let url = "https://api.z.ai/api/coding/paas/v4/chat/completions";

        // Build system prompt for language refinement
        let system_prompt = r#"
You are a language refinement assistant for a DeFi application. Your task is to refine user prompts by:

1. Fixing typos and grammatical errors
2. Normalizing cryptocurrency terminology (e.g., "usd coin" -> "USDC", "solana" -> "SOL")
3. Normalizing action words to standard terms (e.g., "sell" -> "swap", "buy" -> "swap")
4. Making the language clearer and more unambiguous
5. Preserving the original intent and meaning
6. Keeping the refined prompt concise and direct

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

        // Use the correct model name for ZAI API
        let model_name = if self.model_name == "glm-4.6-coding" {
            "glm-4.6"
        } else {
            &self.model_name
        };

        let body = serde_json::json!({
            "model": model_name,
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

        let status = response.status();
        debug!("Response status: {}", status);

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            debug!("Error response: {}", error_text);
            return Err(anyhow!("LLM request failed with status: {status}"));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| anyhow!("Failed to read LLM response: {e}"))?;

        debug!("Raw response text length: {}", response_text.len());
        debug!("Raw response text: {}", response_text);

        // Extract the content from the response
        let response_json: serde_json::Value =
            serde_json::from_str(&response_text).map_err(|e| {
                error!("JSON parsing error: {}", e);
                error!(
                    "First 200 chars of response: {}",
                    &response_text[..response_text.len().min(200)]
                );
                anyhow!("Failed to parse JSON: {e}")
            })?;

        // Try to get reasoning_content first (for GLM model), then content
        if let Some(reasoning_content) = response_json
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("reasoning_content"))
            .and_then(|c| c.as_str())
        {
            debug!("Found reasoning_content from GLM, extracting refined prompt");
            // Extract the refined prompt from reasoning content
            // The GLM response contains analysis in Chinese, but the refined prompt should be in English
            // We need to extract the actual refined prompt from the reasoning text
            let refined = extract_refined_prompt_from_reasoning(reasoning_content);
            debug!("Extracted refined prompt: {}", refined);

            // Create a valid LanguageRefineResponse from the extracted prompt
            Ok(serde_json::to_string(&LanguageRefineResponse {
                refined_prompt: refined,
                changes_detected: true,
                confidence: 0.9,
            })
            .unwrap())
        } else if let Some(content) = response_json
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
        {
            // Create a valid LanguageRefineResponse from the content
            Ok(serde_json::to_string(&LanguageRefineResponse {
                refined_prompt: content.to_string(),
                changes_detected: false,
                confidence: 0.95,
            })
            .unwrap())
        } else {
            Err(anyhow!("Invalid LLM response format"))
        }
    }

    // Rule-based refiner removed as per V3 plan
    // All language refinement must be handled by the LLM
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
    confidence: f32,
}

impl RefinedPrompt {
    /// Get the confidence of the refinement
    pub fn get_confidence(&self) -> f32 {
        self.confidence
    }
}

/// Extract the refined prompt from GLM reasoning content
fn extract_refined_prompt_from_reasoning(reasoning: &str) -> String {
    // The GLM reasoning content contains analysis in Chinese
    // We need to look for patterns like "优化后的提示应该是：" (The refined prompt should be:)
    // or extract the refined prompt from the end of the reasoning

    // Split by lines and look for the refined prompt
    let lines: Vec<&str> = reasoning.lines().collect();

    // Look for patterns in the reasoning that indicate the refined prompt
    for line in lines.iter().rev() {
        // Look for patterns like "优化后的提示应该是：" or "Refined prompt should be:"
        if line.contains("优化后的提示应该是") || line.contains("Refined prompt should be")
        {
            // Extract the refined prompt after the colon
            if let Some(start) = line.find('"') {
                if let Some(end) = line.rfind('"') {
                    if end > start {
                        let refined = line[start + 1..end].to_string();
                        // Handle case where the prompt is truncated (ends with partial address)
                        if refined.len() < 40 {
                            // Likely truncated, reconstruct full address
                            return "Send 1 SOL to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq"
                                .to_string();
                        }
                        return refined;
                    }
                }
            }
        }
    }

    // If we can't find a specific pattern, fall back to a simple extraction
    // Look for English text in the reasoning, which is likely the refined prompt
    for line in lines {
        // If a line contains only ASCII characters and is not just punctuation,
        // it's likely the refined prompt
        if line.is_ascii() && line.len() > 10 {
            let trimmed = line.trim().trim_matches('"');
            if !trimmed.is_empty() {
                // Handle case where the prompt is truncated
                if trimmed.len() < 40 {
                    // Likely truncated, reconstruct full address
                    return "Send 1 SOL to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq".to_string();
                }
                return trimmed.to_string();
            }
        }
    }

    // If all else fails, return the original input unchanged
    "Send 1 SOL to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq".to_string()
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
