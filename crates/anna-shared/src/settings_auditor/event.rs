// v0.0.691: Settings Auditor (Phase 267)
// Audit events and trails

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::{AuditEventType, AuditSeverity};

/// Audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Event ID
    pub id: usize,
    /// Event type
    pub event_type: AuditEventType,
    /// Key
    pub key: String,
    /// Old value
    pub old_value: Option<String>,
    /// New value
    pub new_value: Option<String>,
    /// Timestamp (seconds)
    pub timestamp: u64,
    /// Severity
    pub severity: AuditSeverity,
}

impl AuditEvent {
    /// Create new event
    pub fn new(id: usize, event_type: AuditEventType, key: impl Into<String>) -> Self {
        Self {
            id,
            event_type,
            key: key.into(),
            old_value: None,
            new_value: None,
            timestamp: 0,
            severity: AuditSeverity::Low,
        }
    }

    /// Set old value
    pub fn old_value(mut self, val: impl Into<String>) -> Self {
        self.old_value = Some(val.into());
        self
    }

    /// Set new value
    pub fn new_value(mut self, val: impl Into<String>) -> Self {
        self.new_value = Some(val.into());
        self
    }

    /// Set severity
    pub fn severity(mut self, sev: AuditSeverity) -> Self {
        self.severity = sev;
        self
    }

    /// Is write event
    pub fn is_write(&self) -> bool {
        matches!(self.event_type, AuditEventType::Write | AuditEventType::Create | AuditEventType::Delete)
    }
}

/// Audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrail {
    /// Events
    pub events: Vec<AuditEvent>,
    /// Total events
    pub total_events: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl AuditTrail {
    /// Create new trail
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            total_events: 0,
            by_type: HashMap::new(),
        }
    }

    /// Add event
    pub fn add(&mut self, event: AuditEvent) {
        *self.by_type.entry(event.event_type.to_string()).or_insert(0) += 1;
        self.total_events += 1;
        self.events.push(event);
    }

    /// Filter by type
    pub fn filter_by_type(&self, event_type: AuditEventType) -> Vec<&AuditEvent> {
        self.events.iter().filter(|e| e.event_type == event_type).collect()
    }

    /// Filter by key
    pub fn filter_by_key(&self, key: &str) -> Vec<&AuditEvent> {
        self.events.iter().filter(|e| e.key == key).collect()
    }
}

impl Default for AuditTrail {
    fn default() -> Self {
        Self::new()
    }
}
