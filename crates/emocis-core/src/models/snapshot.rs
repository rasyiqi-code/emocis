use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Git commit snapshot representing temporal state of the graph G=(V,E,S,P)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: String, // e.g. "snapshot:commit_abc123"
    pub project_id: String,
    pub commit_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    pub created_at: DateTime<Utc>,
}
