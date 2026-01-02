// v0.0.573: Settings Audit - Core Types
// Audit event types, severity levels, and entry structure

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;

/// Audit event type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditEventType {
    /// Settings loaded
    Load,
    /// Settings saved
    Save,
    /// Single setting changed
    Change,
    /// Category reset
    Reset,
    /// Profile switched
    ProfileSwitch,
    /// Template applied
    TemplateApply,
    /// Export performed
    Export,
    /// Import performed
    Import,
    /// Validation failed
    ValidationFailed,
    /// Access denied
    AccessDenied,
}

impl std::fmt::Display for AuditEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Load => write!(f, "Load"),
            Self::Save => write!(f, "Save"),
            Self::Change => write!(f, "Change"),
            Self::Reset => write!(f, "Reset"),
            Self::ProfileSwitch => write!(f, "Profile Switch"),
            Self::TemplateApply => write!(f, "Template Apply"),
            Self::Export => write!(f, "Export"),
            Self::Import => write!(f, "Import"),
            Self::ValidationFailed => write!(f, "Validation Failed"),
            Self::AccessDenied => write!(f, "Access Denied"),
        }
    }
}

/// Audit severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AuditSeverity {
    /// Informational
    Info,
    /// Notable
    Notice,
    /// Warning
    Warning,
    /// Security-related
    Security,
    /// Critical
    Critical,
}

impl std::fmt::Display for AuditSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "Info"),
            Self::Notice => write!(f, "Notice"),
            Self::Warning => write!(f, "Warning"),
            Self::Security => write!(f, "Security"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}

/// An audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Entry ID
    pub id: u64,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Event type
    pub event_type: AuditEventType,
    /// Severity
    pub severity: AuditSeverity,
    /// Category affected
    pub category: Option<SettingsCategory>,
    /// Field affected
    pub field: Option<String>,
    /// Old value
    pub old_value: Option<String>,
    /// New value
    pub new_value: Option<String>,
    /// Source (user, system, api)
    pub source: String,
    /// Description
    pub description: String,
    /// Session ID
    pub session_id: Option<String>,
}

impl AuditEntry {
    /// Create new audit entry
    pub fn new(
        id: u64,
        event_type: AuditEventType,
        severity: AuditSeverity,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id,
            timestamp: chrono::Utc::now(),
            event_type,
            severity,
            category: None,
            field: None,
            old_value: None,
            new_value: None,
            source: "user".to_string(),
            description: description.into(),
            session_id: None,
        }
    }

    /// Set category
    pub fn with_category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set field
    pub fn with_field(mut self, field: impl Into<String>) -> Self {
        self.field = Some(field.into());
        self
    }

    /// Set old value
    pub fn with_old_value(mut self, value: impl Into<String>) -> Self {
        self.old_value = Some(value.into());
        self
    }

    /// Set new value
    pub fn with_new_value(mut self, value: impl Into<String>) -> Self {
        self.new_value = Some(value.into());
        self
    }

    /// Set source
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = source.into();
        self
    }

    /// Set session ID
    pub fn with_session(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    /// Is security event?
    pub fn is_security_event(&self) -> bool {
        self.severity >= AuditSeverity::Security
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_event_type_display() {
        assert_eq!(format!("{}", AuditEventType::Change), "Change");
        assert_eq!(format!("{}", AuditEventType::Save), "Save");
    }

    #[test]
    fn test_audit_severity_display() {
        assert_eq!(format!("{}", AuditSeverity::Info), "Info");
        assert_eq!(format!("{}", AuditSeverity::Security), "Security");
    }

    #[test]
    fn test_audit_entry_new() {
        let entry = AuditEntry::new(1, AuditEventType::Change, AuditSeverity::Info, "Test");
        assert_eq!(entry.id, 1);
        assert_eq!(entry.event_type, AuditEventType::Change);
    }

    #[test]
    fn test_audit_entry_builder() {
        let entry = AuditEntry::new(1, AuditEventType::Change, AuditSeverity::Info, "Test")
            .with_category(SettingsCategory::Risk)
            .with_field("level")
            .with_old_value("low")
            .with_new_value("high");
        assert_eq!(entry.category, Some(SettingsCategory::Risk));
        assert_eq!(entry.field, Some("level".to_string()));
    }

    #[test]
    fn test_audit_entry_is_security() {
        let entry = AuditEntry::new(1, AuditEventType::AccessDenied, AuditSeverity::Security, "Test");
        assert!(entry.is_security_event());
    }
}
