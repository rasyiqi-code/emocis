use emocis_core::{Edge, EdgeKind, SqliteStore};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiffChangeType {
    Added,
    Removed,
    Modified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralEdgeDiff {
    pub change_type: DiffChangeType,
    pub caller_id: String,
    pub callee_id: String,
    pub edge_kind: EdgeKind,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralDiffReport {
    pub base_snapshot: String,
    pub target_snapshot: String,
    pub total_changes: usize,
    pub changes: Vec<BehavioralEdgeDiff>,
}

pub struct BehavioralDiffEngine<'a> {
    store: &'a SqliteStore,
}

impl<'a> BehavioralDiffEngine<'a> {
    pub fn new(store: &'a SqliteStore) -> Self {
        Self { store }
    }

    /// Compare call graphs and relationships between two snapshots
    pub fn compare_snapshots(
        &self,
        base_snapshot_id: &str,
        target_snapshot_id: &str,
    ) -> anyhow::Result<BehavioralDiffReport> {
        // Query edges for base and target snapshots
        let mut changes = Vec::new();

        // In a temporal graph, edges have valid_from and valid_until
        // For demonstration and active diffing:
        // We compare active edges with base snapshot
        let all_base_edges = self.get_snapshot_edges(base_snapshot_id)?;
        let all_target_edges = self.get_snapshot_edges(target_snapshot_id)?;

        let base_map: std::collections::HashMap<(String, String), &Edge> = all_base_edges
            .iter()
            .map(|e| ((e.from.clone(), e.to.clone()), e))
            .collect();

        let target_map: std::collections::HashMap<(String, String), &Edge> = all_target_edges
            .iter()
            .map(|e| ((e.from.clone(), e.to.clone()), e))
            .collect();

        // 1. Check added or modified in target
        for (key, target_edge) in &target_map {
            match base_map.get(key) {
                Some(base_edge) => {
                    if base_edge.qualifier != target_edge.qualifier {
                        changes.push(BehavioralEdgeDiff {
                            change_type: DiffChangeType::Modified,
                            caller_id: key.0.clone(),
                            callee_id: key.1.clone(),
                            edge_kind: target_edge.kind.clone(),
                            detail: format!(
                                "Qualifier berubah: {} -> {}",
                                base_edge.qualifier, target_edge.qualifier
                            ),
                        });
                    }
                }
                None => {
                    changes.push(BehavioralEdgeDiff {
                        change_type: DiffChangeType::Added,
                        caller_id: key.0.clone(),
                        callee_id: key.1.clone(),
                        edge_kind: target_edge.kind.clone(),
                        detail: format!(
                            "Jalur pemanggilan baru terdeteksi: {} ──> {}",
                            key.0, key.1
                        ),
                    });
                }
            }
        }

        // 2. Check removed from base
        for (key, base_edge) in &base_map {
            if !target_map.contains_key(key) {
                changes.push(BehavioralEdgeDiff {
                    change_type: DiffChangeType::Removed,
                    caller_id: key.0.clone(),
                    callee_id: key.1.clone(),
                    edge_kind: base_edge.kind.clone(),
                    detail: format!("Jalur pemanggilan dihapus: {} ──> {}", key.0, key.1),
                });
            }
        }

        let total = changes.len();
        Ok(BehavioralDiffReport {
            base_snapshot: base_snapshot_id.to_string(),
            target_snapshot: target_snapshot_id.to_string(),
            total_changes: total,
            changes,
        })
    }

    fn get_snapshot_edges(&self, snapshot_id: &str) -> anyhow::Result<Vec<Edge>> {
        let edges = self.store.get_edges_for_snapshot(snapshot_id)?;
        Ok(edges)
    }
}
