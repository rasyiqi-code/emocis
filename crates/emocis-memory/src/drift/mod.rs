use emocis_core::{NodeKind, SqliteStore};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DriftSeverity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftViolation {
    pub severity: DriftSeverity,
    pub title: String,
    pub description: String,
    pub location: Option<String>,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftReport {
    pub total_violations: usize,
    pub violations: Vec<DriftViolation>,
}

pub struct DriftDetector<'a> {
    store: &'a SqliteStore,
}

impl<'a> DriftDetector<'a> {
    pub fn new(store: &'a SqliteStore) -> Self {
        Self { store }
    }

    /// Check documentation drift: verify that symbols mentioned in knowledge items/docs exist in code
    pub fn detect_doc_drift(&self, project_id: &str) -> anyhow::Result<Vec<DriftViolation>> {
        let mut violations = Vec::new();
        let items = self.store.list_knowledge_items(project_id)?;

        for item in items {
            for target_name in &item.affects {
                // If it's a file path or symbol name, check existence in nodes
                let matched_nodes = self.store.find_nodes_by_name(target_name)?;
                if matched_nodes.is_empty() && !target_name.ends_with(".ts") && !target_name.ends_with(".md") {
                    violations.push(DriftViolation {
                        severity: DriftSeverity::Medium,
                        title: format!("Documentation Drift: Entitas '{}' tidak ditemukan", target_name),
                        description: format!(
                            "Dokumentasi/Keputusan '{}' merujuk ke entitas '{}', namun simbol tersebut tidak ada lagi di kode aktif.",
                            item.title, target_name
                        ),
                        location: item.evidence.first().and_then(|e| e.file.clone()),
                        recommendation: format!("Perbarui referensi di '{}' atau periksa apakah simbol telah di-rename.", item.title),
                    });
                }
            }
        }

        Ok(violations)
    }

    /// Check architecture drift: circular dependencies and boundary violations
    pub fn detect_architecture_drift(&self, _project_id: &str) -> anyhow::Result<Vec<DriftViolation>> {
        let mut violations = Vec::new();

        // Check Controller -> Database direct bypass (PRD Part I Section 6 & Part II Section 32)
        // If a function name ends with 'Controller' and calls a database function directly
        let controller_nodes = self.store.find_nodes_by_name("Controller")?;
        for ctrl in controller_nodes {
            let mut check_targets = vec![ctrl.clone()];
            if ctrl.kind == NodeKind::Class {
                let methods = self.store.find_nodes_by_name(&format!("{}.", ctrl.name))?;
                check_targets.extend(methods);
            }

            for target in check_targets {
                let callees = self.store.get_callees(&target.id)?;
                for (_, callee) in callees {
                    let callee_lower = callee.name.to_lowercase();
                    if callee.kind == NodeKind::Database
                        || callee.kind == NodeKind::Table
                        || callee_lower.contains("database")
                        || callee_lower.contains("repository")
                    {
                        violations.push(DriftViolation {
                            severity: DriftSeverity::Critical,
                            title: "Architecture Violation: Layer Bypassed".to_string(),
                            description: format!(
                                "Controller '{}' ({}) memanggil database '{}' secara langsung tanpa melalui Service layer.",
                                ctrl.name, target.name, callee.name
                            ),
                            location: target.source.as_ref().map(|s| format!("{}:{}", s.file, s.start_line)),
                            recommendation: "Pindahkan pemanggilan database ke Service layer untuk menjaga pemisahan tanggung jawab (Clean Architecture).".to_string(),
                        });
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Run full audit
    pub fn run_audit(&self, project_id: &str) -> anyhow::Result<DriftReport> {
        let mut all_violations = Vec::new();
        all_violations.extend(self.detect_doc_drift(project_id)?);
        all_violations.extend(self.detect_architecture_drift(project_id)?);

        let total = all_violations.len();
        Ok(DriftReport {
            total_violations: total,
            violations: all_violations,
        })
    }
}
