//! Journal log parsers for SystemTriage fast path.
//!
//! v0.0.35: Deterministic parsing of journalctl and systemd-analyze output.
//! v0.45.4: Proper SYSLOG_IDENTIFIER attribution via JSON output.
//! All grouping and ordering is stable across runs.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A top entry from journal logs grouped by unit/source
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalTopItem {
    /// Grouping key (SYSLOG_IDENTIFIER, _SYSTEMD_UNIT, or "unattributed")
    pub key: String,
    /// Count of occurrences
    pub count: u32,
}

/// Parsed journal summary with deterministic grouping (v0.45.4: proper attribution)
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalSummary {
    /// Total count of log entries
    pub count_total: u32,
    /// Top entries sorted by count desc, then key asc (deterministic)
    pub top: Vec<JournalTopItem>,
}

/// Boot time information from systemd-analyze
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BootTimeInfo {
    /// Raw "Startup finished in X" line
    pub raw_line: String,
    /// Kernel time in milliseconds (if parseable)
    pub kernel_ms: Option<u64>,
    /// Userspace time in milliseconds (if parseable)
    pub userspace_ms: Option<u64>,
    /// Total boot time in milliseconds (if parseable)
    pub total_ms: Option<u64>,
}

/// Parse journalctl JSON output with proper SYSLOG_IDENTIFIER attribution (v0.45.4).
///
/// Extracts grouping key using these rules (priority order):
/// 1. SYSLOG_IDENTIFIER field (e.g., "systemd", "kernel", "nginx")
/// 2. _SYSTEMD_UNIT field (e.g., "nginx.service")
/// 3. _COMM field (command name)
/// 4. "unattributed"
///
/// Output is sorted by count descending, then key ascending for determinism.
pub fn parse_journalctl_json(output: &str) -> JournalSummary {
    use std::collections::HashMap;
    let mut by_key: HashMap<String, u32> = HashMap::new();

    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Try to parse as JSON
        if let Ok(entry) = serde_json::from_str::<Value>(trimmed) {
            let key = extract_attribution(&entry);
            *by_key.entry(key).or_default() += 1;
        } else {
            // Fallback to legacy text parsing if not JSON
            let key = extract_grouping_key(trimmed);
            *by_key.entry(key).or_default() += 1;
        }
    }

    let count_total: u32 = by_key.values().sum();

    // Sort by count desc, then key asc (deterministic)
    let mut sorted: Vec<_> = by_key.into_iter().collect();
    sorted.sort_by(|a, b| {
        b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0))
    });

    let top: Vec<JournalTopItem> = sorted
        .into_iter()
        .take(10) // Top 10 for summary
        .map(|(key, count)| JournalTopItem { key, count })
        .collect();

    JournalSummary { count_total, top }
}

/// Extract attribution key from JSON journal entry (v0.45.4).
/// Priority: SYSLOG_IDENTIFIER > _SYSTEMD_UNIT > _COMM > "unattributed"
fn extract_attribution(entry: &Value) -> String {
    // Try SYSLOG_IDENTIFIER first (most specific)
    if let Some(id) = entry.get("SYSLOG_IDENTIFIER").and_then(|v| v.as_str()) {
        if !id.is_empty() {
            return id.to_lowercase();
        }
    }

    // Try _SYSTEMD_UNIT (systemd unit name)
    if let Some(unit) = entry.get("_SYSTEMD_UNIT").and_then(|v| v.as_str()) {
        if !unit.is_empty() {
            // Strip .service suffix for cleaner display
            return unit.trim_end_matches(".service").to_lowercase();
        }
    }

    // Try _COMM (command name)
    if let Some(comm) = entry.get("_COMM").and_then(|v| v.as_str()) {
        if !comm.is_empty() {
            return comm.to_lowercase();
        }
    }

    "unattributed".to_string()
}

/// Parse journalctl output with priority filter (legacy text format).
/// Use parse_journalctl_json for JSON output (v0.45.4 preferred).
pub fn parse_journalctl_priority(output: &str) -> JournalSummary {
    // Detect if input is JSON (starts with '{') or text
    let first_char = output.trim_start().chars().next();
    if first_char == Some('{') {
        return parse_journalctl_json(output);
    }

    use std::collections::HashMap;
    let mut by_key: HashMap<String, u32> = HashMap::new();

    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("--") {
            continue;
        }

        let key = extract_grouping_key(trimmed);
        *by_key.entry(key).or_default() += 1;
    }

    let count_total: u32 = by_key.values().sum();

    // Sort by count desc, then key asc (deterministic)
    let mut sorted: Vec<_> = by_key.into_iter().collect();
    sorted.sort_by(|a, b| {
        b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0))
    });

    let top: Vec<JournalTopItem> = sorted
        .into_iter()
        .take(10) // Top 10 for summary
        .map(|(key, count)| JournalTopItem { key, count })
        .collect();

    JournalSummary { count_total, top }
}

/// Extract grouping key from journal line (legacy text format).
///
/// Format: "Dec 05 10:00:00 hostname unit[pid]: message"
/// We want to extract "unit" as the key.
fn extract_grouping_key(line: &str) -> String {
    // Skip timestamp (3 fields: "Dec 05 10:00:00")
    let parts: Vec<&str> = line.splitn(5, ' ').collect();
    if parts.len() < 5 {
        return "unattributed".to_string();
    }

    // parts[3] = hostname, parts[4] = "unit[pid]: message" or "kernel: message"
    let rest = parts.get(4).unwrap_or(&"unattributed");

    // Extract unit name: everything before '[' or ':'
    let unit = rest.split(['[', ':']).next().unwrap_or("unattributed");

    // Return lowercase for consistent grouping
    unit.to_lowercase()
}

/// Parse systemd-analyze output for boot time.
///
/// Extracts "Startup finished in X" line and parses time values.
/// All times are stored as milliseconds (u64) for determinism.
pub fn parse_boot_time(output: &str) -> BootTimeInfo {
    let mut info = BootTimeInfo::default();

    for line in output.lines() {
        if line.contains("Startup finished in") {
            info.raw_line = line.trim().to_string();

            // Parse kernel time: "kernel X.XXXs"
            if let Some(kernel_ms) = extract_time_ms(line, "kernel") {
                info.kernel_ms = Some(kernel_ms);
            }

            // Parse userspace time: "userspace X.XXXs"
            if let Some(userspace_ms) = extract_time_ms(line, "userspace") {
                info.userspace_ms = Some(userspace_ms);
            }

            // Calculate total if both present
            if let (Some(k), Some(u)) = (info.kernel_ms, info.userspace_ms) {
                info.total_ms = Some(k + u);
            }

            // Or parse "= X.XXXs" at end
            if info.total_ms.is_none() {
                if let Some(total) = extract_total_time_ms(line) {
                    info.total_ms = Some(total);
                }
            }

            break;
        }
    }

    info
}

impl BootTimeInfo {
    /// Get total boot time in seconds (for display)
    pub fn total_secs(&self) -> Option<f32> {
        self.total_ms.map(|ms| ms as f32 / 1000.0)
    }
}

/// Extract time value for a specific component as milliseconds
/// Format: "2.5s (kernel)" - time comes BEFORE the component name
fn extract_time_ms(line: &str, component: &str) -> Option<u64> {
    let lower = line.to_lowercase();
    let idx = lower.find(component)?;

    // Time is BEFORE "(component)", so we need to look backwards
    // Find the opening paren before component name
    let before = &line[..idx];
    let paren_idx = before.rfind('(')?;

    // Time is just before the paren: "2.5s ("
    let time_section = before[..paren_idx].trim();

    // Find the last time value (e.g., "2.5s" in "Startup finished in 2.5s")
    // Split by whitespace and find the time token
    let tokens: Vec<&str> = time_section.split_whitespace().collect();
    let time_token = tokens.last()?;

    parse_time_to_ms(time_token)
}

/// Extract total time from "= X.XXXs" at end of line as milliseconds
fn extract_total_time_ms(line: &str) -> Option<u64> {
    let idx = line.rfind('=')?;
    let after = line[idx + 1..].trim();
    parse_time_to_ms(after)
}

/// Parse time string like "5.123s" or "1min 5.123s" to milliseconds
fn parse_time_to_ms(s: &str) -> Option<u64> {
    let s = s.trim_end_matches('s').trim_end_matches('.');

    if s.contains("min") {
        // Format: "Xmin Y.YYY"
        let parts: Vec<&str> = s.split("min").collect();
        let mins: f64 = parts.get(0)?.trim().parse().ok()?;
        let secs: f64 = parts.get(1)?.trim().parse().unwrap_or(0.0);
        Some(((mins * 60.0 + secs) * 1000.0) as u64)
    } else {
        let secs: f64 = s.trim().parse().ok()?;
        Some((secs * 1000.0) as u64)
    }
}

/// Failed systemd unit
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FailedUnit {
    /// Unit name (e.g., "nginx.service")
    pub name: String,
    /// Load state (loaded, not-found, etc.)
    pub load_state: String,
    /// Active state (failed, inactive, etc.)
    pub active_state: String,
    /// Unit description
    pub description: String,
}

/// Parse systemctl --failed output.
///
/// Handles variable whitespace between columns.
pub fn parse_failed_units(output: &str) -> Vec<FailedUnit> {
    let mut units = Vec::new();

    for line in output.lines() {
        let trimmed = line.trim().trim_start_matches('●').trim();

        // Skip headers, empty lines, and summary
        if trimmed.is_empty()
            || trimmed.starts_with("UNIT")
            || trimmed.contains("loaded units listed")
            || trimmed.starts_with("To show all")
        {
            continue;
        }

        // Parse unit line: "unit.service loaded failed failed Description"
        let parts: Vec<&str> = trimmed.split_whitespace().collect();

        // Need at least: name, load_state, active_state, sub_state
        if parts.len() >= 4 && parts[0].contains('.') {
            units.push(FailedUnit {
                name: parts[0].to_string(),
                load_state: parts.get(1).unwrap_or(&"").to_string(),
                active_state: parts.get(2).unwrap_or(&"").to_string(),
                description: parts.get(4..).map(|s| s.join(" ")).unwrap_or_default(),
            });
        }
    }

    units
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_journalctl_priority_basic() {
        let output = "Dec 05 10:00:00 myhost systemd[1]: Failed to start Some Service.
Dec 05 10:01:00 myhost kernel: Error in something
Dec 05 10:02:00 myhost systemd[1]: Another error message
Dec 05 10:03:00 myhost nginx[1234]: Connection refused";

        let summary = parse_journalctl_priority(output);
        assert_eq!(summary.count_total, 4);
        assert_eq!(summary.top[0].key, "systemd"); // Most frequent
        assert_eq!(summary.top[0].count, 2);
    }

    #[test]
    fn test_parse_journalctl_priority_empty() {
        let summary = parse_journalctl_priority("");
        assert_eq!(summary.count_total, 0);
        assert!(summary.top.is_empty());
    }

    #[test]
    fn test_parse_journalctl_priority_stable_ordering() {
        let output = "Dec 05 10:00:00 host aaa[1]: msg
Dec 05 10:00:01 host bbb[1]: msg
Dec 05 10:00:02 host ccc[1]: msg";

        let summary1 = parse_journalctl_priority(output);
        let summary2 = parse_journalctl_priority(output);

        // Same input must produce identical output (deterministic)
        assert_eq!(summary1, summary2);
        // All have count 1, should be sorted alphabetically
        assert_eq!(summary1.top[0].key, "aaa");
        assert_eq!(summary1.top[1].key, "bbb");
        assert_eq!(summary1.top[2].key, "ccc");
    }

    #[test]
    fn test_parse_boot_time_basic() {
        let output = "Startup finished in 2.5s (kernel) + 5.3s (userspace) = 7.8s";
        let info = parse_boot_time(output);
        assert!(info.raw_line.contains("Startup finished"));
        assert!(info.total_ms.is_some());
        // 7.8s = 7800ms
        assert_eq!(info.total_ms.unwrap(), 7800);
    }

    #[test]
    fn test_parse_boot_time_empty() {
        let info = parse_boot_time("");
        assert!(info.raw_line.is_empty());
        assert!(info.total_ms.is_none());
    }

    #[test]
    fn test_parse_failed_units_basic() {
        let output = "  UNIT                   LOAD   ACTIVE SUB    DESCRIPTION
● nginx.service         loaded failed failed Nginx Web Server
● redis.service         loaded failed failed Redis Database
0 loaded units listed.";

        let units = parse_failed_units(output);
        assert_eq!(units.len(), 2);
        assert_eq!(units[0].name, "nginx.service");
        assert_eq!(units[0].active_state, "failed");
        assert_eq!(units[1].name, "redis.service");
    }

    #[test]
    fn test_parse_failed_units_empty() {
        let output = "0 loaded units listed.";
        let units = parse_failed_units(output);
        assert!(units.is_empty());
    }

    #[test]
    fn test_parse_failed_units_variable_spacing() {
        // Real systemctl output often has variable spacing
        let output = "● foo.service            loaded  failed  failed  Some Description";
        let units = parse_failed_units(output);
        assert_eq!(units.len(), 1);
        assert_eq!(units[0].name, "foo.service");
        assert_eq!(units[0].load_state, "loaded");
        assert_eq!(units[0].active_state, "failed");
    }

    #[test]
    fn test_grouping_key_extraction() {
        assert_eq!(
            extract_grouping_key("Dec 05 10:00:00 host systemd[1]: message"),
            "systemd"
        );
        assert_eq!(
            extract_grouping_key("Dec 05 10:00:00 host kernel: message"),
            "kernel"
        );
        assert_eq!(
            extract_grouping_key("Dec 05 10:00:00 host NGINX[1234]: message"),
            "nginx" // lowercase
        );
    }

    // === v0.45.4: JSON parsing tests ===

    #[test]
    fn test_parse_journalctl_json_syslog_identifier() {
        let output = r#"{"SYSLOG_IDENTIFIER":"systemd","MESSAGE":"Starting service..."}
{"SYSLOG_IDENTIFIER":"systemd","MESSAGE":"Failed to start..."}
{"SYSLOG_IDENTIFIER":"nginx","MESSAGE":"Connection refused"}"#;

        let summary = parse_journalctl_json(output);
        assert_eq!(summary.count_total, 3);
        assert_eq!(summary.top[0].key, "systemd");
        assert_eq!(summary.top[0].count, 2);
        assert_eq!(summary.top[1].key, "nginx");
        assert_eq!(summary.top[1].count, 1);
    }

    #[test]
    fn test_parse_journalctl_json_fallback_to_unit() {
        // No SYSLOG_IDENTIFIER, falls back to _SYSTEMD_UNIT
        let output = r#"{"_SYSTEMD_UNIT":"nginx.service","MESSAGE":"test"}"#;

        let summary = parse_journalctl_json(output);
        assert_eq!(summary.count_total, 1);
        assert_eq!(summary.top[0].key, "nginx"); // .service stripped
    }

    #[test]
    fn test_parse_journalctl_json_unattributed() {
        // No identifying fields
        let output = r#"{"MESSAGE":"anonymous error"}"#;

        let summary = parse_journalctl_json(output);
        assert_eq!(summary.count_total, 1);
        assert_eq!(summary.top[0].key, "unattributed");
    }

    #[test]
    fn test_parse_journalctl_auto_detect_json() {
        // parse_journalctl_priority should auto-detect JSON format
        let json_output = r#"{"SYSLOG_IDENTIFIER":"test","MESSAGE":"test"}"#;
        let summary = parse_journalctl_priority(json_output);
        assert_eq!(summary.top[0].key, "test");
    }
}
