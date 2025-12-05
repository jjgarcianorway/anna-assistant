//! Health Brief - Relevant-only health status for "how is my computer" queries (v0.0.32).
//!
//! Unlike full system reports, HealthBrief only shows actionable items:
//! - Disk warnings (>85%)
//! - Memory pressure (>90%)
//! - Failed/degraded services
//! - High CPU consumers
//!
//! If everything is OK, the brief simply says "all good".

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

/// Health brief with only actionable items
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HealthBrief {
    /// Items that need attention (warnings and errors only)
    pub items: Vec<BriefItem>,
    /// Overall status
    pub overall: BriefSeverity,
    /// Quick summary message
    pub summary: String,
    /// Whether all systems are healthy
    pub all_healthy: bool,
}

impl HealthBrief {
    /// Create a new health brief
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            overall: BriefSeverity::Ok,
            summary: String::new(),
            all_healthy: true,
        }
    }

    /// Add an item (only if Warning or Error)
    pub fn add_item(&mut self, item: BriefItem) {
        if item.severity != BriefSeverity::Ok {
            // Update overall severity
            if item.severity > self.overall {
                self.overall = item.severity;
            }
            self.all_healthy = false;
            self.items.push(item);
        }
    }

    /// Add disk space item
    pub fn add_disk(&mut self, mount: &str, use_percent: u8, avail: &str) {
        let severity = disk_severity(use_percent);
        if severity != BriefSeverity::Ok {
            let msg = format!("{}% used, {} available", use_percent, avail);
            self.add_item(
                BriefItem::new(BriefItemKind::DiskSpace, severity, msg, format!("{}%", use_percent))
                    .with_threshold(if use_percent >= 95 { "95%" } else { "85%" })
                    .with_context(mount),
            );
        }
    }

    /// Add memory pressure item
    pub fn add_memory(&mut self, used_percent: u8, available: &str) {
        let severity = memory_severity(used_percent);
        if severity != BriefSeverity::Ok {
            let msg = format!("{}% used, {} available", used_percent, available);
            self.add_item(
                BriefItem::new(BriefItemKind::Memory, severity, msg, format!("{}%", used_percent))
                    .with_threshold(if used_percent >= 95 { "95%" } else { "90%" }),
            );
        }
    }

    /// Add failed service item
    pub fn add_failed_service(&mut self, service: &str) {
        self.add_item(
            BriefItem::new(
                BriefItemKind::Service,
                BriefSeverity::Error,
                "failed",
                "failed",
            )
            .with_context(service),
        );
    }

    /// Add high CPU process item
    pub fn add_high_cpu(&mut self, process: &str, cpu_percent: f32) {
        if cpu_percent >= 80.0 {
            let severity = if cpu_percent >= 95.0 {
                BriefSeverity::Error
            } else {
                BriefSeverity::Warning
            };
            let msg = format!("using {:.1}% CPU", cpu_percent);
            self.add_item(
                BriefItem::new(BriefItemKind::CpuUsage, severity, msg, format!("{:.1}%", cpu_percent))
                    .with_threshold("80%")
                    .with_context(process),
            );
        }
    }

    /// Finalize the brief and generate summary
    pub fn finalize(&mut self) {
        // Sort by severity (errors first)
        self.items.sort_by(|a, b| b.severity.cmp(&a.severity));

        // Generate summary
        if self.all_healthy {
            self.summary = "Your system is healthy. No issues detected.".to_string();
        } else {
            let errors = self.items.iter().filter(|i| i.severity == BriefSeverity::Error).count();
            let warnings = self.items.iter().filter(|i| i.severity == BriefSeverity::Warning).count();

            self.summary = match (errors, warnings) {
                (0, w) => format!("{} warning{} found.", w, if w == 1 { "" } else { "s" }),
                (e, 0) => format!("{} critical issue{} found.", e, if e == 1 { "" } else { "s" }),
                (e, w) => format!(
                    "{} critical issue{} and {} warning{} found.",
                    e,
                    if e == 1 { "" } else { "s" },
                    w,
                    if w == 1 { "" } else { "s" }
                ),
            };
        }
    }

    /// Format as answer text
    pub fn format_answer(&self) -> String {
        if self.all_healthy {
            return self.summary.clone();
        }

        let mut answer = format!("**Health Status: {}**\n\n", self.summary);

        // Group by kind
        for item in &self.items {
            answer.push_str(&format!("- {}\n", item.format_line()));
        }

        answer
    }

    /// Format as markdown table
    pub fn format_table(&self) -> String {
        if self.all_healthy {
            return self.summary.clone();
        }

        let mut table = String::from("| Status | Category | Issue | Details |\n");
        table.push_str("|--------|----------|-------|----------|\n");

        for item in &self.items {
            let icon = match item.severity {
                BriefSeverity::Ok => "✓",
                BriefSeverity::Warning => "⚠️",
                BriefSeverity::Error => "❌",
            };
            let ctx = item.context.as_deref().unwrap_or("-");
            table.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                icon, item.kind, item.message, ctx
            ));
        }

        table
    }
}

/// Determine disk severity based on usage percent
pub fn disk_severity(use_percent: u8) -> BriefSeverity {
    if use_percent >= 95 {
        BriefSeverity::Error
    } else if use_percent >= 85 {
        BriefSeverity::Warning
    } else {
        BriefSeverity::Ok
    }
}

/// Determine memory severity based on usage percent
pub fn memory_severity(used_percent: u8) -> BriefSeverity {
    if used_percent >= 95 {
        BriefSeverity::Error
    } else if used_percent >= 90 {
        BriefSeverity::Warning
    } else {
        BriefSeverity::Ok
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_healthy() {
        let mut brief = HealthBrief::new();
        brief.add_disk("/", 50, "50G");
        brief.add_disk("/home", 70, "30G");
        brief.finalize();

        assert!(brief.all_healthy);
        assert_eq!(brief.items.len(), 0);
        assert!(brief.summary.contains("healthy"));
    }

    #[test]
    fn test_disk_warning() {
        let mut brief = HealthBrief::new();
        brief.add_disk("/", 87, "13G");
        brief.finalize();

        assert!(!brief.all_healthy);
        assert_eq!(brief.items.len(), 1);
        assert_eq!(brief.items[0].severity, BriefSeverity::Warning);
        assert!(brief.items[0].context.as_ref().unwrap().contains("/"));
    }

    #[test]
    fn test_disk_critical() {
        let mut brief = HealthBrief::new();
        brief.add_disk("/", 96, "4G");
        brief.finalize();

        assert!(!brief.all_healthy);
        assert_eq!(brief.overall, BriefSeverity::Error);
        assert_eq!(brief.items[0].severity, BriefSeverity::Error);
    }

    #[test]
    fn test_failed_service() {
        let mut brief = HealthBrief::new();
        brief.add_failed_service("nginx.service");
        brief.finalize();

        assert!(!brief.all_healthy);
        assert_eq!(brief.overall, BriefSeverity::Error);
        assert!(brief.items[0].context.as_ref().unwrap().contains("nginx"));
    }

    #[test]
    fn test_high_cpu() {
        let mut brief = HealthBrief::new();
        brief.add_high_cpu("firefox", 85.5);
        brief.finalize();

        assert!(!brief.all_healthy);
        assert_eq!(brief.items[0].severity, BriefSeverity::Warning);
    }

    #[test]
    fn test_format_answer_healthy() {
        let mut brief = HealthBrief::new();
        brief.finalize();

        let answer = brief.format_answer();
        assert!(answer.contains("healthy"));
    }

    #[test]
    fn test_format_answer_with_issues() {
        let mut brief = HealthBrief::new();
        brief.add_disk("/home", 96, "4G");
        brief.add_failed_service("docker.service");
        brief.finalize();

        let answer = brief.format_answer();
        assert!(answer.contains("critical"));
        assert!(answer.contains("/home"));
        assert!(answer.contains("docker"));
    }

    #[test]
    fn test_severity_ordering() {
        let mut brief = HealthBrief::new();
        brief.add_disk("/tmp", 87, "13G"); // Warning
        brief.add_disk("/", 96, "4G"); // Error
        brief.add_disk("/var", 88, "12G"); // Warning
        brief.finalize();

        // Errors should come first
        assert_eq!(brief.items[0].severity, BriefSeverity::Error);
        assert_eq!(brief.items[1].severity, BriefSeverity::Warning);
        assert_eq!(brief.items[2].severity, BriefSeverity::Warning);
    }

    // Golden tests
    #[test]
    fn golden_healthy_summary() {
        let mut brief = HealthBrief::new();
        brief.finalize();
        assert_eq!(brief.summary, "Your system is healthy. No issues detected.");
    }

    #[test]
    fn golden_single_warning() {
        let mut brief = HealthBrief::new();
        brief.add_disk("/", 87, "13G");
        brief.finalize();
        assert_eq!(brief.summary, "1 warning found.");
    }

    #[test]
    fn golden_multiple_issues() {
        let mut brief = HealthBrief::new();
        brief.add_disk("/", 96, "4G");
        brief.add_disk("/home", 88, "12G");
        brief.add_failed_service("nginx");
        brief.finalize();
        assert_eq!(brief.summary, "2 critical issues and 1 warning found.");
    }
}
