use crate::error::Error;
use crate::upgrade::claude_client::{ClaudeClient, SectionIntent};
use async_trait::async_trait;
use std::path::PathBuf;
use std::process::Command;

/// CLI-based Claude client that shells out to `claude` command
///
/// Mirrors the implementation in scout-and-wave-go/pkg/agent/backend/cli/client.go
pub struct CliClient {
    claude_path: PathBuf,
}

impl CliClient {
    /// Create a new CLI client with the given path to the claude binary
    pub fn new(claude_path: PathBuf) -> Self {
        Self { claude_path }
    }
}

#[async_trait]
impl ClaudeClient for CliClient {
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

        // Construct analysis prompt (same as API client)
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

        // Shell out to claude CLI
        // Use --print (no tool execution)
        let output = Command::new(&self.claude_path)
            .arg("--print")
            .arg("-p")
            .arg(prompt)
            .output()
            .map_err(|e| Error::ValidationError(format!("Failed to execute claude CLI: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::ApiError(format!(
                "claude CLI exited with status {}: {}",
                output.status, stderr
            )));
        }

        // Parse stdout as JSON (strip markdown code fences if present)
        let stdout = String::from_utf8_lossy(&output.stdout);
        let json_str = stdout.trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();
        let intent: SectionIntent = serde_json::from_str(json_str).map_err(|e| {
            Error::ApiError(format!(
                "Failed to parse claude CLI response as JSON: {}. Response: {}",
                e, stdout
            ))
        })?;

        Ok(intent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_cli_client_new() {
        let client = CliClient::new(PathBuf::from("/usr/local/bin/claude"));
        assert_eq!(
            client.claude_path,
            PathBuf::from("/usr/local/bin/claude")
        );
    }

    #[tokio::test]
    #[ignore] // Requires claude CLI installed
    async fn test_analyze_section_with_cli() {
        // Try to find claude on PATH
        let claude_path = which::which("claude");
        if claude_path.is_err() {
            eprintln!("claude CLI not found on PATH, skipping test");
            return;
        }

        let client = CliClient::new(claude_path.unwrap());
        let result = client
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
    fn test_cli_client_invalid_binary() {
        let client = CliClient::new(PathBuf::from("/nonexistent/claude"));
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(client.analyze_section(
            "Test",
            "Test content",
        ));

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Failed to execute claude CLI"));
    }
}
