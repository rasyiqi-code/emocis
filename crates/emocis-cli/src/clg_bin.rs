mod common;

use clap::{Parser, Subcommand};
use colored::*;
use common::open_or_init_store;
use emocis_cclg::GraphAnalyzer;

#[derive(Parser)]
#[command(
    name = "clg",
    author = "EMOCIS Team",
    version = "0.1.0",
    about = "Canonical Code Logic Graph Query Engine (PRD Part II)"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Menampilkan fungsi yang dipanggil oleh target (Callees)
    Calls {
        /// Nama atau qualified name fungsi pemanggil
        function: String,
    },

    /// Menampilkan fungsi yang memanggil target (Callers)
    Callers {
        /// Nama atau qualified name fungsi yang dipanggil
        function: String,
    },

    /// Menganalisis dampak perubahan simbol (Impact Analysis)
    Impact {
        /// Simbol fungsi/variabel yang ingin dianalisis
        symbol: String,
    },

    /// Mencari jalur eksekusi antara dua simbol logika
    Path {
        /// Titik awal eksekusi
        source: String,
        /// Titik tujuan
        target: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let store = open_or_init_store()?;
    let analyzer = GraphAnalyzer::new(&store);

    match cli.command {
        Commands::Calls { function } => {
            println!("{}", format!("🔍 Mencari fungsi yang dipanggil oleh '{}'...", function).bold().cyan());
            let candidates = store.find_nodes_by_name(&function)?;
            if candidates.is_empty() {
                println!("{}", "Simbol tidak ditemukan di dalam graf CCLG. Jalankan `project map` terlebih dahulu.".yellow());
                return Ok(());
            }

            for node in candidates {
                println!("\nSimbol: {} ({:?}) - {}", node.name.bold(), node.kind, node.identity.qualified_name.cyan());
                let callees = store.get_callees(&node.id)?;
                if callees.is_empty() {
                    println!("  {}", "(tidak memanggil fungsi lain)".dimmed());
                } else {
                    for (edge, callee) in callees {
                        let loc = callee.source.map(|s| format!(" ({}:{})", s.file, s.start_line)).unwrap_or_default();
                        println!("  ├── [calls] ──> {}{}", callee.name.bold().green(), loc.dimmed());
                        if !edge.qualifier.is_null() {
                            println!("  │   qualifier: {}", edge.qualifier.to_string().dimmed());
                        }
                    }
                }
            }
        }
        Commands::Callers { function } => {
            println!("{}", format!("🔍 Mencari fungsi yang memanggil '{}'...", function).bold().cyan());
            let candidates = store.find_nodes_by_name(&function)?;
            if candidates.is_empty() {
                println!("{}", "Simbol tidak ditemukan di dalam graf CCLG. Jalankan `project map` terlebih dahulu.".yellow());
                return Ok(());
            }

            for node in candidates {
                println!("\nTarget: {} ({:?}) - {}", node.name.bold(), node.kind, node.identity.qualified_name.cyan());
                let callers = store.get_callers(&node.id)?;
                if callers.is_empty() {
                    println!("  {}", "(tidak ada pemanggil terdeteksi)".dimmed());
                } else {
                    for (_, caller) in callers {
                        let loc = caller.source.map(|s| format!(" ({}:{})", s.file, s.start_line)).unwrap_or_default();
                        println!("  └── [called by] <── {}{}", caller.name.bold().magenta(), loc.dimmed());
                    }
                }
            }
        }
        Commands::Impact { symbol } => {
            println!("{}", format!("⚡ Menganalisis dampak perubahan '{}'...", symbol).bold().cyan());
            let report = analyzer.analyze_impact(&symbol)?;

            if report.target_node.is_none() {
                println!("{}", "Simbol tidak ditemukan di graf. Pastikan Anda telah menjalankan `project map`.".yellow());
                return Ok(());
            }

            println!("\n{}", "==================================================".magenta());
            println!("  IMPACT ANALYSIS REPORT: {}", report.target_symbol.bold().yellow());
            println!("{}\n", "==================================================".magenta());

            println!("{}", "DIRECT DEPENDENTS:".bold().cyan());
            if report.direct_callers.is_empty() {
                println!("  {}", "(None)".dimmed());
            } else {
                for caller in &report.direct_callers {
                    let loc = caller.source.as_ref().map(|s| format!(" [{}:{}]", s.file, s.start_line)).unwrap_or_default();
                    println!("  • {} ({:?}){}", caller.name.bold(), caller.kind, loc.dimmed());
                }
            }

            println!("\n{}", "INDIRECT CALLERS (Depth 2+):".bold().cyan());
            if report.indirect_callers.is_empty() {
                println!("  {}", "(None)".dimmed());
            } else {
                for caller in &report.indirect_callers {
                    println!("  • {} ({:?})", caller.name, caller.kind);
                }
            }

            println!("\n{}", "EXPOSED APIS / ENDPOINTS:".bold().cyan());
            if report.exposed_endpoints.is_empty() {
                println!("  {}", "(Tidak ada endpoint publik yang terdampak langsung)".dimmed());
            } else {
                for ep in &report.exposed_endpoints {
                    println!("  • {}", ep.name.bold().red());
                }
            }

            println!("\n{}", "AFFECTED DATA STORES:".bold().cyan());
            if report.affected_datastores.is_empty() {
                println!("  {}", "(None)".dimmed());
            } else {
                for ds in &report.affected_datastores {
                    println!("  • {}", ds.bold().yellow());
                }
            }
            println!();
        }
        Commands::Path { source, target } => {
            println!("{}", format!("🔍 Mencari alur eksekusi dari '{}' ke '{}'...", source, target).bold().cyan());
            let result = analyzer.find_path(&source, &target)?;

            if !result.found {
                println!("{}", "Tidak ditemukan lintasan eksekusi yang menghubungkan kedua simbol.".yellow());
            } else {
                println!("{}", "\n✔ Lintasan eksekusi ditemukan:".bold().green());
                for (idx, step) in result.path.iter().enumerate() {
                    let arrow = if idx == 0 { "  Start: " } else { "  ──> " };
                    let loc = step.source.as_ref().map(|s| format!(" ({}:{})", s.file, s.start_line)).unwrap_or_default();
                    println!("{}{}{}", arrow.bold().cyan(), step.name.bold(), loc.dimmed());
                }
                println!();
            }
        }
    }

    Ok(())
}
