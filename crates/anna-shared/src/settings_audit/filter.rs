// v0.0.573: Settings Audit - Filtering
// Filter audit log entries by various criteria

use crate::unified_settings::SettingsCategory;

use super::types::{AuditEntry, AuditEventType, AuditSeverity};

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
