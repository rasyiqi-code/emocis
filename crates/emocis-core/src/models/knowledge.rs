use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use super::provenance::Evidence;

/// PRD Part I Section 19: Knowledge Node Types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeType {
    Decision,
    Constraint,
    Warning,
    BusinessRule,
    FailedAttempt,
    Worklog,
    Architecture,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeStatus {
    Active,
    Modified,
    Strengthened,
    Weakened,
    Superseded,
    Contradicted,
    Violated,
    Archived,
}

impl Default for KnowledgeStatus {
    fn default() -> Self {
        KnowledgeStatus::Active
    }
}

/// Knowledge Node (PRD Part I Section 38)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeItem {
    pub id: String,
    pub project_id: String,
    pub knowledge_type: KnowledgeType,
    pub title: String,
    pub content: String,
    pub status: KnowledgeStatus,
    pub confidence: f32, // 0.0 to 1.0
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<Evidence>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub affects: Vec<String>, // Target symbols, modules, or files
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
