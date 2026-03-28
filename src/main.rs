use agentskills::commands::{DecomposeCommand, LintCommand};
use clap::{Parser, Subcommand};
use std::process;

#[derive(Parser)]
#[command(name = "agentskills")]
#[command(version, about = "Tool for validating and decomposing Agent Skills")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Split large skills into core + reference files for better context economy
    Decompose(DecomposeCommand),
    /// Validate Agent Skill frontmatter and structure
    Lint(LintCommand),
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Decompose(cmd) => agentskills::commands::decompose::run(&cmd),
        Commands::Lint(cmd) => agentskills::commands::lint::run(&cmd),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
