use crate::error::Error;
use crate::models::{RoutingStyle, DecomposeOptions};
use clap::Parser;
use std::path::PathBuf;
use tokio::runtime::Runtime;

#[derive(Parser, Debug)]
pub struct DecomposeCommand {
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

    /// Routing style (smart, inline, table, none)
    #[arg(long, value_name = "STYLE")]
    pub routing_style: Option<String>,

    /// Show timing annotations in routing
    #[arg(long)]
    pub timing: bool,

    /// Generate back-links in reference files
    #[arg(long, default_value = "true")]
    pub back_links: bool,
}

/// Synchronous wrapper for the async run function.
/// This allows main.rs to call the command without async/await.
/// Once main.rs is updated to use an async runtime (e.g., #[tokio::main]),
/// it can call run_async directly.
pub fn run(cmd: &DecomposeCommand) -> Result<(), Error> {
    let rt = Runtime::new().map_err(|e| {
        Error::ValidationError(format!("Failed to create async runtime: {}", e))
    })?;
    rt.block_on(run_async(cmd))
}

/// Async implementation of the decompose command.
/// Handles interactive mode with user confirmation and calls the async decompose_skill function.
pub async fn run_async(cmd: &DecomposeCommand) -> Result<(), Error> {
    // Parse routing style if provided
    let routing_style = cmd.routing_style.as_ref().map(|s| {
        match s.to_lowercase().as_str() {
            "smart" => RoutingStyle::Smart,
            "inline" => RoutingStyle::Inline,
            "table" => RoutingStyle::Table,
            "none" => RoutingStyle::None,
            _ => RoutingStyle::Smart, // Default to smart if invalid
        }
    });

    // Build upgrade options from command flags
    let options = DecomposeOptions {
        dry_run: cmd.dry_run,
        with_agent_references: cmd.with_agent_references,
        interactive: Some(cmd.interactive),
        provider: cmd.provider.clone(),
        routing_style,
        show_timing: cmd.timing,
        back_links: cmd.back_links,
    };

    // Print progress to stderr
    eprintln!("Analyzing...");

    // If interactive mode, show preview and get user confirmation
    if cmd.interactive {
        // Note: Detailed preview (analysis results, routing graph, frontmatter changes)
        // would be implemented here after Agent E's decompose_skill returns structured data.
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
            eprintln!("Decompose cancelled.");
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
    let preview_opt = crate::decompose::decompose_skill(&skill_md_path, &options).await?;

    if let Some(_preview) = preview_opt {
        // Dry-run mode: preview_data was returned, already printed by decompose_skill
        // No additional output needed here
    } else {
        // Non-dry-run mode: files were written
        println!("✓ Decompose complete");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_decompose_command_parsing() {
        let cmd = DecomposeCommand::try_parse_from(&["decompose", "/path/to/skill"]).unwrap();
        assert_eq!(cmd.path, PathBuf::from("/path/to/skill"));
        assert!(!cmd.dry_run);
        assert!(!cmd.with_agent_references);
        assert!(!cmd.interactive);
    }

    #[test]
    fn test_decompose_command_with_dry_run() {
        let cmd =
            DecomposeCommand::try_parse_from(&["decompose", "/path/to/skill", "--dry-run"]).unwrap();
        assert_eq!(cmd.path, PathBuf::from("/path/to/skill"));
        assert!(cmd.dry_run);
        assert!(!cmd.with_agent_references);
        assert!(!cmd.interactive);
    }

    #[test]
    fn test_decompose_command_with_agent_references() {
        let cmd = DecomposeCommand::try_parse_from(&[
            "decompose",
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
    fn test_decompose_command_all_flags() {
        let cmd = DecomposeCommand::try_parse_from(&[
            "decompose",
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
    fn test_decompose_command_missing_path() {
        let result = DecomposeCommand::try_parse_from(&["decompose"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_decompose_command_help() {
        let mut cmd = DecomposeCommand::command();
        let help = cmd.render_help().to_string();
        assert!(help.contains("Path to Agent Skill directory"));
        assert!(help.contains("Show changes without applying"));
        assert!(help.contains("Add agent-references"));
    }

    #[test]
    fn test_decompose_command_with_interactive() {
        let cmd = DecomposeCommand::try_parse_from(&[
            "decompose",
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
    fn test_decompose_options_interactive_field() {
        // Test that DecomposeOptions correctly holds the interactive field
        let options = DecomposeOptions {
            dry_run: false,
            with_agent_references: true,
            interactive: Some(true),
            ..Default::default()
        };
        assert_eq!(options.interactive, Some(true));

        // Test default behavior
        let default_options = DecomposeOptions {
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
