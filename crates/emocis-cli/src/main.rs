use clap::Parser;
use colored::*;

#[derive(Parser)]
#[command(
    name = "emocis",
    author = "EMOCIS Team",
    version = "0.1.0",
    about = "Engineering Memory & Codebase Intelligence System (Unified CLI)"
)]
struct Cli {}

fn main() {
    let _ = Cli::parse();

    println!("{}", "=======================================================".cyan());
    println!("{}", "   EMOCIS: Engineering Memory & Codebase Intelligence   ".bold().cyan());
    println!("{}", "=======================================================".cyan());
    println!("Gunakan binary langsung:");
    println!("  • {}  : Mengelola keputusan, batasan & ringkasan memori", "project".bold().green());
    println!("  • {}      : Menelusuri alur logika kode, panggilan & dampak", "clg".bold().yellow());
    println!("\nContoh Perintah:");
    println!("  $ project init");
    println!("  $ project map");
    println!("  $ project knowledge");
    println!("  $ project search \"idempotency\"");
    println!("  $ clg calls checkout");
    println!("  $ clg callers charge");
    println!("  $ clg impact charge");
    println!("  $ clg path checkout charge");
}
