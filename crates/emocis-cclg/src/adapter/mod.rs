use emocis_core::{Edge, Node};

pub mod typescript;

pub use typescript::TypeScriptAdapter;

#[derive(Debug, Clone)]
pub struct ParsedFileGraph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

pub trait LanguageAdapter: Send + Sync {
    fn language_name(&self) -> &'static str;
    fn supported_extensions(&self) -> &[&'static str];
    fn parse_file(
        &self,
        project_id: &str,
        file_path: &str,
        content: &str,
    ) -> anyhow::Result<ParsedFileGraph>;
}
