use emocis_core::{EdgeKind, IdentityGenerator, Node, NodeKind, SqliteStore};
use std::collections::{HashMap, HashSet};
use std::path::{Component, Path};

/// Resolve explicit relative imports and language-agnostic module bindings through public exports.
/// Unrecognized or ambiguous bindings stay unresolved unless disambiguated by local directory.
pub fn link_project_calls(store: &SqliteStore, project_id: &str) -> anyhow::Result<()> {
    let all_nodes = store.find_nodes_by_project(project_id)?;
    let nodes: HashMap<_, _> = all_nodes
        .iter()
        .map(|n| (n.id.as_str(), n))
        .collect();
    let files: HashMap<_, _> = nodes
        .values()
        .filter(|n| n.kind == NodeKind::File)
        .filter_map(|n| n.source.as_ref().map(|s| (s.file.as_str(), *n)))
        .collect();
    let mut symbols: HashMap<(&str, &str), Vec<&str>> = HashMap::new();
    for node in nodes.values() {
        if node.kind == NodeKind::Function && node.identity.qualified_name == node.name {
            if let Some(source) = &node.source {
                symbols
                    .entry((&source.file, &node.name))
                    .or_default()
                    .push(&node.id);
            }
        }
    }
    let ts_aliases = collect_tsconfig_aliases(&files);
    let resolver = ExportResolver { files, symbols, ts_aliases };
    for mut edge in store.get_edges_by_project(project_id)? {
        if edge.kind != EdgeKind::Calls {
            continue;
        }
        let Some(caller) = nodes.get(edge.from.as_str()) else {
            continue;
        };
        if nodes.contains_key(edge.to.as_str()) {
            continue;
        }
        let Some(source) = &caller.source else {
            continue;
        };
        let Some(file) = resolver.files.get(source.file.as_str()) else {
            continue;
        };
        let Some(raw) = edge.qualifier["raw_callee"].as_str() else {
            continue;
        };

        let mut targets = HashSet::new();

        // 0. IPC Bridges (Tauri IPC invoke <-> #[tauri::command])
        if let Some(cmd) = edge.qualifier["ipc_command"].as_str().or_else(|| raw.strip_prefix("tauri_ipc:")) {
            for node in nodes.values() {
                if (node.kind == NodeKind::Function || node.kind == NodeKind::Method)
                    && (node.name == cmd || node.identity.qualified_name == cmd)
                    && node.metadata["tauri_command"].as_bool() == Some(true)
                {
                    targets.insert(node.id.clone());
                }
            }
        }

        // 1. Direct scoped callee (e.g. crate::sync::run_sync or super::helper::ping)
        if let Some((mod_part, fn_name)) = raw.rsplit_once("::") {
            if let Some(path) = resolver.module_path(&source.file, mod_part) {
                targets.extend(resolver.resolve(&path, fn_name, &mut HashSet::new()));
            }
        }

        // 2. Imports matching
        if let Some(imports) = file.metadata["imports"].as_array() {
            for binding in imports {
                let module = binding["module"].as_str().unwrap_or("");
                let local = binding["local"].as_str().unwrap_or("");
                let imported = binding["imported"].as_str().unwrap_or("");
                let symbol = if imported == "*" {
                    if local == "*" {
                        // Wildcard import (Rust `use crate::models::*;` or Python `from models import *`)
                        Some(raw)
                    } else {
                        // Namespace import (TS `import * as api from './api'; api.fetch()`)
                        raw.strip_prefix(&format!("{local}."))
                    }
                } else if raw == local {
                    Some(imported)
                } else if let Some(stripped) = raw.strip_prefix(&format!("{local}::")) {
                    // Rust module import: `use crate::sync; sync::run()`
                    Some(stripped)
                } else if let Some(stripped) = raw.strip_prefix(&format!("{local}.")) {
                    // TS/JS imported object/service method call: `import { dbService } from './dbService'; dbService.getProducts()`
                    Some(stripped)
                } else {
                    None
                };
                let Some(symbol) = symbol else {
                    continue;
                };
                if let Some(path) = resolver.module_path(&source.file, module) {
                    targets.extend(resolver.resolve(&path, symbol, &mut HashSet::new()));
                }
            }
        }

        // 3. Same file resolution for raw callee or this.method
        if targets.is_empty() {
            let direct_name = raw.strip_prefix("this.").unwrap_or(raw);
            for node in nodes.values() {
                if let Some(n_src) = &node.source {
                    if n_src.file == source.file
                        && (node.kind == NodeKind::Function || node.kind == NodeKind::Method)
                        && (node.name == direct_name || node.identity.qualified_name.ends_with(&format!(".{direct_name}")))
                    {
                        targets.insert(node.id.clone());
                    }
                }
            }
        }

        let resolved_target = if targets.len() == 1 {
            targets.into_iter().next()
        } else if targets.len() > 1 {
            // Disambiguation heuristic: prioritize target defined in the same directory or file
            let caller_dir = Path::new(&source.file).parent().unwrap_or(Path::new(""));
            targets.into_iter().find(|tid| {
                if let Some(tnode) = nodes.get(tid.as_str()) {
                    if let Some(tsrc) = &tnode.source {
                        let target_dir = Path::new(&tsrc.file).parent().unwrap_or(Path::new(""));
                        return caller_dir == target_dir;
                    }
                }
                false
            })
        } else {
            None
        };

        if let Some(target_id) = resolved_target {
            let old_id = edge.id.clone();
            edge.to = target_id;
            edge.id = IdentityGenerator::for_edge("calls", &edge.from, &edge.to);
            store.replace_edge(&old_id, &edge)?;
        }
    }
    Ok(())
}

fn collect_tsconfig_aliases(files: &HashMap<&str, &Node>) -> Vec<(String, Vec<String>)> {
    let mut aliases = Vec::new();
    let candidates = ["tsconfig.json", "jsconfig.json", "tsconfig.app.json"];
    for candidate in candidates {
        if files.contains_key(candidate) || Path::new(candidate).exists() {
            if let Ok(content) = std::fs::read_to_string(candidate) {
                let clean_content: String = content
                    .lines()
                    .filter(|l| !l.trim().starts_with("//"))
                    .collect::<Vec<_>>()
                    .join("\n");
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&clean_content) {
                    if let Some(paths) = val["compilerOptions"]["paths"].as_object() {
                        for (key, target_val) in paths {
                            let prefix = key.trim_end_matches('*').to_string();
                            let mut target_dirs = Vec::new();
                            if let Some(arr) = target_val.as_array() {
                                for item in arr {
                                    if let Some(t_str) = item.as_str() {
                                        let clean_target = t_str.trim_end_matches('*');
                                        target_dirs.push(clean_target.to_string());
                                    }
                                }
                            } else if let Some(t_str) = target_val.as_str() {
                                target_dirs.push(t_str.trim_end_matches('*').to_string());
                            }
                            if !prefix.is_empty() && !target_dirs.is_empty() {
                                aliases.push((prefix, target_dirs));
                            }
                        }
                    }
                }
            }
        }
    }
    aliases
}

struct ExportResolver<'a> {
    files: HashMap<&'a str, &'a Node>,
    symbols: HashMap<(&'a str, &'a str), Vec<&'a str>>,
    ts_aliases: Vec<(String, Vec<String>)>,
}

impl ExportResolver<'_> {
    fn module_path(&self, source: &str, module: &str) -> Option<String> {
        // 1. Rust crate::
        if let Some(rest) = module.strip_prefix("crate::") {
            let rel_path = rest.replace("::", "/");
            let mut candidates = Vec::new();
            if let Some(idx) = source.rfind("/src/") {
                let crate_src = &source[..idx + 5]; // includes trailing "/src/"
                candidates.push(format!("{}{}.rs", crate_src, rel_path));
                candidates.push(format!("{}{}/mod.rs", crate_src, rel_path));
            } else if source.starts_with("src/") {
                candidates.push(format!("src/{}.rs", rel_path));
                candidates.push(format!("src/{}/mod.rs", rel_path));
            } else {
                candidates.push(format!("{}.rs", rel_path));
                candidates.push(format!("{}/mod.rs", rel_path));
            }
            if let Some(found) = candidates.into_iter().find(|p| self.files.contains_key(p.as_str())) {
                return Some(found);
            }
        }

        // 2. Rust super::
        if module == "super" || module.starts_with("super::") {
            let rest = module.strip_prefix("super::").unwrap_or("");
            let rel_sub = rest.replace("::", "/");
            let parent = Path::new(source).parent().unwrap_or(Path::new(""));
            let super_dir = parent.parent().unwrap_or(Path::new(""));
            let joined = super_dir.join(&rel_sub);
            let s = joined.to_string_lossy();
            let s = s.trim_start_matches("./");
            let candidates = vec![
                format!("{}.rs", s),
                format!("{}/mod.rs", s),
            ];
            if let Some(found) = candidates.into_iter().find(|p| self.files.contains_key(p.as_str())) {
                return Some(found);
            }
        }

        // 3. Rust self::
        if let Some(rest) = module.strip_prefix("self::") {
            let rel_sub = rest.replace("::", "/");
            let parent = Path::new(source).parent().unwrap_or(Path::new(""));
            let joined = parent.join(&rel_sub);
            let s = joined.to_string_lossy();
            let s = s.trim_start_matches("./");
            let candidates = vec![
                format!("{}.rs", s),
                format!("{}/mod.rs", s),
            ];
            if let Some(found) = candidates.into_iter().find(|p| self.files.contains_key(p.as_str())) {
                return Some(found);
            }
        }

        // 4. Custom TypeScript Path Aliases (from tsconfig.json / jsconfig.json)
        for (prefix, targets) in &self.ts_aliases {
            if module.starts_with(prefix) {
                let rest = &module[prefix.len()..];
                for base in targets {
                    let mut base_dirs = Vec::new();
                    if let Some(idx) = source.rfind("/src/") {
                        base_dirs.push(source[..idx + 5].to_string());
                    } else if source.starts_with("src/") {
                        base_dirs.push("src/".to_string());
                    }
                    base_dirs.push("src/".to_string());
                    base_dirs.push("".to_string());

                    for p_dir in base_dirs {
                        let candidate_base = format!("{}{}{}", p_dir, base, rest);
                        let mut paths = vec![candidate_base.clone()];
                        for ext in ["ts", "tsx", "js", "jsx", "vue", "svelte"] {
                            paths.push(format!("{}.{}", candidate_base, ext));
                        }
                        for ext in ["ts", "tsx", "js", "jsx"] {
                            paths.push(format!("{}/index.{}", candidate_base, ext));
                        }
                        if let Some(found) = paths.into_iter().find(|p| self.files.contains_key(p.as_str())) {
                            return Some(found);
                        }
                    }
                }
            }
        }

        // 4b. TypeScript / Web Path Aliases: @/ or ~/
        if module.starts_with("@/") || module.starts_with("~/") {
            let alias_path = &module[2..];
            let mut base_dirs = Vec::new();
            if let Some(idx) = source.rfind("/src/") {
                base_dirs.push(source[..idx + 5].to_string());
            } else if source.starts_with("src/") {
                base_dirs.push("src/".to_string());
            }
            base_dirs.push("src/".to_string());
            base_dirs.push("".to_string());

            for base in base_dirs {
                let candidate_base = format!("{}{}", base, alias_path);
                let mut paths = vec![candidate_base.clone()];
                for ext in ["ts", "tsx", "js", "jsx", "vue"] {
                    paths.push(format!("{}.{}", candidate_base, ext));
                }
                for ext in ["ts", "tsx", "js", "jsx"] {
                    paths.push(format!("{}/index.{}", candidate_base, ext));
                }
                if let Some(found) = paths.into_iter().find(|p| self.files.contains_key(p.as_str())) {
                    return Some(found);
                }
            }
        }

        // 5. Python relative imports (starts with .)
        if module.starts_with('.') {
            let num_dots = module.chars().take_while(|&c| c == '.').count();
            let rest = &module[num_dots..];
            let rel_sub = rest.replace('.', "/");
            let mut curr = Path::new(source).parent().unwrap_or(Path::new(""));
            for _ in 1..num_dots {
                curr = curr.parent().unwrap_or(Path::new(""));
            }
            let joined = if rel_sub.is_empty() {
                curr.to_path_buf()
            } else {
                curr.join(&rel_sub)
            };
            let s = joined.to_string_lossy();
            let s = s.trim_start_matches("./");
            let candidates = vec![
                format!("{}.py", s),
                format!("{}/__init__.py", s),
            ];
            if let Some(found) = candidates.into_iter().find(|p| self.files.contains_key(p.as_str())) {
                return Some(found);
            }
        }

        // 6. Python absolute / package imports (e.g. models.user or helpers)
        if !module.contains('/') && !module.starts_with('.') && !module.contains("::") {
            let rel_sub = module.replace('.', "/");
            let mut candidates = vec![
                format!("{}.py", rel_sub),
                format!("{}/__init__.py", rel_sub),
            ];
            let parent = Path::new(source).parent().unwrap_or(Path::new(""));
            let joined = parent.join(&rel_sub);
            let s = joined.to_string_lossy();
            let s = s.trim_start_matches("./");
            candidates.push(format!("{}.py", s));
            candidates.push(format!("{}/__init__.py", s));

            if let Some(found) = candidates.into_iter().find(|p| self.files.contains_key(p.as_str())) {
                return Some(found);
            }
        }

        // 7. Standard relative path resolution (e.g. ./foo, ../bar)
        if module.starts_with("./") || module.starts_with("../") {
            let joined = Path::new(source)
                .parent()
                .unwrap_or(Path::new(""))
                .join(module);
            let mut parts = Vec::new();
            for component in joined.components() {
                match component {
                    Component::Normal(s) => parts.push(s.to_string_lossy().into_owned()),
                    Component::ParentDir => {
                        parts.pop()?;
                    }
                    Component::CurDir => {}
                    _ => return None,
                }
            }
            let base = parts.join("/");
            let mut paths = vec![base.clone()];
            if let Some(stem) = base.strip_suffix(".js") {
                paths.push(format!("{stem}.ts"));
                paths.push(format!("{stem}.tsx"));
            }
            for ext in ["ts", "tsx", "js", "jsx", "rs", "py", "vue"] {
                paths.push(format!("{base}.{ext}"));
            }
            for ext in ["ts", "tsx", "js", "jsx"] {
                paths.push(format!("{base}/index.{ext}"));
            }
            paths
                .into_iter()
                .find(|path| self.files.contains_key(path.as_str()))
        } else {
            None
        }
    }

    fn resolve(
        &self,
        file: &str,
        exported: &str,
        visited: &mut HashSet<(String, String)>,
    ) -> HashSet<String> {
        let mut targets = HashSet::new();
        // Bound recursion and terminate export cycles.
        let key = (file.to_owned(), exported.to_owned());
        if visited.len() >= 64 || !visited.insert(key.clone()) {
            return targets;
        }
        if let Some(node) = self.files.get(file) {
            if let Some(exports) = node.metadata["exports"].as_array() {
                for binding in exports
                    .iter()
                    .filter(|b| b["exported"].as_str() == Some(exported))
                {
                    let local = binding["local"].as_str().unwrap_or("");
                    if let Some(module) = binding["module"].as_str() {
                        if let Some(path) = self.module_path(file, module) {
                            targets.extend(self.resolve(&path, local, visited));
                        }
                    } else if let Some(ids) = self.symbols.get(&(file, local)) {
                        targets.extend(ids.iter().map(|id| (*id).to_owned()));
                    } else if let Some(imports) = node.metadata["imports"].as_array() {
                        for import in imports
                            .iter()
                            .filter(|b| b["local"].as_str() == Some(local))
                        {
                            if let Some(path) =
                                self.module_path(file, import["module"].as_str().unwrap_or(""))
                            {
                                targets.extend(self.resolve(
                                    &path,
                                    import["imported"].as_str().unwrap_or(""),
                                    visited,
                                ));
                            }
                        }
                    }
                }
            } else {
                // Fallback only for files that do not produce explicit export metadata
                if let Some(ids) = self.symbols.get(&(file, exported)) {
                    targets.extend(ids.iter().map(|id| (*id).to_owned()));
                }
            }
        }
        visited.remove(&key);
        targets
    }
}

