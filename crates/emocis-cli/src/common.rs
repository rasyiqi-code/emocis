use std::fs;
use std::path::{Path, PathBuf};
use emocis_cclg::AdapterRegistry;
use emocis_core::{Snapshot, SqliteStore};
use emocis_memory::KnowledgeExtractor;

pub const EMOCIS_DIR: &str = ".emocis";
pub const DB_NAME: &str = "emocis.db";

pub fn get_db_path() -> PathBuf {
    PathBuf::from(EMOCIS_DIR).join(DB_NAME)
}

pub fn open_or_init_store() -> anyhow::Result<SqliteStore> {
    let dir = Path::new(EMOCIS_DIR);
    if !dir.exists() {
        fs::create_dir_all(dir)?;
    }
    let db_path = get_db_path();
    let store = SqliteStore::open(db_path)?;
    Ok(store)
}

/// Recursively index a project directory
pub fn index_directory(root_path: &Path, project_id: &str) -> anyhow::Result<(usize, usize, usize)> {
    let store = open_or_init_store()?;
    store.insert_project(project_id, project_id, root_path.to_str().unwrap_or("."))?;

    let snapshot = Snapshot {
        id: format!("snapshot:{}", chrono::Utc::now().timestamp()),
        project_id: project_id.to_string(),
        commit_hash: "working_tree".to_string(),
        branch: Some("main".to_string()),
        author: None,
        message: Some("Automated indexing".to_string()),
        created_at: chrono::Utc::now(),
    };
    store.insert_snapshot(&snapshot)?;

    let registry = AdapterRegistry::new();
    let knowledge_extractor = KnowledgeExtractor::new();

    let mut total_files = 0;
    let mut total_nodes = 0;
    let mut total_knowledge = 0;

    store.execute_batch("BEGIN TRANSACTION;")?;

    let index_res = index_dir_recursive(
        root_path,
        root_path,
        project_id,
        &registry,
        &knowledge_extractor,
        &store,
        &mut total_files,
        &mut total_nodes,
        &mut total_knowledge,
    );

    match index_res {
        Ok(()) => {
            store.execute_batch("COMMIT;")?;
            Ok((total_files, total_nodes, total_knowledge))
        }
        Err(err) => {
            let _ = store.execute_batch("ROLLBACK;");
            Err(err)
        }
    }
}

fn index_dir_recursive(
    current_dir: &Path,
    root_path: &Path,
    project_id: &str,
    registry: &AdapterRegistry,
    knowledge_extractor: &KnowledgeExtractor,
    store: &SqliteStore,
    total_files: &mut usize,
    total_nodes: &mut usize,
    total_knowledge: &mut usize,
) -> anyhow::Result<()> {
    if !current_dir.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(current_dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // Skip hidden, git, target, node_modules, build outputs
        if file_name.starts_with('.')
            || file_name == "target"
            || file_name == "node_modules"
            || file_name == "dist"
            || file_name == "build"
            || file_name == "out"
            || file_name == "coverage"
        {
            continue;
        }

        if path.is_dir() {
            index_dir_recursive(
                &path,
                root_path,
                project_id,
                registry,
                knowledge_extractor,
                store,
                total_files,
                total_nodes,
                total_knowledge,
            )?;
        } else if path.is_file() {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let relative_path = path.strip_prefix(root_path).unwrap_or(&path).to_str().unwrap_or("");

            // 1. Markdown Documentation indexing
            if ext == "md" {
                if let Ok(content) = fs::read_to_string(&path) {
                    *total_files += 1;
                    let k_items = knowledge_extractor.extract_from_markdown(project_id, relative_path, &content);
                    for item in k_items {
                        *total_knowledge += 1;
                        store.insert_knowledge_item(&item)?;
                    }
                }
            }

            // 2. Code indexing (Polyglot: TypeScript, JavaScript, Rust, etc.)
            if let Some(adapter) = registry.get_adapter_for_path(&path) {
                if let Ok(content) = fs::read_to_string(&path) {
                    *total_files += 1;

                    // Knowledge from comments
                    let comment_items = knowledge_extractor.extract_from_code(project_id, relative_path, &content);
                    for item in comment_items {
                        *total_knowledge += 1;
                        store.insert_knowledge_item(&item)?;
                    }

                    // CCLG Code Graph from AST
                    if let Ok(parsed) = adapter.parse_file(project_id, relative_path, &content) {
                        for node in &parsed.nodes {
                            *total_nodes += 1;
                            store.insert_node(node)?;
                        }
                        for edge in &parsed.edges {
                            store.insert_edge(edge)?;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
