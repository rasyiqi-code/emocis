use emocis_cclg::{link_project_calls, LanguageAdapter, PythonAdapter, RustAdapter, TypeScriptAdapter};
use emocis_core::{EdgeKind, SqliteStore};

#[test]
fn test_rust_cross_file_linking() {
    let store = SqliteStore::open_in_memory().unwrap();
    let project_id = "rust_test_proj";
    store.insert_project(project_id, project_id, ".").unwrap();

    let rust_adapter = RustAdapter::new();

    // File 1: src/local_db.rs
    let code_b = r#"
pub fn get_conn() -> String {
    "connected".to_string()
}
"#;
    let graph_b = rust_adapter.parse_file(project_id, "src/local_db.rs", code_b).unwrap();
    for node in graph_b.nodes {
        store.insert_node(&node).unwrap();
    }
    for edge in graph_b.edges {
        store.insert_edge(&edge).unwrap();
    }

    // File 2: src/services/calc.rs
    let code_a = r#"
use crate::local_db::get_conn;

pub fn calculate() {
    let _conn = get_conn();
}
"#;
    let graph_a = rust_adapter.parse_file(project_id, "src/services/calc.rs", code_a).unwrap();
    for node in graph_a.nodes {
        store.insert_node(&node).unwrap();
    }
    for edge in graph_a.edges {
        store.insert_edge(&edge).unwrap();
    }

    // Execute linker
    link_project_calls(&store, project_id).unwrap();

    // Verify after linking:
    let all_edges = store.get_all_edges().unwrap();
    let call_edges: Vec<_> = all_edges.into_iter().filter(|e| e.kind == EdgeKind::Calls).collect();
    let linked_edge = call_edges.iter().find(|e| e.qualifier["raw_callee"] == "get_conn").expect("Found get_conn call");
    let target_node = store.get_node(&linked_edge.to).unwrap().expect("Target node exists");
    assert_eq!(target_node.name, "get_conn");
    assert_eq!(target_node.source.as_ref().unwrap().file, "src/local_db.rs");
}

#[test]
fn test_typescript_alias_linking() {
    let store = SqliteStore::open_in_memory().unwrap();
    let project_id = "ts_test_proj";
    store.insert_project(project_id, project_id, ".").unwrap();

    let ts_adapter = TypeScriptAdapter::new();

    // File 1: src/utils/format.ts
    let code_format = r#"
export function formatCurrency(val: number): string {
    return "$" + val;
}
"#;
    let graph_format = ts_adapter.parse_file(project_id, "src/utils/format.ts", code_format).unwrap();
    for node in graph_format.nodes {
        store.insert_node(&node).unwrap();
    }
    for edge in graph_format.edges {
        store.insert_edge(&edge).unwrap();
    }

    // File 2: src/views/Dashboard.ts
    let code_dashboard = r#"
import { formatCurrency } from '@/utils/format';

export function renderDashboard() {
    return formatCurrency(150);
}
"#;
    let graph_dashboard = ts_adapter.parse_file(project_id, "src/views/Dashboard.ts", code_dashboard).unwrap();
    for node in graph_dashboard.nodes {
        store.insert_node(&node).unwrap();
    }
    for edge in graph_dashboard.edges {
        store.insert_edge(&edge).unwrap();
    }

    // Execute linker
    link_project_calls(&store, project_id).unwrap();

    // Verify linking
    let all_edges = store.get_all_edges().unwrap();
    let call_edges: Vec<_> = all_edges.into_iter().filter(|e| e.kind == EdgeKind::Calls).collect();
    for e in &call_edges {
        println!("TS Call edge: from={} to={} qual={:?}", e.from, e.to, e.qualifier);
    }
    let linked_edge = call_edges.iter().find(|e| e.qualifier["raw_callee"] == "formatCurrency").expect("Found formatCurrency call");
    let target_node = store.get_node(&linked_edge.to).unwrap().expect("Target node exists");
    assert_eq!(target_node.name, "formatCurrency");
    assert_eq!(target_node.source.as_ref().unwrap().file, "src/utils/format.ts");
}

#[test]
fn test_python_cross_file_linking() {
    let store = SqliteStore::open_in_memory().unwrap();
    let project_id = "py_test_proj";
    store.insert_project(project_id, project_id, ".").unwrap();

    let py_adapter = PythonAdapter::new();

    // File 1: pkg/services/db.py
    let code_db = r#"
def get_session():
    return "session"
"#;
    let graph_db = py_adapter.parse_file(project_id, "pkg/services/db.py", code_db).unwrap();
    for node in graph_db.nodes {
        store.insert_node(&node).unwrap();
    }
    for edge in graph_db.edges {
        store.insert_edge(&edge).unwrap();
    }

    // File 2: pkg/services/tax.py
    let code_tax = r#"
from .db import get_session

def calculate_tax():
    s = get_session()
    return 10
"#;
    let graph_tax = py_adapter.parse_file(project_id, "pkg/services/tax.py", code_tax).unwrap();
    for node in graph_tax.nodes {
        store.insert_node(&node).unwrap();
    }
    for edge in graph_tax.edges {
        store.insert_edge(&edge).unwrap();
    }

    // Execute linker
    link_project_calls(&store, project_id).unwrap();

    // Verify linking
    let all_edges = store.get_all_edges().unwrap();
    let call_edges: Vec<_> = all_edges.into_iter().filter(|e| e.kind == EdgeKind::Calls).collect();
    assert_eq!(call_edges.len(), 1);

    let linked_edge = &call_edges[0];
    let target_node = store.get_node(&linked_edge.to).unwrap().expect("Target node exists");
    assert_eq!(target_node.name, "get_session");
    assert_eq!(target_node.source.as_ref().unwrap().file, "pkg/services/db.py");
}
