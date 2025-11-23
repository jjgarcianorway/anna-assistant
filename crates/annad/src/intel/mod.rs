//! Intelligence Module - Systems Intelligence and Diagnostics
//!
//! Beta.217a: Sysadmin Brain Foundation
//! Beta.270: Proactive Engine and Root-Cause Correlation
//!
//! Provides deterministic rule-based intelligence for system diagnostics
//! without LLM inference. Pure logic-based insights from telemetry data.

pub mod proactive_engine;
pub mod sysadmin_brain;

pub use sysadmin_brain::{analyze_system_health, DiagnosticInsight, DiagnosticSeverity};
pub use proactive_engine::{
    compute_proactive_assessment, ProactiveAssessment, ProactiveInput,
    CorrelatedIssue, RootCause, IssueSeverity, Signal, TrendObservation,
    RecoveryNotice, HistorianContext, MAX_DISPLAYED_ISSUES,
    assessment_to_summaries, root_cause_to_string, severity_to_string,
};
