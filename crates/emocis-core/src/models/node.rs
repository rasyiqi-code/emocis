use serde::{Deserialize, Serialize};
use super::provenance::Provenance;

/// PRD Part II Section 9: Node Taxonomy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeCategory {
    Structural,
    Symbol,
    Execution,
    Data,
    External,
    Knowledge,
    History,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    // 9.1 Structural Nodes
    Project,
    Package,
    Module,
    Directory,
    File,
    Namespace,

    // 9.2 Symbol Nodes
    Class,
    Interface,
    Struct,
    Enum,
    Function,
    Method,
    Constructor,
    Variable,
    Parameter,
    Property,
    Constant,
    Type,
    Generic,

    // 9.3 Execution Nodes
    Entry,
    Exit,
    Statement,
    Expression,
    Condition,
    Branch,
    Loop,
    Return,
    Throw,
    Catch,
    Finally,
    CallSite,

    // 9.4 Data Nodes
    Value,
    Object,
    Collection,
    Field,
    Database,
    Table,
    Column,
    Query,

    // 9.5 External Nodes
    Api,
    Endpoint,
    Service,
    Queue,
    Topic,
    Filesystem,
    Process,

    // 9.6 Knowledge Nodes
    Comment,
    Decision,
    Constraint,
    BusinessRule,
    Warning,
    Assumption,
    FailedAttempt,
    Worklog,

    // 9.7 History Nodes
    Commit,
    BranchNode,
    Tag,
    Change,
    Snapshot,
}

impl NodeKind {
    pub fn category(&self) -> NodeCategory {
        match self {
            NodeKind::Project
            | NodeKind::Package
            | NodeKind::Module
            | NodeKind::Directory
            | NodeKind::File
            | NodeKind::Namespace => NodeCategory::Structural,

            NodeKind::Class
            | NodeKind::Interface
            | NodeKind::Struct
            | NodeKind::Enum
            | NodeKind::Function
            | NodeKind::Method
            | NodeKind::Constructor
            | NodeKind::Variable
            | NodeKind::Parameter
            | NodeKind::Property
            | NodeKind::Constant
            | NodeKind::Type
            | NodeKind::Generic => NodeCategory::Symbol,

            NodeKind::Entry
            | NodeKind::Exit
            | NodeKind::Statement
            | NodeKind::Expression
            | NodeKind::Condition
            | NodeKind::Branch
            | NodeKind::Loop
            | NodeKind::Return
            | NodeKind::Throw
            | NodeKind::Catch
            | NodeKind::Finally
            | NodeKind::CallSite => NodeCategory::Execution,

            NodeKind::Value
            | NodeKind::Object
            | NodeKind::Collection
            | NodeKind::Field
            | NodeKind::Database
            | NodeKind::Table
            | NodeKind::Column
            | NodeKind::Query => NodeCategory::Data,

            NodeKind::Api
            | NodeKind::Endpoint
            | NodeKind::Service
            | NodeKind::Queue
            | NodeKind::Topic
            | NodeKind::Filesystem
            | NodeKind::Process => NodeCategory::External,

            NodeKind::Comment
            | NodeKind::Decision
            | NodeKind::Constraint
            | NodeKind::BusinessRule
            | NodeKind::Warning
            | NodeKind::Assumption
            | NodeKind::FailedAttempt
            | NodeKind::Worklog => NodeCategory::Knowledge,

            NodeKind::Commit
            | NodeKind::BranchNode
            | NodeKind::Tag
            | NodeKind::Change
            | NodeKind::Snapshot => NodeCategory::History,
        }
    }
}

/// PRD Part II Section 8: Source location within code
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceLocation {
    pub file: String,
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
}

/// PRD Part II Section 8: Node identity (stable across line shifting)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeIdentity {
    pub qualified_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeStatus {
    Active,
    Deprecated,
    Archived,
    Deleted,
}

impl Default for NodeStatus {
    fn default() -> Self {
        NodeStatus::Active
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeState {
    pub status: NodeStatus,
}

impl Default for NodeState {
    fn default() -> Self {
        Self {
            status: NodeStatus::Active,
        }
    }
}

/// Canonical Node Schema (PRD Part II Section 8)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub kind: NodeKind,
    pub name: String,
    pub project_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceLocation>,
    pub identity: NodeIdentity,
    pub state: NodeState,
    pub provenance: Provenance,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub metadata: serde_json::Value,
}
