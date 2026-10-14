use anyhow::{anyhow, Result};
use emocis_core::{
    Certainty, Edge, EdgeKind, IdentityGenerator, Node, NodeIdentity, NodeKind, NodeState,
    Provenance, ProvenanceSource, SourceLocation, TemporalRange,
};
use tree_sitter::{Node as TsNode, Parser};

use super::{LanguageAdapter, ParsedFileGraph};

pub struct TypeScriptAdapter;

impl Default for TypeScriptAdapter {
    fn default() -> Self {
        Self
    }
}

impl TypeScriptAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl LanguageAdapter for TypeScriptAdapter {
    fn language_name(&self) -> &'static str {
        "typescript"
    }

    fn supported_extensions(&self) -> &[&'static str] {
        &["ts", "tsx", "js", "jsx"]
    }

    fn parse_file(
        &self,
        project_id: &str,
        file_path: &str,
        content: &str,
    ) -> Result<ParsedFileGraph> {
        let mut parser = Parser::new();
        let language = tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into();
        parser
            .set_language(&language)
            .map_err(|e| anyhow!("Failed to set TypeScript tree-sitter language: {:?}", e))?;

        let tree = parser
            .parse(content, None)
            .ok_or_else(|| anyhow!("Tree-sitter failed to parse file {}", file_path))?;

        let root_node = tree.root_node();
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        // 1. File node
        let file_node_id = IdentityGenerator::for_file(file_path);
        let file_node = Node {
            id: file_node_id.clone(),
            kind: NodeKind::File,
            name: file_path.to_string(),
            project_id: project_id.to_string(),
            language: Some("typescript".to_string()),
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
            metadata: serde_json::Value::Null,
        };
        nodes.push(file_node);

        // 2. Walk AST recursively
        let content_bytes = content.as_bytes();
        let mut call_counter: u32 = 0;
        walk_tree(
            &root_node,
            content_bytes,
            project_id,
            file_path,
            &file_node_id,
            None, // current enclosing function/class
            &mut call_counter,
            &mut nodes,
            &mut edges,
        );

        Ok(ParsedFileGraph { nodes, edges })
    }
}

fn walk_tree(
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
        "function_declaration" => {
            if let Some(name_node) = ts_node.child_by_field_name("name") {
                if let Ok(fn_name) = name_node.utf8_text(code) {
                    let fn_id = IdentityGenerator::for_symbol(
                        "typescript",
                        file_path,
                        fn_name,
                        &NodeKind::Function,
                    );

                    let node = Node {
                        id: fn_id.clone(),
                        kind: NodeKind::Function,
                        name: fn_name.to_string(),
                        project_id: project_id.to_string(),
                        language: Some("typescript".to_string()),
                        source: Some(node_source_loc(ts_node, file_path)),
                        identity: NodeIdentity {
                            qualified_name: fn_name.to_string(),
                            signature: None,
                        },
                        state: NodeState::default(),
                        provenance: Provenance::default(),
                        metadata: serde_json::Value::Null,
                    };

                    // Edge: file contains function
                    let edge = Edge {
                        id: IdentityGenerator::for_edge("contains", file_node_id, &fn_id),
                        from: file_node_id.to_string(),
                        to: fn_id.clone(),
                        kind: EdgeKind::Contains,
                        qualifier: serde_json::Value::Null,
                        provenance: Provenance::default(),
                        temporal: TemporalRange::default(),
                    };

                    nodes.push(node);
                    edges.push(edge);
                    new_enclosing_id = Some(fn_id);
                }
            }
        }
        "class_declaration" => {
            if let Some(name_node) = ts_node.child_by_field_name("name") {
                if let Ok(class_name) = name_node.utf8_text(code) {
                    let class_id = IdentityGenerator::for_symbol(
                        "typescript",
                        file_path,
                        class_name,
                        &NodeKind::Class,
                    );

                    let node = Node {
                        id: class_id.clone(),
                        kind: NodeKind::Class,
                        name: class_name.to_string(),
                        project_id: project_id.to_string(),
                        language: Some("typescript".to_string()),
                        source: Some(node_source_loc(ts_node, file_path)),
                        identity: NodeIdentity {
                            qualified_name: class_name.to_string(),
                            signature: None,
                        },
                        state: NodeState::default(),
                        provenance: Provenance::default(),
                        metadata: serde_json::Value::Null,
                    };

                    let edge = Edge {
                        id: IdentityGenerator::for_edge("contains", file_node_id, &class_id),
                        from: file_node_id.to_string(),
                        to: class_id.clone(),
                        kind: EdgeKind::Contains,
                        qualifier: serde_json::Value::Null,
                        provenance: Provenance::default(),
                        temporal: TemporalRange::default(),
                    };

                    nodes.push(node);
                    edges.push(edge);
                    new_enclosing_id = Some(class_id);
                }
            }
        }
        "method_definition" => {
            if let Some(name_node) = ts_node.child_by_field_name("name") {
                if let Ok(method_name) = name_node.utf8_text(code) {
                    let qualified = match current_enclosing_id {
                        Some(enc) => format!("{}.{}", enc.split(':').nth(3).unwrap_or(""), method_name),
                        None => method_name.to_string(),
                    };

                    let method_id = IdentityGenerator::for_symbol(
                        "typescript",
                        file_path,
                        &qualified,
                        &NodeKind::Method,
                    );

                    let node = Node {
                        id: method_id.clone(),
                        kind: NodeKind::Method,
                        name: method_name.to_string(),
                        project_id: project_id.to_string(),
                        language: Some("typescript".to_string()),
                        source: Some(node_source_loc(ts_node, file_path)),
                        identity: NodeIdentity {
                            qualified_name: qualified,
                            signature: None,
                        },
                        state: NodeState::default(),
                        provenance: Provenance::default(),
                        metadata: serde_json::Value::Null,
                    };

                    if let Some(enc_id) = current_enclosing_id {
                        edges.push(Edge {
                            id: IdentityGenerator::for_edge("contains", enc_id, &method_id),
                            from: enc_id.to_string(),
                            to: method_id.clone(),
                            kind: EdgeKind::Contains,
                            qualifier: serde_json::Value::Null,
                            provenance: Provenance::default(),
                            temporal: TemporalRange::default(),
                        });
                    }

                    nodes.push(node);
                    new_enclosing_id = Some(method_id);
                }
            }
        }
        "call_expression" => {
            // PRD Part II Section 11 & 18: Call Sites
            if let Some(function_node) = ts_node.child_by_field_name("function") {
                if let Ok(callee_text) = function_node.utf8_text(code) {
                    *call_counter += 1;
                    let caller_id = current_enclosing_id.unwrap_or(file_node_id);
                    let callsite_id = IdentityGenerator::for_callsite(caller_id, callee_text, *call_counter);

                    let callsite_node = Node {
                        id: callsite_id.clone(),
                        kind: NodeKind::CallSite,
                        name: callee_text.to_string(),
                        project_id: project_id.to_string(),
                        language: Some("typescript".to_string()),
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

                    // Edge: Caller contains CallSite
                    edges.push(Edge {
                        id: IdentityGenerator::for_edge("contains", caller_id, &callsite_id),
                        from: caller_id.to_string(),
                        to: callsite_id.clone(),
                        kind: EdgeKind::Contains,
                        qualifier: serde_json::Value::Null,
                        provenance: Provenance::default(),
                        temporal: TemporalRange::default(),
                    });

                    // Candidate Target Function ID
                    // (Target will be resolved during graph linking or matches function name)
                    let target_fn_id = IdentityGenerator::for_symbol(
                        "typescript",
                        file_path,
                        callee_text,
                        &NodeKind::Function,
                    );

                    // Edge: CallSite calls Target Function
                    edges.push(Edge {
                        id: IdentityGenerator::for_edge("calls", &callsite_id, &target_fn_id),
                        from: callsite_id.clone(),
                        to: target_fn_id.clone(),
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

                    // Direct caller --calls--> target edge for fast reachability
                    edges.push(Edge {
                        id: IdentityGenerator::for_edge("calls", caller_id, &target_fn_id),
                        from: caller_id.to_string(),
                        to: target_fn_id,
                        kind: EdgeKind::Calls,
                        qualifier: serde_json::json!({ "callsite": callsite_id }),
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

    // Recurse children
    let mut cursor = ts_node.walk();
    for child in ts_node.children(&mut cursor) {
        walk_tree(
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
