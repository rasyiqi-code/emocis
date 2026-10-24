use colored::*;
use crate::common::open_or_init_store;
use emocis_memory::MemoryEngine;
use std::env;

pub fn execute() -> anyhow::Result<()> {
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
    println!(
        "  Total Pengetahuan Terekam: {}",
        total_items.to_string().yellow()
    );

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

    Ok(())
}
