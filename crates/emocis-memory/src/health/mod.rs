use emocis_core::SqliteStore;
use serde::{Deserialize, Serialize};
use crate::drift::DriftDetector;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    pub overall_score: u32,
    pub modularity: u32,
    pub cohesion: u32,
    pub coupling: u32,
    pub boundary_integrity: u32,
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
        let drift_detector = DriftDetector::new(self.store);
        let audit = drift_detector.run_audit(project_id)?;

        // Deduct points based on violations
        let violation_penalty = (audit.total_violations * 15).min(50) as u32;

        let boundary_integrity = 100u32.saturating_sub(violation_penalty);
        let modularity = 85u32.saturating_sub((audit.total_violations * 5) as u32);
        let cohesion = 80u32;
        let coupling = 75u32;

        let overall_score = ((modularity + cohesion + coupling + boundary_integrity) / 4).min(100);

        let technical_debt_index = if overall_score >= 80 {
            "Low (Healthy)".to_string()
        } else if overall_score >= 60 {
            "Moderate (Monitor)".to_string()
        } else {
            "High (Action Required)".to_string()
        };

        Ok(HealthMetrics {
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
