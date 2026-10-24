use emocis_core::{KnowledgeType, SqliteStore};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

const MAX_CONTEXT_CHARS: usize = 7_600;
const TRUNCATION_NOTICE: &str =
    "\n[Context dipotong pada batas butir. Persempit intent untuk hasil lebih lengkap.]\n";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinimalContextPackage {
    pub truncated: bool,
    pub intent: String,
    pub active_constraints: Vec<String>,
    pub relevant_decisions: Vec<String>,
    pub warnings: Vec<String>,
    pub failed_attempts: Vec<String>,
    pub affected_symbols: Vec<String>,
    pub token_estimate: usize,
    pub formatted_markdown: String,
}

pub struct ContextCompressor<'a> {
    store: &'a SqliteStore,
}

impl<'a> ContextCompressor<'a> {
    pub fn new(store: &'a SqliteStore) -> Self {
        Self { store }
    }

    /// Compatibility API for callers that intentionally search the whole local store.
    pub fn build_context(&self, intent: &str) -> anyhow::Result<MinimalContextPackage> {
        self.build_context_inner(None, intent)
    }

    pub fn build_context_for_project(
        &self,
        project_id: &str,
        intent: &str,
    ) -> anyhow::Result<MinimalContextPackage> {
        self.build_context_inner(Some(project_id), intent)
    }

    fn build_context_inner(
        &self,
        project_id: Option<&str>,
        intent: &str,
    ) -> anyhow::Result<MinimalContextPackage> {
        let items = match project_id {
            Some(project) => self.store.search_knowledge_for_project(project, intent)?,
            None => self.store.search_knowledge(intent)?,
        };
        let mut active_constraints = Vec::new();
        let mut relevant_decisions = Vec::new();
        let mut warnings = Vec::new();
        let mut failed_attempts = Vec::new();
        let mut affected_symbols = Vec::new();
        let mut md = format!("### EMOCIS Context Package\n**Intent:** {}\n\nKeyword search; relevance and completeness are not guaranteed. Token savings are not measured.\n\n", intent.chars().take(300).collect::<String>().replace('`', " ").replace('\n', " ").replace('\r', " "));
        let mut emitted = HashSet::new();
        let mut truncated = false;
        let groups = [
            (
                "constraint",
                "#### Constraints\n",
                KnowledgeType::Constraint,
            ),
            (
                "business",
                "#### Business Rules\n",
                KnowledgeType::BusinessRule,
            ),
            (
                "decision",
                "#### Decisions and Architecture\n",
                KnowledgeType::Decision,
            ),
            ("warning", "#### Warnings\n", KnowledgeType::Warning),
            (
                "failure",
                "#### Failed Attempts\n",
                KnowledgeType::FailedAttempt,
            ),
        ];
        for (key, heading, kind) in groups {
            for item in items.iter().filter(|item| match key {
                "business" => item.knowledge_type == KnowledgeType::BusinessRule,
                "decision" => matches!(
                    item.knowledge_type,
                    KnowledgeType::Decision | KnowledgeType::Architecture
                ),
                "constraint" => item.knowledge_type == KnowledgeType::Constraint,
                "warning" => item.knowledge_type == KnowledgeType::Warning,
                _ => item.knowledge_type == KnowledgeType::FailedAttempt,
            }) {
                let line = format!("- **{}**: {}\n", item.title, item.content);
                let prefix = if emitted.contains(key) { "" } else { heading };
                if md.chars().count() + prefix.chars().count() + line.chars().count()
                    > MAX_CONTEXT_CHARS
                {
                    truncated = true;
                    continue;
                }
                md.push_str(prefix);
                md.push_str(&line);
                emitted.insert(key);
                match kind {
                    KnowledgeType::Constraint => {
                        active_constraints.push(format!("{}: {}", item.title, item.content))
                    }
                    KnowledgeType::BusinessRule => {
                        active_constraints.push(format!("{}: {}", item.title, item.content))
                    }
                    KnowledgeType::Decision => {
                        relevant_decisions.push(format!("{}: {}", item.title, item.content))
                    }
                    KnowledgeType::Warning => {
                        warnings.push(format!("{}: {}", item.title, item.content))
                    }
                    KnowledgeType::FailedAttempt => {
                        failed_attempts.push(format!("{}: {}", item.title, item.content))
                    }
                    _ => {}
                }
                if affected_symbols.len() < 100 {
                    affected_symbols.extend(
                        item.affects
                            .iter()
                            .take(10)
                            .take(100 - affected_symbols.len())
                            .map(|s| s.chars().take(200).collect::<String>()),
                    );
                }
            }
            if emitted.contains(key) {
                md.push('\n');
            }
        }

        // Match individual intent terms and scope results to the requested project.
        let mut found = HashMap::new();
        let terms: Vec<_> = intent
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|term| term.chars().count() >= 3)
            .take(8)
            .collect();
        for term in &terms {
            for node in self.store.find_nodes_by_name(term)?.into_iter().take(100) {
                if project_id.is_some_and(|project| node.project_id != project) {
                    continue;
                }
                found.entry(node.id.clone()).or_insert(node);
            }
        }
        let mut matches: Vec<_> = found.into_values().collect();
        matches.sort_by(|a, b| {
            let score = |node: &emocis_core::Node| terms_for_score(intent, &node.name);
            score(b).cmp(&score(a)).then_with(|| a.id.cmp(&b.id))
        });
        for node in matches.into_iter().take(30) {
            let label = format!(
                "{} ({:?}) in {}",
                node.name,
                node.kind,
                node.source.as_ref().map(|s| s.file.as_str()).unwrap_or("?")
            );
            let line = format!("- {}\n", label);
            let heading = if !emitted.contains("symbols") {
                "#### Matching symbols\n"
            } else {
                ""
            };
            if md.chars().count() + heading.chars().count() + line.chars().count()
                > MAX_CONTEXT_CHARS
            {
                truncated = true;
                continue;
            }
            md.push_str(heading);
            md.push_str(&line);
            emitted.insert("symbols");
            affected_symbols.push(label);
        }
        if truncated {
            md.push_str(TRUNCATION_NOTICE);
        }
        affected_symbols.sort();
        affected_symbols.dedup();
        affected_symbols.truncate(100);
        let token_estimate = md.chars().count().div_ceil(4);
        Ok(MinimalContextPackage {
            truncated,
            intent: intent.chars().take(300).collect(),
            active_constraints,
            relevant_decisions,
            warnings,
            failed_attempts,
            affected_symbols,
            token_estimate,
            formatted_markdown: md,
        })
    }
}

fn terms_for_score(query: &str, candidate: &str) -> usize {
    query
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|term| term.chars().count() >= 3)
        .map(|term| usize::from(candidate.to_lowercase().contains(&term.to_lowercase())))
        .sum()
}
