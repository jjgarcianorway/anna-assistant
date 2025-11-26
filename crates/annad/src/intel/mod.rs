//! Intelligence Module - Systems Intelligence and Diagnostics
//!
//! v6.57.0: Cleaned up - removed legacy sysadmin_brain with hardcoded rules.
//! DiagnosticInsight types moved to proactive_engine.
//!
//! Proactive engine remains for telemetry-based trend detection.

pub mod proactive_engine;
// REMOVED in 6.57.0: sysadmin_brain - hardcoded diagnostic rules

pub use proactive_engine::{
    compute_proactive_assessment, ProactiveAssessment, ProactiveInput,
    CorrelatedIssue, RootCause, IssueSeverity, Signal, TrendObservation,
    RecoveryNotice, HistorianContext, MAX_DISPLAYED_ISSUES,
    assessment_to_summaries, root_cause_to_string, severity_to_string,
    // v6.57.0: Types moved from deleted sysadmin_brain
    DiagnosticInsight, DiagnosticSeverity,
    // v6.57.0: Stub functions for backwards compatibility
    analyze_system_health, format_insights,
};
