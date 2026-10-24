use tree_sitter::{Language, Node, Parser};

pub(super) struct Complexity {
    pub value: Option<usize>,
    pub function: Option<String>,
    pub method: &'static str,
}

pub(super) fn max_function_complexity(path: &str, source: &str) -> Complexity {
    let (language, method): (Option<Language>, _) = match std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
    {
        Some("ts" | "tsx" | "js" | "jsx") => (
            Some(if path.ends_with(".tsx") || path.ends_with(".jsx") {
                tree_sitter_typescript::LANGUAGE_TSX.into()
            } else {
                tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()
            }),
            "tree-sitter TypeScript/JavaScript",
        ),
        Some("rs") => (Some(tree_sitter_rust::LANGUAGE.into()), "tree-sitter Rust"),
        Some("py") => (
            Some(tree_sitter_python::LANGUAGE.into()),
            "tree-sitter Python",
        ),
        _ => (None, "unavailable for this language"),
    };
    let Some(language) = language else {
        return Complexity {
            value: None,
            function: None,
            method,
        };
    };
    let mut parser = Parser::new();
    if parser.set_language(&language).is_err() {
        return Complexity {
            value: None,
            function: None,
            method,
        };
    }
    let Some(tree) = parser.parse(source, None) else {
        return Complexity {
            value: None,
            function: None,
            method,
        };
    };
    let root = tree.root_node();
    if root.has_error() {
        return Complexity {
            value: None,
            function: None,
            method,
        };
    }
    let mut functions = Vec::new();
    collect_functions(root, source.as_bytes(), &mut functions);
    let mut best: Option<(usize, String)> = None;
    for function in functions {
        let mut branches = 0;
        count_branches(function, function, source.as_bytes(), &mut branches);
        let complexity = branches + 1;
        let name = function_name(function, source.as_bytes());
        if best
            .as_ref()
            .is_none_or(|(current, _)| complexity > *current)
        {
            best = Some((complexity, name));
        }
    }
    Complexity {
        value: best.as_ref().map(|(value, _)| *value),
        function: best.map(|(_, name)| name),
        method,
    }
}

fn is_function(kind: &str) -> bool {
    matches!(
        kind,
        "function_declaration"
            | "function_expression"
            | "arrow_function"
            | "method_definition"
            | "function_item"
            | "closure_expression"
            | "function_definition"
            | "lambda"
    )
}

fn collect_functions<'tree>(node: Node<'tree>, source: &[u8], output: &mut Vec<Node<'tree>>) {
    if is_function(node.kind()) {
        output.push(node);
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_functions(child, source, output);
    }
    let _ = source;
}

fn count_branches(node: Node<'_>, root_function: Node<'_>, source: &[u8], count: &mut usize) {
    if node.id() != root_function.id() && is_function(node.kind()) {
        return;
    }
    if matches!(
        node.kind(),
        "if_statement"
            | "if_expression"
            | "for_statement"
            | "for_in_statement"
            | "for_expression"
            | "while_statement"
            | "while_expression"
            | "loop_expression"
            | "catch_clause"
            | "except_clause"
            | "conditional_expression"
            | "switch_case"
            | "match_arm"
    ) {
        *count += 1;
    }
    if node.kind() == "binary_expression" {
        let mut cursor = node.walk();
        if node
            .children(&mut cursor)
            .any(|child| matches!(child.kind(), "&&" | "||" | "??"))
        {
            *count += 1;
        }
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        count_branches(child, root_function, source, count);
    }
    let _ = source;
}

fn function_name(node: Node<'_>, source: &[u8]) -> String {
    if let Some(name) = node
        .parent()
        .filter(|p| p.kind() == "variable_declarator")
        .and_then(|parent| parent.child_by_field_name("name"))
        .and_then(|name| name.utf8_text(source).ok())
    {
        return name.to_owned();
    }
    if let Some(name) = node.child_by_field_name("name") {
        if let Ok(text) = name.utf8_text(source) {
            return text.to_owned();
        }
    }
    node.parent()
        .and_then(|parent| {
            (parent.kind() == "variable_declarator")
                .then(|| parent.child_by_field_name("name"))
                .flatten()
                .and_then(|name| name.utf8_text(source).ok())
                .map(str::to_owned)
        })
        .unwrap_or_else(|| format!("<anonymous@{}>", node.start_position().row + 1))
}
