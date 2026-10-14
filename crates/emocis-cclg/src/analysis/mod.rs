use std::collections::{HashSet, VecDeque};
use emocis_core::{Node, NodeKind, SqliteStore};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactReport {
    pub target_symbol: String,
    pub target_node: Option<Node>,
    pub direct_callers: Vec<Node>,
    pub indirect_callers: Vec<Node>,
    pub exposed_endpoints: Vec<Node>,
    pub affected_datastores: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPath {
    pub source: String,
    pub target: String,
    pub path: Vec<Node>,
    pub found: bool,
}

pub struct GraphAnalyzer<'a> {
    store: &'a SqliteStore,
}

impl<'a> GraphAnalyzer<'a> {
    pub fn new(store: &'a SqliteStore) -> Self {
        Self { store }
    }

    /// PRD Part II Section 28: Impact Analysis Engine
    pub fn analyze_impact(&self, symbol_query: &str) -> anyhow::Result<ImpactReport> {
        let matching_nodes = self.store.find_nodes_by_name(symbol_query)?;
        let target_node = matching_nodes.first().cloned();

        let mut direct_callers = Vec::new();
        let mut indirect_callers = Vec::new();
        let mut exposed_endpoints = Vec::new();
        let mut affected_datastores = HashSet::new();

        if let Some(target) = &target_node {
            // BFS traversal for callers
            let mut visited = HashSet::new();
            let mut queue = VecDeque::new();

            visited.insert(target.id.clone());

            // 1. Direct callers
            let direct = self.store.get_callers(&target.id)?;
            for (edge, caller) in direct {
                if visited.insert(caller.id.clone()) {
                    queue.push_back((caller.clone(), 1)); // depth 1
                    direct_callers.push(caller);
                }
                if let Some(ds) = edge.qualifier.get("datastore").and_then(|v| v.as_str()) {
                    affected_datastores.insert(ds.to_string());
                }
            }

            // 2. Transitive indirect callers (depth 2+)
            while let Some((curr_node, depth)) = queue.pop_front() {
                if depth >= 5 {
                    continue; // max depth
                }

                let callers = self.store.get_callers(&curr_node.id)?;
                for (_, parent_caller) in callers {
                    if visited.insert(parent_caller.id.clone()) {
                        if parent_caller.kind == NodeKind::Endpoint || parent_caller.kind == NodeKind::Api {
                            exposed_endpoints.push(parent_caller.clone());
                        } else {
                            indirect_callers.push(parent_caller.clone());
                        }
                        queue.push_back((parent_caller, depth + 1));
                    }
                }
            }
        }

        Ok(ImpactReport {
            target_symbol: symbol_query.to_string(),
            target_node,
            direct_callers,
            indirect_callers,
            exposed_endpoints,
            affected_datastores: affected_datastores.into_iter().collect(),
        })
    }

    /// PRD Part II Section 30: End-to-End Path Analysis
    pub fn find_path(&self, source_query: &str, target_query: &str) -> anyhow::Result<ExecutionPath> {
        let source_candidates = self.store.find_nodes_by_name(source_query)?;
        let target_candidates = self.store.find_nodes_by_name(target_query)?;

        let source = source_candidates.first();
        let target = target_candidates.first();

        if let (Some(src), Some(dst)) = (source, target) {
            let mut visited = HashSet::new();
            let mut queue = VecDeque::new();
            let mut parent_map = std::collections::HashMap::new();

            visited.insert(src.id.clone());
            queue.push_back(src.clone());

            let mut found = false;

            while let Some(curr) = queue.pop_front() {
                if curr.id == dst.id {
                    found = true;
                    break;
                }

                let callees = self.store.get_callees(&curr.id)?;
                for (_, next_node) in callees {
                    if visited.insert(next_node.id.clone()) {
                        parent_map.insert(next_node.id.clone(), curr.clone());
                        queue.push_back(next_node);
                    }
                }
            }

            if found {
                let mut path = Vec::new();
                let mut curr_id = dst.id.clone();
                if let Some(node) = self.store.get_node(&curr_id)? {
                    path.push(node);
                }

                while let Some(parent) = parent_map.get(&curr_id) {
                    path.push(parent.clone());
                    curr_id = parent.id.clone();
                    if curr_id == src.id {
                        break;
                    }
                }
                path.reverse();

                return Ok(ExecutionPath {
                    source: source_query.to_string(),
                    target: target_query.to_string(),
                    path,
                    found: true,
                });
            }
        }

        Ok(ExecutionPath {
            source: source_query.to_string(),
            target: target_query.to_string(),
            path: Vec::new(),
            found: false,
        })
    }
}
