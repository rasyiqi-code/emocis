use colored::*;
use crate::common::open_or_init_store;
use emocis_memory::DriftDetector;
use std::env;

pub fn execute() -> anyhow::Result<()> {
    let cwd = env::current_dir()?;
    let project_name = cwd
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");

    let store = open_or_init_store()?;
    let detector = DriftDetector::new(&store);
    let report = detector.run_audit(project_name)?;

    println!(
        "\n{}",
        "==================================================".red()
    );
    println!(
        "{}",
        "        DRIFT & BOUNDARY VIOLATION AUDIT          "
            .bold()
            .red()
    );
    println!(
        "{}\n",
        "==================================================".red()
    );

    if report.total_violations == 0 {
        println!("{}", "✔ Bersih! Tidak ditemukan drift dokumentasi maupun pelanggaran batas arsitektur.".bold().green());
    } else {
        println!(
            "{}",
            format!(
                "Ditemukan {} potensi drift/pelanggaran:",
                report.total_violations
            )
            .bold()
            .yellow()
        );
        for (idx, v) in report.violations.iter().enumerate() {
            let loc_str = v.location.as_deref().unwrap_or("Unknown location");
            println!("\n[{}] [{:?}] {}", idx + 1, v.severity, v.title.bold());
            println!("    Lokasi: {}", loc_str.cyan());
            println!("    Detail: {}", v.description);
            println!("    Saran : {}", v.recommendation.green());
        }
    }
    println!();

    Ok(())
}
