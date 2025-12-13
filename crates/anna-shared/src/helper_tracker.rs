//! Helper Tracker - Phase 83
//!
//! Tracks helpers (tools) installed by Anna vs user.
//! VISION.md: "Track what helpers she installed vs user installed"
//! Anna-installed helpers can be removed on uninstall, user-installed are preserved.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Who installed the helper
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InstallerSource {
    Anna,
    User,
    System,
    Unknown,
}

impl InstallerSource {
    pub fn name(&self) -> &'static str {
        match self {
            InstallerSource::Anna => "Anna",
            InstallerSource::User => "User",
            InstallerSource::System => "System",
            InstallerSource::Unknown => "Unknown",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            InstallerSource::Anna => "A",
            InstallerSource::User => "U",
            InstallerSource::System => "S",
            InstallerSource::Unknown => "?",
        }
    }
}

/// Purpose of the helper
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HelperPurpose {
    SystemInfo,
    NetworkDiag,
    DiskUtil,
    ProcessMon,
    LogAnalysis,
    Security,
    Performance,
    Development,
    Multimedia,
    General,
}

impl HelperPurpose {
    pub fn name(&self) -> &'static str {
        match self {
            HelperPurpose::SystemInfo => "System Info",
            HelperPurpose::NetworkDiag => "Network Diagnostics",
            HelperPurpose::DiskUtil => "Disk Utilities",
            HelperPurpose::ProcessMon => "Process Monitoring",
            HelperPurpose::LogAnalysis => "Log Analysis",
            HelperPurpose::Security => "Security",
            HelperPurpose::Performance => "Performance",
            HelperPurpose::Development => "Development",
            HelperPurpose::Multimedia => "Multimedia",
            HelperPurpose::General => "General",
        }
    }
}

/// A helper (tool) record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelperRecord {
    /// Helper name/command
    pub name: String,
    /// Package name (may differ from command)
    pub package_name: Option<String>,
    /// Who installed it
    pub installed_by: InstallerSource,
    /// Purpose category
    pub purpose: HelperPurpose,
    /// Description of what it does
    pub description: String,
    /// When it was first detected/installed
    pub installed_at: u64,
    /// Times Anna has used this helper
    pub usage_count: u64,
    /// Last time used
    pub last_used: Option<u64>,
    /// Whether it's currently available
    pub available: bool,
    /// Why Anna installed it (if Anna-installed)
    pub install_reason: Option<String>,
    /// Ticket ID that triggered installation
    pub ticket_id: Option<String>,
}

/// Helper tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HelperTracker {
    /// All helper records
    pub helpers: Vec<HelperRecord>,
    /// Count by installer source
    pub by_source: HashMap<String, u64>,
    /// Count by purpose
    pub by_purpose: HashMap<String, u64>,
    /// Total usage count
    pub total_usage: u64,
}

impl HelperTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a helper
    pub fn register(&mut self, helper: HelperRecord) {
        *self.by_source.entry(helper.installed_by.name().to_string()).or_insert(0) += 1;
        *self.by_purpose.entry(helper.purpose.name().to_string()).or_insert(0) += 1;
        self.helpers.push(helper);
    }

    /// Record helper usage
    pub fn record_usage(&mut self, name: &str, timestamp: u64) -> bool {
        let found = self.helpers.iter().position(|h| h.name == name);
        if let Some(idx) = found {
            self.helpers[idx].usage_count += 1;
            self.helpers[idx].last_used = Some(timestamp);
            self.total_usage += 1;
            true
        } else {
            false
        }
    }

    /// Mark helper as unavailable
    pub fn mark_unavailable(&mut self, name: &str) -> bool {
        let found = self.helpers.iter().position(|h| h.name == name);
        if let Some(idx) = found {
            self.helpers[idx].available = false;
            true
        } else {
            false
        }
    }

    /// Get helpers installed by Anna
    pub fn anna_installed(&self) -> Vec<&HelperRecord> {
        self.helpers.iter().filter(|h| h.installed_by == InstallerSource::Anna).collect()
    }

    /// Get helpers installed by user
    pub fn user_installed(&self) -> Vec<&HelperRecord> {
        self.helpers.iter().filter(|h| h.installed_by == InstallerSource::User).collect()
    }

    /// Get available helpers
    pub fn available(&self) -> Vec<&HelperRecord> {
        self.helpers.iter().filter(|h| h.available).collect()
    }

    /// Get helpers by purpose
    pub fn by_helper_purpose(&self, purpose: HelperPurpose) -> Vec<&HelperRecord> {
        self.helpers.iter().filter(|h| h.purpose == purpose).collect()
    }

    /// Get helper by name
    pub fn get(&self, name: &str) -> Option<&HelperRecord> {
        self.helpers.iter().find(|h| h.name == name)
    }

    /// Check if helper exists
    pub fn has(&self, name: &str) -> bool {
        self.helpers.iter().any(|h| h.name == name)
    }

    /// Total helper count
    pub fn total_count(&self) -> usize {
        self.helpers.len()
    }

    /// Available helper count
    pub fn available_count(&self) -> usize {
        self.helpers.iter().filter(|h| h.available).count()
    }

    /// Most used helper
    pub fn most_used(&self) -> Option<(&str, u64)> {
        self.helpers
            .iter()
            .max_by_key(|h| h.usage_count)
            .map(|h| (h.name.as_str(), h.usage_count))
    }

    /// Most common purpose
    pub fn most_common_purpose(&self) -> Option<(&str, u64)> {
        self.by_purpose
            .iter()
            .max_by_key(|(_, v)| *v)
            .map(|(k, v)| (k.as_str(), *v))
    }

    /// Helpers that can be removed on uninstall (Anna-installed only)
    pub fn removable_on_uninstall(&self) -> Vec<&HelperRecord> {
        self.helpers
            .iter()
            .filter(|h| h.installed_by == InstallerSource::Anna && h.available)
            .collect()
    }
}

/// Format helper tracker for display
pub fn format_helper_tracker(tracker: &HelperTracker) -> String {
    let mut lines = vec!["=== Helper Tools ===".to_string()];
    lines.push(String::new());

    if tracker.helpers.is_empty() {
        lines.push("No helpers registered yet.".to_string());
        return lines.join("\n");
    }

    // Summary
    lines.push(format!("Total helpers: {}", tracker.total_count()));
    lines.push(format!("Available: {}", tracker.available_count()));
    lines.push(format!("Total usage: {}", tracker.total_usage));

    // By source
    if !tracker.by_source.is_empty() {
        lines.push(String::new());
        lines.push("By installer:".to_string());
        for (source, count) in &tracker.by_source {
            lines.push(format!("  {}: {}", source, count));
        }
    }

    // Most used
    if let Some((name, count)) = tracker.most_used() {
        lines.push(String::new());
        lines.push(format!("Most used: {} ({} times)", name, count));
    }

    // Anna-installed (removable on uninstall)
    let removable = tracker.removable_on_uninstall();
    if !removable.is_empty() {
        lines.push(String::new());
        lines.push(format!("Anna-installed (removable): {}", removable.len()));
        for h in removable.iter().take(5) {
            lines.push(format!("  - {}", h.name));
        }
    }

    lines.join("\n")
}

/// Format helper tracker compact
pub fn format_helper_tracker_compact(tracker: &HelperTracker) -> String {
    let anna_count = tracker.anna_installed().len();
    let user_count = tracker.user_installed().len();
    format!(
        "Helpers: {} total | {} Anna-installed | {} user-installed",
        tracker.total_count(),
        anna_count,
        user_count
    )
}

/// Format helper tracker one-line
pub fn format_helper_tracker_oneline(tracker: &HelperTracker) -> String {
    format!(
        "{} helpers ({} available)",
        tracker.total_count(),
        tracker.available_count()
    )
}

/// Check if query is about helpers
pub fn is_helper_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "helper",
        "helpers",
        "tools installed",
        "what tools",
        "installed tools",
        "available tools",
        "anna install",
        "which packages",
        "did anna install",
    ];
    keywords.iter().any(|k| q.contains(k))
}

/// Generate fun fact about helpers
pub fn helper_fun_fact(tracker: &HelperTracker) -> String {
    if tracker.helpers.is_empty() {
        return "No helper tools registered yet!".to_string();
    }

    let facts = [
        format!("Anna knows about {} helper tools.", tracker.total_count()),
        format!("{} helpers are currently available.", tracker.available_count()),
        {
            if let Some((name, count)) = tracker.most_used() {
                format!("{} is the most used helper ({} times).", name, count)
            } else {
                "No helper usage recorded yet.".to_string()
            }
        },
        format!(
            "{} helpers were installed by Anna.",
            tracker.anna_installed().len()
        ),
        format!(
            "{} helpers can be removed on uninstall.",
            tracker.removable_on_uninstall().len()
        ),
    ];

    facts[tracker.total_count() % facts.len()].clone()
}

/// Detect helper purpose from name
pub fn detect_purpose(name: &str) -> HelperPurpose {
    let name_lower = name.to_lowercase();

    if name_lower.contains("net") || name_lower.contains("ping") || name_lower.contains("ip") {
        HelperPurpose::NetworkDiag
    } else if name_lower.contains("disk") || name_lower.contains("df") || name_lower.contains("du") {
        HelperPurpose::DiskUtil
    } else if name_lower.contains("top") || name_lower.contains("ps") || name_lower.contains("proc") {
        HelperPurpose::ProcessMon
    } else if name_lower.contains("log") || name_lower.contains("journal") {
        HelperPurpose::LogAnalysis
    } else if name_lower.contains("sec") || name_lower.contains("crypt") || name_lower.contains("ssh") {
        HelperPurpose::Security
    } else if name_lower.contains("perf") || name_lower.contains("bench") {
        HelperPurpose::Performance
    } else if name_lower.contains("git") || name_lower.contains("make") || name_lower.contains("gcc") {
        HelperPurpose::Development
    } else if name_lower.contains("audio") || name_lower.contains("video") || name_lower.contains("ffmpeg") {
        HelperPurpose::Multimedia
    } else if name_lower.contains("sys") || name_lower.contains("info") || name_lower.contains("stat") {
        HelperPurpose::SystemInfo
    } else {
        HelperPurpose::General
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_helper(name: &str, source: InstallerSource) -> HelperRecord {
        HelperRecord {
            name: name.to_string(),
            package_name: Some(name.to_string()),
            installed_by: source,
            purpose: detect_purpose(name),
            description: format!("{} helper", name),
            installed_at: 1234567890,
            usage_count: 0,
            last_used: None,
            available: true,
            install_reason: None,
            ticket_id: None,
        }
    }

    #[test]
    fn test_installer_source() {
        assert_eq!(InstallerSource::Anna.name(), "Anna");
        assert_eq!(InstallerSource::User.symbol(), "U");
    }

    #[test]
    fn test_helper_purpose() {
        assert_eq!(HelperPurpose::SystemInfo.name(), "System Info");
        assert_eq!(HelperPurpose::NetworkDiag.name(), "Network Diagnostics");
    }

    #[test]
    fn test_detect_purpose() {
        assert_eq!(detect_purpose("netstat"), HelperPurpose::NetworkDiag);
        assert_eq!(detect_purpose("htop"), HelperPurpose::ProcessMon);
        assert_eq!(detect_purpose("sysinfo"), HelperPurpose::SystemInfo);
        assert_eq!(detect_purpose("git"), HelperPurpose::Development);
        assert_eq!(detect_purpose("ffmpeg"), HelperPurpose::Multimedia);
    }

    #[test]
    fn test_helper_tracker_register() {
        let mut tracker = HelperTracker::new();
        tracker.register(make_helper("htop", InstallerSource::Anna));

        assert_eq!(tracker.total_count(), 1);
        assert!(tracker.has("htop"));
    }

    #[test]
    fn test_record_usage() {
        let mut tracker = HelperTracker::new();
        tracker.register(make_helper("htop", InstallerSource::Anna));

        assert!(tracker.record_usage("htop", 1234567890));
        assert_eq!(tracker.total_usage, 1);
        assert_eq!(tracker.get("htop").unwrap().usage_count, 1);
    }

    #[test]
    fn test_mark_unavailable() {
        let mut tracker = HelperTracker::new();
        tracker.register(make_helper("htop", InstallerSource::Anna));

        assert!(tracker.mark_unavailable("htop"));
        assert!(!tracker.get("htop").unwrap().available);
        assert_eq!(tracker.available_count(), 0);
    }

    #[test]
    fn test_anna_vs_user_installed() {
        let mut tracker = HelperTracker::new();
        tracker.register(make_helper("htop", InstallerSource::Anna));
        tracker.register(make_helper("vim", InstallerSource::User));
        tracker.register(make_helper("netstat", InstallerSource::Anna));

        assert_eq!(tracker.anna_installed().len(), 2);
        assert_eq!(tracker.user_installed().len(), 1);
    }

    #[test]
    fn test_removable_on_uninstall() {
        let mut tracker = HelperTracker::new();
        tracker.register(make_helper("htop", InstallerSource::Anna));
        tracker.register(make_helper("vim", InstallerSource::User));

        let removable = tracker.removable_on_uninstall();
        assert_eq!(removable.len(), 1);
        assert_eq!(removable[0].name, "htop");
    }

    #[test]
    fn test_most_used() {
        let mut tracker = HelperTracker::new();
        tracker.register(make_helper("htop", InstallerSource::Anna));
        tracker.register(make_helper("vim", InstallerSource::User));

        tracker.record_usage("htop", 1);
        tracker.record_usage("htop", 2);
        tracker.record_usage("vim", 3);

        let (name, count) = tracker.most_used().unwrap();
        assert_eq!(name, "htop");
        assert_eq!(count, 2);
    }

    #[test]
    fn test_format_helper_tracker() {
        let mut tracker = HelperTracker::new();
        tracker.register(make_helper("htop", InstallerSource::Anna));

        let output = format_helper_tracker(&tracker);
        assert!(output.contains("Helper Tools"));
        assert!(output.contains("Total helpers: 1"));
    }

    #[test]
    fn test_is_helper_query() {
        assert!(is_helper_query("what helpers are installed?"));
        assert!(is_helper_query("show me available tools"));
        assert!(is_helper_query("what did anna install?"));
        assert!(!is_helper_query("what is the weather?"));
    }

    #[test]
    fn test_helper_fun_fact() {
        let mut tracker = HelperTracker::new();
        tracker.register(make_helper("htop", InstallerSource::Anna));

        let fact = helper_fun_fact(&tracker);
        assert!(!fact.is_empty());
    }
}
