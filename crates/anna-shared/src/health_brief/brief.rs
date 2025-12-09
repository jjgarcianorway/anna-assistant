//! HealthBrief struct and methods (v0.0.207).

use serde::{Deserialize, Serialize};

use super::severity::{disk_severity, memory_severity};
use super::types::{BriefItem, BriefItemKind, BriefSeverity};

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
                BriefItem::new(
                    BriefItemKind::DiskSpace,
                    severity,
                    msg,
                    format!("{}%", use_percent),
                )
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
                BriefItem::new(
                    BriefItemKind::Memory,
                    severity,
                    msg,
                    format!("{}%", used_percent),
                )
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
                BriefItem::new(
                    BriefItemKind::CpuUsage,
                    severity,
                    msg,
                    format!("{:.1}%", cpu_percent),
                )
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
            let errors = self
                .items
                .iter()
                .filter(|i| i.severity == BriefSeverity::Error)
                .count();
            let warnings = self
                .items
                .iter()
                .filter(|i| i.severity == BriefSeverity::Warning)
                .count();

            self.summary = match (errors, warnings) {
                (0, w) => format!("{} warning{} found.", w, if w == 1 { "" } else { "s" }),
                (e, 0) => format!(
                    "{} critical issue{} found.",
                    e,
                    if e == 1 { "" } else { "s" }
                ),
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
