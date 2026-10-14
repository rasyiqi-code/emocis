use serde::{Deserialize, Serialize};
use super::provenance::Provenance;

/// PRD Part II Section 13: Canonical Edge Taxonomy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeCategory {
    Structural,
    Symbol,
    Behavioral,
    ControlFlow,
    DataFlow,
    External,
    Knowledge,
    History,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind {
    // 13.1 Structural
    Contains,
    Declares,
    Defines,
    Exports,
    Imports,
    BelongsTo,

    // 13.2 Symbol
    References,
    Uses,
    Extends,
    Implements,
    Instantiates,
    Overrides,
    Aliases,
    ResolvesTo,

    // 13.3 Behavioral
    Calls,
    Returns,
    Passes,
    Reads,
    Writes,
    Assigns,
    Mutates,
    Creates,
    Destroys,
    Throws,
    Catches,
    Handles,

    // 13.4 Control Flow
    FlowsTo,
    BranchesTo,
    LoopsTo,

    // 13.5 Data Flow
    DerivesFrom,
    Transforms,
    Filters,
    Maps,
    Aggregates,

    // 13.6 External
    CallsApi,
    ReadsDatabase,
    WritesDatabase,
    Publishes,
    Subscribes,
    Sends,
    Receives,
    ReadsFile,
    WritesFile,
    Spawns,

    // 13.7 Knowledge
    DocumentedBy,
    ConstrainedBy,
    ImplementsRule,
    Affects,
    Contradicts,
    Supersedes,
    CausedBy,
    ReplacedBy,

    // 13.8 History
    IntroducedBy,
    ModifiedBy,
}

impl EdgeKind {
    pub fn category(&self) -> EdgeCategory {
        match self {
            EdgeKind::Contains
            | EdgeKind::Declares
            | EdgeKind::Defines
            | EdgeKind::Exports
            | EdgeKind::Imports
            | EdgeKind::BelongsTo => EdgeCategory::Structural,

            EdgeKind::References
            | EdgeKind::Uses
            | EdgeKind::Extends
            | EdgeKind::Implements
            | EdgeKind::Instantiates
            | EdgeKind::Overrides
            | EdgeKind::Aliases
            | EdgeKind::ResolvesTo => EdgeCategory::Symbol,

            EdgeKind::Calls
            | EdgeKind::Returns
            | EdgeKind::Passes
            | EdgeKind::Reads
            | EdgeKind::Writes
            | EdgeKind::Assigns
            | EdgeKind::Mutates
            | EdgeKind::Creates
            | EdgeKind::Destroys
            | EdgeKind::Throws
            | EdgeKind::Catches
            | EdgeKind::Handles => EdgeCategory::Behavioral,

            EdgeKind::FlowsTo
            | EdgeKind::BranchesTo
            | EdgeKind::LoopsTo => EdgeCategory::ControlFlow,

            EdgeKind::DerivesFrom
            | EdgeKind::Transforms
            | EdgeKind::Filters
            | EdgeKind::Maps
            | EdgeKind::Aggregates => EdgeCategory::DataFlow,

            EdgeKind::CallsApi
            | EdgeKind::ReadsDatabase
            | EdgeKind::WritesDatabase
            | EdgeKind::Publishes
            | EdgeKind::Subscribes
            | EdgeKind::Sends
            | EdgeKind::Receives
            | EdgeKind::ReadsFile
            | EdgeKind::WritesFile
            | EdgeKind::Spawns => EdgeCategory::External,

            EdgeKind::DocumentedBy
            | EdgeKind::ConstrainedBy
            | EdgeKind::ImplementsRule
            | EdgeKind::Affects
            | EdgeKind::Contradicts
            | EdgeKind::Supersedes
            | EdgeKind::CausedBy
            | EdgeKind::ReplacedBy => EdgeCategory::Knowledge,

            EdgeKind::IntroducedBy
            | EdgeKind::ModifiedBy => EdgeCategory::History,
        }
    }
}

/// Temporal validity of an edge (commit snapshot binding)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalRange {
    pub valid_from: String, // e.g. "snapshot:commit_abc123"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<String>,
}

impl Default for TemporalRange {
    fn default() -> Self {
        Self {
            valid_from: "snapshot:initial".to_string(),
            valid_until: None,
        }
    }
}

/// Canonical Edge Schema (PRD Part II Section 12)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub id: String,
    pub from: String, // Node ID
    pub to: String,   // Node ID
    pub kind: EdgeKind,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub qualifier: serde_json::Value,
    pub provenance: Provenance,
    pub temporal: TemporalRange,
}
