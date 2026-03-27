use crate::error::Error;
use crate::upgrade::semantic_analyzer::{SemanticAnalyzer, SectionIntent};
use anthropic_sdk::Client;
use async_trait::async_trait;
use serde_json::json;
use std::sync::{Arc, Mutex};

/// API-based Anthropic analyzer using Anthropic SDK
pub struct AnthropicApi {
    api_key: String,
}

impl AnthropicApi {
    /// Create a new Anthropic API analyzer with the provided API key
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

#[async_trait]
impl SemanticAnalyzer for AnthropicApi {
    async fn analyze_section(
        &self,
        section_header: &str,
        section_content: &str,
    ) -> Result<SectionIntent, Error> {
        // Truncate content to 500 chars to stay within token limits
        let truncated_content = if section_content.len() > 500 {
            &section_content[..500]
        } else {
            section_content
        };

        // Construct analysis prompt
        let prompt = format!(
            r#"This is a section from an Agent Skill. Section header: "{section_header}".

Section content (first 500 chars):
{truncated_content}

Determine if this section is:
(1) command-specific (e.g., only for /scout command)
(2) agent-specific (e.g., only for wave-agent)
(3) conditional (only loaded when prompt matches pattern)
(4) always-loaded (core section for all invocations)

Respond ONLY with valid JSON in this exact format:
{{
  "is_command_specific": true/false,
  "command": "command_name or null",
  "is_agent_specific": true/false,
  "agent_type": "agent_type_name or null",
  "is_conditional": true/false,
  "condition_pattern": "pattern or null",
  "reasoning": "brief explanation"
}}"#
        );

        // Build request using SDK builder pattern
        let request = Client::new()
            .auth(&self.api_key)
            .model("claude-3-haiku-20240307")
            .messages(&json!([
                {"role": "user", "content": prompt}
            ]))
            .max_tokens(500)
            .stream(false)
            .build()
            .map_err(|e| Error::ApiError(format!("Failed to build request: {}", e)))?;

        // Execute request and collect response
        let response_text = Arc::new(Mutex::new(String::new()));
        let response_clone = response_text.clone();

        request
            .execute(move |text| {
                let response_clone = response_clone.clone();
                async move {
                    let mut response = response_clone.lock().unwrap();
                    response.push_str(&text);
                }
            })
            .await
            .map_err(|e| Error::ApiError(format!("Claude API call failed: {}", e)))?;

        let final_response = response_text.lock().unwrap().clone();

        // Parse JSON response
        let intent: SectionIntent = serde_json::from_str(&final_response).map_err(|e| {
            Error::ApiError(format!(
                "Failed to parse Claude response as JSON: {}. Response: {}",
                e, final_response
            ))
        })?;

        Ok(intent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anthropic_api_new() {
        let analyzer = AnthropicApi::new("test-api-key".to_string());
        assert_eq!(analyzer.api_key, "test-api-key");
    }

    #[tokio::test]
    #[ignore] // Requires live API key in ANTHROPIC_API_KEY env var
    async fn test_analyze_section_command_specific() {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .expect("ANTHROPIC_API_KEY env var required for live API tests");
        let analyzer = AnthropicApi::new(api_key);

        let result = analyzer
            .analyze_section(
                "Scout Agent Instructions",
                "This section provides detailed instructions for the Scout agent when executing /saw scout commands...",
            )
            .await;

        assert!(result.is_ok());
        let intent = result.unwrap();
        // Scout-specific section should be detected as command-specific
        assert!(intent.is_command_specific || intent.is_agent_specific);
    }

    #[tokio::test]
    #[ignore] // Requires live API key
    async fn test_analyze_section_agent_specific() {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .expect("ANTHROPIC_API_KEY env var required for live API tests");
        let analyzer = AnthropicApi::new(api_key);

        let result = analyzer
            .analyze_section(
                "Wave Agent Protocol",
                "You are a Wave Agent in the Scout-and-Wave protocol. You implement a specific feature component...",
            )
            .await;

        assert!(result.is_ok());
        let intent = result.unwrap();
        assert!(intent.is_agent_specific);
    }

    #[tokio::test]
    #[ignore] // Requires live API key
    async fn test_analyze_section_always_loaded() {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .expect("ANTHROPIC_API_KEY env var required for live API tests");
        let analyzer = AnthropicApi::new(api_key);

        let result = analyzer
            .analyze_section(
                "General Instructions",
                "These are general purpose instructions that apply to all agent invocations regardless of context...",
            )
            .await;

        assert!(result.is_ok());
        let intent = result.unwrap();
        // Always-loaded section should have all flags false
        assert!(!intent.is_command_specific);
        assert!(!intent.is_agent_specific);
        assert!(!intent.is_conditional);
    }
}
