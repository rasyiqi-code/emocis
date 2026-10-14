use emocis_core::{
    Evidence, IdentityGenerator, KnowledgeItem, KnowledgeStatus, KnowledgeType, SqliteStore,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedAttemptRecord {
    pub id: String,
    pub approach: String,
    pub reason_rejected: String,
    pub replaced_by: String,
    pub provenance_source: String,
    pub affected_modules: Vec<String>,
}

pub struct FailedAttemptsManager<'a> {
    store: &'a SqliteStore,
}

impl<'a> FailedAttemptsManager<'a> {
    pub fn new(store: &'a SqliteStore) -> Self {
        Self { store }
    }

    /// Record a failed approach so future engineers and AI agents do not repeat it
    pub fn record_failure(
        &self,
        project_id: &str,
        approach: &str,
        reason_rejected: &str,
        replaced_by: &str,
        provenance_pr_or_commit: &str,
        affected_modules: Vec<String>,
    ) -> anyhow::Result<String> {
        let unique_id = format!("failure_{}", chrono::Utc::now().timestamp());
        let id = IdentityGenerator::for_knowledge("failed_attempt", &unique_id);

        let content = format!(
            "PENDEKATAN GAGAL: {}\nALASAN DITOLAK: {}\nDIGANTIKAN OLEH: {}\nSUMBER BUKTI: {}",
            approach, reason_rejected, replaced_by, provenance_pr_or_commit
        );

        let item = KnowledgeItem {
            id: id.clone(),
            project_id: project_id.to_string(),
            knowledge_type: KnowledgeType::FailedAttempt,
            title: format!("Failed Approach: {}", approach),
            content,
            status: KnowledgeStatus::Active,
            confidence: 1.0,
            evidence: vec![Evidence {
                evidence_type: "incident_or_pr".to_string(),
                file: None,
                start_line: None,
                end_line: None,
                commit_id: Some(provenance_pr_or_commit.to_string()),
                snippet: Some(reason_rejected.to_string()),
            }],
            affects: affected_modules,
            tags: vec!["failed_attempt".to_string(), "anti_pattern".to_string()],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        self.store.insert_knowledge_item(&item)?;
        Ok(id)
    }

    /// List all recorded failed attempts
    pub fn list_failures(&self, project_id: &str) -> anyhow::Result<Vec<FailedAttemptRecord>> {
        let items = self.store.list_knowledge_items(project_id)?;
        let mut failures = Vec::new();

        for item in items {
            if item.knowledge_type == KnowledgeType::FailedAttempt {
                failures.push(FailedAttemptRecord {
                    id: item.id,
                    approach: item.title.replace("Failed Approach: ", ""),
                    reason_rejected: item.content.clone(),
                    replaced_by: "".to_string(),
                    provenance_source: item.evidence.first().and_then(|e| e.commit_id.clone()).unwrap_or_default(),
                    affected_modules: item.affects,
                });
            }
        }

        Ok(failures)
    }

    /// Check if a proposed approach matches a known past failure
    pub fn check_proposed_approach(&self, proposal: &str) -> anyhow::Result<Option<KnowledgeItem>> {
        let items = self.store.search_knowledge(proposal)?;
        for item in items {
            if item.knowledge_type == KnowledgeType::FailedAttempt {
                return Ok(Some(item));
            }
        }
        Ok(None)
    }
}
