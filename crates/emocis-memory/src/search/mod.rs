use std::collections::BTreeMap;
use emocis_core::{KnowledgeItem, KnowledgeType, SqliteStore};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TocSection {
    pub section_number: String,
    pub title: String,
    pub items: Vec<KnowledgeItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptualToc {
    pub project_id: String,
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
    pub fn generate_toc(&self, project_id: &str) -> anyhow::Result<ConceptualToc> {
        let all_items = self.store.list_knowledge_items(project_id)?;

        let mut groups: BTreeMap<&'static str, (&'static str, Vec<KnowledgeItem>)> = BTreeMap::new();
        groups.insert("01", ("01 Architecture", Vec::new()));
        groups.insert("02", ("02 Decisions", Vec::new()));
        groups.insert("03", ("03 Constraints", Vec::new()));
        groups.insert("04", ("04 Business Rules", Vec::new()));
        groups.insert("05", ("05 Warnings & Caveats", Vec::new()));
        groups.insert("06", ("06 Failed Attempts", Vec::new()));
        groups.insert("07", ("07 Worklog", Vec::new()));

        for item in all_items {
            let section_key = match item.knowledge_type {
                KnowledgeType::Architecture => "01",
                KnowledgeType::Decision => "02",
                KnowledgeType::Constraint => "03",
                KnowledgeType::BusinessRule => "04",
                KnowledgeType::Warning => "05",
                KnowledgeType::FailedAttempt => "06",
                KnowledgeType::Worklog => "07",
            };

            if let Some((_, list)) = groups.get_mut(section_key) {
                list.push(item);
            }
        }

        let mut sections = Vec::new();
        for (sec_num, (sec_title, items)) in groups {
            sections.push(TocSection {
                section_number: sec_num.to_string(),
                title: sec_title.to_string(),
                items,
            });
        }

        Ok(ConceptualToc {
            project_id: project_id.to_string(),
            sections,
        })
    }

    /// Search knowledge items by natural query (PRD Part I Section 23)
    pub fn search(&self, query: &str) -> anyhow::Result<Vec<KnowledgeItem>> {
        let results = self.store.search_knowledge(query)?;
        Ok(results)
    }
}
