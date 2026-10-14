use std::collections::{HashMap, HashSet};
use emocis_core::{NodeKind, SqliteStore};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactorDiagnosis {
    pub file_path: String,
    pub loc: usize,
    pub function_count: usize,
    pub distinct_conceptual_clusters: usize,
    pub top_concept_clusters: Vec<String>,
    pub incoming_dependents_count: usize,
    pub outgoing_calls_count: usize,
    pub cyclomatic_complexity_est: usize,
    pub severity: String, // "CRITICAL", "HIGH", "MEDIUM", "HEALTHY"
    pub recommendation: String,
}

pub struct RefactoringRadar<'a> {
    store: &'a SqliteStore,
}

impl<'a> RefactoringRadar<'a> {
    pub fn new(store: &'a SqliteStore) -> Self {
        Self { store }
    }

    /// Fully dynamic diagnosis applicable to ANY codebase (game engines, compilers, web, ML, etc.)
    pub fn diagnose_file(&self, file_path: &str, content: &str) -> anyhow::Result<RefactorDiagnosis> {
        let lines: Vec<&str> = content.lines().collect();
        let loc = lines.len();

        // 1. Dynamic AST token extraction from declared symbols
        let nodes = self.store.find_nodes_by_name(file_path)?;
        let mut functions_in_file = Vec::new();
        let mut incoming_dependents = HashSet::new();
        let mut outgoing_calls = HashSet::new();

        for node in &nodes {
            if node.kind == NodeKind::Function || node.kind == NodeKind::Method {
                functions_in_file.push(node.name.clone());
            }

            if let Ok(callers) = self.store.get_callers(&node.id) {
                for (_, caller) in callers {
                    if caller.source.as_ref().map(|s| s.file.as_str()) != Some(file_path) {
                        incoming_dependents.insert(caller.id);
                    }
                }
            }
            if let Ok(callees) = self.store.get_callees(&node.id) {
                for (_, callee) in callees {
                    if callee.source.as_ref().map(|s| s.file.as_str()) != Some(file_path) {
                        outgoing_calls.insert(callee.id);
                    }
                }
            }
        }

        // 2. Dynamic Word-Boundary Tokenizer (CamelCase, snake_case, kebab-case)
        let mut word_frequencies: HashMap<String, usize> = HashMap::new();
        for fn_name in &functions_in_file {
            let tokens = tokenize_identifier(fn_name);
            for tok in tokens {
                if tok.len() > 2 {
                    *word_frequencies.entry(tok.to_lowercase()).or_insert(0) += 1;
                }
            }
        }

        // Filter out generic programming keywords
        let generic_stops = [
            "get", "set", "handle", "process", "run", "do", "make", "create",
            "update", "delete", "build", "init", "check", "on", "is", "has", "to",
        ];
        let mut clusters: Vec<(String, usize)> = word_frequencies
            .into_iter()
            .filter(|(w, _)| !generic_stops.contains(&w.as_str()))
            .collect();

        clusters.sort_by(|a, b| b.1.cmp(&a.1));

        let top_concept_clusters: Vec<String> = clusters
            .iter()
            .take(5)
            .map(|(w, c)| format!("{} ({} referensi)", capitalize(w), c))
            .collect();

        let distinct_clusters = clusters.len();

        // 3. Dynamic Cyclomatic Complexity Estimation (Branch counting)
        let mut cyclomatic_branches = 1;
        for line in &lines {
            let trimmed = line.trim();
            if trimmed.starts_with("if ") || trimmed.starts_with("if(")
                || trimmed.starts_with("for ") || trimmed.starts_with("for(")
                || trimmed.starts_with("while ") || trimmed.starts_with("while(")
                || trimmed.starts_with("match ") || trimmed.starts_with("switch ")
                || trimmed.starts_with("catch ") || trimmed.starts_with("catch(")
            {
                cyclomatic_branches += 1;
            }
        }

        let incoming_count = incoming_dependents.len();
        let outgoing_count = outgoing_calls.len();
        let function_count = functions_in_file.len();

        // 4. Dynamic Severity Assessment
        let (severity, recommendation) = if loc > 1200 || (function_count > 25 && distinct_clusters >= 6) {
            let cluster_names: Vec<String> = clusters.iter().take(3).map(|(w, _)| capitalize(w)).collect();
            let suggestion = if cluster_names.is_empty() {
                "Pecah file ke beberapa sub-modul berdasarkan domain".to_string()
            } else {
                format!("Pisahkan modul ini menjadi: {}Handler / {}Service", cluster_names.join(", "), cluster_names.join(", "))
            };

            (
                "CRITICAL".to_string(),
                format!(
                    "File Bloat terdeteksi ({} LOC, {} fungsi, {} klaster konsep berbeda). {}",
                    loc, function_count, distinct_clusters, suggestion
                ),
            )
        } else if loc > 500 || (function_count > 15 && distinct_clusters >= 4) {
            (
                "HIGH".to_string(),
                format!(
                    "Modul mulai menampung akumulasi tanggung jawab ganda ({} fungsi). Pertimbangkan memisahkan fungsi pendukung.",
                    function_count
                ),
            )
        } else if cyclomatic_branches > 30 {
            (
                "MEDIUM".to_string(),
                format!(
                    "Kompleksitas percabangan tinggi ({} cabang logika). Pertimbangkan ekstraksi fungsi kecil.",
                    cyclomatic_branches
                ),
            )
        } else {
            (
                "HEALTHY".to_string(),
                "Struktur ukuran file, jumlah fungsi, dan kohesi tanggung jawab berada dalam batas wajar.".to_string(),
            )
        };

        Ok(RefactorDiagnosis {
            file_path: file_path.to_string(),
            loc,
            function_count,
            distinct_conceptual_clusters: distinct_clusters,
            top_concept_clusters,
            incoming_dependents_count: incoming_count,
            outgoing_calls_count: outgoing_count,
            cyclomatic_complexity_est: cyclomatic_branches,
            severity,
            recommendation,
        })
    }

    /// Dynamic repository-wide bloat candidate scanner
    pub fn scan_bloat_candidates(&self) -> anyhow::Result<Vec<RefactorDiagnosis>> {
        let all_nodes = self.store.find_nodes_by_name("")?;
        let mut files_checked = HashMap::new();

        for node in all_nodes {
            if node.kind == NodeKind::File {
                if let Some(src) = node.source {
                    if let Ok(content) = std::fs::read_to_string(&src.file) {
                        if let Ok(diag) = self.diagnose_file(&src.file, &content) {
                            files_checked.insert(src.file, diag);
                        }
                    }
                }
            }
        }

        let mut results: Vec<RefactorDiagnosis> = files_checked.into_values().collect();
        results.sort_by(|a, b| b.loc.cmp(&a.loc));
        Ok(results)
    }
}

/// Tokenize camelCase, PascalCase, snake_case, and kebab-case dynamically
fn tokenize_identifier(ident: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();

    for c in ident.chars() {
        if c == '_' || c == '-' || c == '.' || c == '/' || c == ':' {
            if !current.is_empty() {
                tokens.push(current);
                current = String::new();
            }
        } else if c.is_uppercase() {
            if !current.is_empty() {
                tokens.push(current);
                current = String::new();
            }
            current.push(c);
        } else {
            current.push(c);
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}
