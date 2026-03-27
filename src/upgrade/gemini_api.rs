use crate::error::Error;
use crate::upgrade::semantic_analyzer::{SemanticAnalyzer, SectionIntent};
use async_trait::async_trait;
use gemini_client_rs::{
    GeminiClient,
    types::{Content, ContentPart, GenerateContentRequest, Role},
};

/// API-based Gemini analyzer using Google Gemini API
pub struct GeminiApi {
    api_key: String,
}

impl GeminiApi {
    /// Create a new Gemini API analyzer with the provided API key
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

#[async_trait]
impl SemanticAnalyzer for GeminiApi {
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

        // Construct analysis prompt (same as Anthropic)
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

        // Build Gemini client and request
        let client = GeminiClient::new(self.api_key.clone());

        // Create request with proper structure
        let content_part = ContentPart::new_text(&prompt, false);
        let content = Content {
            parts: vec![content_part],
            role: Some(Role::User),
        };
        let request = GenerateContentRequest {
            contents: vec![content],
            system_instruction: None,
            tools: vec![],
            tool_config: None,
            generation_config: None,
        };

        // Execute request
        let response = client
            .generate_content("gemini-1.5-flash", &request)
            .await
            .map_err(|e| Error::ApiError(format!("Gemini API call failed: {}", e)))?;

        // Extract text from response
        let response_text = response
            .candidates
            .first()
            .and_then(|c| c.content.as_ref())
            .and_then(|content| content.parts.first())
            .and_then(|p| {
                match &p.data {
                    gemini_client_rs::types::ContentData::Text(text) => Some(text.as_str()),
                    _ => None,
                }
            })
            .ok_or_else(|| Error::ApiError("Gemini response missing text content".to_string()))?;

        // Parse JSON response (strip markdown code fences if present)
        let json_str = response_text
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        let intent: SectionIntent = serde_json::from_str(json_str).map_err(|e| {
            Error::ApiError(format!(
                "Failed to parse Gemini response as JSON: {}. Response: {}",
                e, response_text
            ))
        })?;

        Ok(intent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gemini_api_new() {
        let analyzer = GeminiApi::new("test-api-key".to_string());
        assert_eq!(analyzer.api_key, "test-api-key");
    }

    #[tokio::test]
    #[ignore] // Requires live API key in GOOGLE_API_KEY env var
    async fn test_analyze_section_command_specific() {
        let api_key = std::env::var("GOOGLE_API_KEY")
            .expect("GOOGLE_API_KEY env var required for live API tests");
        let analyzer = GeminiApi::new(api_key);

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
        let api_key = std::env::var("GOOGLE_API_KEY")
            .expect("GOOGLE_API_KEY env var required for live API tests");
        let analyzer = GeminiApi::new(api_key);

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
        let api_key = std::env::var("GOOGLE_API_KEY")
            .expect("GOOGLE_API_KEY env var required for live API tests");
        let analyzer = GeminiApi::new(api_key);

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
