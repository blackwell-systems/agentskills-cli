use crate::error::Error;
use serde::{Deserialize, Serialize};
use std::env;

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

/// Trait for Claude-based semantic analysis backends
///
/// Implementations:
/// - ApiClient: Uses Anthropic API SDK
/// - CliClient: Shells out to `claude` CLI
pub trait ClaudeClient: Send + Sync {
    /// Analyze a section to determine its routing intent
    ///
    /// Uses Claude to classify whether a section is:
    /// - Command-specific (only for one subcommand like /scout)
    /// - Agent-specific (only for one agent type like wave-agent)
    /// - Conditional (only loaded when prompt matches pattern)
    /// - Always-loaded (core section for all invocations)
    fn analyze_section(
        &self,
        section_header: &str,
        section_content: &str,
    ) -> impl std::future::Future<Output = Result<SectionIntent, Error>> + Send;
}

/// Create a Claude client using auto-detection
///
/// Detection order:
/// 1. ANTHROPIC_API_KEY env var → ApiClient
/// 2. `claude` binary on PATH → CliClient
/// 3. Neither → None (mechanical split fallback)
///
/// This mirrors the authentication pattern in scout-and-wave-go's backend selection.
pub fn new_client() -> Option<Box<dyn ClaudeClient>> {
    // 1. API key
    if let Ok(api_key) = env::var("ANTHROPIC_API_KEY") {
        if !api_key.trim().is_empty() {
            return Some(Box::new(crate::upgrade::api_client::ApiClient::new(
                api_key,
            )));
        }
    }

    // 2. Claude CLI
    if let Ok(claude_path) = which::which("claude") {
        return Some(Box::new(crate::upgrade::cli_client::CliClient::new(
            claude_path,
        )));
    }

    // 3. No client available
    None
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_new_client_with_api_key() {
        env::set_var("ANTHROPIC_API_KEY", "test-key");
        let client = new_client();
        assert!(client.is_some());
        env::remove_var("ANTHROPIC_API_KEY");
    }

    #[test]
    fn test_new_client_with_empty_api_key() {
        env::set_var("ANTHROPIC_API_KEY", "   ");
        let _client = new_client();
        // Should skip to CLI check (we don't assert result since it depends on PATH)
        env::remove_var("ANTHROPIC_API_KEY");
    }
}
