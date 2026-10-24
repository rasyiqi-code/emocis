use emocis_cclg::{link_project_calls, LanguageAdapter, RustAdapter, TypeScriptAdapter};
use emocis_core::{EdgeKind, SqliteStore};

#[test]
fn test_tauri_ipc_semantic_bridge() {
    let store = SqliteStore::open_in_memory().unwrap();
    let project_id = "tauri_pos_proj";
    store.insert_project(project_id, project_id, ".").unwrap();

    let rust_adapter = RustAdapter::new();
    let ts_adapter = TypeScriptAdapter::new();

    // 1. Backend Rust with #[tauri::command]
    let rust_code = r#"
#[tauri::command]
pub fn db_create_transaction(amount: f64) -> bool {
    true
}

pub fn internal_calc() {}
"#;
    let graph_rust = rust_adapter.parse_file(project_id, "src-tauri/src/db.rs", rust_code).unwrap();
    for node in graph_rust.nodes {
        store.insert_node(&node).unwrap();
    }
    for edge in graph_rust.edges {
        store.insert_edge(&edge).unwrap();
    }

    // 2. Frontend TypeScript calling invoke('db_create_transaction', ...)
    let ts_code = r#"
import { invoke } from '@tauri-apps/api/core';

export async function processPayment(amount: number) {
    return await invoke<string>('db_create_transaction', { amount });
}
"#;
    let graph_ts = ts_adapter.parse_file(project_id, "src/views/ReceiptModal.ts", ts_code).unwrap();
    for node in graph_ts.nodes {
        store.insert_node(&node).unwrap();
    }
    for edge in graph_ts.edges {
        store.insert_edge(&edge).unwrap();
    }

    // 3. Run linker
    link_project_calls(&store, project_id).unwrap();

    // 4. Verify cross-language IPC call edge
    let all_edges = store.get_all_edges().unwrap();
    let ipc_call_edges: Vec<_> = all_edges
        .into_iter()
        .filter(|e| e.kind == EdgeKind::Calls && e.qualifier["ipc_command"] == "db_create_transaction")
        .collect();

    assert!(!ipc_call_edges.is_empty(), "Should find IPC call edges");

    for edge in ipc_call_edges {
        let target_node = store.get_node(&edge.to).unwrap().expect("Target node exists");
        assert_eq!(target_node.name, "db_create_transaction");
        assert_eq!(target_node.source.as_ref().unwrap().file, "src-tauri/src/db.rs");
        assert_eq!(target_node.language.as_deref(), Some("rust"));
    }
}
