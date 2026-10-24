use emocis_core::{EdgeKind, NodeKind, SqliteStore};
use petgraph::{algo::kosaraju_scc, graphmap::DiGraphMap};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDependencies {
    pub file: String,
    /// Number of distinct other files calling into this file.
    pub fan_in: usize,
    /// Number of distinct other files called by this file.
    pub fan_out: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMeasurements {
    pub indexed_files: usize,
    pub indexed_nodes: usize,
    pub indexed_functions: usize,
    /// Unique caller-target pairs; callsite mirror edges are excluded.
    pub call_relationships: usize,
    pub resolved_calls: usize,
    pub unresolved_calls: usize,
    pub internal_file_calls: usize,
    pub cross_file_calls: usize,
    pub resolution_percent: Option<f64>,
    pub cross_file_percent: Option<f64>,
    /// Strongly connected file components, sorted for reproducible output.
    pub dependency_cycles: Vec<Vec<String>>,
    pub files: Vec<FileDependencies>,
}

impl GraphMeasurements {
    pub fn compute(store: &SqliteStore, project_id: &str) -> anyhow::Result<Self> {
        let all_nodes = store.find_nodes_by_name("")?;
        let nodes: HashMap<_, _> = all_nodes
            .iter()
            .filter(|n| n.project_id == project_id)
            .map(|n| (n.id.as_str(), n))
            .collect();
        let mut files = BTreeSet::new();
        for node in nodes.values() {
            if let Some(source) = &node.source {
                files.insert(source.file.as_str());
            }
        }
        let mut graph = DiGraphMap::<&str, ()>::new();
        let mut incoming: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
        let mut outgoing: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
        for file in &files {
            graph.add_node(*file);
        }
        let mut seen = HashSet::new();
        let mut resolved_calls = 0;
        let mut internal_file_calls = 0;
        let mut cross_file_calls = 0;
        for edge in store.get_all_edges()? {
            if edge.kind != EdgeKind::Calls {
                continue;
            }
            let Some(caller) = nodes.get(edge.from.as_str()) else {
                continue;
            };
            if caller.kind == NodeKind::CallSite
                || !seen.insert((edge.from.clone(), edge.to.clone()))
            {
                continue;
            }
            let Some(target) = nodes.get(edge.to.as_str()) else {
                continue;
            };
            resolved_calls += 1;
            if let (Some(from), Some(to)) = (&caller.source, &target.source) {
                if from.file == to.file {
                    internal_file_calls += 1;
                } else {
                    cross_file_calls += 1;
                    graph.add_edge(from.file.as_str(), to.file.as_str(), ());
                    incoming.entry(&to.file).or_default().insert(&from.file);
                    outgoing.entry(&from.file).or_default().insert(&to.file);
                }
            }
        }
        let mut dependency_cycles: Vec<Vec<String>> = kosaraju_scc(&graph)
            .into_iter()
            .filter(|component| component.len() > 1)
            .map(|component| {
                let mut paths: Vec<_> = component.into_iter().map(str::to_owned).collect();
                paths.sort();
                paths
            })
            .collect();
        dependency_cycles.sort();
        let percent =
            |part: usize, total: usize| (total > 0).then(|| part as f64 * 100.0 / total as f64);
        Ok(Self {
            indexed_files: files.len(),
            indexed_nodes: nodes.len(),
            indexed_functions: nodes
                .values()
                .filter(|n| matches!(n.kind, NodeKind::Function | NodeKind::Method))
                .count(),
            call_relationships: seen.len(),
            resolved_calls,
            unresolved_calls: seen.len() - resolved_calls,
            internal_file_calls,
            cross_file_calls,
            resolution_percent: percent(resolved_calls, seen.len()),
            cross_file_percent: percent(cross_file_calls, resolved_calls),
            dependency_cycles,
            files: files
                .into_iter()
                .map(|file| FileDependencies {
                    file: file.to_owned(),
                    fan_in: incoming.get(file).map_or(0, BTreeSet::len),
                    fan_out: outgoing.get(file).map_or(0, BTreeSet::len),
                })
                .collect(),
        })
    }
}
