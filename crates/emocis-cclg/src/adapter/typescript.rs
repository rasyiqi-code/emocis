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
        &["ts", "tsx", "js", "jsx", "vue", "svelte"]
    }

    fn parse_file(
        &self,
        project_id: &str,
        file_path: &str,
        content: &str,
    ) -> Result<ParsedFileGraph> {
        let mut parser = Parser::new();
        let language = if file_path.ends_with(".tsx") || file_path.ends_with(".jsx") {
            tree_sitter_typescript::LANGUAGE_TSX.into()
        } else {
            tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()
        };
        parser
            .set_language(&language)
            .map_err(|e| anyhow!("Failed to set TypeScript tree-sitter language: {:?}", e))?;

        let effective_content = if file_path.ends_with(".vue") || file_path.ends_with(".svelte") {
            extract_script_content(content)
        } else {
            content.to_string()
        };

        let tree = parser
            .parse(&effective_content, None)
            .ok_or_else(|| anyhow!("Tree-sitter failed to parse file {}", file_path))?;

        let root_node = tree.root_node();
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        let content_bytes = effective_content.as_bytes();

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
            metadata: serde_json::json!({"imports": collect_imports(root_node, content_bytes), "exports": collect_exports(root_node, content_bytes)}),
        };
        nodes.push(file_node);

        // 2. Walk AST recursively
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

        // 3. For SFC components (Vue / Svelte), extract template event bindings
        if file_path.ends_with(".vue") || file_path.ends_with(".svelte") {
            extract_template_event_calls(
                content,
                project_id,
                file_path,
                &file_node_id,
                &mut call_counter,
                &mut nodes,
                &mut edges,
            );
        }

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
        "function_declaration" | "function_expression" | "arrow_function" => {
            if let Some(fn_name) = callable_name(*ts_node, code) {
                let parent_id = current_enclosing_id.unwrap_or(file_node_id);
                let qualified = current_enclosing_id
                    .and_then(|id| nodes.iter().find(|n| n.id == id))
                    .map(|parent| format!("{}.{}", parent.identity.qualified_name, fn_name))
                    .unwrap_or_else(|| fn_name.clone());
                let fn_id = IdentityGenerator::for_symbol(
                    "typescript",
                    file_path,
                    &qualified,
                    &NodeKind::Function,
                );
                nodes.push(Node {
                    id: fn_id.clone(),
                    kind: NodeKind::Function,
                    name: fn_name,
                    project_id: project_id.to_owned(),
                    language: Some("typescript".to_string()),
                    source: Some(node_source_loc(ts_node, file_path)),
                    identity: NodeIdentity {
                        qualified_name: qualified,
                        signature: None,
                    },
                    state: NodeState::default(),
                    provenance: Provenance::default(),
                    metadata: serde_json::Value::Null,
                });
                edges.push(Edge {
                    id: IdentityGenerator::for_edge("contains", parent_id, &fn_id),
                    from: parent_id.to_owned(),
                    to: fn_id.clone(),
                    kind: EdgeKind::Contains,
                    qualifier: serde_json::Value::Null,
                    provenance: Provenance::default(),
                    temporal: TemporalRange::default(),
                });
                new_enclosing_id = Some(fn_id);
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
                        Some(enc) => {
                            format!("{}.{}", enc.split(':').nth(3).unwrap_or(""), method_name)
                        }
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

                    let clean_callee = callee_text.trim().trim_start_matches("await ").trim();
                    let clean_callee = clean_callee.split('<').next().unwrap_or(clean_callee).trim();
                    let clean_callee = clean_callee.trim_matches(['(', ')']).trim();
                    let is_invoke_call = clean_callee == "invoke"
                        || clean_callee.starts_with("invoke<")
                        || clean_callee.ends_with(".invoke")
                        || clean_callee.contains(".invoke<");

                    let ipc_cmd = if is_invoke_call {
                        ts_node.child_by_field_name("arguments").and_then(|args| {
                            let mut cursor = args.walk();
                            for child in args.named_children(&mut cursor) {
                                if child.kind() == "string" {
                                    let raw_str = child.utf8_text(code).unwrap_or("");
                                    let cleaned = raw_str.trim_matches(['\'', '"', '`']);
                                    if !cleaned.is_empty() {
                                        return Some(cleaned.to_string());
                                    }
                                }
                                break;
                            }
                            None
                        })
                    } else {
                        None
                    };

                    let (node_display_name, raw_callee_val, qualifier_json) = if let Some(ref cmd) = ipc_cmd {
                        (
                            format!("invoke('{}')", cmd),
                            format!("tauri_ipc:{}", cmd),
                            serde_json::json!({
                                "raw_callee": format!("tauri_ipc:{}", cmd),
                                "ipc_command": cmd,
                                "ipc_type": "tauri"
                            }),
                        )
                    } else {
                        (
                            clean_callee.to_string(),
                            clean_callee.to_string(),
                            serde_json::json!({ "raw_callee": clean_callee }),
                        )
                    };

                    let callsite_id =
                        IdentityGenerator::for_callsite(caller_id, &raw_callee_val, *call_counter);

                    let callsite_node = Node {
                        id: callsite_id.clone(),
                        kind: NodeKind::CallSite,
                        name: node_display_name,
                        project_id: project_id.to_string(),
                        language: Some("typescript".to_string()),
                        source: Some(node_source_loc(ts_node, file_path)),
                        identity: NodeIdentity {
                            qualified_name: format!("call:{}", raw_callee_val),
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
                    let target_fn_id = IdentityGenerator::for_symbol(
                        "typescript",
                        file_path,
                        &raw_callee_val,
                        &NodeKind::Function,
                    );

                    // Edge: CallSite calls Target Function
                    edges.push(Edge {
                        id: IdentityGenerator::for_edge("calls", &callsite_id, &target_fn_id),
                        from: callsite_id.clone(),
                        to: target_fn_id.clone(),
                        kind: EdgeKind::Calls,
                        qualifier: qualifier_json.clone(),
                        provenance: Provenance {
                            source: ProvenanceSource::Ast,
                            confidence: 0.9,
                            certainty: Certainty::Likely,
                            evidence: Vec::new(),
                        },
                        temporal: TemporalRange::default(),
                    });

                    // Direct caller --calls--> target edge for fast reachability
                    let mut direct_qual = qualifier_json;
                    if let Some(obj) = direct_qual.as_object_mut() {
                        obj.insert("callsite".to_string(), serde_json::json!(callsite_id));
                    }
                    edges.push(Edge {
                        id: IdentityGenerator::for_edge("calls", caller_id, &target_fn_id),
                        from: caller_id.to_string(),
                        to: target_fn_id,
                        kind: EdgeKind::Calls,
                        qualifier: direct_qual,
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

/// Read import bindings from syntax nodes, so comments and strings cannot create links.
fn collect_imports(root: TsNode, code: &[u8]) -> Vec<serde_json::Value> {
    let mut imports = Vec::new();
    let mut cursor = root.walk();
    for statement in root.named_children(&mut cursor) {
        if statement.kind() != "import_statement" || has_token(statement, "type") {
            continue;
        }
        let Some(source) = statement.child_by_field_name("source") else {
            continue;
        };
        let module = source
            .utf8_text(code)
            .unwrap_or("")
            .trim_matches(['\'', '"']);
        fn bindings(node: TsNode, code: &[u8], module: &str, out: &mut Vec<serde_json::Value>) {
            if node.kind() == "import_specifier" && has_token(node, "type") {
                return;
            }
            if node.kind() == "identifier"
                && node.parent().is_some_and(|p| p.kind() == "import_clause")
            {
                out.push(serde_json::json!({"module": module, "imported": "default", "local": node.utf8_text(code).unwrap_or("")}));
                return;
            }
            if node.kind() == "import_specifier" {
                if let Some(name) = node.child_by_field_name("name") {
                    let alias = node.child_by_field_name("alias").unwrap_or(name);
                    out.push(serde_json::json!({"module": module,
                        "imported": name.utf8_text(code).unwrap_or(""),
                        "local": alias.utf8_text(code).unwrap_or("")}));
                }
                return;
            }
            if node.kind() == "namespace_import" {
                let mut c = node.walk();
                for child in node.named_children(&mut c) {
                    if child.kind() == "identifier" {
                        out.push(serde_json::json!({"module": module, "imported": "*", "local": child.utf8_text(code).unwrap_or("")}));
                    }
                }
                return;
            }
            let mut c = node.walk();
            for child in node.named_children(&mut c) {
                bindings(child, code, module, out);
            }
        }
        bindings(statement, code, module, &mut imports);
    }
    imports
}

fn has_token(node: TsNode, kind: &str) -> bool {
    let mut cursor = node.walk();
    let found = node.children(&mut cursor).any(|child| child.kind() == kind);
    found
}

fn callable_name(node: TsNode, code: &[u8]) -> Option<String> {
    if let Some(parent) = node.parent() {
        if parent.kind() == "variable_declarator" {
            let name = parent.child_by_field_name("name")?;
            if name.kind() == "identifier" {
                return name.utf8_text(code).ok().map(str::to_owned);
            }
        }
        if parent.kind() == "export_statement" && has_token(parent, "default") {
            return Some(
                node.child_by_field_name("name")
                    .and_then(|n| n.utf8_text(code).ok())
                    .unwrap_or("<default>")
                    .to_owned(),
            );
        }
    }
    if node.kind() == "function_declaration" {
        return node
            .child_by_field_name("name")
            .and_then(|n| n.utf8_text(code).ok())
            .map(str::to_owned);
    }
    None
}

/// Explicit exported names only. Type-only exports and export-star are not callable bindings.
fn collect_exports(root: TsNode, code: &[u8]) -> Vec<serde_json::Value> {
    let mut exports = Vec::new();
    let mut cursor = root.walk();
    for statement in root.named_children(&mut cursor) {
        if statement.kind() != "export_statement" || has_token(statement, "type") {
            continue;
        }
        let source = statement
            .child_by_field_name("source")
            .and_then(|n| n.utf8_text(code).ok())
            .map(|s| s.trim_matches(['\'', '"']).to_owned());
        if has_token(statement, "default") {
            if let Some(value) = statement
                .child_by_field_name("declaration")
                .or_else(|| statement.child_by_field_name("value"))
            {
                if value.kind() == "object" {
                    let mut obj_cursor = value.walk();
                    for prop in value.named_children(&mut obj_cursor) {
                        if prop.kind() == "method_definition" {
                            if let Some(m_name) = prop.child_by_field_name("name") {
                                if let Ok(m_str) = m_name.utf8_text(code) {
                                    exports.push(serde_json::json!({"exported": m_str, "local": m_str}));
                                }
                            }
                        } else if prop.kind() == "pair" {
                            if let Some(k_name) = prop.child_by_field_name("key") {
                                if let Ok(k_str) = k_name.utf8_text(code) {
                                    exports.push(serde_json::json!({"exported": k_str, "local": k_str}));
                                }
                            }
                        }
                    }
                }
                let local = if value.kind() == "identifier" {
                    value.utf8_text(code).ok().map(str::to_owned)
                } else {
                    callable_name(value, code)
                };
                if let Some(local) = local {
                    exports.push(serde_json::json!({"exported": "default", "local": local}));
                }
            }
            continue;
        }
        if let Some(declaration) = statement.child_by_field_name("declaration") {
            if declaration.kind() == "function_declaration" {
                if let Some(name) = callable_name(declaration, code) {
                    exports.push(serde_json::json!({"exported": name, "local": name}));
                }
            } else if matches!(
                declaration.kind(),
                "lexical_declaration" | "variable_declaration"
            ) {
                let mut c = declaration.walk();
                for variable in declaration.named_children(&mut c) {
                    if let (Some(name), Some(value)) = (
                        variable.child_by_field_name("name"),
                        variable.child_by_field_name("value"),
                    ) {
                        if name.kind() == "identifier" {
                            let obj_name = name.utf8_text(code).unwrap_or("");
                            if matches!(value.kind(), "arrow_function" | "function_expression") {
                                exports.push(serde_json::json!({"exported": obj_name, "local": obj_name}));
                            } else if value.kind() == "object" {
                                exports.push(serde_json::json!({"exported": obj_name, "local": obj_name}));
                                let mut obj_cursor = value.walk();
                                for prop in value.named_children(&mut obj_cursor) {
                                    if prop.kind() == "method_definition" {
                                        if let Some(m_name) = prop.child_by_field_name("name") {
                                            if let Ok(m_str) = m_name.utf8_text(code) {
                                                exports.push(serde_json::json!({
                                                    "exported": format!("{}.{}", obj_name, m_str),
                                                    "local": m_str
                                                }));
                                            }
                                        }
                                    } else if prop.kind() == "pair" {
                                        if let Some(k_name) = prop.child_by_field_name("key") {
                                            if let Ok(k_str) = k_name.utf8_text(code) {
                                                exports.push(serde_json::json!({
                                                    "exported": format!("{}.{}", obj_name, k_str),
                                                    "local": k_str
                                                }));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        let mut c = statement.walk();
        for clause in statement.named_children(&mut c) {
            if clause.kind() != "export_clause" {
                continue;
            }
            let mut spec_cursor = clause.walk();
            for spec in clause.named_children(&mut spec_cursor) {
                if spec.kind() != "export_specifier" || has_token(spec, "type") {
                    continue;
                }
                if let Some(name) = spec.child_by_field_name("name") {
                    let alias = spec.child_by_field_name("alias").unwrap_or(name);
                    exports.push(
                        serde_json::json!({"local": name.utf8_text(code).unwrap_or(""),
                        "exported": alias.utf8_text(code).unwrap_or(""), "module": source}),
                    );
                }
            }
        }
    }
    exports
}

fn extract_script_content(content: &str) -> String {
    let mut bytes = content.as_bytes().to_vec();
    let len = bytes.len();
    let lower = content.to_ascii_lowercase();

    let mut mask = vec![false; len];
    let mut cursor = 0;

    while cursor < len {
        if let Some(rel_start) = lower[cursor..].find("<script") {
            let tag_start = cursor + rel_start;
            // Find end of <script ...> tag (i.e. '>')
            if let Some(rel_gt) = lower[tag_start..].find('>') {
                let gt_pos = tag_start + rel_gt;
                let script_content_start = gt_pos + 1;

                // Check self-closing tag like <script ... />
                let is_self_closing = gt_pos > tag_start && bytes[gt_pos - 1] == b'/';
                if is_self_closing {
                    cursor = script_content_start;
                    continue;
                }

                if let Some(rel_close) = lower[script_content_start..].find("</script") {
                    let script_content_end = script_content_start + rel_close;
                    for i in script_content_start..script_content_end {
                        mask[i] = true;
                    }
                    if let Some(rel_close_gt) = lower[script_content_end..].find('>') {
                        cursor = script_content_end + rel_close_gt + 1;
                    } else {
                        cursor = script_content_end + 8;
                    }
                } else {
                    for i in script_content_start..len {
                        mask[i] = true;
                    }
                    break;
                }
            } else {
                break;
            }
        } else {
            break;
        }
    }

    for i in 0..len {
        if !mask[i] {
            let b = bytes[i];
            if b != b'\n' && b != b'\r' {
                bytes[i] = b' ';
            }
        }
    }

    String::from_utf8(bytes).unwrap_or_else(|_| content.to_string())
}

fn extract_template_event_calls(
    content: &str,
    project_id: &str,
    file_path: &str,
    file_node_id: &str,
    call_counter: &mut u32,
    nodes: &mut Vec<Node>,
    edges: &mut Vec<Edge>,
) {
    let lower = content.to_ascii_lowercase();
    let template_end = lower.find("<script").unwrap_or(content.len());
    let template_part = &content[..template_end];

    for (line_idx, line) in template_part.lines().enumerate() {
        let line_num = (line_idx + 1) as u32;
        let mut rest = line;
        while let Some(at_pos) = rest.find('@').or_else(|| rest.find("v-on:")).or_else(|| rest.find("on:")) {
            let sub = &rest[at_pos..];
            if let Some(eq_pos) = sub.find('=') {
                let after_eq = sub[eq_pos + 1..].trim_start();
                if let Some(quote) = after_eq.chars().next() {
                    if quote == '"' || quote == '\'' {
                        let inner = &after_eq[1..];
                        if let Some(end_quote) = inner.find(quote) {
                            let handler_expr = inner[..end_quote].trim();
                            let callee = handler_expr
                                .split('(')
                                .next()
                                .unwrap_or(handler_expr)
                                .trim();

                            if !callee.is_empty()
                                && callee.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '$' || c == '.')
                                && !callee.starts_with('.')
                                && !callee.ends_with('.')
                                && callee != "true"
                                && callee != "false"
                                && callee != "null"
                            {
                                *call_counter += 1;
                                let callsite_id = IdentityGenerator::for_callsite(file_node_id, callee, *call_counter);
                                let target_id = IdentityGenerator::for_symbol(
                                    "typescript",
                                    file_path,
                                    callee,
                                    &NodeKind::Function,
                                );

                                nodes.push(Node {
                                    id: callsite_id.clone(),
                                    kind: NodeKind::CallSite,
                                    name: format!("@event -> {}", callee),
                                    project_id: project_id.to_string(),
                                    language: Some("typescript".to_string()),
                                    source: Some(SourceLocation {
                                        file: file_path.to_string(),
                                        start_line: line_num,
                                        start_column: 1,
                                        end_line: line_num,
                                        end_column: 1,
                                    }),
                                    identity: NodeIdentity {
                                        qualified_name: format!("template_call:{}", callee),
                                        signature: None,
                                    },
                                    state: NodeState::default(),
                                    provenance: Provenance::default(),
                                    metadata: serde_json::json!({ "template_binding": true, "raw_callee": callee }),
                                });

                                edges.push(Edge {
                                    id: IdentityGenerator::for_edge("contains", file_node_id, &callsite_id),
                                    from: file_node_id.to_string(),
                                    to: callsite_id.clone(),
                                    kind: EdgeKind::Contains,
                                    qualifier: serde_json::Value::Null,
                                    provenance: Provenance::default(),
                                    temporal: TemporalRange::default(),
                                });

                                edges.push(Edge {
                                    id: IdentityGenerator::for_edge("calls", file_node_id, &target_id),
                                    from: file_node_id.to_string(),
                                    to: target_id.clone(),
                                    kind: EdgeKind::Calls,
                                    qualifier: serde_json::json!({
                                        "raw_callee": callee,
                                        "template_event": true
                                    }),
                                    provenance: Provenance::default(),
                                    temporal: TemporalRange::default(),
                                });
                            }
                            rest = &inner[end_quote + 1..];
                            continue;
                        }
                    }
                }
            }
            rest = &sub[1..];
        }
    }
}

