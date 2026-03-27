use crate::error::Error;
use anthropic_sdk::{Client, MessagesRequest};
use serde::{Deserialize, Serialize};

/// Intent classification for a skill section, determined by semantic analysis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SectionIntent {
    pub is_command_specific: bool,
    pub command: Option<String>,
    pub is_agent_specific: bool,
    pub agent_type: Option<String>,
    pub is_conditional: bool,
    pub condition_pattern: Option<String>,
    pub reasoning: String,
}

/// Semantic analyzer using Claude API for section intent classification
pub struct SemanticAnalyzer {
    client: Client,
}

impl SemanticAnalyzer {
    /// Create a new semantic analyzer with the provided API key
    pub fn new(api_key: String) -> Self {
        let client = Client::new(api_key);
        Self { client }
    }

    /// Analyze a section to determine its routing intent
    ///
    /// Uses Claude API to classify whether a section is:
    /// - Command-specific (only for one subcommand like /scout)
    /// - Agent-specific (only for one agent type like wave-agent)
    /// - Conditional (only loaded when prompt matches pattern)
    /// - Always-loaded (core section for all invocations)
    pub async fn analyze_section(
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

        // Call Claude API
        let request = MessagesRequest {
            model: "claude-3-haiku-20240307".to_string(),
            max_tokens: 500,
            messages: vec![anthropic_sdk::Message {
                role: "user".to_string(),
                content: prompt,
            }],
            system: None,
            temperature: None,
            top_p: None,
            top_k: None,
            metadata: None,
            stop_sequences: None,
            stream: None,
        };

        let response = self
            .client
            .messages(request)
            .await
            .map_err(|e| Error::ApiError(format!("Claude API call failed: {}", e)))?;

        // Extract text from response
        let response_text = response
            .content
            .first()
            .ok_or_else(|| Error::ApiError("Empty response from Claude API".to_string()))?
            .text
            .as_ref()
            .ok_or_else(|| Error::ApiError("No text in Claude API response".to_string()))?;

        // Parse JSON response
        let intent: SectionIntent = serde_json::from_str(response_text).map_err(|e| {
            Error::ApiError(format!(
                "Failed to parse Claude response as JSON: {}. Response: {}",
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
    fn test_semantic_analyzer_new() {
        let analyzer = SemanticAnalyzer::new("test-api-key".to_string());
        // Just verify construction works
        assert!(std::ptr::addr_of!(analyzer.client) as usize != 0);
    }

    #[test]
    #[ignore] // Requires live API key in ANTHROPIC_API_KEY env var
    async fn test_analyze_section_command_specific() {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .expect("ANTHROPIC_API_KEY env var required for live API tests");
        let analyzer = SemanticAnalyzer::new(api_key);

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

    #[test]
    #[ignore] // Requires live API key
    async fn test_analyze_section_agent_specific() {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .expect("ANTHROPIC_API_KEY env var required for live API tests");
        let analyzer = SemanticAnalyzer::new(api_key);

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

    #[test]
    #[ignore] // Requires live API key
    async fn test_analyze_section_always_loaded() {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .expect("ANTHROPIC_API_KEY env var required for live API tests");
        let analyzer = SemanticAnalyzer::new(api_key);

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

    // Mock tests for offline testing
    #[test]
    fn test_section_intent_serialization() {
        let intent = SectionIntent {
            is_command_specific: true,
            command: Some("scout".to_string()),
            is_agent_specific: false,
            agent_type: None,
            is_conditional: false,
            condition_pattern: None,
            reasoning: "Test reasoning".to_string(),
        };

        let json = serde_json::to_string(&intent).unwrap();
        let deserialized: SectionIntent = serde_json::from_str(&json).unwrap();
        assert_eq!(intent, deserialized);
    }
}
