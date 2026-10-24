pub mod graph;
use crate::drift::DriftDetector;
use emocis_core::SqliteStore;
pub use graph::{FileDependencies, GraphMeasurements};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    pub graph: GraphMeasurements,
    pub status: String,
    pub explanation: String,
    pub indexed_symbols: usize,
    pub overall_score: Option<u32>,
    pub modularity: Option<u32>,
    pub cohesion: Option<u32>,
    pub coupling: Option<u32>,
    pub boundary_integrity: Option<u32>,
    pub technical_debt_index: String,
    pub active_violations_count: usize,
}

pub struct HealthScorer<'a> {
    store: &'a SqliteStore,
}

impl<'a> HealthScorer<'a> {
    pub fn new(store: &'a SqliteStore) -> Self {
        Self { store }
    }

    pub fn compute_health(&self, project_id: &str) -> anyhow::Result<HealthMetrics> {
        let graph = GraphMeasurements::compute(self.store, project_id)?;
        let drift_detector = DriftDetector::new(self.store);
        let audit = drift_detector.run_audit_with_graph(project_id, &graph)?;
        let indexed_symbols = graph.indexed_nodes;

        let (overall_score, modularity, cohesion, coupling, boundary_integrity, technical_debt_index) =
            if graph.indexed_files > 0 {
                // 1. Modularity (0 - 100):
                // Measures module segregation and absence of cyclic dependencies.
                // Each cycle is a severe modularity violation (-25 per cycle component).
                let cycle_penalty = (graph.dependency_cycles.len() as u32) * 25;
                let god_files_count = graph
                    .files
                    .iter()
                    .filter(|f| f.fan_in + f.fan_out > 30)
                    .count() as u32;
                let mod_score = 100u32
                    .saturating_sub(cycle_penalty)
                    .saturating_sub(god_files_count * 5)
                    .clamp(0, 100);

                // 2. Cohesion (0 - 100):
                // Internal relatedness: percentage of calls staying within the same file/subsystem.
                let coh_score = if graph.resolved_calls > 0 {
                    let internal_ratio = graph.internal_file_calls as f64 / graph.resolved_calls as f64;
                    (50.0 + internal_ratio * 50.0).round() as u32
                } else {
                    75
                }.clamp(0, 100);

                // 3. Coupling (0 - 100):
                // Clean separation: low tangled cross-file cycles and balanced fan-out.
                let excessive_cross_file = if let Some(pct) = graph.cross_file_percent {
                    if pct > 85.0 { 15 } else { 0 }
                } else {
                    0
                };
                let coup_score = 100u32
                    .saturating_sub(cycle_penalty)
                    .saturating_sub(excessive_cross_file)
                    .clamp(0, 100);

                // 4. Boundary Integrity (0 - 100):
                // Layer boundary preservation, penalized by active architectural drift issues.
                let drift_penalty = (audit.total_violations as u32) * 5;
                let bound_score = 100u32
                    .saturating_sub(drift_penalty)
                    .saturating_sub(cycle_penalty / 2)
                    .clamp(0, 100);

                // 5. Overall Health Score (0 - 100):
                let overall = ((mod_score as f64 * 0.30)
                    + (coh_score as f64 * 0.25)
                    + (coup_score as f64 * 0.25)
                    + (bound_score as f64 * 0.20))
                    .round() as u32;
                let overall = overall.clamp(0, 100);

                // 6. Technical Debt Index
                let debt_index = if overall >= 80 {
                    "Low (Maintainable)".to_string()
                } else if overall >= 60 {
                    "Moderate (Technical Debt Accruing)".to_string()
                } else {
                    "High (Immediate Refactoring Recommended)".to_string()
                };

                (
                    Some(overall),
                    Some(mod_score),
                    Some(coh_score),
                    Some(coup_score),
                    Some(bound_score),
                    debt_index,
                )
            } else {
                (
                    None,
                    None,
                    None,
                    None,
                    None,
                    "Not assessed".to_string(),
                )
            };

        Ok(HealthMetrics {
            status: if graph.indexed_files == 0 { "not_indexed" } else { "calibrated_static_analysis" }.to_string(),
            graph,
            explanation: "Calibrated codebase architectural health based on CCLG static graph analysis, modularity, coupling, cohesion, and boundary integrity.".to_string(),
            indexed_symbols,
            overall_score,
            modularity,
            cohesion,
            coupling,
            boundary_integrity,
            technical_debt_index,
            active_violations_count: audit.total_violations,
        })
    }
}
