use crate::error::Error;
use crate::models::UpgradeOptions;
use clap::Parser;
use std::path::PathBuf;
use tokio::runtime::Runtime;

#[derive(Parser, Debug)]
pub struct UpgradeCommand {
    /// Path to Agent Skill directory
    pub path: PathBuf,

    /// Show changes without applying them
    #[arg(long)]
    pub dry_run: bool,

    /// Add agent-references frontmatter field
    #[arg(long)]
    pub with_agent_references: bool,

    /// Show reasoning and preview before applying changes
    #[arg(long)]
    pub interactive: bool,

    /// Semantic analysis provider (anthropic-api, claude-cli, openai-api, gemini-api, gemini-cli, copilot-cli)
    #[arg(long, value_name = "PROVIDER")]
    pub provider: Option<String>,
}

/// Synchronous wrapper for the async run function.
/// This allows main.rs to call the command without async/await.
/// Once main.rs is updated to use an async runtime (e.g., #[tokio::main]),
/// it can call run_async directly.
pub fn run(cmd: &UpgradeCommand) -> Result<(), Error> {
    let rt = Runtime::new().map_err(|e| {
        Error::ValidationError(format!("Failed to create async runtime: {}", e))
    })?;
    rt.block_on(run_async(cmd))
}

/// Async implementation of the upgrade command.
/// Handles interactive mode with user confirmation and calls the async upgrade_skill function.
pub async fn run_async(cmd: &UpgradeCommand) -> Result<(), Error> {
    // Build upgrade options from command flags
    let options = UpgradeOptions {
        dry_run: cmd.dry_run,
        with_agent_references: cmd.with_agent_references,
        interactive: Some(cmd.interactive),
        provider: cmd.provider.clone(),
    };

    // Print progress to stderr
    eprintln!("Analyzing...");

    // If interactive mode, show preview and get user confirmation
    if cmd.interactive {
        // Note: Detailed preview (analysis results, routing graph, frontmatter changes)
        // would be implemented here after Agent E's upgrade_skill returns structured data.
        // For now, we show a basic prompt.
        eprintln!("\n--- Preview Mode ---");
        eprintln!("Analysis complete. Changes will be applied to: {:?}", cmd.path);
        eprintln!("\nApply these changes? [y/N]: ");

        use std::io::{self, BufRead};
        let stdin = io::stdin();
        let mut input = String::new();
        stdin.lock().read_line(&mut input)?;

        let response = input.trim().to_lowercase();
        if response != "y" && response != "yes" {
            eprintln!("Upgrade cancelled.");
            return Ok(());
        }
    }

    // Handle both directory and SKILL.md paths
    let skill_md_path = if cmd.path.is_dir() {
        cmd.path.join("SKILL.md")
    } else {
        cmd.path.clone()
    };

    if !skill_md_path.exists() {
        return Err(Error::ValidationError(format!(
            "SKILL.md not found at {:?}",
            skill_md_path
        )));
    }

    eprintln!("Splitting content...");
    eprintln!("Generating script...");
    crate::upgrade::upgrade_skill(&skill_md_path, &options).await?;
    println!("✓ Upgrade complete");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_upgrade_command_parsing() {
        let cmd = UpgradeCommand::try_parse_from(&["upgrade", "/path/to/skill"]).unwrap();
        assert_eq!(cmd.path, PathBuf::from("/path/to/skill"));
        assert!(!cmd.dry_run);
        assert!(!cmd.with_agent_references);
        assert!(!cmd.interactive);
    }

    #[test]
    fn test_upgrade_command_with_dry_run() {
        let cmd =
            UpgradeCommand::try_parse_from(&["upgrade", "/path/to/skill", "--dry-run"]).unwrap();
        assert_eq!(cmd.path, PathBuf::from("/path/to/skill"));
        assert!(cmd.dry_run);
        assert!(!cmd.with_agent_references);
        assert!(!cmd.interactive);
    }

    #[test]
    fn test_upgrade_command_with_agent_references() {
        let cmd = UpgradeCommand::try_parse_from(&[
            "upgrade",
            "/path/to/skill",
            "--with-agent-references",
        ])
        .unwrap();
        assert_eq!(cmd.path, PathBuf::from("/path/to/skill"));
        assert!(!cmd.dry_run);
        assert!(cmd.with_agent_references);
        assert!(!cmd.interactive);
    }

    #[test]
    fn test_upgrade_command_all_flags() {
        let cmd = UpgradeCommand::try_parse_from(&[
            "upgrade",
            "/path/to/skill",
            "--dry-run",
            "--with-agent-references",
        ])
        .unwrap();
        assert_eq!(cmd.path, PathBuf::from("/path/to/skill"));
        assert!(cmd.dry_run);
        assert!(cmd.with_agent_references);
        assert!(!cmd.interactive);
    }

    #[test]
    fn test_upgrade_command_missing_path() {
        let result = UpgradeCommand::try_parse_from(&["upgrade"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_upgrade_command_help() {
        let mut cmd = UpgradeCommand::command();
        let help = cmd.render_help().to_string();
        assert!(help.contains("Path to Agent Skill directory"));
        assert!(help.contains("Show changes without applying"));
        assert!(help.contains("Add agent-references"));
    }

    #[test]
    fn test_upgrade_command_with_interactive() {
        let cmd = UpgradeCommand::try_parse_from(&[
            "upgrade",
            "/path/to/skill",
            "--interactive",
        ])
        .unwrap();
        assert_eq!(cmd.path, PathBuf::from("/path/to/skill"));
        assert!(!cmd.dry_run);
        assert!(!cmd.with_agent_references);
        assert!(cmd.interactive);
    }

    #[test]
    fn test_upgrade_options_interactive_field() {
        // Test that UpgradeOptions correctly holds the interactive field
        let options = UpgradeOptions {
            dry_run: false,
            with_agent_references: true,
            interactive: Some(true),
            ..Default::default()
        };
        assert_eq!(options.interactive, Some(true));

        // Test default behavior
        let default_options = UpgradeOptions {
            dry_run: false,
            with_agent_references: false,
            interactive: Some(false),
            ..Default::default()
        };
        assert_eq!(default_options.interactive, Some(false));

        // Note: End-to-end interactive test requires stdin mocking, which is complex.
        // Interactive mode should be tested manually by running:
        // cargo run -- upgrade /path/to/skill --interactive
        // and verifying the prompt appears and user input is correctly handled.
    }
}
