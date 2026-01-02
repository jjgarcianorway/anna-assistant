//! Source-of-Truth Layer - v0.0.443.
//!
//! Make Anna auditable, honest, and inspectable:
//!
//! 1. Source providers (man, help, wiki) with citations
//! 2. Debug trace observability
//! 3. Model inventory (no duplicates, proper attribution)
//! 4. Helper inventory and attribution
//! 5. Clean stats and dialogs

pub mod helper_inventory;
pub mod inventory_common;
pub mod model_inventory;
pub mod providers;
pub mod providers_archwiki;
pub mod providers_config;
pub mod providers_help;
pub mod providers_man;
pub mod providers_types;
pub mod research;
pub mod research_builder;
pub mod research_types;
pub mod stats_ui;
pub mod trace;
pub mod trace_file_manager;
pub mod trace_types;

// Re-export main types
pub use helper_inventory::{HelperEntry, HelperRegistry, HelperStatus, InstallMethod};
pub use inventory_common::InstalledBy;
pub use model_inventory::{ModelEntry, ModelRegistry, ModelRole, OllamaModel, RegistrySummary};
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
