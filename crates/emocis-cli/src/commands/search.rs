use colored::*;
use crate::common::{current_project_id, open_or_init_store};
use emocis_memory::MemoryEngine;

pub fn execute(query: &str) -> anyhow::Result<()> {
    let store = open_or_init_store()?;
    let engine = MemoryEngine::new(&store);

    println!(
        "{}",
        format!("🔍 Mencari pengetahuan dengan query: '{}'", query)
            .bold()
            .cyan()
    );
    let results = engine.search_for_project(&current_project_id()?, query)?;

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

    Ok(())
}
