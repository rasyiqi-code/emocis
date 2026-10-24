use anyhow::{anyhow, Result};
use emocis_core::{
    Certainty, Edge, EdgeKind, IdentityGenerator, Node, NodeIdentity, NodeKind, NodeState,
    Provenance, ProvenanceSource, SourceLocation, TemporalRange,
};
use tree_sitter::{Node as TsNode, Parser};

use super::{LanguageAdapter, ParsedFileGraph};

pub struct RustAdapter;

impl Default for RustAdapter {
    fn default() -> Self {
        Self
    }
}

impl RustAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl LanguageAdapter for RustAdapter {
    fn language_name(&self) -> &'static str {
        "rust"
    }

    fn supported_extensions(&self) -> &[&'static str] {
        &["rs"]
    }

    fn parse_file(
        &self,
        project_id: &str,
        file_path: &str,
        content: &str,
    ) -> Result<ParsedFileGraph> {
        let mut parser = Parser::new();
        let language = tree_sitter_rust::LANGUAGE.into();
        parser
            .set_language(&language)
            .map_err(|e| anyhow!("Failed to set Rust tree-sitter language: {:?}", e))?;

        let tree = parser
            .parse(content, None)
            .ok_or_else(|| anyhow!("Tree-sitter failed to parse file {}", file_path))?;

        let root_node = tree.root_node();
        let content_bytes = content.as_bytes();
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        // 1. File node
        let file_node_id = IdentityGenerator::for_file(file_path);
        let file_node = Node {
            id: file_node_id.clone(),
            kind: NodeKind::File,
            name: file_path.to_string(),
            project_id: project_id.to_string(),
            language: Some("rust".to_string()),
            source: Some(SourceLocation {
                file: file_path.to_string(),
                start_line: 1,
                start_column: 1,
                end_line: root_node.end_position().row as u32 + 1,
                end_column: root_node.end_position().column as u32 + 1,
            }),
            identity: NodeIdentity {
                qualified_name: file_path.to_string(),
                signature: None,
            },
            state: NodeState::default(),
            provenance: Provenance::default(),
            metadata: serde_json::json!({
                "imports": collect_rust_imports(root_node, content_bytes),
                "exports": collect_rust_exports(root_node, content_bytes),
            }),
        };
        nodes.push(file_node);

        // 2. Walk AST recursively
        let mut call_counter: u32 = 0;
        walk_rust_tree(
            &root_node,
            content_bytes,
            project_id,
            file_path,
            &file_node_id,
            None, // enclosing impl or fn
            &mut call_counter,
            &mut nodes,
            &mut edges,
        );

        Ok(ParsedFileGraph { nodes, edges })
    }
}

fn walk_rust_tree(
    ts_node: &TsNode,
    code: &[u8],
    project_id: &str,
    file_path: &str,
    file_node_id: &str,
    current_enclosing_id: Option<&str>,
    call_counter: &mut u32,
    nodes: &mut Vec<Node>,
    edges: &mut Vec<Edge>,
) {
    let kind = ts_node.kind();
    let mut new_enclosing_id: Option<String> = None;

    match kind {
        "function_item" => {
            if let Some(name_node) = ts_node.child_by_field_name("name") {
                if let Ok(fn_name) = name_node.utf8_text(code) {
                    let qualified = match current_enclosing_id {
                        Some(enc) => format!("{}.{}", enc.split(':').nth(3).unwrap_or(""), fn_name),
                        None => fn_name.to_string(),
                    };

                    let fn_kind = if current_enclosing_id.is_some() {
                        NodeKind::Method
                    } else {
                        NodeKind::Function
                    };

                    let fn_id =
                        IdentityGenerator::for_symbol("rust", file_path, &qualified, &fn_kind);

                    let is_tauri_cmd = is_tauri_command_fn(ts_node, code);

                    let node = Node {
                        id: fn_id.clone(),
                        kind: fn_kind,
                        name: fn_name.to_string(),
                        project_id: project_id.to_string(),
                        language: Some("rust".to_string()),
                        source: Some(node_source_loc(ts_node, file_path)),
                        identity: NodeIdentity {
                            qualified_name: qualified,
                            signature: None,
                        },
                        state: NodeState::default(),
                        provenance: Provenance::default(),
                        metadata: serde_json::json!({
                            "public": has_public_visibility(*ts_node, code),
                            "tauri_command": is_tauri_cmd,
                            "ipc_command": fn_name,
                        }),
                    };

                    let parent_id = current_enclosing_id.unwrap_or(file_node_id);
                    edges.push(Edge {
                        id: IdentityGenerator::for_edge("contains", parent_id, &fn_id),
                        from: parent_id.to_string(),
                        to: fn_id.clone(),
                        kind: EdgeKind::Contains,
                        qualifier: serde_json::Value::Null,
                        provenance: Provenance::default(),
                        temporal: TemporalRange::default(),
                    });

                    nodes.push(node);
                    new_enclosing_id = Some(fn_id);
                }
            }
        }
        "struct_item" => {
            if let Some(name_node) = ts_node.child_by_field_name("name") {
                if let Ok(struct_name) = name_node.utf8_text(code) {
                    let struct_id = IdentityGenerator::for_symbol(
                        "rust",
                        file_path,
                        struct_name,
                        &NodeKind::Struct,
                    );

                    let node = Node {
                        id: struct_id.clone(),
                        kind: NodeKind::Struct,
                        name: struct_name.to_string(),
                        project_id: project_id.to_string(),
                        language: Some("rust".to_string()),
                        source: Some(node_source_loc(ts_node, file_path)),
                        identity: NodeIdentity {
                            qualified_name: struct_name.to_string(),
                            signature: None,
                        },
                        state: NodeState::default(),
                        provenance: Provenance::default(),
                        metadata: serde_json::Value::Null,
                    };

                    edges.push(Edge {
                        id: IdentityGenerator::for_edge("contains", file_node_id, &struct_id),
                        from: file_node_id.to_string(),
                        to: struct_id.clone(),
                        kind: EdgeKind::Contains,
                        qualifier: serde_json::Value::Null,
                        provenance: Provenance::default(),
                        temporal: TemporalRange::default(),
                    });

                    nodes.push(node);
                }
            }
        }
        "impl_item" => {
            if let Some(type_node) = ts_node.child_by_field_name("type") {
                if let Ok(impl_type) = type_node.utf8_text(code) {
                    let impl_id = IdentityGenerator::for_symbol(
                        "rust",
                        file_path,
                        impl_type,
                        &NodeKind::Class,
                    );
                    new_enclosing_id = Some(impl_id);
                }
            }
        }
        "call_expression" => {
            if let Some(function_node) = ts_node.child_by_field_name("function") {
                if let Ok(callee_text) = function_node.utf8_text(code) {
                    *call_counter += 1;
                    let caller_id = current_enclosing_id.unwrap_or(file_node_id);
                    let callsite_id =
                        IdentityGenerator::for_callsite(caller_id, callee_text, *call_counter);

                    let callsite_node = Node {
                        id: callsite_id.clone(),
                        kind: NodeKind::CallSite,
                        name: callee_text.to_string(),
                        project_id: project_id.to_string(),
                        language: Some("rust".to_string()),
                        source: Some(node_source_loc(ts_node, file_path)),
                        identity: NodeIdentity {
                            qualified_name: format!("call:{}", callee_text),
                            signature: None,
                        },
                        state: NodeState::default(),
                        provenance: Provenance {
                            source: ProvenanceSource::Ast,
                            confidence: 1.0,
                            certainty: Certainty::Certain,
                            evidence: Vec::new(),
                        },
                        metadata: serde_json::Value::Null,
                    };

                    edges.push(Edge {
                        id: IdentityGenerator::for_edge("contains", caller_id, &callsite_id),
                        from: caller_id.to_string(),
                        to: callsite_id.clone(),
                        kind: EdgeKind::Contains,
                        qualifier: serde_json::Value::Null,
                        provenance: Provenance::default(),
                        temporal: TemporalRange::default(),
                    });

                    let target_fn_id = IdentityGenerator::for_symbol(
                        "rust",
                        file_path,
                        callee_text,
                        &NodeKind::Function,
                    );

                    edges.push(Edge {
                        id: IdentityGenerator::for_edge("calls", caller_id, &target_fn_id),
                        from: caller_id.to_string(),
                        to: target_fn_id,
                        kind: EdgeKind::Calls,
                        qualifier: serde_json::json!({ "raw_callee": callee_text }),
                        provenance: Provenance {
                            source: ProvenanceSource::Ast,
                            confidence: 0.9,
                            certainty: Certainty::Likely,
                            evidence: Vec::new(),
                        },
                        temporal: TemporalRange::default(),
                    });

                    nodes.push(callsite_node);
                }
            }
        }
        "token_tree" => {
            let mut cursor = ts_node.walk();
            let children: Vec<_> = ts_node.children(&mut cursor).collect();
            for i in 0..children.len() {
                if children[i].kind() == "identifier" && i + 1 < children.len() {
                    let next = &children[i + 1];
                    if next.kind() == "token_tree" {
                        if let Ok(next_text) = next.utf8_text(code) {
                            if next_text.starts_with('(') {
                                if let Ok(callee_text) = children[i].utf8_text(code) {
                                    if !matches!(
                                        callee_text,
                                        "vec"
                                            | "format"
                                            | "println"
                                            | "print"
                                            | "eprintln"
                                            | "write"
                                            | "writeln"
                                            | "panic"
                                            | "Some"
                                            | "Ok"
                                            | "Err"
                                            | "json"
                                    ) {
                                        *call_counter += 1;
                                        let caller_id =
                                            current_enclosing_id.unwrap_or(file_node_id);
                                        let callsite_id = IdentityGenerator::for_callsite(
                                            caller_id,
                                            callee_text,
                                            *call_counter,
                                        );

                                        let callsite_node = Node {
                                            id: callsite_id.clone(),
                                            kind: NodeKind::CallSite,
                                            name: callee_text.to_string(),
                                            project_id: project_id.to_string(),
                                            language: Some("rust".to_string()),
                                            source: Some(node_source_loc(&children[i], file_path)),
                                            identity: NodeIdentity {
                                                qualified_name: format!("call:{}", callee_text),
                                                signature: None,
                                            },
                                            state: NodeState::default(),
                                            provenance: Provenance {
                                                source: ProvenanceSource::Ast,
                                                confidence: 0.85,
                                                certainty: Certainty::Likely,
                                                evidence: Vec::new(),
                                            },
                                            metadata: serde_json::Value::Null,
                                        };

                                        edges.push(Edge {
                                            id: IdentityGenerator::for_edge(
                                                "contains",
                                                caller_id,
                                                &callsite_id,
                                            ),
                                            from: caller_id.to_string(),
                                            to: callsite_id.clone(),
                                            kind: EdgeKind::Contains,
                                            qualifier: serde_json::Value::Null,
                                            provenance: Provenance::default(),
                                            temporal: TemporalRange::default(),
                                        });

                                        let target_fn_id = IdentityGenerator::for_symbol(
                                            "rust",
                                            file_path,
                                            callee_text,
                                            &NodeKind::Function,
                                        );

                                        edges.push(Edge {
                                            id: IdentityGenerator::for_edge(
                                                "calls",
                                                caller_id,
                                                &target_fn_id,
                                            ),
                                            from: caller_id.to_string(),
                                            to: target_fn_id,
                                            kind: EdgeKind::Calls,
                                            qualifier: serde_json::json!({ "raw_callee": callee_text, "in_macro": true }),
                                            provenance: Provenance {
                                                source: ProvenanceSource::Ast,
                                                confidence: 0.85,
                                                certainty: Certainty::Likely,
                                                evidence: Vec::new(),
                                            },
                                            temporal: TemporalRange::default(),
                                        });

                                        nodes.push(callsite_node);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }

    let next_enclosing = new_enclosing_id.as_deref().or(current_enclosing_id);

    let mut cursor = ts_node.walk();
    for child in ts_node.children(&mut cursor) {
        walk_rust_tree(
            &child,
            code,
            project_id,
            file_path,
            file_node_id,
            next_enclosing,
            call_counter,
            nodes,
            edges,
        );
    }
}

fn node_source_loc(node: &TsNode, file: &str) -> SourceLocation {
    let start = node.start_position();
    let end = node.end_position();
    SourceLocation {
        file: file.to_string(),
        start_line: start.row as u32 + 1,
        start_column: start.column as u32 + 1,
        end_line: end.row as u32 + 1,
        end_column: end.column as u32 + 1,
    }
}

fn has_public_visibility(node: TsNode, code: &[u8]) -> bool {
    let mut cursor = node.walk();
    let is_public = node.children(&mut cursor).any(|child| {
        child.kind() == "visibility_modifier"
            && child
                .utf8_text(code)
                .is_ok_and(|text| text.trim_start().starts_with("pub"))
    });
    is_public
}

fn is_tauri_command_fn(node: &TsNode, code: &[u8]) -> bool {
    let mut prev = node.prev_named_sibling();
    while let Some(sibling) = prev {
        if sibling.kind() == "attribute_item" {
            if let Ok(text) = sibling.utf8_text(code) {
                if text.contains("tauri::command") || text.contains("command") {
                    return true;
                }
            }
            prev = sibling.prev_named_sibling();
        } else {
            break;
        }
    }
    false
}

struct RustUseBinding {
    module: String,
    imported: String,
    local: String,
}

fn collect_rust_imports(root: TsNode, code: &[u8]) -> Vec<serde_json::Value> {
    let mut imports = Vec::new();
    let mut cursor = root.walk();
    for statement in root.named_children(&mut cursor) {
        if statement.kind() == "use_declaration" {
            if let Some(arg) = statement.child_by_field_name("argument") {
                let mut bindings = Vec::new();
                extract_use_items(arg, code, "", &mut bindings);
                for b in bindings {
                    imports.push(serde_json::json!({
                        "module": b.module,
                        "imported": b.imported,
                        "local": b.local,
                    }));
                }
            }
        }
    }
    imports
}

fn collect_rust_exports(root: TsNode, code: &[u8]) -> Vec<serde_json::Value> {
    let mut exports = Vec::new();
    let mut cursor = root.walk();
    for statement in root.named_children(&mut cursor) {
        match statement.kind() {
            "use_declaration" => {
                if has_public_visibility(statement, code) {
                    if let Some(arg) = statement.child_by_field_name("argument") {
                        let mut bindings = Vec::new();
                        extract_use_items(arg, code, "", &mut bindings);
                        for b in bindings {
                            exports.push(serde_json::json!({
                                "local": b.local,
                                "exported": b.local,
                                "module": Some(b.module),
                            }));
                        }
                    }
                }
            }
            "function_item" => {
                let is_pub = has_public_visibility(statement, code);
                let is_tauri = is_tauri_command_fn(&statement, code);
                if is_pub || is_tauri {
                    if let Some(name_node) = statement.child_by_field_name("name") {
                        if let Ok(name) = name_node.utf8_text(code) {
                            exports.push(serde_json::json!({
                                "local": name,
                                "exported": name,
                            }));
                            if is_tauri {
                                exports.push(serde_json::json!({
                                    "local": name,
                                    "exported": format!("tauri_ipc:{}", name),
                                }));
                            }
                        }
                    }
                }
            }
            "struct_item" | "enum_item" | "type_item" => {
                if has_public_visibility(statement, code) {
                    if let Some(name_node) = statement.child_by_field_name("name") {
                        if let Ok(name) = name_node.utf8_text(code) {
                            exports.push(serde_json::json!({
                                "local": name,
                                "exported": name,
                            }));
                        }
                    }
                }
            }
            _ => {}
        }
    }
    exports
}

fn extract_use_items(
    node: TsNode,
    code: &[u8],
    prefix: &str,
    out: &mut Vec<RustUseBinding>,
) {
    match node.kind() {
        "scoped_identifier" => {
            let path_text = node
                .child_by_field_name("path")
                .and_then(|p| p.utf8_text(code).ok())
                .unwrap_or("");
            let name_text = node
                .child_by_field_name("name")
                .and_then(|n| n.utf8_text(code).ok())
                .unwrap_or("");
            let module = if prefix.is_empty() {
                path_text.to_string()
            } else if path_text.is_empty() {
                prefix.to_string()
            } else {
                format!("{}::{}", prefix, path_text)
            };
            out.push(RustUseBinding {
                module,
                imported: name_text.to_string(),
                local: name_text.to_string(),
            });
        }
        "scoped_use_list" => {
            let path_text = node
                .child_by_field_name("path")
                .and_then(|p| p.utf8_text(code).ok())
                .unwrap_or("");
            let new_prefix = if prefix.is_empty() {
                path_text.to_string()
            } else if path_text.is_empty() {
                prefix.to_string()
            } else {
                format!("{}::{}", prefix, path_text)
            };
            if let Some(list) = node.child_by_field_name("list") {
                extract_use_items(list, code, &new_prefix, out);
            }
        }
        "use_list" => {
            let mut cursor = node.walk();
            for child in node.named_children(&mut cursor) {
                extract_use_items(child, code, prefix, out);
            }
        }
        "use_as_clause" => {
            let alias = node
                .child_by_field_name("alias")
                .and_then(|a| a.utf8_text(code).ok())
                .unwrap_or("");
            if let Some(path_node) = node.child_by_field_name("path") {
                if path_node.kind() == "identifier" {
                    let imp = path_node.utf8_text(code).unwrap_or("");
                    out.push(RustUseBinding {
                        module: prefix.to_string(),
                        imported: imp.to_string(),
                        local: alias.to_string(),
                    });
                } else if path_node.kind() == "scoped_identifier" {
                    let p_text = path_node
                        .child_by_field_name("path")
                        .and_then(|p| p.utf8_text(code).ok())
                        .unwrap_or("");
                    let n_text = path_node
                        .child_by_field_name("name")
                        .and_then(|n| n.utf8_text(code).ok())
                        .unwrap_or("");
                    let module = if prefix.is_empty() {
                        p_text.to_string()
                    } else if p_text.is_empty() {
                        prefix.to_string()
                    } else {
                        format!("{}::{}", prefix, p_text)
                    };
                    out.push(RustUseBinding {
                        module,
                        imported: n_text.to_string(),
                        local: alias.to_string(),
                    });
                }
            }
        }
        "use_wildcard" => {
            let mut cursor = node.walk();
            let mut prefix_path = String::new();
            for child in node.children(&mut cursor) {
                if child.kind() == "scoped_identifier" || child.kind() == "identifier" {
                    if let Ok(t) = child.utf8_text(code) {
                        prefix_path = t.to_string();
                        break;
                    }
                }
            }
            let module = if prefix.is_empty() {
                prefix_path
            } else if prefix_path.is_empty() {
                prefix.to_string()
            } else {
                format!("{}::{}", prefix, prefix_path)
            };
            out.push(RustUseBinding {
                module,
                imported: "*".to_string(),
                local: "*".to_string(),
            });
        }
        "identifier" => {
            let name = node.utf8_text(code).unwrap_or("");
            out.push(RustUseBinding {
                module: prefix.to_string(),
                imported: name.to_string(),
                local: name.to_string(),
            });
        }
        "self" => {
            let last_segment = prefix.rsplit("::").next().unwrap_or(prefix);
            let parent_module = prefix.rsplit_once("::").map(|(p, _)| p).unwrap_or(prefix);
            out.push(RustUseBinding {
                module: parent_module.to_string(),
                imported: last_segment.to_string(),
                local: last_segment.to_string(),
            });
        }
        _ => {}
    }
}

