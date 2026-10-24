use crate::health::GraphMeasurements;
use emocis_core::SqliteStore;
use emocis_core::{EdgeKind, NodeKind};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

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
                let matched_nodes: Vec<_> = self
                    .store
                    .find_nodes_by_name(target_name)?
                    .into_iter()
                    .filter(|n| n.project_id == project_id)
                    .collect();
                if matched_nodes.is_empty()
                    && !target_name.ends_with(".ts")
                    && !target_name.ends_with(".rs")
                    && !target_name.ends_with(".md")
                {
                    violations.push(DriftViolation {
                        severity: DriftSeverity::Medium,
                        title: format!("Documentation Drift: Entitas '{}' tidak ditemukan", target_name),
                        description: format!(
                            "Dokumentasi/Keputusan '{}' merujuk ke entitas '{}', namun simbol tersebut tidak ada lagi di kode aktif.",
                            item.title, target_name
                        ),
                        location: item.evidence.first().and_then(|e| e.file.clone()),
                        recommendation: format!("Perbarui referensi di '{}' atau sinkronkan dengan nama simbol terkini.", item.title),
                    });
                }
            }
        }

        Ok(violations)
    }

    /// Detect file dependency cycles and unreferenced functions within this project.
    pub fn detect_architecture_drift(
        &self,
        project_id: &str,
    ) -> anyhow::Result<Vec<DriftViolation>> {
        let graph = GraphMeasurements::compute(self.store, project_id)?;
        self.detect_architecture_drift_with_graph(project_id, &graph)
    }

    fn detect_architecture_drift_with_graph(
        &self,
        project_id: &str,
        graph: &GraphMeasurements,
    ) -> anyhow::Result<Vec<DriftViolation>> {
        let mut violations = Vec::new();
        for component in graph.dependency_cycles.iter().cloned() {
            violations.push(DriftViolation {
                severity: DriftSeverity::Critical,
                title: format!("Circular Dependency: {} files", component.len()),
                description: format!("File-file ini saling terjangkau melalui panggilan pada graf statis: {}. Urutan daftar bukan urutan jalur eksekusi.", component.join(", ")),
                location: Some(component.join(", ")),
                recommendation: "Tinjau arah dependensi dan ekstrak kontrak bersama bila diperlukan.".to_string(),
            });
        }
        let nodes = self.store.find_nodes_by_name("")?;
        let file_nodes: std::collections::HashMap<_, _> = nodes
            .iter()
            .filter(|n| n.project_id == project_id && n.kind == NodeKind::File)
            .filter_map(|n| n.source.as_ref().map(|source| (source.file.as_str(), n)))
            .collect();
        let project_nodes: HashSet<_> = nodes
            .iter()
            .filter(|n| n.project_id == project_id)
            .map(|n| n.id.as_str())
            .collect();
        let incoming: HashSet<_> = self
            .store
            .get_all_edges()?
            .into_iter()
            .filter(|e| e.kind == EdgeKind::Calls && project_nodes.contains(e.from.as_str()))
            .map(|e| e.to)
            .collect();
        for node in nodes.iter().filter(|n| {
            n.project_id == project_id && n.kind == NodeKind::Function && !incoming.contains(&n.id)
        }) {
            let Some(source) = &node.source else {
                continue;
            };
            let path = std::path::Path::new(&source.file);
            if node.name == "main"
                || node.name == "activate"
                || node.name == "deactivate"
                || node.name == "run_mcp_server"
                || node.name.starts_with("test_")
                || source.file.ends_with("_test.rs")
                || source.file.ends_with("_tests.rs")
                || source.file.contains(".test.")
                || source.file.contains(".spec.")
                || source.file.ends_with(".vue")
                || source.file.ends_with(".svelte")
                || node.metadata["is_test"] == true
                || node.metadata["tauri_command"] == true
                || path.components().any(|part| {
                    matches!(
                        part.as_os_str().to_str(),
                        Some(
                            "test"
                                | "tests"
                                | "spec"
                                | "specs"
                                | "examples"
                                | "example"
                                | "benches"
                                | "fixtures"
                                | "stores"
                                | "store"
                                | "composables"
                                | "composable"
                        )
                    )
                })
            {
                continue;
            }
            if node.language.as_deref() == Some("typescript")
                && node.identity.qualified_name.contains('.')
            {
                continue;
            }
            if node.language.as_deref() == Some("rust") && node.metadata["public"] == true {
                continue;
            }
            if node.language.as_deref() == Some("python") && !node.name.starts_with('_') {
                continue;
            }
            if node.language.as_deref() == Some("typescript") {
                let exported = file_nodes.get(source.file.as_str()).is_some_and(|file| {
                    file.metadata["exports"].as_array().is_some_and(|exports| {
                        exports
                            .iter()
                            .any(|binding| binding["local"].as_str() == Some(&node.name))
                    })
                });
                if exported {
                    continue;
                }
            }
            violations.push(DriftViolation {
                severity: DriftSeverity::Low,
                title: format!("Fungsi privat kandidat tanpa pemanggil terindeks: '{}'", node.name),
                description: "Tidak ditemukan pemanggil untuk fungsi yang tampak privat pada graf parsial. Pola callback, refleksi, dan penggunaan dinamis masih dapat luput.".to_string(),
                location: Some(format!("{}:{}", source.file, source.start_line)),
                recommendation: "Verifikasi penggunaan sebelum menghapus kode.".to_string(),
            });
        }
        Ok(violations)
    }

    /// Run full audit
    pub fn run_audit(&self, project_id: &str) -> anyhow::Result<DriftReport> {
        let graph = GraphMeasurements::compute(self.store, project_id)?;
        self.run_audit_with_graph(project_id, &graph)
    }

    pub(crate) fn run_audit_with_graph(
        &self,
        project_id: &str,
        graph: &GraphMeasurements,
    ) -> anyhow::Result<DriftReport> {
        let mut all_violations = self.detect_doc_drift(project_id)?;
        all_violations.extend(self.detect_architecture_drift_with_graph(project_id, graph)?);
        let total_violations = all_violations.len();
        Ok(DriftReport {
            total_violations,
            violations: all_violations,
        })
    }
}
