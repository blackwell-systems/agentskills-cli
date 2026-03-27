use agentskills::commands::{LintCommand, UpgradeCommand};
use clap::{Parser, Subcommand};
use std::process;

#[derive(Parser)]
#[command(name = "agentskills")]
#[command(version, about = "Tool for validating and upgrading Agent Skills")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Lint(LintCommand),
    Upgrade(UpgradeCommand),
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Lint(cmd) => agentskills::commands::lint::run(&cmd),
        Commands::Upgrade(cmd) => agentskills::commands::upgrade::run(&cmd),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
