use serde::{Deserialize, Serialize};

/// PRD Section 25: Certainty Model
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Certainty {
    /// Terbukti secara deterministik melalui AST direct call
    Certain,
    /// Resolusi antarmuka polimorfik atau dynamic dispatch dengan kandidat terbatas
    Likely,
    /// Resolusi berbasis inferensi semantik atau prediksi model
    Inferred,
    /// Perilaku yang sangat bergantung pada konfigurasi dinamis eksternal
    Unknown,
}

impl Default for Certainty {
    fn default() -> Self {
        Certainty::Certain
    }
}

/// Source of provenance detection
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProvenanceSource {
    Parser,
    Ast,
    TypeChecker,
    SymbolResolver,
    CfgAnalysis,
    DataflowAnalysis,
    SqlAnalysis,
    Git,
    Comment,
    Documentation,
    Human,
    AiInference,
}

/// Evidence supporting a node, edge, or knowledge fact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub evidence_type: String, // "comment", "ast", "commit", "doc", "test"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snippet: Option<String>,
}

/// PRD Section 39 & Part II Section 24: Provenance Tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provenance {
    pub source: ProvenanceSource,
    pub confidence: f32, // 0.0 - 1.0
    pub certainty: Certainty,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<Evidence>,
}

impl Default for Provenance {
    fn default() -> Self {
        Self {
            source: ProvenanceSource::Parser,
            confidence: 1.0,
            certainty: Certainty::Certain,
            evidence: Vec::new(),
        }
    }
}
