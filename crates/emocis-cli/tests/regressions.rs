use emocis_core::SqliteStore;
use emocis_memory::{ContextCompressor, KnowledgeExtractor, RefactoringRadar};
use serde_json::{json, Value};
use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Output, Stdio},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "emocis-test-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap()
        ));
        let path = root.join(name);
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
    fn write(&self, name: &str, source: &str) {
        fs::create_dir_all(self.0.join(name).parent().unwrap()).unwrap();
        fs::write(self.0.join(name), source).unwrap();
    }
    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_project"))
            .args(args)
            .current_dir(&self.0)
            .output()
            .unwrap()
    }
    fn map(&self) {
        let out = self.run(&["map"]);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    fn store(&self) -> SqliteStore {
        SqliteStore::open(self.0.join(".emocis/emocis.db")).unwrap()
    }
    fn tool(&self, name: &str, args: Value) -> Value {
        let mut process = Command::new(env!("CARGO_BIN_EXE_emocis"))
            .arg("mcp")
            .current_dir(&self.0)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        writeln!(process.stdin.take().unwrap(), "{}", json!({"jsonrpc":"2.0", "id":1, "method":"tools/call", "params":{"name":name,"arguments":args}})).unwrap();
        let output = process.wait_with_output().unwrap();
        assert!(output.status.success());
        let response: Value = serde_json::from_slice(&output.stdout).unwrap();
        serde_json::from_str(response["result"]["content"][0]["text"].as_str().unwrap()).unwrap()
    }
}

#[test]
fn default_arrow_expression_and_reexport_bindings_resolve() {
    let f = Fixture::new("export-bindings");
    f.write("main.ts", "import renamed from './default';\nimport anonymous from './anonymous';\nimport named from './named';\nimport { renamedArrow, expression, execute } from './barrel';\nexport const run = () => { renamed(); anonymous(); named(); renamedArrow(); expression(); execute(); };\n");
    f.write(
        "default.ts",
        "const implementation = () => 42; export default implementation;",
    );
    f.write(
        "anonymous.ts",
        "export default () => helper(); function helper() {};",
    );
    f.write("named.ts", "export default function realName() {};");
    f.write(
        "arrow.ts",
        "export const arrow = () => 1; export const expression = function() { return 2; };",
    );
    f.write("barrel.ts", "export { arrow as renamedArrow, expression } from './arrow'; export { default as execute } from './default';");
    f.map();
    let store = f.store();
    let run = store
        .find_nodes_by_name("run")
        .unwrap()
        .into_iter()
        .find(|n| n.kind == emocis_core::NodeKind::Function)
        .unwrap();
    let callees = store.get_callees(&run.id).unwrap();
    // Two import aliases point at the same implementation, counted once.
    assert_eq!(callees.len(), 5);
    assert!(callees.iter().all(|(_, target)| target.source.is_some()));
    let helper = store
        .find_nodes_by_name("helper")
        .unwrap()
        .into_iter()
        .find(|n| n.kind == emocis_core::NodeKind::Function)
        .unwrap();
    assert_eq!(
        store.get_callers(&helper.id).unwrap()[0].1.name,
        "<default>"
    );
}

#[test]
fn export_resolution_rejects_private_type_only_and_cyclic_bindings() {
    let f = Fixture::new("unresolved-exports");
    f.write("main.ts", "import { hidden } from './private'; import type { typed } from './private'; import { loop } from './one'; function run() { hidden(); typed(); loop(); }");
    f.write(
        "private.ts",
        "function hidden() {} export function typed() {}",
    );
    f.write("one.ts", "export { loop } from './two';");
    f.write("two.ts", "export { loop } from './one';");
    f.map();
    let health = f.tool("emocis_health", json!({}));
    assert_eq!(health["graph"]["unresolved_calls"], 3);
    assert_eq!(health["graph"]["resolved_calls"], 0);
}

#[test]
fn graph_measurements_count_unique_relations_and_detect_long_cycles() {
    let f = Fixture::new("measurements");
    f.write("a.ts", "import { b } from './b'; export function a() { b(); b(); helper(); console.log('external'); } function helper() {}");
    f.write(
        "b.ts",
        "import { c } from './c'; export const b = () => c();",
    );
    f.write(
        "c.ts",
        "import { a } from './a'; export function c() { a(); }",
    );
    f.map();
    let health = f.tool("emocis_health", json!({}));
    let graph = &health["graph"];
    assert_eq!(graph["indexed_files"], 3);
    assert_eq!(graph["indexed_functions"], 4);
    assert_eq!(graph["call_relationships"], 5);
    assert_eq!(graph["resolved_calls"], 4);
    assert_eq!(graph["unresolved_calls"], 1);
    assert_eq!(graph["internal_file_calls"], 1);
    assert_eq!(graph["cross_file_calls"], 3);
    assert_eq!(graph["resolution_percent"], 80.0);
    assert_eq!(graph["cross_file_percent"], 75.0);
    assert_eq!(
        graph["dependency_cycles"],
        json!([["a.ts", "b.ts", "c.ts"]])
    );
    assert!(graph["files"]
        .as_array()
        .unwrap()
        .iter()
        .all(|f| f["fan_in"] == 1 && f["fan_out"] == 1));
    assert!(health["overall_score"].as_u64().is_some());
    let audit = f.tool("emocis_audit", json!({}));
    assert_eq!(audit["total_violations"], 1);
    assert!(audit["violations"][0]["title"]
        .as_str()
        .unwrap()
        .contains("3 files"));
}

#[test]
fn empty_graph_ratios_are_unavailable_and_other_projects_are_excluded() {
    use emocis_cclg::{LanguageAdapter, TypeScriptAdapter};
    let store = SqliteStore::open_in_memory().unwrap();
    store.insert_project("empty", "empty", ".").unwrap();
    store.insert_project("other", "other", ".").unwrap();
    let parsed = TypeScriptAdapter::new()
        .parse_file("other", "other.ts", "function unused() { external(); }")
        .unwrap();
    for node in parsed.nodes {
        store.insert_node(&node).unwrap();
    }
    for edge in parsed.edges {
        store.insert_edge(&edge).unwrap();
    }
    let health = emocis_memory::HealthScorer::new(&store)
        .compute_health("empty")
        .unwrap();
    assert_eq!(health.graph.indexed_files, 0);
    assert_eq!(health.graph.call_relationships, 0);
    assert_eq!(health.active_violations_count, 0);
    assert!(health.graph.resolution_percent.is_none());
    assert!(health.graph.cross_file_percent.is_none());
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(self.0.parent().unwrap());
    }
}

#[test]
fn reindex_preserves_manual_memory_and_removes_stale_extractions() {
    let f = Fixture::new("sample");
    f.write("a.ts", "// @warning stale warning\nfunction old() {}\n");
    f.map();
    for approach in ["Redis lock", "File lock"] {
        assert!(f
            .run(&[
                "failed",
                "--approach",
                approach,
                "--reason",
                "race",
                "--replaced-by",
                "DB lock",
                "--provenance",
                "PR1"
            ])
            .status
            .success());
    }
    f.write("a.ts", "function replacement() {}\n");
    f.map();
    let store = f.store();
    let items = store.list_knowledge_items("sample").unwrap();
    assert_eq!(items.len(), 2);
    assert!(items.iter().all(|i| !i.evidence.is_empty()));
    assert!(store.find_nodes_by_name("old").unwrap().is_empty());
    assert_eq!(store.find_nodes_by_name("replacement").unwrap().len(), 1);
}

#[test]
fn relative_imports_link_to_the_correct_file_without_double_counting_callsites() {
    let f = Fixture::new("imports");
    f.write("a.ts", "import { target as alias } from './b';\nimport * as api from './b';\nexport function caller() { alias(); api.target(); }\n");
    f.write("b.ts", "export function target() { return 1; }\n");
    f.write("decoy.ts", "export function target() { return 2; }\n");
    f.map();
    let store = f.store();
    let target = store
        .find_nodes_by_file("b.ts")
        .unwrap()
        .into_iter()
        .find(|n| n.name == "target")
        .unwrap();
    let callers = store.get_callers(&target.id).unwrap();
    assert_eq!(callers.len(), 1);
    assert_eq!(callers[0].1.name, "caller");
    let decoy = store
        .find_nodes_by_file("decoy.ts")
        .unwrap()
        .into_iter()
        .find(|n| n.name == "target")
        .unwrap();
    assert!(store.get_callers(&decoy.id).unwrap().is_empty());
    let impact = f.tool("emocis_impact", json!({"symbol": "target"}));
    assert_eq!(impact["direct_callers"].as_array().unwrap().len(), 1);
}

#[test]
fn unicode_and_quoted_project_names_are_safe() {
    let f = Fixture::new("owner's-repo");
    f.write(
        "a.ts",
        &format!("// @warning {}\nfunction caller() {{}}\n", "😀".repeat(80)),
    );
    f.map();
    assert_eq!(
        f.store()
            .list_knowledge_items("owner's-repo")
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn failed_read_rolls_back_previous_index() {
    let f = Fixture::new("rollback");
    f.write("a.ts", "function original() {}\n");
    f.map();
    fs::write(f.0.join("a.ts"), [0xff, 0xfe]).unwrap();
    assert!(!f.run(&["map"]).status.success());
    assert_eq!(f.store().find_nodes_by_name("original").unwrap().len(), 1);
}

#[test]
fn refactoring_counts_functions_by_source_file() {
    let f = Fixture::new("refactor");
    f.write("a.ts", "function caller() {}\nfunction another() {}\n");
    f.map();
    let store = f.store();
    let report = RefactoringRadar::new(&store)
        .diagnose_file("a.ts", "function caller() {}\nfunction another() {}\n")
        .unwrap();
    assert_eq!(report.function_count, 2);
}

#[test]
fn mcp_uses_actual_project_and_unmeasured_health_stays_null() {
    let f = Fixture::new("real-project");
    f.map();
    let store = f.store();
    let mut item = KnowledgeExtractor::new()
        .extract_from_code("real-project", "a.ts", "// @warning missing entity")
        .remove(0);
    item.affects = vec!["DeletedSymbol".to_owned()];
    store.insert_knowledge_item(&item).unwrap();
    let audit = f.tool("emocis_audit", json!({}));
    assert_eq!(audit["total_violations"], 1);
    assert_eq!(audit["violations"].as_array().unwrap().len(), 1);
    let health = f.tool("emocis_health", json!({}));
    assert_eq!(health["active_violations_count"], 1);
    assert!(health["overall_score"].is_null());
    assert_eq!(health["status"], "not_indexed");
}

#[test]
fn keyword_context_is_bounded_and_includes_business_rules() {
    let store = SqliteStore::open_in_memory().unwrap();
    store.insert_project("p", "p", ".").unwrap();
    let extractor = KnowledgeExtractor::new();
    for i in 0..100 {
        let items = extractor.extract_from_code(
            "p",
            &format!("{i}.ts"),
            &format!("// @rule Payment retry {}", "😀".repeat(80)),
        );
        store.insert_knowledge_item(&items[0]).unwrap();
    }
    assert_eq!(
        store
            .search_knowledge("refactor payment workflow")
            .unwrap()
            .len(),
        100
    );
    let context = ContextCompressor::new(&store)
        .build_context("refactor payment workflow")
        .unwrap();
    assert!(!context.active_constraints.is_empty());
    assert!(context.truncated);
    assert!(context.formatted_markdown.chars().count() <= 8000);
}

#[test]
fn source_cleanup_is_project_scoped_and_preserves_unattributed_manual_items() {
    let store = SqliteStore::open_in_memory().unwrap();
    let extractor = KnowledgeExtractor::new();
    for project in ["first", "second"] {
        store.insert_project(project, project, ".").unwrap();
        let source = extractor
            .extract_from_code(
                project,
                &format!("{project}.ts"),
                "// @warning indexed warning",
            )
            .remove(0);
        store.insert_knowledge_item(&source).unwrap();
    }
    let mut manual = extractor
        .extract_from_code("first", "manual.ts", "// @decision manually entered")
        .remove(0);
    manual.evidence.clear();
    store.insert_knowledge_item(&manual).unwrap();
    store.execute_batch("BEGIN;").unwrap();
    store.clear_source_index("first").unwrap();
    store.execute_batch("COMMIT;").unwrap();
    assert_eq!(store.list_knowledge_items("first").unwrap().len(), 1);
    assert_eq!(
        store.list_knowledge_items("first").unwrap()[0].id,
        manual.id
    );
    assert_eq!(store.list_knowledge_items("second").unwrap().len(), 1);
    assert_eq!(
        store.list_knowledge_items("second").unwrap()[0]
            .evidence
            .len(),
        1
    );
}
