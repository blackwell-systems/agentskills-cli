use crate::error::Error;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;
use std::fmt;

/// Why a provider detection attempt failed
#[derive(Debug, Clone)]
pub enum DetectionFailure {
    EnvVarMissing(String),
    EnvVarEmpty(String),
    BinaryNotFound(String),
}

impl fmt::Display for DetectionFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DetectionFailure::EnvVarMissing(var) => write!(f, "{} not set", var),
            DetectionFailure::EnvVarEmpty(var) => write!(f, "{} is empty", var),
            DetectionFailure::BinaryNotFound(bin) => write!(f, "'{}' not found on PATH", bin),
        }
    }
}

/// Result of provider detection with rich error context
pub struct DetectionResult {
    pub analyzer: Option<Box<dyn SemanticAnalyzer>>,
    pub attempts: Vec<(String, DetectionFailure)>,
}

impl DetectionResult {
    /// Create a failed detection result with all attempts
    fn not_found(attempts: Vec<(String, DetectionFailure)>) -> Self {
        Self {
            analyzer: None,
            attempts,
        }
    }

    /// Format a user-friendly error message
    pub fn error_message(&self) -> String {
        if self.attempts.is_empty() {
            return "No semantic analyzer providers configured.".to_string();
        }

        let mut msg = String::from("No semantic analyzer found. Tried:\n");
        for (provider, failure) in &self.attempts {
            msg.push_str(&format!("  - {}: {}\n", provider, failure));
        }
        msg.push_str("\nInstall a provider or the tool will use mechanical splitting.");
        msg
    }
}

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
/// - CopilotCli: Shells out to `copilot` CLI (GitHub Copilot)
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
    OpenAiApi,
    GeminiApi,
    GeminiCli,
    CopilotCli,
}

impl Provider {
    /// Get a human-readable name for this provider
    fn name(&self) -> &str {
        match self {
            Provider::AnthropicApi => "Anthropic API",
            Provider::AnthropicCli => "Claude CLI",
            Provider::OpenAiApi => "OpenAI API",
            Provider::GeminiApi => "Gemini API",
            Provider::GeminiCli => "Gemini CLI",
            Provider::CopilotCli => "Copilot CLI",
        }
    }

    /// Get the environment variable name for API key (if applicable)
    fn env_var(&self) -> Option<&str> {
        match self {
            Provider::AnthropicApi => Some("ANTHROPIC_API_KEY"),
            Provider::OpenAiApi => Some("OPENAI_API_KEY"),
            Provider::GeminiApi => Some("GOOGLE_API_KEY"),
            _ => None,
        }
    }

    /// Get the CLI binary name (if applicable)
    fn cli_name(&self) -> Option<&str> {
        match self {
            Provider::AnthropicCli => Some("claude"),
            Provider::GeminiCli => Some("gemini"),
            Provider::CopilotCli => Some("copilot"),
            _ => None,
        }
    }

    /// Try to create an analyzer instance for this provider
    /// Returns Ok(analyzer) on success, Err(reason) on failure
    fn try_create(&self) -> Result<Box<dyn SemanticAnalyzer>, DetectionFailure> {
        match self {
            // API providers: check env var
            Provider::AnthropicApi | Provider::OpenAiApi | Provider::GeminiApi => {
                let env_var_name = self.env_var().ok_or_else(|| {
                    DetectionFailure::EnvVarMissing("unknown".to_string())
                })?;

                let api_key = env::var(env_var_name).map_err(|_| {
                    DetectionFailure::EnvVarMissing(env_var_name.to_string())
                })?;

                if api_key.trim().is_empty() {
                    return Err(DetectionFailure::EnvVarEmpty(env_var_name.to_string()));
                }

                Ok(self.create_api_client(api_key))
            }
            // CLI providers: check binary on PATH
            Provider::AnthropicCli | Provider::GeminiCli | Provider::CopilotCli => {
                let cli_name = self.cli_name().ok_or_else(|| {
                    DetectionFailure::BinaryNotFound("unknown".to_string())
                })?;

                let cli_path = which::which(cli_name).map_err(|_| {
                    DetectionFailure::BinaryNotFound(cli_name.to_string())
                })?;

                Ok(self.create_cli_client(cli_path))
            }
        }
    }

    /// Create an API-based analyzer
    fn create_api_client(&self, api_key: String) -> Box<dyn SemanticAnalyzer> {
        match self {
            Provider::AnthropicApi => {
                Box::new(crate::upgrade::anthropic_api::AnthropicApi::new(api_key))
            }
            Provider::OpenAiApi => {
                Box::new(crate::upgrade::openai_api::OpenAiApi::new(api_key))
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
            Provider::CopilotCli => {
                Box::new(crate::upgrade::copilot_cli::CopilotCli::new(cli_path))
            }
            _ => unreachable!("create_cli_client called on API provider"),
        }
    }
}

/// Create a semantic analyzer using auto-detection with rich error context
///
/// Detection order:
/// 1. ANTHROPIC_API_KEY env var → AnthropicApi
/// 2. `claude` binary on PATH → AnthropicCli
/// 3. OPENAI_API_KEY env var → OpenAiApi
/// 4. GOOGLE_API_KEY env var → GeminiApi
/// 5. `gemini` binary on PATH → GeminiCli
/// 6. `copilot` binary on PATH → CopilotCli
/// 7. None → mechanical split fallback
///
/// Returns DetectionResult with either an analyzer or detailed failure reasons.
pub fn new_analyzer() -> DetectionResult {
    const PROVIDERS: &[Provider] = &[
        Provider::AnthropicApi,
        Provider::AnthropicCli,
        Provider::OpenAiApi,
        Provider::GeminiApi,
        Provider::GeminiCli,
        Provider::CopilotCli,
    ];

    let mut attempts = Vec::new();

    for provider in PROVIDERS {
        match provider.try_create() {
            Ok(analyzer) => {
                return DetectionResult {
                    analyzer: Some(analyzer),
                    attempts: vec![], // Success case - no failures to report
                }
            }
            Err(failure) => {
                attempts.push((provider.name().to_string(), failure));
            }
        }
    }

    DetectionResult::not_found(attempts)
}

/// Create a semantic analyzer for a specific provider by name
///
/// Provider names:
/// - "anthropic-api" → AnthropicApi (requires ANTHROPIC_API_KEY)
/// - "claude-cli" → AnthropicCli (requires `claude` binary)
/// - "openai-api" → OpenAiApi (requires OPENAI_API_KEY)
/// - "gemini-api" → GeminiApi (requires GOOGLE_API_KEY)
/// - "gemini-cli" → GeminiCli (requires `gemini` binary)
/// - "copilot-cli" → CopilotCli (requires `copilot` binary)
///
/// Returns DetectionResult with either an analyzer or the failure reason.
pub fn new_analyzer_by_name(provider_name: &str) -> DetectionResult {
    let provider = match provider_name.to_lowercase().as_str() {
        "anthropic-api" => Provider::AnthropicApi,
        "claude-cli" => Provider::AnthropicCli,
        "openai-api" => Provider::OpenAiApi,
        "gemini-api" => Provider::GeminiApi,
        "gemini-cli" => Provider::GeminiCli,
        "copilot-cli" => Provider::CopilotCli,
        _ => {
            return DetectionResult {
                analyzer: None,
                attempts: vec![(
                    provider_name.to_string(),
                    DetectionFailure::BinaryNotFound(format!(
                        "Unknown provider '{}'. Valid: anthropic-api, claude-cli, openai-api, gemini-api, gemini-cli, copilot-cli",
                        provider_name
                    )),
                )],
            };
        }
    };

    match provider.try_create() {
        Ok(analyzer) => DetectionResult {
            analyzer: Some(analyzer),
            attempts: vec![],
        },
        Err(failure) => DetectionResult {
            analyzer: None,
            attempts: vec![(provider.name().to_string(), failure)],
        },
    }
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
        let result = new_analyzer();
        assert!(result.analyzer.is_some());
        env::remove_var("ANTHROPIC_API_KEY");
    }

    #[test]
    fn test_new_analyzer_with_empty_anthropic_api_key() {
        env::set_var("ANTHROPIC_API_KEY", "   ");
        let result = new_analyzer();
        // Should try other providers after Anthropic API fails with EnvVarEmpty
        // Final result depends on whether CLI binaries are on PATH
        // If a CLI succeeds, attempts vec will be empty (success case)
        // If all fail, attempts vec will have all failures
        env::remove_var("ANTHROPIC_API_KEY");
        // Just verify it doesn't crash - the result depends on system state
        let _ = result;
    }

    #[test]
    fn test_new_analyzer_with_google_api_key() {
        // Clear Anthropic to test Gemini path
        env::remove_var("ANTHROPIC_API_KEY");
        env::set_var("GOOGLE_API_KEY", "test-gemini-key");
        let result = new_analyzer();
        // Result depends on whether claude CLI is on PATH
        // If claude exists, it will be selected first (step 2 beats step 3)
        // If not, Gemini API will be selected
        env::remove_var("GOOGLE_API_KEY");
        // We can't assert specifics without knowing PATH state, but attempts should be tracked
        let _ = result;
    }

    #[test]
    fn test_detection_result_error_message() {
        // Test error message formatting with mock failures
        let attempts = vec![
            (
                "Test Provider 1".to_string(),
                DetectionFailure::EnvVarMissing("TEST_KEY".to_string()),
            ),
            (
                "Test Provider 2".to_string(),
                DetectionFailure::BinaryNotFound("test-bin".to_string()),
            ),
        ];

        let result = DetectionResult::not_found(attempts);

        // Should have no analyzer
        assert!(result.analyzer.is_none());

        // Should have 2 failures
        assert_eq!(result.attempts.len(), 2);

        // Error message should be formatted correctly
        let msg = result.error_message();
        assert!(msg.contains("No semantic analyzer found"));
        assert!(msg.contains("Test Provider 1"));
        assert!(msg.contains("TEST_KEY not set"));
        assert!(msg.contains("Test Provider 2"));
        assert!(msg.contains("'test-bin' not found on PATH"));
    }
}
