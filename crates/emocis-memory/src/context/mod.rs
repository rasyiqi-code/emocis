use emocis_core::{KnowledgeType, SqliteStore};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinimalContextPackage {
    pub intent: String,
    pub active_constraints: Vec<String>,
    pub relevant_decisions: Vec<String>,
    pub warnings: Vec<String>,
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

    /// Build a Minimal Context Package for AI Coding Agents
    pub fn build_context(&self, intent: &str) -> anyhow::Result<MinimalContextPackage> {
        let items = self.store.search_knowledge(intent)?;

        let mut active_constraints = Vec::new();
        let mut relevant_decisions = Vec::new();
        let mut warnings = Vec::new();
        let mut affected_symbols = Vec::new();

        for item in items {
            match item.knowledge_type {
                KnowledgeType::Constraint => {
                    active_constraints.push(format!("{}: {}", item.title, item.content));
                }
                KnowledgeType::Decision => {
                    relevant_decisions.push(format!("{}: {}", item.title, item.content));
                }
                KnowledgeType::Warning => {
                    warnings.push(format!("{}: {}", item.title, item.content));
                }
                _ => {}
            }
            affected_symbols.extend(item.affects);
        }

        // Also search symbols matching the intent
        let matching_nodes = self.store.find_nodes_by_name(intent)?;
        for node in matching_nodes {
            affected_symbols.push(format!("{} ({:?}) in {}", node.name, node.kind, node.source.map(|s| s.file).unwrap_or_default()));
        }

        affected_symbols.sort();
        affected_symbols.dedup();

        // Format Markdown
        let mut md = String::new();
        md.push_str(&format!("### 🧠 EMOCIS Minimal Context Package for AI Agent\n"));
        md.push_str(&format!("**Target Intent / Task:** `{}`\n\n", intent));

        if !active_constraints.is_empty() {
            md.push_str("#### ⛔ CRITICAL CONSTRAINTS (DO NOT VIOLATE):\n");
            for c in &active_constraints {
                md.push_str(&format!("- **{}**\n", c));
            }
            md.push('\n');
        }

        if !relevant_decisions.is_empty() {
            md.push_str("#### 🏛 ARCHITECTURAL DECISIONS:\n");
            for d in &relevant_decisions {
                md.push_str(&format!("- {}\n", d));
            }
            md.push('\n');
        }

        if !warnings.is_empty() {
            md.push_str("#### ⚠️ WARNINGS & CAVEATS:\n");
            for w in &warnings {
                md.push_str(&format!("- {}\n", w));
            }
            md.push('\n');
        }

        if !affected_symbols.is_empty() {
            md.push_str("#### 📦 RELEVANT CODE SYMBOLS:\n");
            for s in &affected_symbols {
                md.push_str(&format!("- {}\n", s));
            }
            md.push('\n');
        }

        let token_estimate = md.len() / 4; // approximate rule of thumb: 4 chars per token

        Ok(MinimalContextPackage {
            intent: intent.to_string(),
            active_constraints,
            relevant_decisions,
            warnings,
            affected_symbols,
            token_estimate,
            formatted_markdown: md,
        })
    }
}
