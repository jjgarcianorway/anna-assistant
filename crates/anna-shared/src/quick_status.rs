//! Quick Status Summary (v0.0.484).
//!
//! Provides at-a-glance system status summaries.
//! Designed for quick checks without full status display.

use serde::{Deserialize, Serialize};

/// Quick status health level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthLevel {
    /// Everything is fine
    Good,
    /// Minor issues, not critical
    Warning,
    /// Critical issues need attention
    Critical,
    /// Status unknown
    Unknown,
}

impl HealthLevel {
    /// Get display symbol
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Good => "[OK]",
            Self::Warning => "[!]",
            Self::Critical => "[X]",
            Self::Unknown => "[?]",
        }
    }

    /// Get color hint for display
    pub fn color_hint(&self) -> &'static str {
        match self {
            Self::Good => "green",
            Self::Warning => "yellow",
            Self::Critical => "red",
            Self::Unknown => "gray",
        }
    }
}

/// Individual status item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusItem {
    /// Name of the item
    pub name: String,
    /// Health level
    pub health: HealthLevel,
    /// Brief status message
    pub message: String,
    /// Optional value (e.g., "85%")
    pub value: Option<String>,
}

impl StatusItem {
    /// Create a new status item
    pub fn new(name: &str, health: HealthLevel, message: &str) -> Self {
        Self {
            name: name.to_string(),
            health,
            message: message.to_string(),
            value: None,
        }
    }

    /// Add a value
    pub fn with_value(mut self, value: &str) -> Self {
        self.value = Some(value.to_string());
        self
    }

    /// Format for display
    pub fn format(&self) -> String {
        let value_str = self
            .value
            .as_ref()
            .map(|v| format!(" ({})", v))
            .unwrap_or_default();

        format!(
            "{} {}: {}{}",
            self.health.symbol(),
            self.name,
            self.message,
            value_str
        )
    }
}

/// Quick status summary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuickStatus {
    /// Individual status items
    pub items: Vec<StatusItem>,
    /// Overall health (worst of all items)
    pub overall: HealthLevel,
    /// Summary message
    pub summary: String,
}

impl Default for HealthLevel {
    fn default() -> Self {
        Self::Unknown
    }
}

impl QuickStatus {
    /// Create empty status
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a status item
    pub fn add(&mut self, item: StatusItem) {
        // Update overall health to worst level
        self.overall = match (&self.overall, &item.health) {
            (_, HealthLevel::Critical) => HealthLevel::Critical,
            (HealthLevel::Critical, _) => HealthLevel::Critical,
            (_, HealthLevel::Warning) => HealthLevel::Warning,
            (HealthLevel::Warning, _) => HealthLevel::Warning,
            (HealthLevel::Unknown, HealthLevel::Good) => HealthLevel::Good,
            (HealthLevel::Good, _) => HealthLevel::Good,
            _ => self.overall,
        };

        self.items.push(item);
    }

    /// Add multiple items
    pub fn add_all(&mut self, items: Vec<StatusItem>) {
        for item in items {
            self.add(item);
        }
    }

    /// Set summary message
    pub fn set_summary(&mut self, summary: &str) {
        self.summary = summary.to_string();
    }

    /// Generate summary from items
    pub fn generate_summary(&mut self) {
        let critical = self
            .items
            .iter()
            .filter(|i| i.health == HealthLevel::Critical)
            .count();
        let warning = self
            .items
            .iter()
            .filter(|i| i.health == HealthLevel::Warning)
            .count();

        self.summary = if critical > 0 {
            format!("{} critical issue{}", critical, if critical > 1 { "s" } else { "" })
        } else if warning > 0 {
            format!("{} warning{}", warning, if warning > 1 { "s" } else { "" })
        } else {
            "All systems operational".to_string()
        };
    }

    /// Get count of issues by health level
    pub fn count_by_health(&self, health: HealthLevel) -> usize {
        self.items.iter().filter(|i| i.health == health).count()
    }

    /// Check if there are any critical issues
    pub fn has_critical(&self) -> bool {
        self.items.iter().any(|i| i.health == HealthLevel::Critical)
    }

    /// Check if there are any warnings
    pub fn has_warnings(&self) -> bool {
        self.items.iter().any(|i| i.health == HealthLevel::Warning)
    }

    /// Check if everything is good
    pub fn all_good(&self) -> bool {
        self.items.iter().all(|i| i.health == HealthLevel::Good)
    }
}

/// Format quick status for one-line display
pub fn format_quick_status_oneline(status: &QuickStatus) -> String {
    let symbol = status.overall.symbol();
    format!("{} {}", symbol, status.summary)
}

/// Format quick status for compact display
pub fn format_quick_status_compact(status: &QuickStatus) -> String {
    let mut output = String::new();

    output.push_str(&format!("{} {}\n", status.overall.symbol(), status.summary));

    // Only show non-good items in compact mode
    for item in &status.items {
        if item.health != HealthLevel::Good {
            output.push_str(&format!("  {}\n", item.format()));
        }
    }

    output
}

/// Format quick status for full display
pub fn format_quick_status_full(status: &QuickStatus) -> String {
    let mut output = String::new();

    output.push_str("Quick Status\n");
    output.push_str("══════════════════════════════════════\n\n");

    output.push_str(&format!("{} {}\n\n", status.overall.symbol(), status.summary));

    for item in &status.items {
        output.push_str(&format!("  {}\n", item.format()));
    }

    output
}

/// Create status from memory usage percentage
pub fn memory_status(used_percent: f32) -> StatusItem {
    let health = if used_percent > 90.0 {
        HealthLevel::Critical
    } else if used_percent > 75.0 {
        HealthLevel::Warning
    } else {
        HealthLevel::Good
    };

    let message = if used_percent > 90.0 {
        "Memory critically low"
    } else if used_percent > 75.0 {
        "Memory usage high"
    } else {
        "Memory OK"
    };

    StatusItem::new("Memory", health, message).with_value(&format!("{:.0}%", used_percent))
}

/// Create status from disk usage percentage
pub fn disk_status(used_percent: f32, mount_point: &str) -> StatusItem {
    let health = if used_percent > 95.0 {
        HealthLevel::Critical
    } else if used_percent > 85.0 {
        HealthLevel::Warning
    } else {
        HealthLevel::Good
    };

    let message = if used_percent > 95.0 {
        format!("{} almost full", mount_point)
    } else if used_percent > 85.0 {
        format!("{} getting full", mount_point)
    } else {
        format!("{} OK", mount_point)
    };

    StatusItem::new("Disk", health, &message).with_value(&format!("{:.0}%", used_percent))
}

/// Create status from CPU load
pub fn cpu_status(load_1min: f32, core_count: u32) -> StatusItem {
    let load_per_core = load_1min / core_count as f32;

    let health = if load_per_core > 2.0 {
        HealthLevel::Critical
    } else if load_per_core > 1.0 {
        HealthLevel::Warning
    } else {
        HealthLevel::Good
    };

    let message = if load_per_core > 2.0 {
        "CPU overloaded"
    } else if load_per_core > 1.0 {
        "CPU busy"
    } else {
        "CPU OK"
    };

    StatusItem::new("CPU", health, message).with_value(&format!("{:.1}", load_1min))
}

/// Create status from service state
pub fn service_status(name: &str, running: bool, failed: bool) -> StatusItem {
    let health = if failed {
        HealthLevel::Critical
    } else if !running {
        HealthLevel::Warning
    } else {
        HealthLevel::Good
    };

    let message = if failed {
        format!("{} failed", name)
    } else if !running {
        format!("{} stopped", name)
    } else {
        format!("{} running", name)
    };

    StatusItem::new("Service", health, &message)
}

/// Detect if query is asking for quick status
pub fn is_quick_status_query(query: &str) -> bool {
    let lower = query.to_lowercase();

    let patterns = [
        "quick status",
        "quick check",
        "health check",
        "system ok",
        "everything ok",
        "any problems",
        "any issues",
        "status check",
        "how's the system",
        "how is the system",
    ];

    for pattern in patterns {
        if lower.contains(pattern) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_level_symbol() {
        assert_eq!(HealthLevel::Good.symbol(), "[OK]");
        assert_eq!(HealthLevel::Warning.symbol(), "[!]");
        assert_eq!(HealthLevel::Critical.symbol(), "[X]");
        assert_eq!(HealthLevel::Unknown.symbol(), "[?]");
    }

    #[test]
    fn test_status_item_format() {
        let item = StatusItem::new("Test", HealthLevel::Good, "All good");
        assert!(item.format().contains("[OK]"));
        assert!(item.format().contains("Test"));
        assert!(item.format().contains("All good"));

        let item_with_value = item.with_value("100%");
        assert!(item_with_value.format().contains("100%"));
    }

    #[test]
    fn test_quick_status_overall() {
        let mut status = QuickStatus::new();

        status.add(StatusItem::new("A", HealthLevel::Good, "OK"));
        assert_eq!(status.overall, HealthLevel::Good);

        status.add(StatusItem::new("B", HealthLevel::Warning, "Warn"));
        assert_eq!(status.overall, HealthLevel::Warning);

        status.add(StatusItem::new("C", HealthLevel::Critical, "Bad"));
        assert_eq!(status.overall, HealthLevel::Critical);
    }

    #[test]
    fn test_generate_summary() {
        let mut status = QuickStatus::new();
        status.add(StatusItem::new("A", HealthLevel::Good, "OK"));
        status.add(StatusItem::new("B", HealthLevel::Good, "OK"));
        status.generate_summary();
        assert_eq!(status.summary, "All systems operational");

        let mut status2 = QuickStatus::new();
        status2.add(StatusItem::new("A", HealthLevel::Warning, "Warn"));
        status2.generate_summary();
        assert!(status2.summary.contains("1 warning"));

        let mut status3 = QuickStatus::new();
        status3.add(StatusItem::new("A", HealthLevel::Critical, "Bad"));
        status3.add(StatusItem::new("B", HealthLevel::Critical, "Bad"));
        status3.generate_summary();
        assert!(status3.summary.contains("2 critical issues"));
    }

    #[test]
    fn test_memory_status() {
        let ok = memory_status(50.0);
        assert_eq!(ok.health, HealthLevel::Good);

        let warn = memory_status(80.0);
        assert_eq!(warn.health, HealthLevel::Warning);

        let critical = memory_status(95.0);
        assert_eq!(critical.health, HealthLevel::Critical);
    }

    #[test]
    fn test_disk_status() {
        let ok = disk_status(50.0, "/");
        assert_eq!(ok.health, HealthLevel::Good);

        let warn = disk_status(90.0, "/home");
        assert_eq!(warn.health, HealthLevel::Warning);

        let critical = disk_status(98.0, "/");
        assert_eq!(critical.health, HealthLevel::Critical);
    }

    #[test]
    fn test_cpu_status() {
        let ok = cpu_status(2.0, 4); // 0.5 per core
        assert_eq!(ok.health, HealthLevel::Good);

        let warn = cpu_status(6.0, 4); // 1.5 per core
        assert_eq!(warn.health, HealthLevel::Warning);

        let critical = cpu_status(12.0, 4); // 3.0 per core
        assert_eq!(critical.health, HealthLevel::Critical);
    }

    #[test]
    fn test_service_status() {
        let running = service_status("nginx", true, false);
        assert_eq!(running.health, HealthLevel::Good);

        let stopped = service_status("nginx", false, false);
        assert_eq!(stopped.health, HealthLevel::Warning);

        let failed = service_status("nginx", false, true);
        assert_eq!(failed.health, HealthLevel::Critical);
    }

    #[test]
    fn test_format_quick_status_oneline() {
        let mut status = QuickStatus::new();
        status.add(StatusItem::new("A", HealthLevel::Good, "OK"));
        status.set_summary("All good");

        let output = format_quick_status_oneline(&status);
        assert!(output.contains("[OK]"));
        assert!(output.contains("All good"));
    }

    #[test]
    fn test_is_quick_status_query() {
        assert!(is_quick_status_query("quick status"));
        assert!(is_quick_status_query("any problems?"));
        assert!(is_quick_status_query("health check"));
        assert!(is_quick_status_query("how's the system?"));

        assert!(!is_quick_status_query("restart nginx"));
        assert!(!is_quick_status_query("show disk usage"));
    }

    #[test]
    fn test_has_critical() {
        let mut status = QuickStatus::new();
        status.add(StatusItem::new("A", HealthLevel::Good, "OK"));
        assert!(!status.has_critical());

        status.add(StatusItem::new("B", HealthLevel::Critical, "Bad"));
        assert!(status.has_critical());
    }

    #[test]
    fn test_all_good() {
        let mut status = QuickStatus::new();
        status.add(StatusItem::new("A", HealthLevel::Good, "OK"));
        status.add(StatusItem::new("B", HealthLevel::Good, "OK"));
        assert!(status.all_good());

        status.add(StatusItem::new("C", HealthLevel::Warning, "Warn"));
        assert!(!status.all_good());
    }
}
