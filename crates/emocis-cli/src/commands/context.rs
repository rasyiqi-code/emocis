use colored::*;
use crate::common::{current_project_id, open_or_init_store};
use emocis_memory::ContextCompressor;

pub fn execute(intent: &str) -> anyhow::Result<()> {
    let store = open_or_init_store()?;
    let compressor = ContextCompressor::new(&store);
    let pkg = compressor.build_context_for_project(&current_project_id()?, intent)?;

    println!(
        "\n{}",
        "==================================================".cyan()
    );
    println!(
        "{}",
        "   MINIMAL CONTEXT PACKAGE FOR AI CODING AGENT    "
            .bold()
            .cyan()
    );
    println!(
        "{}\n",
        "==================================================".cyan()
    );

    println!("{}", pkg.formatted_markdown);
    println!(
        "{}",
        "--------------------------------------------------".dimmed()
    );
    println!(
        "{}",
        format!(
            "Estimasi token: ~{} tokens (perkiraan; penghematan tidak diukur)",
            pkg.token_estimate
        )
        .bold()
        .green()
    );
    println!();

    Ok(())
}
