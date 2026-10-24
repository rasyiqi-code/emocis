use colored::*;
use crate::common::index_directory_ext;
use std::env;

pub fn execute(force: bool) -> anyhow::Result<()> {
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

    let stats = index_directory_ext(&cwd, project_name, force)?;

    println!("{}", "✔ Pemetaan selesai:".bold().green());
    println!("  • Total file dipindai : {}", stats.total_files.to_string().yellow());
    if stats.cached_files > 0 {
        println!("  • File cache (skip)   : {}", stats.cached_files.to_string().cyan());
    }
    println!("  • File baru/dimodif   : {}", stats.indexed_files.to_string().yellow());
    println!("  • Simbol logika (CCLG): {}", stats.total_nodes.to_string().yellow());
    println!(
        "  • Item pengetahuan    : {}",
        stats.total_knowledge.to_string().yellow()
    );

    Ok(())
}
