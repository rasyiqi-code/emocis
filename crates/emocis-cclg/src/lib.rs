pub mod adapter;
pub mod analysis;

pub use adapter::{LanguageAdapter, ParsedFileGraph, TypeScriptAdapter};
pub use analysis::{BehavioralDiffEngine, BehavioralDiffReport, ExecutionPath, GraphAnalyzer, ImpactReport};
