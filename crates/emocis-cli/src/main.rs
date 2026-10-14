use clap::{Parser, Subcommand};
use colored::*;
use emocis_cli::mcp::run_mcp_server;

#[derive(Parser)]
#[command(
    name = "emocis",
    author = "EMOCIS Team",
    version = "0.1.0",
    about = "Engineering Memory & Codebase Intelligence System (Unified CLI & MCP Server)"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Menjalankan Model Context Protocol (MCP) Server untuk AI Coding Agent via stdio
    Mcp,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Mcp) => {
            // Run MCP stdio server
            run_mcp_server()?;
        }
        None => {
            println!("{}", "=======================================================".cyan());
            println!("{}", "   EMOCIS: Engineering Memory & Codebase Intelligence   ".bold().cyan());
            println!("{}", "=======================================================".cyan());
            println!("Interface Utama:");
            println!("  • {}      : Model Context Protocol (MCP) Server untuk AI Coding Agent", "emocis mcp".bold().magenta());
            println!("  • {}      : Mengelola keputusan, batasan & ringkasan memori", "project".bold().green());
            println!("  • {}          : Menelusuri alur logika kode, panggilan & dampak", "clg".bold().yellow());
            println!("\nContoh Perintah:");
            println!("  $ project init");
            println!("  $ project map");
            println!("  $ project knowledge");
            println!("  $ project context \"refactor payment\"");
            println!("  $ project health");
            println!("  $ project audit");
            println!("  $ clg calls checkout");
            println!("  $ clg callers charge");
            println!("  $ clg impact charge");
            println!("  $ clg path checkout charge");
            println!("  $ emocis mcp");
        }
    }

    Ok(())
}
