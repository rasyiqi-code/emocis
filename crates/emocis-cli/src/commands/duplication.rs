use colored::*;
use crate::common::{current_project_id, open_or_init_store};
use emocis_memory::DuplicationDetector;

pub fn execute() -> anyhow::Result<()> {
    let store = open_or_init_store()?;
    let detector = DuplicationDetector::new(&store);
    let results = detector.scan_duplications_for_project(&current_project_id()?)?;

    println!(
        "\n{}",
        "==================================================".cyan()
    );
    println!(
        "{}",
        "       HEURISTIC SIMILARITY CANDIDATES (PRD Bab 10)    "
            .bold()
            .cyan()
    );
    println!(
        "{}\n",
        "==================================================".cyan()
    );

    if results.is_empty() {
        println!(
            "{}",
            "✔ Tidak ada kandidat kemiripan berbasis kode yang terdeteksi.".green()
        );
    } else {
        println!(
            "{}",
            format!(
                "Ditemukan {} pasangan fungsi yang perlu ditinjau:",
                results.len()
            )
            .yellow()
        );
        for (idx, d) in results.iter().enumerate() {
            println!(
                "\n[{}] {} <───> {}",
                idx + 1,
                d.function_a.bold().yellow(),
                d.function_b.bold().yellow()
            );
            println!("    Lokasi A  : {}", d.location_a.dimmed());
            println!("    Lokasi B  : {}", d.location_b.dimmed());
            println!("    Klasifikasi: {}", d.similarity_level.cyan());
            println!("    Tanggung Jawab: {}", d.shared_responsibility);
            println!("    Saran     : {}", d.recommendation.green());
        }
    }
    println!();

    Ok(())
}
