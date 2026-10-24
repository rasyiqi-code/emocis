use emocis_cclg::{link_project_calls, AdapterRegistry};
use emocis_core::{Snapshot, SqliteStore};
use emocis_memory::KnowledgeExtractor;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

pub const EMOCIS_DIR: &str = ".emocis";
pub const DB_NAME: &str = "emocis.db";

pub fn current_project_id() -> anyhow::Result<String> {
    Ok(std::env::current_dir()?
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("project")
        .to_owned())
}

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

#[derive(Debug, Clone, Copy)]
pub struct IndexStats {
    pub total_files: usize,
    pub indexed_files: usize,
    pub cached_files: usize,
    pub total_nodes: usize,
    pub total_knowledge: usize,
}


/// Recursively index a project directory with optional forced re-indexing
pub fn index_directory_ext(
    root_path: &Path,
    project_id: &str,
    force: bool,
) -> anyhow::Result<IndexStats> {
    let store = open_or_init_store()?;
    store.execute_batch("BEGIN IMMEDIATE;")?;
    let result = (|| -> anyhow::Result<IndexStats> {
        store.insert_project(project_id, project_id, root_path.to_str().unwrap_or("."))?;

        let snapshot = Snapshot {
            id: format!(
                "snapshot:{}:{}",
                project_id,
                chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
            ),
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

        if force {
            store.clear_source_index(project_id)?;
        }

        let stored_hashes = if force {
            HashMap::new()
        } else {
            store.get_file_hashes(project_id)?
        };

        let mut seen_files = HashSet::new();
        let mut total_files = 0;
        let mut indexed_files = 0;
        let mut cached_files = 0;

        index_dir_recursive(
            root_path,
            root_path,
            project_id,
            &registry,
            &knowledge_extractor,
            &store,
            &stored_hashes,
            &mut seen_files,
            &mut total_files,
            &mut indexed_files,
            &mut cached_files,
        )?;

        // Cleanup deleted files
        for old_file in stored_hashes.keys() {
            if !seen_files.contains(old_file) {
                store.clear_file_index(project_id, old_file)?;
            }
        }

        // Re-link cross-file calls if files were indexed/modified or if forced
        if indexed_files > 0 || force || stored_hashes.len() != seen_files.len() {
            link_project_calls(&store, project_id)?;
        }

        let total_nodes = store.count_nodes(project_id)?;
        let total_knowledge = store.count_knowledge_items(project_id)?;

        Ok(IndexStats {
            total_files,
            indexed_files,
            cached_files,
            total_nodes,
            total_knowledge,
        })
    })();
    match result {
        Ok(stats) => {
            store.execute_batch("COMMIT;")?;
            Ok(stats)
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
    stored_hashes: &HashMap<String, String>,
    seen_files: &mut HashSet<String>,
    total_files: &mut usize,
    indexed_files: &mut usize,
    cached_files: &mut usize,
) -> anyhow::Result<()> {
    if !current_dir.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(current_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_symlink() {
            continue;
        }
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
                stored_hashes,
                seen_files,
                total_files,
                indexed_files,
                cached_files,
            )?;
        } else if path.is_file() {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let relative_path = path
                .strip_prefix(root_path)
                .unwrap_or(&path)
                .to_str()
                .unwrap_or("");

            if ext == "md" {
                *total_files += 1;
                seen_files.insert(relative_path.to_string());
                let content = fs::read_to_string(&path)?;
                let hash = format!("{:x}", Sha256::digest(content.as_bytes()));

                if stored_hashes.get(relative_path) == Some(&hash) {
                    *cached_files += 1;
                } else {
                    *indexed_files += 1;
                    if stored_hashes.contains_key(relative_path) {
                        store.clear_file_index(project_id, relative_path)?;
                    }
                    for item in knowledge_extractor.extract_from_markdown(project_id, relative_path, &content) {
                        store.insert_knowledge_item(&item)?;
                    }
                    store.set_file_hash(project_id, relative_path, &hash)?;
                }
            }

            // Comments are supported in these languages; AST coverage depends on the registry.
            let is_code_ext = matches!(
                ext,
                "ts" | "tsx"
                    | "js"
                    | "jsx"
                    | "rs"
                    | "py"
                    | "kt"
                    | "java"
                    | "go"
                    | "vue"
                    | "svelte"
                    | "c"
                    | "cpp"
                    | "h"
                    | "hpp"
                    | "cs"
                    | "php"
                    | "rb"
                    | "swift"
                    | "sql"
            );
            if is_code_ext {
                *total_files += 1;
                seen_files.insert(relative_path.to_string());
                let content = fs::read_to_string(&path)?;
                let hash = format!("{:x}", Sha256::digest(content.as_bytes()));

                if stored_hashes.get(relative_path) == Some(&hash) {
                    *cached_files += 1;
                } else {
                    *indexed_files += 1;
                    if stored_hashes.contains_key(relative_path) {
                        store.clear_file_index(project_id, relative_path)?;
                    }
                    for item in knowledge_extractor.extract_from_code(project_id, relative_path, &content) {
                        store.insert_knowledge_item(&item)?;
                    }
                    if let Some(adapter) = registry.get_adapter_for_path(&path) {
                        let parsed = adapter.parse_file(project_id, relative_path, &content)?;
                        for node in &parsed.nodes {
                            store.insert_node(node)?;
                        }
                        for edge in &parsed.edges {
                            store.insert_edge(edge)?;
                        }
                    }
                    store.set_file_hash(project_id, relative_path, &hash)?;
                }
            }
        }
    }
    Ok(())
}
