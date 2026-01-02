// v0.0.573: Settings Audit - Log Management
// Main audit log structure and operations

use crate::unified_settings::SettingsCategory;

use super::filter::AuditFilter;
use super::types::{AuditEntry, AuditEventType, AuditSeverity};

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
