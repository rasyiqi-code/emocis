use colored::*;
use crate::common::{current_project_id, open_or_init_store};
use emocis_memory::RefactoringRadar;
use std::env;

pub fn execute(file: Option<String>) -> anyhow::Result<()> {
    let store = open_or_init_store()?;
    let radar = RefactoringRadar::new(&store);

    println!(
        "\n{}",
        "==================================================".magenta()
    );
    println!(
        "{}",
        "           REFACTORING RADAR (PRD Bab 28)         "
            .bold()
            .magenta()
    );
    println!(
        "{}\n",
        "==================================================".magenta()
    );

    match file {
        Some(target_file) => {
            let cwd = env::current_dir()?;
            let supplied = std::path::Path::new(&target_file);
            let full_path = if supplied.is_absolute() {
                supplied.to_path_buf()
            } else {
                cwd.join(supplied)
            };
            let relative_path = full_path
                .strip_prefix(&cwd)
                .unwrap_or(&full_path)
                .to_string_lossy()
                .replace('\\', "/");
            let content = std::fs::read_to_string(&full_path).map_err(|e| {
                anyhow::anyhow!("Gagal membaca file '{}': {}", target_file, e)
            })?;
            let diag = radar.diagnose_file(&relative_path, &content)?;

            println!("Target File: {}", diag.file_path.bold().yellow());
            println!(
                "  • Baris (semua / nonkosong): {} / {}",
                diag.loc, diag.non_blank_lines
            );
            println!(
                "  • Jumlah Fungsi             : {}",
                diag.function_count.to_string().cyan()
            );
            println!(
                "  • Kompleksitas maksimum    : {} ({})",
                diag.cyclomatic_complexity_est
                    .map_or("N/A".to_string(), |n| n.to_string()),
                diag.complexity_method
            );
            if let Some(target) = &diag.complexity_target {
                println!("    Fungsi: {}", target);
            }
            println!(
                "  • Klaster Konsep Terdeteksi : {} klaster leksikal",
                diag.distinct_conceptual_clusters.to_string().cyan()
            );
            for d in &diag.top_concept_clusters {
                println!("    - {}", d);
            }
            println!(
                "  • Incoming Dependents       : {}",
                diag.incoming_dependents_count
            );
            println!(
                "  • Outgoing Calls            : {}",
                diag.outgoing_calls_count
            );
            println!(
                "  • Severity                  : {}",
                diag.severity.bold().red()
            );
            println!("\n  💡 {}", "REKOMENDASI:".bold().green());
            println!("  {}", diag.recommendation);
        }
        None => {
            println!(
                "{}",
                "Memindai proyek terindeks ini untuk sinyal ukuran...".cyan()
            );
            let candidates =
                radar.scan_bloat_candidates_for_project(&current_project_id()?)?;
            if candidates.is_empty() {
                println!(
                    "{}",
                    "✔ Tidak ada sinyal ukuran yang melewati ambang tinjau.".green()
                );
            } else {
                for c in candidates {
                    println!(
                        "\n• {} ({} baris, {} fungsi) - [{}]",
                        c.file_path.bold(),
                        c.loc,
                        c.function_count,
                        c.severity.bold().red()
                    );
                    println!("  Klaster Konsep: {}", c.top_concept_clusters.join(", "));
                    println!("  Saran : {}", c.recommendation.dimmed());
                }
            }
        }
    }
    println!();

    Ok(())
}
