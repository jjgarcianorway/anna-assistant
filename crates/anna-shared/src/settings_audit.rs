// v0.0.573: Settings Audit (Phase 149)
// Track and report settings changes for compliance and security

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

/// Audit log filter
#[derive(Debug, Clone, Default)]
pub struct AuditFilter {
    /// Filter by event types
    pub event_types: Vec<AuditEventType>,
    /// Filter by severity
    pub min_severity: Option<AuditSeverity>,
    /// Filter by category
    pub category: Option<SettingsCategory>,
    /// Filter by source
    pub source: Option<String>,
    /// Filter from timestamp
    pub from: Option<chrono::DateTime<chrono::Utc>>,
    /// Filter to timestamp
    pub to: Option<chrono::DateTime<chrono::Utc>>,
}

impl AuditFilter {
    /// Create new filter
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by event type
    pub fn event_type(mut self, event_type: AuditEventType) -> Self {
        self.event_types.push(event_type);
        self
    }

    /// Filter by minimum severity
    pub fn severity(mut self, severity: AuditSeverity) -> Self {
        self.min_severity = Some(severity);
        self
    }

    /// Filter by category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Check if entry matches filter
    pub fn matches(&self, entry: &AuditEntry) -> bool {
        // Check event types
        if !self.event_types.is_empty() && !self.event_types.contains(&entry.event_type) {
            return false;
        }

        // Check severity
        if let Some(min) = self.min_severity {
            if entry.severity < min {
                return false;
            }
        }

        // Check category
        if let Some(cat) = self.category {
            if entry.category != Some(cat) {
                return false;
            }
        }

        // Check source
        if let Some(ref src) = self.source {
            if entry.source != *src {
                return false;
            }
        }

        // Check time range
        if let Some(from) = self.from {
            if entry.timestamp < from {
                return false;
            }
        }
        if let Some(to) = self.to {
            if entry.timestamp > to {
                return false;
            }
        }

        true
    }
}

/// Audit log
#[derive(Debug, Clone, Default)]
pub struct AuditLog {
    /// Log entries
    entries: Vec<AuditEntry>,
    /// Next ID
    next_id: u64,
    /// Max entries
    max_entries: usize,
    /// Current session ID
    session_id: Option<String>,
}

impl AuditLog {
    /// Create new audit log
    pub fn new() -> Self {
        Self {
            max_entries: 10000,
            ..Default::default()
        }
    }

    /// Set session ID
    pub fn set_session(&mut self, session_id: impl Into<String>) {
        self.session_id = Some(session_id.into());
    }

    /// Log an event
    pub fn log(&mut self, event_type: AuditEventType, severity: AuditSeverity, description: &str) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let mut entry = AuditEntry::new(id, event_type, severity, description);
        if let Some(ref session) = self.session_id {
            entry.session_id = Some(session.clone());
        }

        self.entries.push(entry);
        self.trim();
        id
    }

    /// Log a change
    pub fn log_change(
        &mut self,
        category: SettingsCategory,
        field: &str,
        old_value: &str,
        new_value: &str,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let mut entry = AuditEntry::new(
            id,
            AuditEventType::Change,
            AuditSeverity::Info,
            &format!("Changed {}.{}", category, field),
        )
        .with_category(category)
        .with_field(field)
        .with_old_value(old_value)
        .with_new_value(new_value);

        if let Some(ref session) = self.session_id {
            entry.session_id = Some(session.clone());
        }

        self.entries.push(entry);
        self.trim();
        id
    }

    /// Log security event
    pub fn log_security(&mut self, description: &str) -> u64 {
        self.log(AuditEventType::AccessDenied, AuditSeverity::Security, description)
    }

    /// Trim to max size
    fn trim(&mut self) {
        while self.entries.len() > self.max_entries {
            self.entries.remove(0);
        }
    }

    /// Get all entries
    pub fn all(&self) -> &[AuditEntry] {
        &self.entries
    }

    /// Get filtered entries
    pub fn filter(&self, filter: &AuditFilter) -> Vec<&AuditEntry> {
        self.entries.iter().filter(|e| filter.matches(e)).collect()
    }

    /// Get recent entries
    pub fn recent(&self, count: usize) -> Vec<&AuditEntry> {
        self.entries.iter().rev().take(count).collect()
    }

    /// Get security events
    pub fn security_events(&self) -> Vec<&AuditEntry> {
        self.entries.iter().filter(|e| e.is_security_event()).collect()
    }

    /// Get changes for a category
    pub fn changes_for(&self, category: SettingsCategory) -> Vec<&AuditEntry> {
        self.entries
            .iter()
            .filter(|e| e.event_type == AuditEventType::Change && e.category == Some(category))
            .collect()
    }

    /// Count by severity
    pub fn count_by_severity(&self, severity: AuditSeverity) -> usize {
        self.entries.iter().filter(|e| e.severity == severity).count()
    }

    /// Get entry by ID
    pub fn get(&self, id: u64) -> Option<&AuditEntry> {
        self.entries.iter().find(|e| e.id == id)
    }

    /// Count entries
    pub fn count(&self) -> usize {
        self.entries.len()
    }

    /// Clear log
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

/// Format audit log for display
pub fn format_audit_log(log: &AuditLog, limit: usize) -> String {
    let mut output = String::new();

    output.push_str("=== Settings Audit Log ===\n\n");
    output.push_str(&format!("Total entries: {}\n", log.count()));
    output.push_str(&format!("Security events: {}\n\n", log.security_events().len()));

    let recent = log.recent(limit);
    if recent.is_empty() {
        output.push_str("No entries.\n");
        return output;
    }

    output.push_str("Recent Activity:\n");
    for entry in recent {
        let time = entry.timestamp.format("%H:%M:%S");
        output.push_str(&format!(
            "[{}] {} [{}] {}\n",
            time, entry.event_type, entry.severity, entry.description
        ));
    }

    output
}

/// Check if query is about audit
pub fn is_audit_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("audit")
        || lower.contains("settings log")
        || lower.contains("change history")
        || lower.contains("who changed")
}

/// Fun fact about audit
pub fn audit_fun_fact() -> &'static str {
    "The audit log tracks all settings changes for compliance and security!"
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

    #[test]
    fn test_audit_filter_new() {
        let filter = AuditFilter::new();
        assert!(filter.event_types.is_empty());
    }

    #[test]
    fn test_audit_filter_matches() {
        let filter = AuditFilter::new().severity(AuditSeverity::Warning);
        let entry = AuditEntry::new(1, AuditEventType::Change, AuditSeverity::Warning, "Test");
        assert!(filter.matches(&entry));
    }

    #[test]
    fn test_audit_log_new() {
        let log = AuditLog::new();
        assert_eq!(log.count(), 0);
    }

    #[test]
    fn test_audit_log_log() {
        let mut log = AuditLog::new();
        let id = log.log(AuditEventType::Save, AuditSeverity::Info, "Saved settings");
        assert!(log.get(id).is_some());
    }

    #[test]
    fn test_audit_log_log_change() {
        let mut log = AuditLog::new();
        let id = log.log_change(SettingsCategory::Risk, "level", "low", "high");
        let entry = log.get(id).unwrap();
        assert_eq!(entry.event_type, AuditEventType::Change);
        assert_eq!(entry.old_value, Some("low".to_string()));
    }

    #[test]
    fn test_audit_log_security_events() {
        let mut log = AuditLog::new();
        log.log_security("Unauthorized access");
        assert_eq!(log.security_events().len(), 1);
    }

    #[test]
    fn test_audit_log_filter() {
        let mut log = AuditLog::new();
        log.log(AuditEventType::Save, AuditSeverity::Info, "Test");
        log.log(AuditEventType::Change, AuditSeverity::Warning, "Test");
        let filter = AuditFilter::new().event_type(AuditEventType::Change);
        assert_eq!(log.filter(&filter).len(), 1);
    }

    #[test]
    fn test_format_audit_log() {
        let log = AuditLog::new();
        let output = format_audit_log(&log, 10);
        assert!(output.contains("Audit"));
    }

    #[test]
    fn test_is_audit_query() {
        assert!(is_audit_query("show audit log"));
        assert!(is_audit_query("change history"));
        assert!(!is_audit_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = audit_fun_fact();
        assert!(fact.contains("audit"));
    }
}
