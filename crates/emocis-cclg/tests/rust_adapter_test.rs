use emocis_cclg::{LanguageAdapter, RustAdapter};
use emocis_core::NodeKind;

#[test]
fn test_rust_adapter_parsing() {
    let adapter = RustAdapter::new();
    let rust_code = r#"
use crate::local_db::get_conn;
use crate::models::{Buyer, Product};
use super::helper;

pub fn calculate_total(qty: u32, price: f64) -> f64 {
    helper();
    let conn = get_conn();
    (qty as f64) * price
}

fn private_helper() {
    println!("test");
}
"#;

    let parsed = adapter.parse_file("proj1", "src/services/calc.rs", rust_code).expect("Should parse rust code");
    
    // Check nodes
    let names: Vec<&str> = parsed.nodes.iter().map(|n| n.name.as_str()).collect();
    assert!(names.contains(&"src/services/calc.rs"));
    assert!(names.contains(&"calculate_total"));
    assert!(names.contains(&"private_helper"));

    let fn_node = parsed.nodes.iter().find(|n| n.name == "calculate_total").unwrap();
    assert_eq!(fn_node.kind, NodeKind::Function);

    let file_node = parsed.nodes.iter().find(|n| n.kind == NodeKind::File).unwrap();
    let imports = file_node.metadata["imports"].as_array().expect("imports should be array");
    assert!(imports.iter().any(|i| i["module"] == "crate::local_db" && i["local"] == "get_conn"));
    assert!(imports.iter().any(|i| i["module"] == "crate::models" && i["local"] == "Buyer"));
    assert!(imports.iter().any(|i| i["module"] == "crate::models" && i["local"] == "Product"));
    assert!(imports.iter().any(|i| i["module"] == "super" && i["local"] == "helper"));

    let exports = file_node.metadata["exports"].as_array().expect("exports should be array");
    assert!(exports.iter().any(|e| e["exported"] == "calculate_total"));
    assert!(!exports.iter().any(|e| e["exported"] == "private_helper"));
}

#[test]
fn test_rust_reexports_and_wildcards() {
    let adapter = RustAdapter::new();
    let code = r#"
pub use crate::db::query;
use super::helper::*;
use crate::models::{Product as Prod};

pub fn run() {
    query();
}
"#;
    let parsed = adapter.parse_file("proj1", "src/services/query_svc.rs", code).unwrap();
    let file_node = parsed.nodes.iter().find(|n| n.kind == NodeKind::File).unwrap();
    let imports = file_node.metadata["imports"].as_array().unwrap();
    assert!(imports.iter().any(|i| i["module"] == "super::helper" && i["local"] == "*"));
    assert!(imports.iter().any(|i| i["module"] == "crate::models" && i["local"] == "Prod" && i["imported"] == "Product"));

    let exports = file_node.metadata["exports"].as_array().unwrap();
    assert!(exports.iter().any(|e| e["exported"] == "query"));
    assert!(exports.iter().any(|e| e["exported"] == "run"));
}

#[test]
fn test_tauri_command_extraction() {
    let adapter = RustAdapter::new();
    let code = r#"
#[tauri::command]
pub fn create_transaction(amount: f64) -> bool {
    true
}
"#;
    let parsed = adapter.parse_file("proj1", "src-tauri/src/cmd.rs", code).unwrap();
    let fn_node = parsed.nodes.iter().find(|n| n.name == "create_transaction").unwrap();
    assert_eq!(fn_node.metadata["tauri_command"], true);
    assert_eq!(fn_node.metadata["ipc_command"], "create_transaction");

    let file_node = parsed.nodes.iter().find(|n| n.kind == NodeKind::File).unwrap();
    let exports = file_node.metadata["exports"].as_array().unwrap();
    assert!(exports.iter().any(|e| e["exported"] == "tauri_ipc:create_transaction"));
}






