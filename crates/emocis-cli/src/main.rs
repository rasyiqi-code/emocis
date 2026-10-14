mod common;

use clap::{Parser, Subcommand};
use colored::*;

#[derive(Parser)]
#[command(
    name = "emocis",
    author = "EMOCIS Team",
    version = "0.1.0",
    about = "Engineering Memory & Codebase Intelligence System (Unified CLI)"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Akses Engineering Memory Layer (Pilar I - Why & Evolution)
    Project {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Akses Canonical Code Logic Graph (Pilar II - How & Behavior)
    Clg {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

fn main() {
    println!("{}", "=======================================================".cyan());
    println!("{}", "   EMOCIS: Engineering Memory & Codebase Intelligence   ".bold().cyan());
    println!("{}", "=======================================================".cyan());
    println!("Gunakan binary langsung:");
    println!("  • {}  : Mengelola keputusan, batasan & ringkasan memori", "project".bold().green());
    println!("  • {}      : Menelusuri alur logika kode, panggilan & dampak", "clg".bold().yellow());
    println!("\nContoh:");
    println!("  $ project init");
    println!("  $ project map");
    println!("  $ project knowledge");
    println!("  $ clg calls <FUNCTION>");
    println!("  $ clg impact <SYMBOL>");
}
