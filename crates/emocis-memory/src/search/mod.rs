use emocis_core::{KnowledgeItem, KnowledgeType, SqliteStore};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TocSection {
    pub section_number: String,
    pub title: String,
    pub items: Vec<KnowledgeItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptualToc {
    pub project_id: String,
    pub doc_sections: Vec<TocSection>,
    pub code_sections: Vec<TocSection>,
    pub sections: Vec<TocSection>,
}

pub struct MemoryEngine<'a> {
    store: &'a SqliteStore,
}

impl<'a> MemoryEngine<'a> {
    pub fn new(store: &'a SqliteStore) -> Self {
        Self { store }
    }

    /// PRD Part I Section 21: Generate Conceptual Table of Contents (TOC)
    /// Separates Documentation / ADRs from In-Code Annotations & Comments
    pub fn generate_toc(&self, project_id: &str) -> anyhow::Result<ConceptualToc> {
        let all_items = self.store.list_knowledge_items(project_id)?;

        let (doc_items, code_items): (Vec<KnowledgeItem>, Vec<KnowledgeItem>) =
            all_items.into_iter().partition(|item| {
                item.evidence
                    .iter()
                    .any(|e| e.evidence_type == "documentation" || e.file.as_deref().unwrap_or("").ends_with(".md"))
                    || item.tags.iter().any(|t| t == "markdown")
                    || item.affects.iter().any(|a| a.ends_with(".md"))
            });

        // 1. Group Documentation items (.md, ADR, specs)
        let mut doc_groups: BTreeMap<&'static str, (&'static str, Vec<KnowledgeItem>)> =
            BTreeMap::new();
        doc_groups.insert("01", ("01 Architecture & Specifications", Vec::new()));
        doc_groups.insert("02", ("02 Architectural Decisions (ADR)", Vec::new()));
        doc_groups.insert("03", ("03 System Constraints & Security", Vec::new()));
        doc_groups.insert("04", ("04 Business Rules & Requirements", Vec::new()));
        doc_groups.insert("05", ("05 Bug Reports & Caveats", Vec::new()));
        doc_groups.insert("06", ("06 Failed Attempts", Vec::new()));
        doc_groups.insert("07", ("07 Worklog & Changelog", Vec::new()));

        for item in doc_items {
            let key = match item.knowledge_type {
                KnowledgeType::Architecture => "01",
                KnowledgeType::Decision => "02",
                KnowledgeType::Constraint => "03",
                KnowledgeType::BusinessRule => "04",
                KnowledgeType::Warning => "05",
                KnowledgeType::FailedAttempt => "06",
                KnowledgeType::Worklog => "07",
            };
            if let Some((_, list)) = doc_groups.get_mut(key) {
                list.push(item);
            }
        }

        let mut doc_sections = Vec::new();
        for (sec_num, (sec_title, items)) in doc_groups {
            doc_sections.push(TocSection {
                section_number: sec_num.to_string(),
                title: sec_title.to_string(),
                items,
            });
        }

        // 2. Group In-Code Annotations & Comments
        let mut code_groups: BTreeMap<&'static str, (&'static str, Vec<KnowledgeItem>)> =
            BTreeMap::new();
        code_groups.insert("01", ("01 Decisions & Rationale", Vec::new()));
        code_groups.insert("02", ("02 System Constraints & Idempotency", Vec::new()));
        code_groups.insert("03", ("03 Business Logic Rules", Vec::new()));
        code_groups.insert("04", ("04 Warnings & Workarounds", Vec::new()));
        code_groups.insert("05", ("05 Bug Fix Notes", Vec::new()));
        code_groups.insert("06", ("06 Architecture Notes", Vec::new()));
        code_groups.insert("07", ("07 Failed Attempts", Vec::new()));
        code_groups.insert("08", ("08 Worklog", Vec::new()));

        for item in code_items {
            let is_bugfix = item.tags.iter().any(|t| t == "bugfix" || t == "bug fix")
                || item.title.starts_with("BUG FIX");

            let key = if is_bugfix {
                "05"
            } else {
                match item.knowledge_type {
                    KnowledgeType::Decision => "01",
                    KnowledgeType::Constraint => "02",
                    KnowledgeType::BusinessRule => "03",
                    KnowledgeType::Warning => "04",
                    KnowledgeType::Architecture => "06",
                    KnowledgeType::FailedAttempt => "07",
                    KnowledgeType::Worklog => "08",
                }
            };
            if let Some((_, list)) = code_groups.get_mut(key) {
                list.push(item);
            }
        }

        let mut code_sections = Vec::new();
        for (sec_num, (sec_title, items)) in code_groups {
            code_sections.push(TocSection {
                section_number: sec_num.to_string(),
                title: sec_title.to_string(),
                items,
            });
        }

        let mut sections = Vec::new();
        sections.extend(doc_sections.clone());
        sections.extend(code_sections.clone());

        Ok(ConceptualToc {
            project_id: project_id.to_string(),
            doc_sections,
            code_sections,
            sections,
        })
    }

    /// Search knowledge items by natural query (PRD Part I Section 23)
    pub fn search(&self, query: &str) -> anyhow::Result<Vec<KnowledgeItem>> {
        Ok(self.store.search_knowledge(query)?)
    }

    pub fn search_for_project(
        &self,
        project_id: &str,
        query: &str,
    ) -> anyhow::Result<Vec<KnowledgeItem>> {
        Ok(self.store.search_knowledge_for_project(project_id, query)?)
    }
}
