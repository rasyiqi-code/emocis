use emocis_core::{Node, NodeKind, SqliteStore};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use tree_sitter::{Language, Node as SyntaxNode, Parser};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicationCandidate {
    pub function_a: String,
    pub location_a: String,
    pub function_b: String,
    pub location_b: String,
    /// A review signal: identical name, similar call sets, AST shape, or lexical names.
    pub similarity_level: &'static str,
    pub shared_responsibility: String,
    pub recommendation: String,
}

pub struct DuplicationDetector<'a> {
    store: &'a SqliteStore,
}

impl<'a> DuplicationDetector<'a> {
    pub fn new(store: &'a SqliteStore) -> Self {
        Self { store }
    }

    pub fn scan_duplications(&self) -> anyhow::Result<Vec<DuplicationCandidate>> {
        self.scan(None)
    }

    pub fn scan_duplications_for_project(
        &self,
        project_id: &str,
    ) -> anyhow::Result<Vec<DuplicationCandidate>> {
        self.scan(Some(project_id))
    }

    fn scan(&self, project_id: Option<&str>) -> anyhow::Result<Vec<DuplicationCandidate>> {
        let all_nodes = self.store.find_nodes_by_name("")?;
        let functions: Vec<&Node> = all_nodes
            .iter()
            .filter(|node| {
                (node.kind == NodeKind::Function || node.kind == NodeKind::Method)
                    && project_id.is_none_or(|project| node.project_id == project)
                    && node.source.is_some()
            })
            .collect();
        let mut calls = HashMap::<String, HashSet<String>>::new();
        let mut file_shapes = HashMap::<
            String,
            Option<HashMap<(String, usize, usize), BTreeMap<String, usize>>>,
        >::new();
        let mut candidates = Vec::new();

        for (index, left) in functions.iter().enumerate() {
            let left_file = left.source.as_ref().unwrap().file.as_str();
            if !file_shapes.contains_key(left_file) {
                file_shapes.insert(left_file.to_owned(), load_file_shapes(left_file));
            }
            if !calls.contains_key(&left.id) {
                calls.insert(left.id.clone(), call_names(self.store, left)?);
            }
            let left_shape = shape_for(&file_shapes, left).cloned();
            for right in functions.iter().skip(index + 1) {
                let right_file = right.source.as_ref().unwrap().file.as_str();
                if left_file == right_file {
                    continue;
                }
                if !file_shapes.contains_key(right_file) {
                    file_shapes.insert(right_file.to_owned(), load_file_shapes(right_file));
                }
                if !calls.contains_key(&right.id) {
                    calls.insert(right.id.clone(), call_names(self.store, right)?);
                }
                let right_shape = shape_for(&file_shapes, right).cloned();
                if left.name == right.name && left.name.len() > 3 {
                    candidates.push(candidate(left, right, "Identical name", format!("Kedua simbol bernama '{}'. Kesamaan nama tidak membuktikan kesamaan perilaku.", left.name), "Bandingkan implementasi dan kontrak sebelum menggabungkan."));
                    continue;
                }
                if let (Some(a), Some(b)) = (left_shape.as_ref(), right_shape.as_ref()) {
                    let min_len = a.len().min(b.len());
                    let max_len = a.len().max(b.len());
                    if max_len > 0 && (min_len as f32 / max_len as f32) >= 0.7 && a.len() >= 5 {
                        let similarity = multiset_jaccard(a, b);
                        if similarity >= 0.82 {
                            candidates.push(candidate(left, right, "AST shape", format!("Struktur AST ternormalisasi mirip ({:.0}%); identifier dan nilai literal diabaikan.", similarity * 100.0), "Tinjau kontrol alur, efek samping, tipe, dan hasil fungsi secara manual."));
                            continue;
                        }
                    }
                }
                let a = &calls[&left.id];
                let b = &calls[&right.id];
                let common = a.intersection(b).count();
                let union = a.union(b).count();
                let call_similarity = if union == 0 {
                    0.0
                } else {
                    common as f32 / union as f32
                };
                if common >= 2 && call_similarity >= 0.7 {
                    let names: Vec<_> = a.intersection(b).cloned().collect();
                    candidates.push(candidate(left, right, "Shared call set", format!("Berbagi {} dari {} nama pemanggilan ({:.0}%).", common, union, call_similarity * 100.0), &format!("Nama pemanggilan bersama: {}. Urutan, argumen, dan perilaku tidak dibandingkan.", names.join(", "))));
                    continue;
                }
                let tokens_a = tokenize(&left.name);
                let tokens_b = tokenize(&right.name);
                let shared = tokens_a.intersection(&tokens_b).count();
                let names_union = tokens_a.union(&tokens_b).count();
                if shared >= 2 && names_union > 0 && shared as f32 / names_union as f32 >= 0.6 {
                    candidates.push(candidate(
                        left,
                        right,
                        "Name similarity",
                        format!("Nama berbagi {shared} token leksikal."),
                        "Kesamaan nama saja lemah; tinjau sumber sebelum mengambil keputusan.",
                    ));
                }
            }
        }
        Ok(candidates)
    }
}

fn candidate(
    left: &Node,
    right: &Node,
    level: &'static str,
    evidence: String,
    recommendation: &str,
) -> DuplicationCandidate {
    DuplicationCandidate {
        function_a: left.name.clone(),
        location_a: left
            .source
            .as_ref()
            .map(|s| format!("{}:{}", s.file, s.start_line))
            .unwrap_or_default(),
        function_b: right.name.clone(),
        location_b: right
            .source
            .as_ref()
            .map(|s| format!("{}:{}", s.file, s.start_line))
            .unwrap_or_default(),
        similarity_level: level,
        shared_responsibility: evidence,
        recommendation: recommendation.to_owned(),
    }
}

fn call_names(store: &SqliteStore, function: &Node) -> anyhow::Result<HashSet<String>> {
    Ok(store
        .get_callees(&function.id)?
        .into_iter()
        .map(|(_, node)| node.name)
        .collect())
}

type FunctionShapes = HashMap<(String, usize, usize), BTreeMap<String, usize>>;

fn shape_for<'a>(
    files: &'a HashMap<String, Option<FunctionShapes>>,
    node: &Node,
) -> Option<&'a BTreeMap<String, usize>> {
    let source = node.source.as_ref()?;
    files.get(&source.file)?.as_ref()?.get(&(
        node.name.clone(),
        source.start_line as usize - 1,
        source.end_line as usize - 1,
    ))
}

fn load_file_shapes(file: &str) -> Option<FunctionShapes> {
    let source = std::fs::read_to_string(file).ok()?;
    let extension = std::path::Path::new(file).extension()?.to_str()?;
    let language: Language = match extension {
        "ts" | "js" => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        "tsx" | "jsx" => tree_sitter_typescript::LANGUAGE_TSX.into(),
        "rs" => tree_sitter_rust::LANGUAGE.into(),
        "py" => tree_sitter_python::LANGUAGE.into(),
        _ => return None,
    };
    let mut parser = Parser::new();
    parser.set_language(&language).ok()?;
    let tree = parser.parse(&source, None)?;
    let mut shapes = HashMap::new();
    collect_file_shapes(tree.root_node(), source.as_bytes(), &mut shapes);
    Some(shapes)
}

fn collect_file_shapes(node: SyntaxNode<'_>, source: &[u8], output: &mut FunctionShapes) {
    if is_function_node(node.kind()) {
        let name = node
            .child_by_field_name("name")
            .and_then(|n| n.utf8_text(source).ok())
            .or_else(|| {
                node.parent()
                    .filter(|p| p.kind() == "variable_declarator")
                    .and_then(|p| p.child_by_field_name("name"))
                    .and_then(|n| n.utf8_text(source).ok())
            })
            .unwrap_or("<default>");
        let start = node.start_position().row;
        let end = node.end_position().row;
        let mut features = BTreeMap::new();
        collect_features(node, node, 0, &mut features);
        output.insert((name.to_owned(), start, end), features);
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_file_shapes(child, source, output);
    }
}

fn is_function_node(kind: &str) -> bool {
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

fn collect_features(
    node: SyntaxNode<'_>,
    root: SyntaxNode<'_>,
    depth: usize,
    output: &mut BTreeMap<String, usize>,
) {
    if node.id() != root.id() && is_function_node(node.kind()) {
        return;
    }
    if !matches!(
        node.kind(),
        "identifier"
            | "type_identifier"
            | "property_identifier"
            | "shorthand_property_identifier"
            | "string"
            | "string_content"
            | "template_string"
            | "number"
            | "integer"
            | "float"
            | "comment"
    ) {
        let key = format!("{}:{}", depth.min(12), node.kind());
        *output.entry(key).or_default() += 1;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.is_named() {
            collect_features(child, root, depth + 1, output);
        } else if matches!(
            child.kind(),
            "&&" | "||" | "??" | "and" | "or" | "==" | "!=" | "<" | ">" | "+" | "-" | "*" | "/"
        ) {
            *output
                .entry(format!("{}:operator:{}", depth.min(12), child.kind()))
                .or_default() += 1;
        }
    }
}

fn multiset_jaccard(a: &BTreeMap<String, usize>, b: &BTreeMap<String, usize>) -> f32 {
    let keys: HashSet<_> = a.keys().chain(b.keys()).collect();
    let (mut intersection, mut union) = (0usize, 0usize);
    for key in keys {
        let (left, right) = (
            a.get(key).copied().unwrap_or(0),
            b.get(key).copied().unwrap_or(0),
        );
        intersection += left.min(right);
        union += left.max(right);
    }
    if union == 0 {
        0.0
    } else {
        intersection as f32 / union as f32
    }
}

fn tokenize(name: &str) -> HashSet<String> {
    let mut words = HashSet::new();
    let mut current = String::new();
    for ch in name.chars() {
        if ch == '_' || ch == '-' || ch == '.' || ch == ':' {
            if !current.is_empty() {
                words.insert(current.to_lowercase());
                current.clear();
            }
        } else if ch.is_uppercase() {
            if !current.is_empty() {
                words.insert(current.to_lowercase());
                current.clear();
            }
            current.push(ch);
        } else {
            current.push(ch);
        }
    }
    if !current.is_empty() {
        words.insert(current.to_lowercase());
    }
    words
}
