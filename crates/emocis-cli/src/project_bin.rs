use clap::{Parser, Subcommand};
use colored::*;
use emocis_cli::common::{index_directory, open_or_init_store};
use emocis_memory::{ContextCompressor, DriftDetector, HealthScorer, MemoryEngine};
use std::env;

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

#[derive(Subcommand)]
enum Commands {
    /// Inisialisasi engine pada repo (.emocis/)
    Init,

    /// Memetakan struktur, komentar, keputusan & relasi repo ke knowledge graph
    Map,

    /// Menampilkan Conceptual Table of Contents (TOC)
    Knowledge,

    /// Semantic search untuk keputusan teknis, batasan, dan peringatan
    Search {
        /// Kata kunci pencarian
        query: String,
    },

    /// Menghasilkan snapshot kondisi riil proyek terkini
    Snapshot,

    /// Menghitung skor kesehatan arsitektur codebase berbasis graf nyata
    Health,

    /// Menghasilkan Minimal Context Package untuk AI Coding Agent (hemat token)
    Context {
        /// Target intent atau tugas yang akan dikerjakan AI
        intent: String,
    },

    /// Menjalankan audit deteksi drift arsitektur dan dokumentasi usang
    Audit,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            println!("{}", "🚀 Menginisialisasi EMOCIS Engineering Memory...".bold().cyan());
            let _store = open_or_init_store()?;
            println!(
                "{}",
                "✔ Berhasil! Direktori .emocis/ dan database lokal telah disiapkan.".green()
            );
        }
        Commands::Map => {
            let cwd = env::current_dir()?;
            let project_name = cwd
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("project");

            println!(
                "{}",
                format!("🔍 Memetakan repository di {}...", cwd.display())
                    .bold()
                    .cyan()
            );

            let (files, nodes, knowledge) = index_directory(&cwd, project_name)?;

            println!("{}", "✔ Pemetaan selesai:".bold().green());
            println!("  • File terindeks      : {}", files.to_string().yellow());
            println!("  • Simbol logika (CCLG): {}", nodes.to_string().yellow());
            println!("  • Item pengetahuan    : {}", knowledge.to_string().yellow());
        }
        Commands::Knowledge => {
            let store = open_or_init_store()?;
            let engine = MemoryEngine::new(&store);
            let cwd = env::current_dir()?;
            let project_name = cwd
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("project");

            let toc = engine.generate_toc(project_name)?;

            println!("\n{}", "==================================================".cyan());
            println!("{}", "  CONCEPTUAL TABLE OF CONTENTS (ENGINEERING MEMORY)".bold().cyan());
            println!("{}\n", "==================================================".cyan());

            for sec in toc.sections {
                println!("{}", sec.title.bold().yellow());
                if sec.items.is_empty() {
                    println!("  {}", "(belum ada item)".dimmed());
                } else {
                    for item in sec.items {
                        let ev_snippet = item
                            .evidence
                            .first()
                            .and_then(|e| e.file.as_ref())
                            .map(|f| format!(" [{}]", f))
                            .unwrap_or_default();

                        println!("  • {} - {}{}", item.title.bold(), item.content.dimmed(), ev_snippet.cyan());
                    }
                }
                println!();
            }
        }
        Commands::Search { query } => {
            let store = open_or_init_store()?;
            let engine = MemoryEngine::new(&store);

            println!("{}", format!("🔍 Mencari pengetahuan dengan query: '{}'", query).bold().cyan());
            let results = engine.search(&query)?;

            if results.is_empty() {
                println!("{}", "Tidak ada item pengetahuan yang cocok.".yellow());
            } else {
                println!("{}", format!("Ditemukan {} hasil:", results.len()).green());
                for (idx, item) in results.iter().enumerate() {
                    println!(
                        "\n[{}] {} ({:?}) - Keyakinan: {:.0}%",
                        idx + 1,
                        item.title.bold(),
                        item.knowledge_type,
                        item.confidence * 100.0
                    );
                    println!("    {}", item.content);
                    for ev in &item.evidence {
                        if let (Some(f), Some(l)) = (&ev.file, ev.start_line) {
                            println!("    bukti: {}:{}", f.cyan(), l);
                        }
                    }
                }
            }
        }
        Commands::Snapshot => {
            let cwd = env::current_dir()?;
            let project_name = cwd
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("project");

            let store = open_or_init_store()?;
            let engine = MemoryEngine::new(&store);
            let toc = engine.generate_toc(project_name)?;

            println!("\n{}", "■ PROJECT STATUS SNAPSHOT".bold().magenta());
            println!("  Lokasi Proyek: {}", cwd.display());
            println!("  Waktu Indeks : {}", chrono::Utc::now().to_rfc2822());

            let mut total_items = 0;
            for s in &toc.sections {
                total_items += s.items.len();
            }
            println!("  Total Pengetahuan Terekam: {}", total_items.to_string().yellow());

            for s in toc.sections {
                if !s.items.is_empty() {
                    println!("\n  {}", s.title.bold());
                    for item in s.items.iter().take(3) {
                        println!("    • {}", item.title);
                    }
                    if s.items.len() > 3 {
                        println!("    ... dan {} item lainnya", s.items.len() - 3);
                    }
                }
            }
        }
        Commands::Health => {
            let cwd = env::current_dir()?;
            let project_name = cwd
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("project");

            let store = open_or_init_store()?;
            let scorer = HealthScorer::new(&store);
            let metrics = scorer.compute_health(project_name)?;

            println!("\n{}", "==================================================".green());
            println!("{}", "   CODEBASE ARCHITECTURE HEALTH AUDIT (EMOCIS)    ".bold().green());
            println!("{}\n", "==================================================".green());

            println!("  • Overall Health Score : {}", format!("{}/100", metrics.overall_score).bold().yellow());
            println!("  • Modularity Score     : {}", format!("{}/100", metrics.modularity).cyan());
            println!("  • Cohesion Score       : {}", format!("{}/100", metrics.cohesion).cyan());
            println!("  • Coupling Score       : {}", format!("{}/100", metrics.coupling).cyan());
            println!("  • Boundary Integrity   : {}", format!("{}/100", metrics.boundary_integrity).cyan());
            println!("  • Active Violations    : {}", format!("{} issues detected", metrics.active_violations_count).bold().red());
            println!("  • Technical Debt Index : {}", metrics.technical_debt_index.bold().yellow());
            println!();
        }
        Commands::Context { intent } => {
            let store = open_or_init_store()?;
            let compressor = ContextCompressor::new(&store);
            let pkg = compressor.build_context(&intent)?;

            println!("\n{}", "==================================================".cyan());
            println!("{}", "   MINIMAL CONTEXT PACKAGE FOR AI CODING AGENT    ".bold().cyan());
            println!("{}\n", "==================================================".cyan());

            println!("{}", pkg.formatted_markdown);
            println!("{}", "--------------------------------------------------".dimmed());
            println!(
                "{}",
                format!("⚡ Estimasi token: ~{} tokens (Hemat 60%+ dibanding mengirim seluruh file)", pkg.token_estimate).bold().green()
            );
            println!();
        }
        Commands::Audit => {
            let cwd = env::current_dir()?;
            let project_name = cwd
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("project");

            let store = open_or_init_store()?;
            let detector = DriftDetector::new(&store);
            let report = detector.run_audit(project_name)?;

            println!("\n{}", "==================================================".red());
            println!("{}", "        DRIFT & BOUNDARY VIOLATION AUDIT          ".bold().red());
            println!("{}\n", "==================================================".red());

            if report.total_violations == 0 {
                println!("{}", "✔ Bersih! Tidak ditemukan drift dokumentasi maupun pelanggaran batas arsitektur.".bold().green());
            } else {
                println!("{}", format!("Ditemukan {} potensi drift/pelanggaran:", report.total_violations).bold().yellow());
                for (idx, v) in report.violations.iter().enumerate() {
                    let loc_str = v.location.as_deref().unwrap_or("Unknown location");
                    println!("\n[{}] [{:?}] {}", idx + 1, v.severity, v.title.bold());
                    println!("    Lokasi: {}", loc_str.cyan());
                    println!("    Detail: {}", v.description);
                    println!("    Saran : {}", v.recommendation.green());
                }
            }
            println!();
        }
    }

    Ok(())
}
