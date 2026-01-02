// v0.0.691: Settings Auditor (Phase 267)
// Settings auditor and statistics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::event::{AuditEvent, AuditTrail};
use super::types::{AuditEventType, AuditorConfig, AuditSeverity};

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
