use colored::*;
use crate::common::open_or_init_store;
use emocis_memory::FailedAttemptsManager;
use std::env;

pub fn execute(
    approach: Option<String>,
    reason: Option<String>,
    replaced_by: Option<String>,
    provenance: Option<String>,
) -> anyhow::Result<()> {
    let cwd = env::current_dir()?;
    let project_name = cwd
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");

    let store = open_or_init_store()?;
    let manager = FailedAttemptsManager::new(&store);

    println!(
        "\n{}",
        "==================================================".yellow()
    );
    println!(
        "{}",
        "       FAILED ATTEMPTS MEMORY (PRD Bab 13)        "
            .bold()
            .yellow()
    );
    println!(
        "{}\n",
        "==================================================".yellow()
    );

    match (approach, reason, replaced_by, provenance) {
        (Some(app), Some(rsn), Some(repl), Some(prov)) => {
            let id =
                manager.record_failure(project_name, &app, &rsn, &repl, &prov, vec![])?;
            println!(
                "{}",
                "✔ Berhasil mencatat eksperimen yang gagal!".bold().green()
            );
            println!("  ID       : {}", id.cyan());
            println!("  Pendekatan: {}", app.bold());
            println!("  Alasan   : {}", rsn);
            println!("  Solusi   : {}", repl.green());
            println!("  Bukti    : {}", prov);
            println!("\nEngineer lain dan AI Coding Agent kini tidak akan mengulangi kesalahan yang sama.");
        }
        _ => {
            let failures = manager.list_failures(project_name)?;
            if failures.is_empty() {
                println!(
                    "{}",
                    "(Belum ada catatan eksperimen gagal yang terekam)".dimmed()
                );
                println!("\nUntuk mencatat pendekatan yang gagal gunakan:");
                println!("  $ project failed --approach \"<PENDEKATAN>\" --reason \"<ALASAN>\" --replaced-by \"<SOLUSI>\" --provenance \"<PR/COMMIT>\"");
            } else {
                for (idx, f) in failures.iter().enumerate() {
                    println!("[{}] {}", idx + 1, f.approach.bold().red());
                    println!("    Detail: {}", f.reason_rejected);
                    if !f.provenance_source.is_empty() {
                        println!("    Bukti : {}", f.provenance_source.cyan());
                    }
                    println!();
                }
            }
        }
    }
    println!();

    Ok(())
}
