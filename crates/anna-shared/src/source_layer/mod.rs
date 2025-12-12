//! Source-of-Truth Layer - v0.0.443.
//!
//! Make Anna auditable, honest, and inspectable:
//!
//! 1. Source providers (man, help, wiki) with citations
//! 2. Debug trace observability
//! 3. Model inventory (no duplicates, proper attribution)
//! 4. Helper inventory and attribution
//! 5. Clean stats and dialogs

pub mod inventory;
pub mod providers;
pub mod research;
pub mod stats_ui;
pub mod trace;

// Re-export main types
pub use inventory::{
    HelperEntry, HelperRegistry, HelperStatus, InstallMethod, InstalledBy, ModelEntry,
    ModelRegistry, ModelRole, OllamaModel, RegistrySummary,
};
pub use providers::{
    commands_for_intent, ArchWikiProvider, HelpProvider, IntentCommands, LocalConfigProvider,
    ManProvider, SourceContent, SourceRequest, SourceType,
};
pub use research::{
    Citation, CitationBuilder, CitedAnswer, ResearchConstraints, ResearchPlan, ResearchResult,
};
pub use stats_ui::{
    CleanStats, ConfirmDialog, DialogChoice, DialogQuestion, DialogResult, OutputMode,
    ProgressIndicator,
};
pub use trace::{DebugSummary, RequestTrace, TraceEvent, TraceFileManager, TraceStage};
