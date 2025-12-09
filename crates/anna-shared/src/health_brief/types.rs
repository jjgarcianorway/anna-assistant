//! Type definitions for health brief module (v0.0.207).

use serde::{Deserialize, Serialize};

/// Severity level for health brief items
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum BriefSeverity {
    /// Everything is fine
    #[default]
    Ok,
    /// Needs attention but not critical
    Warning,
    /// Critical issue requiring action
    Error,
}

impl std::fmt::Display for BriefSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BriefSeverity::Ok => write!(f, "OK"),
            BriefSeverity::Warning => write!(f, "Warning"),
            BriefSeverity::Error => write!(f, "Error"),
        }
    }
}

/// Category of health check item
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BriefItemKind {
    DiskSpace,
    Memory,
    CpuUsage,
    Service,
    SwapUsage,
    LoadAverage,
}

impl std::fmt::Display for BriefItemKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BriefItemKind::DiskSpace => write!(f, "Disk"),
            BriefItemKind::Memory => write!(f, "Memory"),
            BriefItemKind::CpuUsage => write!(f, "CPU"),
            BriefItemKind::Service => write!(f, "Service"),
            BriefItemKind::SwapUsage => write!(f, "Swap"),
            BriefItemKind::LoadAverage => write!(f, "Load"),
        }
    }
}

/// A single item in the health brief
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BriefItem {
    /// What kind of check this is
    pub kind: BriefItemKind,
    /// Severity of this item
    pub severity: BriefSeverity,
    /// Human-readable message
    pub message: String,
    /// Current value (e.g., "92%")
    pub value: String,
    /// Threshold that triggered this (e.g., "85%")
    pub threshold: Option<String>,
    /// Additional context (e.g., mount point, service name)
    pub context: Option<String>,
}

impl BriefItem {
    /// Create a new brief item
    pub fn new(
        kind: BriefItemKind,
        severity: BriefSeverity,
        message: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            severity,
            message: message.into(),
            value: value.into(),
            threshold: None,
            context: None,
        }
    }

    /// Add threshold info
    pub fn with_threshold(mut self, threshold: impl Into<String>) -> Self {
        self.threshold = Some(threshold.into());
        self
    }

    /// Add context info
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Format for display
    pub fn format_line(&self) -> String {
        let icon = match self.severity {
            BriefSeverity::Ok => "✓",
            BriefSeverity::Warning => "⚠",
            BriefSeverity::Error => "✗",
        };
        match &self.context {
            Some(ctx) => format!("{} {} {} ({})", icon, self.kind, self.message, ctx),
            None => format!("{} {} {}", icon, self.kind, self.message),
        }
    }
}
