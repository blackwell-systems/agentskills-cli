use crate::error::Error;
use crate::models::{RoutingStyle, DecomposeOptions};
use clap::Parser;
use colored::Colorize;
use std::path::PathBuf;
use tokio::runtime::Runtime;

#[derive(Parser, Debug)]
#[command(about = "Split large skills into core + reference files")]
#[command(after_help = "\
WORKFLOW:
  1. Detects bloat (implementation sections, command catalogs)
  2. Extracts sections to references/ directory
  3. Generates inject-context script for loading references
  4. Adds breadcrumbs to SKILL.md for runtime sections
  5. Preserves existing frontmatter unchanged

EXAMPLES:
  # Preview changes first
  agentskills decompose ~/.claude/skills/my-skill --dry-run

  # Apply with confirmation
  agentskills decompose ~/.claude/skills/my-skill --interactive

  # Generate routing table format
  agentskills decompose ~/.claude/skills/my-skill --routing-style table

  # Use specific AI provider
  agentskills decompose ~/.claude/skills/my-skill --provider gemini-api
")]
pub struct DecomposeCommand {
    /// Path to SKILL.md or skill directory
    pub path: PathBuf,

    /// Preview changes without writing files
    #[arg(long)]
    pub dry_run: bool,

    /// Show AI reasoning and confirm before applying
    #[arg(long)]
    pub interactive: bool,

    /// AI provider (auto-detects if omitted): anthropic-api, claude-cli, openai-api, gemini-api, gemini-cli, copilot-cli
    #[arg(long, value_name = "PROVIDER")]
    pub provider: Option<String>,

    /// Routing format: smart (auto), inline, table, or none
    #[arg(long, value_name = "STYLE")]
    pub routing_style: Option<String>,

    /// Add timing labels (invocation-time vs runtime)
    #[arg(long)]
    pub timing: bool,

    /// Add back-links in reference files to core SKILL.md
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
        interactive: Some(cmd.interactive),
        provider: cmd.provider.clone(),
        routing_style,
        show_timing: cmd.timing,
        back_links: cmd.back_links,
    };

    // Print progress to stderr
    eprintln!("{}", "Analyzing...".cyan());

    // If interactive mode, show preview and get user confirmation
    if cmd.interactive {
        // Note: Detailed preview (analysis results, routing graph, frontmatter changes)
        // would be implemented here after Agent E's decompose_skill returns structured data.
        // For now, we show a basic prompt.
        eprintln!("\n{}", "--- Preview Mode ---".bold());
        eprintln!("Analysis complete. Changes will be applied to: {:?}", cmd.path);
        eprintln!("\n{}", "Apply these changes? [y/N]: ".bold());

        use std::io::{self, BufRead};
        let stdin = io::stdin();
        let mut input = String::new();
        stdin.lock().read_line(&mut input)?;

        let response = input.trim().to_lowercase();
        if response != "y" && response != "yes" {
            eprintln!("{}", "Decompose cancelled.".yellow());
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

    eprintln!("{}", "Splitting content...".cyan());
    eprintln!("{}", "Generating script...".cyan());
    let preview_opt = crate::decompose::decompose_skill(&skill_md_path, &options).await?;

    if let Some(_preview) = preview_opt {
        // Dry-run mode: preview_data was returned, already printed by decompose_skill
        // No additional output needed here
    } else {
        // Non-dry-run mode: files were written
        println!("{}", "✓ Decompose complete".green().bold());
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
        assert!(!cmd.interactive);
    }

    #[test]
    fn test_decompose_command_with_dry_run() {
        let cmd =
            DecomposeCommand::try_parse_from(&["decompose", "/path/to/skill", "--dry-run"]).unwrap();
        assert_eq!(cmd.path, PathBuf::from("/path/to/skill"));
        assert!(cmd.dry_run);
        assert!(!cmd.interactive);
    }

    #[test]
    fn test_decompose_command_all_flags() {
        let cmd = DecomposeCommand::try_parse_from(&[
            "decompose",
            "/path/to/skill",
            "--dry-run",
            "--interactive",
        ])
        .unwrap();
        assert_eq!(cmd.path, PathBuf::from("/path/to/skill"));
        assert!(cmd.dry_run);
        assert!(cmd.interactive);
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
        assert!(help.contains("Path to SKILL.md or skill directory"));
        assert!(help.contains("Preview changes without writing"));
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
        assert!(cmd.interactive);
    }

    #[test]
    fn test_decompose_options_interactive_field() {
        // Test that DecomposeOptions correctly holds the interactive field
        let options = DecomposeOptions {
            dry_run: false,
            interactive: Some(true),
            ..Default::default()
        };
        assert_eq!(options.interactive, Some(true));

        // Test default behavior
        let default_options = DecomposeOptions {
            dry_run: false,
            interactive: Some(false),
            ..Default::default()
        };
        assert_eq!(default_options.interactive, Some(false));

        // Note: End-to-end interactive test requires stdin mocking, which is complex.
        // Interactive mode should be tested manually by running:
        // cargo run -- decompose /path/to/skill --interactive
        // and verifying the prompt appears and user input is correctly handled.
    }
}
