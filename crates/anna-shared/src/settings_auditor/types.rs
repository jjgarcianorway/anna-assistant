// v0.0.691: Settings Auditor (Phase 267)
// Audit types and configuration

use serde::{Deserialize, Serialize};

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
