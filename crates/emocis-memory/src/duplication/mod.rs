use std::collections::HashSet;
use emocis_core::{Node, NodeKind, SqliteStore};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicationCandidate {
    pub function_a: String,
    pub location_a: String,
    pub function_b: String,
    pub location_b: String,
    pub similarity_level: &'static str, // "Level 1 (Textual)", "Level 2 (Structural)", "Level 3 (Semantic)"
    pub shared_responsibility: String,
    pub recommendation: String,
}

pub struct DuplicationDetector<'a> {
    store: &'a SqliteStore,
}

impl<'a> DuplicationDetector<'a> {
    pub fn new(store: &'a SqliteStore) -> Self {
        Self { store }
    }

    /// Detect duplications dynamically across ANY codebase using Jaccard token similarity & call signatures
    pub fn scan_duplications(&self) -> anyhow::Result<Vec<DuplicationCandidate>> {
        let all_nodes = self.store.find_nodes_by_name("")?;
        let functions: Vec<&Node> = all_nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Function || n.kind == NodeKind::Method)
            .collect();

        let mut duplicates = Vec::new();

        for i in 0..functions.len() {
            for j in (i + 1)..functions.len() {
                let f1 = functions[i];
                let f2 = functions[j];

                // Skip if same file
                let loc1 = f1.source.as_ref().map(|s| s.file.clone()).unwrap_or_default();
                let loc2 = f2.source.as_ref().map(|s| s.file.clone()).unwrap_or_default();
                if loc1 == loc2 || loc1.is_empty() || loc2.is_empty() {
                    continue;
                }

                // 1. Level 1: Exact Name Duplication across different files
                if f1.name == f2.name && f1.name.len() > 3 {
                    duplicates.push(DuplicationCandidate {
                        function_a: f1.name.clone(),
                        location_a: loc1.clone(),
                        function_b: f2.name.clone(),
                        location_b: loc2.clone(),
                        similarity_level: "Level 1 (Identical Name)",
                        shared_responsibility: format!("Nama fungsi identik: '{}'", f1.name),
                        recommendation: format!(
                            "Fungsi bernama identik '{}' didefinisikan di dua modul berbeda. Pertimbangkan konsolidasi ke modul utilitas/domain terpusat.",
                            f1.name
                        ),
                    });
                    continue;
                }

                // 2. Level 2: Call-Pattern Structural Similarity
                let callees1 = self.store.get_callees(&f1.id)?;
                let callees2 = self.store.get_callees(&f2.id)?;

                let calls_set1: HashSet<String> = callees1.into_iter().map(|(_, n)| n.name).collect();
                let calls_set2: HashSet<String> = callees2.into_iter().map(|(_, n)| n.name).collect();

                if !calls_set1.is_empty() && !calls_set2.is_empty() {
                    let intersection: HashSet<_> = calls_set1.intersection(&calls_set2).collect();
                    let union: HashSet<_> = calls_set1.union(&calls_set2).collect();
                    let call_jaccard = (intersection.len() as f32) / (union.len() as f32);

                    if call_jaccard >= 0.70 && intersection.len() >= 2 {
                        let shared_calls: Vec<String> = intersection.into_iter().cloned().collect();
                        duplicates.push(DuplicationCandidate {
                            function_a: f1.name.clone(),
                            location_a: loc1.clone(),
                            function_b: f2.name.clone(),
                            location_b: loc2.clone(),
                            similarity_level: "Level 2 (Structural Call-Graph)",
                            shared_responsibility: format!("Memanggil subrutin yang sama ({}% overlap): {}", (call_jaccard * 100.0) as u32, shared_calls.join(", ")),
                            recommendation: format!(
                                "Kedua fungsi memiliki pola alur pemanggilan logika yang hampir identik. Ekstrak pola alur pemanggilan bersama."
                            ),
                        });
                        continue;
                    }
                }

                // 3. Level 3: Semantic Identifier Token Jaccard Similarity
                let tokens1: HashSet<String> = tokenize_words(&f1.name);
                let tokens2: HashSet<String> = tokenize_words(&f2.name);

                if !tokens1.is_empty() && !tokens2.is_empty() {
                    let intersection: HashSet<_> = tokens1.intersection(&tokens2).collect();
                    let union: HashSet<_> = tokens1.union(&tokens2).collect();
                    let jaccard = (intersection.len() as f32) / (union.len() as f32);

                    if jaccard >= 0.60 && intersection.len() >= 2 {
                        let shared_concepts: Vec<String> = intersection.into_iter().cloned().collect();
                        duplicates.push(DuplicationCandidate {
                            function_a: f1.name.clone(),
                            location_a: loc1,
                            function_b: f2.name.clone(),
                            location_b: loc2,
                            similarity_level: "Level 3 (Semantic Concept)",
                            shared_responsibility: format!("Konsep leksikal bersama: {}", shared_concepts.join(", ")),
                            recommendation: format!(
                                "Kedua fungsi '{}' dan '{}' memiliki kesamaan konsep tanggung jawab. Evaluasi apakah implementasinya dapat disatukan.",
                                f1.name, f2.name
                            ),
                        });
                    }
                }
            }
        }

        Ok(duplicates)
    }
}

fn tokenize_words(ident: &str) -> HashSet<String> {
    let mut words = HashSet::new();
    let mut current = String::new();

    for c in ident.chars() {
        if c == '_' || c == '-' || c == '.' || c == ':' {
            if !current.is_empty() {
                words.insert(current.to_lowercase());
                current = String::new();
            }
        } else if c.is_uppercase() {
            if !current.is_empty() {
                words.insert(current.to_lowercase());
                current = String::new();
            }
            current.push(c);
        } else {
            current.push(c);
        }
    }

    if !current.is_empty() {
        words.insert(current.to_lowercase());
    }

    words
}
