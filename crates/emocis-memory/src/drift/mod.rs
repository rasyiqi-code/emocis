use std::collections::{HashMap, HashSet};
use emocis_core::SqliteStore;
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
                let matched_nodes = self.store.find_nodes_by_name(target_name)?;
                if matched_nodes.is_empty() && !target_name.ends_with(".ts") && !target_name.ends_with(".rs") && !target_name.ends_with(".md") {
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

    /// Universal architecture drift detector:
    /// 1. Circular dependencies between directories/modules (Universal: applies to any project)
    /// 2. Utility / leaf layer inversion (Leaf files calling orchestrators)
    pub fn detect_architecture_drift(&self, _project_id: &str) -> anyhow::Result<Vec<DriftViolation>> {
        let mut violations = Vec::new();

        // 1. Module-level Circular Dependency Detection
        // Map edges between different top-level / sub directories
        let mut dir_dependencies: HashMap<String, HashSet<String>> = HashMap::new();

        let cross_calls = self.store.get_cross_file_calls()?;
        for (file1, file2) in cross_calls {
            let dir1 = get_module_dir(&file1);
            let dir2 = get_module_dir(&file2);

            if dir1 != dir2 && !dir1.is_empty() && !dir2.is_empty() {
                dir_dependencies.entry(dir1.clone()).or_default().insert(dir2.clone());

                // Check if reverse dependency exists: dir2 already calls dir1
                if let Some(target_deps) = dir_dependencies.get(&dir2) {
                    if target_deps.contains(&dir1) {
                        violations.push(DriftViolation {
                            severity: DriftSeverity::Critical,
                            title: format!("Circular Dependency Terdeteksi: {} <───> {}", dir1, dir2),
                            description: format!(
                                "Modul '{}' memanggil '{}' dan sebaliknya modul '{}' memanggil '{}'. Keterikatan siklik ini merusak modularitas dan menyulitkan testing.",
                                dir1, dir2, dir2, dir1
                            ),
                            location: Some(format!("Antara {} dan {}", file1, file2)),
                            recommendation: "Pecah dependensi siklik dengan memindahkan interface atau tipe bersama ke modul terpisah (DIP / Dependency Inversion).".to_string(),
                        });
                    }
                }
            }
        }

        // 2. Dead Logic / Orphan Function Detection (PRD Section 34)
        let uncalled = self.store.get_uncalled_functions(50)?;
        for (fn_name, file_path, line) in uncalled {
            // Non-main / non-index files with unreferenced functions
            if !file_path.contains("main") && !file_path.contains("index") && !file_path.contains("test") {
                violations.push(DriftViolation {
                    severity: DriftSeverity::Low,
                    title: format!("Possible Dead Logic: Fungsi '{}' tidak memiliki pemanggil", fn_name),
                    description: format!(
                        "Fungsi '{}' di '{}' tidak dipanggil oleh bagian manapun di dalam graf kode.",
                        fn_name, file_path
                    ),
                    location: Some(format!("{}:{}", file_path, line)),
                    recommendation: "Periksa apakah fungsi ini masih digunakan, merupakan API publik eksternal, atau kode mati yang aman dibersihkan.".to_string(),
                });
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

fn get_module_dir(file_path: &str) -> String {
    let path = std::path::Path::new(file_path);
    if let Some(parent) = path.parent() {
        parent.to_str().unwrap_or("").to_string()
    } else {
        "".to_string()
    }
}
