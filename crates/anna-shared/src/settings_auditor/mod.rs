// v0.0.691: Settings Auditor (Phase 267)
// Audit settings changes and access

mod auditor;
mod event;
mod registry;
mod types;

#[cfg(test)]
mod tests;

// Re-export all public types to preserve the original API
pub use auditor::{AuditorStats, SettingsAuditor};
pub use event::{AuditEvent, AuditTrail};
pub use registry::{auditor_fun_fact, format_auditor_registry, is_auditor_query, AuditorRegistry};
pub use types::{AuditEventType, AuditSeverity, AuditorConfig};
