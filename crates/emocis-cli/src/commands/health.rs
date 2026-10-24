use colored::*;
use crate::common::open_or_init_store;
use emocis_memory::HealthScorer;
use std::env;

pub fn execute() -> anyhow::Result<()> {
    let cwd = env::current_dir()?;
    let project_name = cwd
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");

    let store = open_or_init_store()?;
    let scorer = HealthScorer::new(&store);
    let metrics = scorer.compute_health(project_name)?;

    println!(
        "\n{}",
        "==================================================".green()
    );
    println!(
        "{}",
        "   CODEBASE ARCHITECTURE HEALTH AUDIT (EMOCIS)    "
            .bold()
            .green()
    );
    println!(
        "{}\n",
        "==================================================".green()
    );

    println!("  {}", metrics.explanation);
    let graph = &metrics.graph;
    let percent = |value: Option<f64>| {
        value
            .map(|n| format!("{n:.1}%"))
            .unwrap_or_else(|| "N/A".to_string())
    };
    println!(
        "  • File / node / fungsi     : {} / {} / {}",
        graph.indexed_files, graph.indexed_nodes, graph.indexed_functions
    );
    println!(
        "  • Relasi terhubung        : {}/{} ({})",
        graph.resolved_calls,
        graph.call_relationships,
        percent(graph.resolution_percent)
    );
    println!("  • Relasi belum terhubung  : {}", graph.unresolved_calls);
    println!(
        "  • Relasi lintas file      : {} ({})",
        graph.cross_file_calls,
        percent(graph.cross_file_percent)
    );
    println!(
        "  • Kelompok siklus file    : {}",
        graph.dependency_cycles.len()
    );
    let mut files: Vec<_> = graph
        .files
        .iter()
        .filter(|f| f.fan_in + f.fan_out > 0)
        .collect();
    files.sort_by_key(|f| (std::cmp::Reverse(f.fan_in + f.fan_out), &f.file));
    for file in files.into_iter().take(5) {
        println!(
            "    {}: {} file masuk / {} file keluar",
            file.file, file.fan_in, file.fan_out
        );
    }

    if metrics.overall_score.is_some() {
        println!(
            "  • Overall Health Score : {}",
            format_score(metrics.overall_score).bold().yellow()
        );
        println!(
            "  • Modularity Score     : {}",
            format_score(metrics.modularity).cyan()
        );
        println!(
            "  • Cohesion Score       : {}",
            format_score(metrics.cohesion).cyan()
        );
        println!(
            "  • Coupling Score       : {}",
            format_score(metrics.coupling).cyan()
        );
        println!(
            "  • Boundary Integrity   : {}",
            format_score(metrics.boundary_integrity).cyan()
        );
    }
    println!(
        "  • Active Violations    : {}",
        format!("{} issues detected", metrics.active_violations_count)
            .bold()
            .red()
    );
    println!(
        "  • Technical Debt Index : {}",
        metrics.technical_debt_index.bold().yellow()
    );
    println!();

    Ok(())
}

fn format_score(score: Option<u32>) -> String {
    score
        .map(|s| format!("{s}/100"))
        .unwrap_or_else(|| "N/A (belum diukur)".to_string())
}
