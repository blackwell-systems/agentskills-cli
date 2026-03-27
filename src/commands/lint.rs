use crate::error::Error;
use crate::validation;
use clap::Parser;
use colored::Colorize;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct LintCommand {
    /// Path to Agent Skill directory
    pub path: PathBuf,

    /// Output as JSON instead of colored text
    #[arg(long)]
    pub json: bool,
}

pub fn run(cmd: &LintCommand) -> Result<(), Error> {
    // Run validation
    let result = validation::validate_skill(&cmd.path)?;

    if cmd.json {
        // JSON output
        let json_value = serde_json::json!({
            "valid": result.is_valid(),
            "errors": result.errors.iter().map(|e| {
                serde_json::json!({
                    "type": e.error_type,
                    "message": e.message,
                    "file": e.file.as_ref().map(|f| f.display().to_string()),
                    "line": e.line,
                    "severity": "error",
                })
            }).collect::<Vec<_>>(),
            "warnings": result.warnings.iter().map(|w| {
                serde_json::json!({
                    "type": w.error_type,
                    "message": w.message,
                    "file": w.file.as_ref().map(|f| f.display().to_string()),
                    "line": w.line,
                    "severity": "warning",
                })
            }).collect::<Vec<_>>(),
        });
        let json = serde_json::to_string_pretty(&json_value)
            .map_err(|e| Error::ParseError(format!("Failed to serialize JSON: {}", e)))?;
        println!("{}", json);
    } else {
        // Colored text output
        print!("{}", result.format_output());

        // Print summary
        if result.is_valid() {
            println!("{}", "✓ Valid Agent Skill".green().bold());
        } else {
            println!("{}", "✗ Validation failed".red().bold());
        }
    }

    // Exit with error if validation failed
    if !result.is_valid() {
        return Err(Error::ValidationError(
            "Validation failed with errors".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_lint_command_parsing() {
        let cmd = LintCommand::try_parse_from(&["lint", "/path/to/skill"]).unwrap();
        assert_eq!(cmd.path, PathBuf::from("/path/to/skill"));
        assert!(!cmd.json);
    }

    #[test]
    fn test_lint_command_with_json_flag() {
        let cmd = LintCommand::try_parse_from(&["lint", "/path/to/skill", "--json"]).unwrap();
        assert_eq!(cmd.path, PathBuf::from("/path/to/skill"));
        assert!(cmd.json);
    }

    #[test]
    fn test_lint_command_missing_path() {
        let result = LintCommand::try_parse_from(&["lint"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_lint_command_help() {
        let mut cmd = LintCommand::command();
        let help = cmd.render_help().to_string();
        assert!(help.contains("Path to Agent Skill directory"));
        assert!(help.contains("Output as JSON"));
    }
}
