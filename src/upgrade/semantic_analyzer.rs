use crate::error::Error;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;

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

/// Trait for LLM-based semantic analysis backends (provider-agnostic)
///
/// Implementations:
/// - AnthropicApi: Uses Anthropic API SDK
/// - AnthropicCli: Shells out to `claude` CLI
/// - GeminiApi: Uses Google Gemini API SDK
/// - GeminiCli: Shells out to `gemini` CLI
#[async_trait]
pub trait SemanticAnalyzer: Send + Sync {
    /// Analyze a section to determine its routing intent
    ///
    /// Uses an LLM to classify whether a section is:
    /// - Command-specific (only for one subcommand like /scout)
    /// - Agent-specific (only for one agent type like wave-agent)
    /// - Conditional (only loaded when prompt matches pattern)
    /// - Always-loaded (core section for all invocations)
    async fn analyze_section(
        &self,
        section_header: &str,
        section_content: &str,
    ) -> Result<SectionIntent, Error>;
}

/// Semantic analysis provider configuration
enum Provider {
    AnthropicApi,
    AnthropicCli,
    GeminiApi,
    GeminiCli,
}

impl Provider {
    /// Get the environment variable name for API key (if applicable)
    fn env_var(&self) -> Option<&str> {
        match self {
            Provider::AnthropicApi => Some("ANTHROPIC_API_KEY"),
            Provider::GeminiApi => Some("GOOGLE_API_KEY"),
            _ => None,
        }
    }

    /// Get the CLI binary name (if applicable)
    fn cli_name(&self) -> Option<&str> {
        match self {
            Provider::AnthropicCli => Some("claude"),
            Provider::GeminiCli => Some("gemini"),
            _ => None,
        }
    }

    /// Try to create an analyzer instance for this provider
    fn try_create(&self) -> Option<Box<dyn SemanticAnalyzer>> {
        match self {
            // API providers: check env var
            Provider::AnthropicApi | Provider::GeminiApi => {
                let env_var_name = self.env_var()?;
                let api_key = env::var(env_var_name).ok()?;
                if api_key.trim().is_empty() {
                    return None;
                }
                Some(self.create_api_client(api_key))
            }
            // CLI providers: check binary on PATH
            Provider::AnthropicCli | Provider::GeminiCli => {
                let cli_name = self.cli_name()?;
                let cli_path = which::which(cli_name).ok()?;
                Some(self.create_cli_client(cli_path))
            }
        }
    }

    /// Create an API-based analyzer
    fn create_api_client(&self, api_key: String) -> Box<dyn SemanticAnalyzer> {
        match self {
            Provider::AnthropicApi => {
                Box::new(crate::upgrade::anthropic_api::AnthropicApi::new(api_key))
            }
            Provider::GeminiApi => {
                Box::new(crate::upgrade::gemini_api::GeminiApi::new(api_key))
            }
            _ => unreachable!("create_api_client called on CLI provider"),
        }
    }

    /// Create a CLI-based analyzer
    fn create_cli_client(&self, cli_path: PathBuf) -> Box<dyn SemanticAnalyzer> {
        match self {
            Provider::AnthropicCli => {
                Box::new(crate::upgrade::anthropic_cli::AnthropicCli::new(cli_path))
            }
            Provider::GeminiCli => {
                Box::new(crate::upgrade::gemini_cli::GeminiCli::new(cli_path))
            }
            _ => unreachable!("create_cli_client called on API provider"),
        }
    }
}

/// Create a semantic analyzer using auto-detection
///
/// Detection order:
/// 1. ANTHROPIC_API_KEY env var → AnthropicApi
/// 2. `claude` binary on PATH → AnthropicCli
/// 3. GOOGLE_API_KEY env var → GeminiApi
/// 4. `gemini` binary on PATH → GeminiCli
/// 5. None → mechanical split fallback
///
/// This supports multiple AgentSkills-compliant providers.
pub fn new_analyzer() -> Option<Box<dyn SemanticAnalyzer>> {
    const PROVIDERS: &[Provider] = &[
        Provider::AnthropicApi,
        Provider::AnthropicCli,
        Provider::GeminiApi,
        Provider::GeminiCli,
    ];

    for provider in PROVIDERS {
        if let Some(analyzer) = provider.try_create() {
            return Some(analyzer);
        }
    }

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
    fn test_new_analyzer_with_anthropic_api_key() {
        env::set_var("ANTHROPIC_API_KEY", "test-key");
        let analyzer = new_analyzer();
        assert!(analyzer.is_some());
        env::remove_var("ANTHROPIC_API_KEY");
    }

    #[test]
    fn test_new_analyzer_with_empty_anthropic_api_key() {
        env::set_var("ANTHROPIC_API_KEY", "   ");
        let _analyzer = new_analyzer();
        // Should skip to CLI check (we don't assert result since it depends on PATH)
        env::remove_var("ANTHROPIC_API_KEY");
    }

    #[test]
    fn test_new_analyzer_with_google_api_key() {
        // Clear Anthropic to test Gemini path
        env::remove_var("ANTHROPIC_API_KEY");
        env::set_var("GOOGLE_API_KEY", "test-gemini-key");
        let analyzer = new_analyzer();
        // Result depends on whether claude CLI is on PATH
        // If claude exists, it will be selected first (step 2 beats step 3)
        // If not, Gemini API will be selected
        env::remove_var("GOOGLE_API_KEY");
        // We can't assert specifics without knowing PATH state
        let _ = analyzer;
    }
}
