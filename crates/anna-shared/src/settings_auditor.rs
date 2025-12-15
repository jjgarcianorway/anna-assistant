// v0.0.691: Settings Auditor (Phase 267)
// Audit settings changes and access

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Audit event type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AuditEventType {
    /// Read access
    #[default]
    Read,
    /// Write access
    Write,
    /// Delete access
    Delete,
    /// Create access
    Create,
}

impl std::fmt::Display for AuditEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Read => write!(f, "read"),
            Self::Write => write!(f, "write"),
            Self::Delete => write!(f, "delete"),
            Self::Create => write!(f, "create"),
        }
    }
}

/// Audit severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AuditSeverity {
    /// Low severity
    #[default]
    Low,
    /// Medium severity
    Medium,
    /// High severity
    High,
    /// Critical severity
    Critical,
}

impl std::fmt::Display for AuditSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

/// Auditor config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditorConfig {
    /// Enable auditing
    pub enabled: bool,
    /// Log reads
    pub log_reads: bool,
    /// Log writes
    pub log_writes: bool,
    /// Max events
    pub max_events: usize,
}

impl AuditorConfig {
    /// Create new config
    pub fn new() -> Self {
        Self {
            enabled: true,
            log_reads: false,
            log_writes: true,
            max_events: 1000,
        }
    }

    /// Set log reads
    pub fn log_reads(mut self, log: bool) -> Self {
        self.log_reads = log;
        self
    }

    /// Set log writes
    pub fn log_writes(mut self, log: bool) -> Self {
        self.log_writes = log;
        self
    }

    /// Set max events
    pub fn max_events(mut self, max: usize) -> Self {
        self.max_events = max;
        self
    }
}

impl Default for AuditorConfig {
    fn default() -> Self {
        Self::new()
    }
}

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

/// Auditor stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditorStats {
    /// Total events
    pub total_events: usize,
    /// Read events
    pub read_events: usize,
    /// Write events
    pub write_events: usize,
    /// By severity
    pub by_severity: HashMap<String, usize>,
}

impl AuditorStats {
    /// Record event
    pub fn record(&mut self, event: &AuditEvent) {
        self.total_events += 1;
        if event.is_write() {
            self.write_events += 1;
        } else {
            self.read_events += 1;
        }
        *self.by_severity.entry(event.severity.to_string()).or_insert(0) += 1;
    }

    /// Write ratio
    pub fn write_ratio(&self) -> f64 {
        if self.total_events == 0 {
            0.0
        } else {
            self.write_events as f64 / self.total_events as f64
        }
    }
}

/// Settings auditor
#[derive(Debug, Clone, Default)]
pub struct SettingsAuditor {
    /// Config
    config: AuditorConfig,
    /// Trail
    trail: AuditTrail,
    /// Stats
    stats: AuditorStats,
    /// Next ID
    next_id: usize,
}

impl SettingsAuditor {
    /// Create new auditor
    pub fn new(config: AuditorConfig) -> Self {
        Self {
            config,
            trail: AuditTrail::new(),
            stats: AuditorStats::default(),
            next_id: 1,
        }
    }

    /// Log read
    pub fn log_read(&mut self, key: &str) {
        if !self.config.enabled || !self.config.log_reads {
            return;
        }
        let event = AuditEvent::new(self.next_id, AuditEventType::Read, key);
        self.next_id += 1;
        self.stats.record(&event);
        self.trail.add(event);
        self.trim_events();
    }

    /// Log write
    pub fn log_write(&mut self, key: &str, old: Option<&str>, new: Option<&str>) {
        if !self.config.enabled || !self.config.log_writes {
            return;
        }
        let mut event = AuditEvent::new(self.next_id, AuditEventType::Write, key);
        if let Some(o) = old {
            event = event.old_value(o);
        }
        if let Some(n) = new {
            event = event.new_value(n);
        }
        self.next_id += 1;
        self.stats.record(&event);
        self.trail.add(event);
        self.trim_events();
    }

    /// Log create
    pub fn log_create(&mut self, key: &str, value: &str) {
        if !self.config.enabled || !self.config.log_writes {
            return;
        }
        let event = AuditEvent::new(self.next_id, AuditEventType::Create, key)
            .new_value(value);
        self.next_id += 1;
        self.stats.record(&event);
        self.trail.add(event);
        self.trim_events();
    }

    /// Log delete
    pub fn log_delete(&mut self, key: &str, old_value: &str) {
        if !self.config.enabled || !self.config.log_writes {
            return;
        }
        let event = AuditEvent::new(self.next_id, AuditEventType::Delete, key)
            .old_value(old_value)
            .severity(AuditSeverity::Medium);
        self.next_id += 1;
        self.stats.record(&event);
        self.trail.add(event);
        self.trim_events();
    }

    /// Trim events to max
    fn trim_events(&mut self) {
        while self.trail.events.len() > self.config.max_events {
            self.trail.events.remove(0);
        }
    }

    /// Get trail
    pub fn trail(&self) -> &AuditTrail {
        &self.trail
    }

    /// Get stats
    pub fn stats(&self) -> &AuditorStats {
        &self.stats
    }
}

/// Auditor registry
#[derive(Debug, Clone, Default)]
pub struct AuditorRegistry {
    /// Auditors by ID
    auditors: HashMap<String, SettingsAuditor>,
}

impl AuditorRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register auditor
    pub fn register(&mut self, id: impl Into<String>, auditor: SettingsAuditor) {
        self.auditors.insert(id.into(), auditor);
    }

    /// Unregister auditor
    pub fn unregister(&mut self, id: &str) -> bool {
        self.auditors.remove(id).is_some()
    }

    /// Get auditor
    pub fn get(&self, id: &str) -> Option<&SettingsAuditor> {
        self.auditors.get(id)
    }

    /// Get auditor mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsAuditor> {
        self.auditors.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.auditors.len()
    }
}

/// Format auditor registry
pub fn format_auditor_registry(registry: &AuditorRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Auditor Registry:\n");
    output.push_str(&format!("  Auditors: {}\n", registry.count()));
    output
}

/// Check if query is about auditor
pub fn is_auditor_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("audit settings") || lower.contains("settings audit") || lower.contains("settings history")
}

/// Fun fact about auditor
pub fn auditor_fun_fact() -> &'static str {
    "Anna's settings auditor tracks every change for complete accountability!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_display() {
        assert_eq!(format!("{}", AuditEventType::Read), "read");
        assert_eq!(format!("{}", AuditEventType::Write), "write");
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(format!("{}", AuditSeverity::High), "high");
        assert_eq!(format!("{}", AuditSeverity::Critical), "critical");
    }

    #[test]
    fn test_config_new() {
        let c = AuditorConfig::new();
        assert!(c.enabled);
    }

    #[test]
    fn test_config_builder() {
        let c = AuditorConfig::new()
            .log_reads(true)
            .max_events(500);
        assert!(c.log_reads);
        assert_eq!(c.max_events, 500);
    }

    #[test]
    fn test_event_new() {
        let e = AuditEvent::new(1, AuditEventType::Write, "key");
        assert!(e.is_write());
    }

    #[test]
    fn test_event_values() {
        let e = AuditEvent::new(1, AuditEventType::Write, "key")
            .old_value("old")
            .new_value("new");
        assert_eq!(e.old_value, Some("old".to_string()));
        assert_eq!(e.new_value, Some("new".to_string()));
    }

    #[test]
    fn test_trail_new() {
        let t = AuditTrail::new();
        assert_eq!(t.total_events, 0);
    }

    #[test]
    fn test_trail_add() {
        let mut t = AuditTrail::new();
        t.add(AuditEvent::new(1, AuditEventType::Write, "key"));
        assert_eq!(t.total_events, 1);
    }

    #[test]
    fn test_stats_record() {
        let mut s = AuditorStats::default();
        s.record(&AuditEvent::new(1, AuditEventType::Write, "key"));
        assert_eq!(s.write_events, 1);
    }

    #[test]
    fn test_auditor_new() {
        let a = SettingsAuditor::new(AuditorConfig::default());
        assert_eq!(a.stats().total_events, 0);
    }

    #[test]
    fn test_auditor_log_write() {
        let mut a = SettingsAuditor::new(AuditorConfig::default());
        a.log_write("key", Some("old"), Some("new"));
        assert_eq!(a.stats().write_events, 1);
    }

    #[test]
    fn test_auditor_log_create() {
        let mut a = SettingsAuditor::new(AuditorConfig::default());
        a.log_create("key", "value");
        assert_eq!(a.stats().write_events, 1);
    }

    #[test]
    fn test_auditor_log_delete() {
        let mut a = SettingsAuditor::new(AuditorConfig::default());
        a.log_delete("key", "old_value");
        assert_eq!(a.stats().write_events, 1);
    }

    #[test]
    fn test_auditor_disabled_reads() {
        let mut a = SettingsAuditor::new(AuditorConfig::default());
        a.log_read("key");
        assert_eq!(a.stats().read_events, 0); // log_reads is false by default
    }

    #[test]
    fn test_registry_new() {
        let r = AuditorRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = AuditorRegistry::new();
        r.register("a1", SettingsAuditor::new(AuditorConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_auditor_query() {
        assert!(is_auditor_query("audit settings"));
        assert!(!is_auditor_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = auditor_fun_fact();
        assert!(fact.contains("auditor"));
    }
}
