pub mod context;
pub mod drift;
pub mod duplication;
pub mod extractor;
pub mod failed_attempts;
pub mod health;
pub mod refactor;
pub mod search;

pub use context::{ContextCompressor, MinimalContextPackage};
pub use drift::{DriftDetector, DriftReport, DriftSeverity, DriftViolation};
pub use duplication::{DuplicationCandidate, DuplicationDetector};
pub use extractor::KnowledgeExtractor;
pub use failed_attempts::{FailedAttemptRecord, FailedAttemptsManager};
pub use health::{HealthMetrics, HealthScorer};
pub use refactor::{RefactorDiagnosis, RefactoringRadar};
pub use search::{ConceptualToc, MemoryEngine, TocSection};
