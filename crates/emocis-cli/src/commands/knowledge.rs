use colored::*;
use crate::common::open_or_init_store;
use emocis_memory::MemoryEngine;
use std::env;

pub fn execute() -> anyhow::Result<()> {
    let store = open_or_init_store()?;
    let engine = MemoryEngine::new(&store);
    let cwd = env::current_dir()?;
    let project_name = cwd
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");

    let toc = engine.generate_toc(project_name)?;

    println!(
        "\n{}",
        "==================================================".cyan()
    );
    println!(
        "{}",
        "  CONCEPTUAL TABLE OF CONTENTS (ENGINEERING MEMORY)"
            .bold()
            .cyan()
    );
    println!(
        "{}\n",
        "==================================================".cyan()
    );

    let print_group = |header: &str, icon: &str, sections: &[emocis_memory::TocSection]| {
        println!("{}", format!("{} {}", icon, header).bold().bright_white().underline());
        println!();
        let mut has_any = false;
        for sec in sections {
            if !sec.items.is_empty() {
                has_any = true;
                println!("  {}", sec.title.bold().yellow());
                for item in &sec.items {
                    let ev_snippet = item
                        .evidence
                        .first()
                        .and_then(|e| {
                            let f = e.file.as_deref().unwrap_or("");
                            let line = e.start_line.unwrap_or(1);
                            if !f.is_empty() {
                                Some(format!(" [{}:{}]", f, line))
                            } else {
                                None
                            }
                        })
                        .unwrap_or_default();

                    let first_line = item.content.lines().next().unwrap_or("").trim();
                    let snippet_display = if first_line.chars().count() > 80 {
                        format!("{}...", first_line.chars().take(77).collect::<String>())
                    } else {
                        first_line.to_string()
                    };

                    println!(
                        "    • {} - {}{}",
                        item.title.bold(),
                        snippet_display.dimmed(),
                        ev_snippet.cyan()
                    );
                }
                println!();
            }
        }
        if !has_any {
            println!("  {}", "(tidak ada item)".dimmed());
            println!();
        }
    };

    print_group(
        "PROJECT DOCUMENTATION & SPECIFICATIONS (MARKDOWN)",
        "📄",
        &toc.doc_sections,
    );
    print_group(
        "SOURCE CODE ANNOTATIONS & INLINE MEMORY",
        "💻",
        &toc.code_sections,
    );

    Ok(())
}
