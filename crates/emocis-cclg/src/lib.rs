pub mod adapter;
pub mod analysis;

pub use adapter::{AdapterRegistry, LanguageAdapter, ParsedFileGraph, PythonAdapter, RustAdapter, TypeScriptAdapter};
pub use analysis::{BehavioralDiffEngine, BehavioralDiffReport, ExecutionPath, GraphAnalyzer, ImpactReport};
