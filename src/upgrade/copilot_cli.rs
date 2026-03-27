use crate::error::Error;
use crate::upgrade::semantic_analyzer::{SemanticAnalyzer, SectionIntent};
use async_trait::async_trait;
use std::path::PathBuf;
use std::process::Command;

/// CLI-based analyzer using GitHub Copilot CLI
///
/// Shells out to `copilot` command (requires GitHub Copilot subscription)
pub struct CopilotCli {
    copilot_path: PathBuf,
}

impl CopilotCli {
    /// Create a new Copilot CLI analyzer with the given path to the copilot binary
    pub fn new(copilot_path: PathBuf) -> Self {
        Self { copilot_path }
    }
}

#[async_trait]
impl SemanticAnalyzer for CopilotCli {
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

        // Shell out to copilot
        // Use -p for non-interactive mode with --allow-all-tools
        // Note: GITHUB_TOKEN env var must not contain classic PAT (ghp_)
        let output = Command::new(&self.copilot_path)
            .arg("-p")
            .arg(&prompt)
            .arg("--allow-all-tools")
            .env_remove("GITHUB_TOKEN") // Remove classic PAT if present
            .output()
            .map_err(|e| {
                Error::ValidationError(format!("Failed to execute copilot CLI: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::ApiError(format!(
                "copilot CLI exited with status {}: {}",
                output.status, stderr
            )));
        }

        // Parse stdout as JSON (strip markdown code fences and usage stats)
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Find the JSON content between code fences or before usage stats
        let json_str = stdout
            .lines()
            .skip_while(|line| !line.trim().starts_with('{')) // Skip until JSON starts
            .take_while(|line| {
                // Stop at usage stats or empty lines after JSON
                !line.starts_with("Total usage")
                    && !line.starts_with("API time")
                    && !line.starts_with("Breakdown")
            })
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim()
            .to_string();

        let intent: SectionIntent = serde_json::from_str(&json_str).map_err(|e| {
            Error::ApiError(format!(
                "Failed to parse copilot CLI response as JSON: {}. Response: {}",
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
    fn test_copilot_cli_new() {
        let analyzer = CopilotCli::new(PathBuf::from("/usr/local/bin/copilot"));
        assert_eq!(
            analyzer.copilot_path,
            PathBuf::from("/usr/local/bin/copilot")
        );
    }

    #[tokio::test]
    #[ignore] // Requires copilot CLI installed
    async fn test_analyze_section_with_cli() {
        // Try to find copilot on PATH
        let copilot_path = which::which("copilot");
        if copilot_path.is_err() {
            eprintln!("copilot CLI not found on PATH, skipping test");
            return;
        }

        let analyzer = CopilotCli::new(copilot_path.unwrap());
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
    fn test_copilot_cli_invalid_binary() {
        let analyzer = CopilotCli::new(PathBuf::from("/nonexistent/copilot"));
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(analyzer.analyze_section("Test", "Test content"));

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Failed to execute copilot CLI"));
    }
}
