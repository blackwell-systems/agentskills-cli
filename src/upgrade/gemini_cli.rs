use crate::error::Error;
use crate::upgrade::semantic_analyzer::{SemanticAnalyzer, SectionIntent};
use async_trait::async_trait;
use std::path::PathBuf;
use std::process::Command;

/// CLI-based Gemini analyzer that shells out to `gemini` command
pub struct GeminiCli {
    gemini_path: PathBuf,
}

impl GeminiCli {
    /// Create a new Gemini CLI analyzer with the given path to the gemini binary
    pub fn new(gemini_path: PathBuf) -> Self {
        Self { gemini_path }
    }
}

#[async_trait]
impl SemanticAnalyzer for GeminiCli {
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
        let prompt = format!(
            r#"This is a section from an Agent Skill. Section header: "{section_header}".

Section content (first 500 chars):
{truncated_content}

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
}}"#
        );

        // Shell out to gemini CLI
        // Use -p for headless mode (non-interactive)
        let output = Command::new(&self.gemini_path)
            .arg("-p")
            .arg(prompt)
            .output()
            .map_err(|e| Error::ValidationError(format!("Failed to execute gemini CLI: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::ApiError(format!(
                "gemini CLI exited with status {}: {}",
                output.status, stderr
            )));
        }

        // Parse stdout as JSON (strip markdown code fences if present)
        let stdout = String::from_utf8_lossy(&output.stdout);
        let json_str = stdout
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();
        let intent: SectionIntent = serde_json::from_str(json_str).map_err(|e| {
            Error::ApiError(format!(
                "Failed to parse gemini CLI response as JSON: {}. Response: {}",
                e, stdout
            ))
        })?;

        Ok(intent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gemini_cli_new() {
        let analyzer = GeminiCli::new(PathBuf::from("/usr/local/bin/gemini"));
        assert_eq!(
            analyzer.gemini_path,
            PathBuf::from("/usr/local/bin/gemini")
        );
    }

    #[tokio::test]
    #[ignore] // Requires gemini CLI installed
    async fn test_analyze_section_with_cli() {
        // Try to find gemini on PATH
        let gemini_path = which::which("gemini");
        if gemini_path.is_err() {
            eprintln!("gemini CLI not found on PATH, skipping test");
            return;
        }

        let analyzer = GeminiCli::new(gemini_path.unwrap());
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

    #[test]
    fn test_gemini_cli_invalid_binary() {
        let analyzer = GeminiCli::new(PathBuf::from("/nonexistent/gemini"));
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(analyzer.analyze_section("Test", "Test content"));

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Failed to execute gemini CLI"));
    }
}
