mod complexity;
use emocis_core::{NodeKind, SqliteStore};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactorDiagnosis {
    pub file_path: String,
    pub loc: usize,
    pub non_blank_lines: usize,
    pub function_count: usize,
    pub distinct_conceptual_clusters: usize,
    pub top_concept_clusters: Vec<String>,
    pub incoming_dependents_count: usize,
    pub outgoing_calls_count: usize,
    pub cyclomatic_complexity_est: Option<usize>,
    pub complexity_method: String,
    pub complexity_target: Option<String>,
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
    pub fn diagnose_file(
        &self,
        file_path: &str,
        content: &str,
    ) -> anyhow::Result<RefactorDiagnosis> {
        let lines: Vec<&str> = content.lines().collect();
        let loc = lines.len();
        let non_blank_lines = lines.iter().filter(|line| !line.trim().is_empty()).count();

        // 1. Dynamic AST token extraction from declared symbols
        let nodes = self.store.find_nodes_by_file(file_path)?;
        let mut functions_in_file = Vec::new();
        let mut incoming_dependents = HashSet::new();
        let mut outgoing_calls = HashSet::new();

        for node in &nodes {
            if node.kind == NodeKind::Function || node.kind == NodeKind::Method {
                functions_in_file.push(node.name.clone());
            }

            for (_, caller) in self.store.get_callers(&node.id)? {
                if caller.source.as_ref().map(|s| s.file.as_str()) != Some(file_path) {
                    incoming_dependents.insert(caller.id);
                }
            }
            for (_, callee) in self.store.get_callees(&node.id)? {
                if callee.source.as_ref().map(|s| s.file.as_str()) != Some(file_path) {
                    outgoing_calls.insert(callee.id);
                }
            }
        }

        // 2. Identifier vocabulary (a lexical signal, not a measure of responsibility).
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
            "get", "set", "handle", "process", "run", "do", "make", "create", "update", "delete",
            "build", "init", "check", "on", "is", "has", "to",
        ];
        let mut clusters: Vec<(String, usize)> = word_frequencies
            .into_iter()
            .filter(|(w, _)| !generic_stops.contains(&w.as_str()))
            .collect();

        clusters.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        let top_concept_clusters: Vec<String> = clusters
            .iter()
            .take(5)
            .map(|(w, c)| format!("{} ({} referensi)", capitalize(w), c))
            .collect();

        let distinct_clusters = clusters.len();

        // AST complexity is measured within each function, excluding nested function bodies.
        let complexity = complexity::max_function_complexity(file_path, content);

        let incoming_count = incoming_dependents.len();
        let outgoing_count = outgoing_calls.len();
        let function_count = functions_in_file.len();

        // 4. Dynamic Severity Assessment
        let (severity, recommendation) = if non_blank_lines > 1200
            || (function_count > 25 && distinct_clusters >= 6)
        {
            (
                "SIZE REVIEW".to_string(),
                format!(
                    "File memiliki {} baris nonkosong dan {} fungsi terindeks. Ini sinyal ukuran untuk ditinjau; analisis kohesi belum tersedia.",
                    non_blank_lines, function_count
                ),
            )
        } else if non_blank_lines > 500 || (function_count > 15 && distinct_clusters >= 4) {
            (
                "SIZE REVIEW".to_string(),
                format!(
                    "File memiliki {} baris nonkosong dan {} fungsi terindeks. Tinjau secara manual apakah modul perlu dipecah.",
                    non_blank_lines, function_count
                ),
            )
        } else if complexity.value.is_some_and(|value| value > 10) {
            (
                "COMPLEXITY REVIEW".to_string(),
                format!(
                    "Kompleksitas fungsi tertinggi terukur {} pada {}. Periksa jalur kondisional dan kasus uji sebelum refactor.",
                    complexity.value.unwrap_or_default(), complexity.function.as_deref().unwrap_or("fungsi tanpa nama")
                ),
            )
        } else {
            (
                "NO SIZE FLAG".to_string(),
                "Heuristik ukuran tidak melewati ambang tinjau; hasil ini tidak menilai kohesi atau kualitas desain.".to_string(),
            )
        };

        Ok(RefactorDiagnosis {
            file_path: file_path.to_string(),
            loc,
            non_blank_lines,
            function_count,
            distinct_conceptual_clusters: distinct_clusters,
            top_concept_clusters,
            incoming_dependents_count: incoming_count,
            outgoing_calls_count: outgoing_count,
            cyclomatic_complexity_est: complexity.value,
            complexity_method: complexity.method.to_owned(),
            complexity_target: complexity.function,
            severity,
            recommendation,
        })
    }

    /// Compatibility API for callers that intentionally scan all projects in this store.
    pub fn scan_bloat_candidates(&self) -> anyhow::Result<Vec<RefactorDiagnosis>> {
        self.scan_bloat_candidates_inner(None)
    }

    /// Scan only one indexed project and report file read/analysis failures.
    pub fn scan_bloat_candidates_for_project(
        &self,
        project_id: &str,
    ) -> anyhow::Result<Vec<RefactorDiagnosis>> {
        self.scan_bloat_candidates_inner(Some(project_id))
    }

    fn scan_bloat_candidates_inner(
        &self,
        project_id: Option<&str>,
    ) -> anyhow::Result<Vec<RefactorDiagnosis>> {
        let all_nodes: Vec<_> = self
            .store
            .find_nodes_by_name("")?
            .into_iter()
            .filter(|n| project_id.is_none_or(|project| n.project_id == project))
            .collect();
        let mut files_checked = HashMap::new();

        for node in all_nodes {
            if node.kind == NodeKind::File {
                if let Some(src) = node.source {
                    let content = std::fs::read_to_string(&src.file)?;
                    let diag = self.diagnose_file(&src.file, &content)?;
                    files_checked.insert(src.file, diag);
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

