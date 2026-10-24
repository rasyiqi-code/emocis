use anyhow::{anyhow, Result};
use emocis_core::{
    Certainty, Edge, EdgeKind, IdentityGenerator, Node, NodeIdentity, NodeKind, NodeState,
    Provenance, ProvenanceSource, SourceLocation, TemporalRange,
};
use tree_sitter::{Node as TsNode, Parser};

use super::{LanguageAdapter, ParsedFileGraph};

pub struct PythonAdapter;

impl Default for PythonAdapter {
    fn default() -> Self {
        Self
    }
}

impl PythonAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl LanguageAdapter for PythonAdapter {
    fn language_name(&self) -> &'static str {
        "python"
    }

    fn supported_extensions(&self) -> &[&'static str] {
        &["py"]
    }

    fn parse_file(
        &self,
        project_id: &str,
        file_path: &str,
        content: &str,
    ) -> Result<ParsedFileGraph> {
        let mut parser = Parser::new();
        let language = tree_sitter_python::LANGUAGE.into();
        parser
            .set_language(&language)
            .map_err(|e| anyhow!("Failed to set Python tree-sitter language: {:?}", e))?;

        let tree = parser
            .parse(content, None)
            .ok_or_else(|| anyhow!("Tree-sitter failed to parse Python file {}", file_path))?;

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
            language: Some("python".to_string()),
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
                "imports": collect_python_imports(root_node, content_bytes),
                "exports": collect_python_exports(root_node, content_bytes),
            }),
        };
        nodes.push(file_node);

        // 2. Walk AST recursively
        let mut call_counter: u32 = 0;
        walk_python_tree(
            &root_node,
            content_bytes,
            project_id,
            file_path,
            &file_node_id,
            None, // enclosing class or function
            &mut call_counter,
            &mut nodes,
            &mut edges,
        );

        Ok(ParsedFileGraph { nodes, edges })
    }
}

fn walk_python_tree(
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
        "function_definition" => {
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
                        IdentityGenerator::for_symbol("python", file_path, &qualified, &fn_kind);

                    let node = Node {
                        id: fn_id.clone(),
                        kind: fn_kind,
                        name: fn_name.to_string(),
                        project_id: project_id.to_string(),
                        language: Some("python".to_string()),
                        source: Some(node_source_loc(ts_node, file_path)),
                        identity: NodeIdentity {
                            qualified_name: qualified,
                            signature: None,
                        },
                        state: NodeState::default(),
                        provenance: Provenance::default(),
                        metadata: serde_json::Value::Null,
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
        "class_definition" => {
            if let Some(name_node) = ts_node.child_by_field_name("name") {
                if let Ok(class_name) = name_node.utf8_text(code) {
                    let class_id = IdentityGenerator::for_symbol(
                        "python",
                        file_path,
                        class_name,
                        &NodeKind::Class,
                    );

                    let node = Node {
                        id: class_id.clone(),
                        kind: NodeKind::Class,
                        name: class_name.to_string(),
                        project_id: project_id.to_string(),
                        language: Some("python".to_string()),
                        source: Some(node_source_loc(ts_node, file_path)),
                        identity: NodeIdentity {
                            qualified_name: class_name.to_string(),
                            signature: None,
                        },
                        state: NodeState::default(),
                        provenance: Provenance::default(),
                        metadata: serde_json::Value::Null,
                    };

                    edges.push(Edge {
                        id: IdentityGenerator::for_edge("contains", file_node_id, &class_id),
                        from: file_node_id.to_string(),
                        to: class_id.clone(),
                        kind: EdgeKind::Contains,
                        qualifier: serde_json::Value::Null,
                        provenance: Provenance::default(),
                        temporal: TemporalRange::default(),
                    });

                    nodes.push(node);
                    new_enclosing_id = Some(class_id);
                }
            }
        }
        "call" => {
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
                        language: Some("python".to_string()),
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
                        "python",
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
        _ => {}
    }

    let next_enclosing = new_enclosing_id.as_deref().or(current_enclosing_id);

    let mut cursor = ts_node.walk();
    for child in ts_node.children(&mut cursor) {
        walk_python_tree(
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

fn collect_python_imports(root: TsNode, code: &[u8]) -> Vec<serde_json::Value> {
    let mut imports = Vec::new();
    let mut cursor = root.walk();
    for statement in root.named_children(&mut cursor) {
        match statement.kind() {
            "import_from_statement" => {
                let module = statement
                    .child_by_field_name("module_name")
                    .and_then(|m| m.utf8_text(code).ok())
                    .unwrap_or("");
                let mut c = statement.walk();
                for child in statement.children(&mut c) {
                    match child.kind() {
                        "dotted_name" => {
                            // Only collect names after the 'import' keyword (not the module_name)
                            if child != statement.child_by_field_name("module_name").unwrap_or(child) {
                                let name = child.utf8_text(code).unwrap_or("");
                                imports.push(serde_json::json!({
                                    "module": module,
                                    "imported": name,
                                    "local": name,
                                }));
                            }
                        }
                        "aliased_import" => {
                            let imp = child
                                .child_by_field_name("name")
                                .and_then(|n| n.utf8_text(code).ok())
                                .unwrap_or("");
                            let local = child
                                .child_by_field_name("alias")
                                .and_then(|a| a.utf8_text(code).ok())
                                .unwrap_or(imp);
                            imports.push(serde_json::json!({
                                "module": module,
                                "imported": imp,
                                "local": local,
                            }));
                        }
                        "wildcard_import" => {
                            imports.push(serde_json::json!({
                                "module": module,
                                "imported": "*",
                                "local": "*",
                            }));
                        }
                        _ => {}
                    }
                }
            }
            "import_statement" => {
                let mut c = statement.walk();
                for child in statement.children(&mut c) {
                    match child.kind() {
                        "dotted_name" => {
                            let name = child.utf8_text(code).unwrap_or("");
                            imports.push(serde_json::json!({
                                "module": name,
                                "imported": "*",
                                "local": name,
                            }));
                        }
                        "aliased_import" => {
                            let module = child
                                .child_by_field_name("name")
                                .and_then(|n| n.utf8_text(code).ok())
                                .unwrap_or("");
                            let local = child
                                .child_by_field_name("alias")
                                .and_then(|a| a.utf8_text(code).ok())
                                .unwrap_or(module);
                            imports.push(serde_json::json!({
                                "module": module,
                                "imported": "*",
                                "local": local,
                            }));
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    imports
}

fn collect_python_exports(root: TsNode, code: &[u8]) -> Vec<serde_json::Value> {
    let mut exports = Vec::new();
    let mut cursor = root.walk();
    for statement in root.named_children(&mut cursor) {
        match statement.kind() {
            "function_definition" | "class_definition" => {
                if let Some(name_node) = statement.child_by_field_name("name") {
                    if let Ok(name) = name_node.utf8_text(code) {
                        if !name.starts_with('_') {
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

