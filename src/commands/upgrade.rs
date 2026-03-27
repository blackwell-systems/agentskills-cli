use crate::error::Error;
use crate::models::UpgradeOptions;
use clap::Parser;
use std::path::PathBuf;

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
}

pub fn run(cmd: &UpgradeCommand) -> Result<(), Error> {
    // Build upgrade options from command flags
    let options = UpgradeOptions {
        dry_run: cmd.dry_run,
        with_agent_references: cmd.with_agent_references,
    };

    // Print progress to stderr
    eprintln!("Analyzing...");

    // INTEGRATION NOTE: The upgrade module is being implemented by another Wave 2 agent.
    // After merge, uncomment the following lines and remove the temporary error:
    //
    // eprintln!("Splitting content...");
    // eprintln!("Generating script...");
    // crate::upgrade::upgrade_skill(&cmd.path, &options)?;
    // println!("✓ Upgrade complete");

    // Temporary placeholder until upgrade module is merged
    let _ = options; // Suppress unused variable warning
    let _ = cmd; // Suppress unused variable warning

    Err(Error::ValidationError(
        "Upgrade functionality will be available after Wave 2 upgrade module is merged".to_string(),
    ))
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
    }

    #[test]
    fn test_upgrade_command_with_dry_run() {
        let cmd =
            UpgradeCommand::try_parse_from(&["upgrade", "/path/to/skill", "--dry-run"]).unwrap();
        assert_eq!(cmd.path, PathBuf::from("/path/to/skill"));
        assert!(cmd.dry_run);
        assert!(!cmd.with_agent_references);
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
}
