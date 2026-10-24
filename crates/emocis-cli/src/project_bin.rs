use clap::Parser;
use emocis_cli::commands::{run, Commands};

#[derive(Parser)]
#[command(
    name = "project",
    author = "EMOCIS Team",
    version = "0.1.0",
    about = "Engineering Memory & Codebase Knowledge System (PRD Part I)"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    run(cli.command)
}
