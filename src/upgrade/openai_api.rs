use crate::error::Error;
use crate::upgrade::semantic_analyzer::{SemanticAnalyzer, SectionIntent};
use async_openai::config::OpenAIConfig;
use async_openai::types::{
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
    ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
};
use async_openai::Client;
use async_trait::async_trait;

/// API-based analyzer using OpenAI API
///
/// Requires OPENAI_API_KEY environment variable
pub struct OpenAiApi {
    api_key: String,
}

impl OpenAiApi {
    /// Create a new OpenAI API analyzer with the given API key
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

#[async_trait]
impl SemanticAnalyzer for OpenAiApi {
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

        // Construct analysis prompt (same as other providers)
        let user_prompt = format!(
            r#"This is a section from an Agent Skill. Section header: "{}".

Section content (first 500 chars):
{}

Determine if this section is:
(1) command-specific (e.g., only for /scout command)
(2) agent-specific (e.g., only for wave-agent)
(3) conditional (only loaded when prompt matches pattern)
(4) always-loaded (core section for all invocations)

If conditional, classify the trigger timing:
- "invocation": Triggered by initial user request (subcommand, flag, agent type, explicit topic)
  Examples: "--dry-run flag", "scout subcommand", "when user asks about X"
- "runtime": Triggered by state discovered during execution (failures, errors, missing resources)
  Examples: "if CI fails", "when artifact not found", "on retry", "after error", "return to previous step"

Respond ONLY with valid JSON in this exact format:
{{
  "is_command_specific": true/false,
  "command": "command_name or null",
  "is_agent_specific": true/false,
  "agent_type": "agent_type_name or null",
  "is_conditional": true/false,
  "condition_pattern": "pattern or null",
  "trigger_timing": "invocation or runtime or null",
  "reasoning": "brief explanation"
}}"#,
            section_header, truncated_content
        );

        // Create OpenAI client with API key
        let config = OpenAIConfig::new().with_api_key(&self.api_key);
        let client = Client::with_config(config);

        // Build chat completion request
        let request = CreateChatCompletionRequestArgs::default()
            .model("gpt-4o-mini") // Cost-effective model for structured output
            .messages(vec![
                ChatCompletionRequestMessage::System(
                    ChatCompletionRequestSystemMessageArgs::default()
                        .content("You are a helpful assistant that analyzes Agent Skill sections and outputs JSON.")
                        .build()
                        .map_err(|e| Error::ApiError(format!("Failed to build system message: {}", e)))?,
                ),
                ChatCompletionRequestMessage::User(
                    ChatCompletionRequestUserMessageArgs::default()
                        .content(user_prompt)
                        .build()
                        .map_err(|e| Error::ApiError(format!("Failed to build user message: {}", e)))?,
                ),
            ])
            .build()
            .map_err(|e| Error::ApiError(format!("Failed to build request: {}", e)))?;

        // Call API
        let response = client
            .chat()
            .create(request)
            .await
            .map_err(|e| Error::ApiError(format!("OpenAI API error: {}", e)))?;

        // Extract text from first choice
        let content = response
            .choices
            .first()
            .and_then(|c| c.message.content.as_ref())
            .ok_or_else(|| Error::ApiError("No response content from OpenAI".to_string()))?;

        // Parse JSON response (strip markdown code fences if present)
        let json_str = content
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        let intent: SectionIntent = serde_json::from_str(json_str).map_err(|e| {
            Error::ApiError(format!(
                "Failed to parse OpenAI response as JSON: {}. Response: {}",
                e, content
            ))
        })?;

        Ok(intent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_api_new() {
        let analyzer = OpenAiApi::new("test-api-key".to_string());
        assert_eq!(analyzer.api_key, "test-api-key");
    }

    #[tokio::test]
    #[ignore] // Requires OPENAI_API_KEY
    async fn test_analyze_section_with_api() {
        let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
        let analyzer = OpenAiApi::new(api_key);

        let result = analyzer
            .analyze_section(
                "Scout Agent Instructions",
                "This section provides detailed instructions for the Scout agent when executing /saw scout commands...",
            )
            .await;

        assert!(result.is_ok());
        let intent = result.unwrap();
        // Scout-specific section should be detected
        assert!(intent.is_command_specific || intent.is_agent_specific);
    }
}
