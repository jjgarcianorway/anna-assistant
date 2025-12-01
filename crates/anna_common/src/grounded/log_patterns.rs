//! Log Pattern Extraction v7.16.0
//!
//! Normalizes log messages into patterns for grouping and counting.
//! Replaces variable parts (IPs, paths, PIDs, timestamps) with placeholders.
//!
//! v7.16.0: Multi-window history (this boot, 24h, 7d, 30d)
//!
//! Sources:
//! - journalctl -b -u UNIT -p warning..alert
//! - journalctl --since "X days ago" -u UNIT -p warning..alert

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;

/// A normalized log pattern with occurrence tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogPattern {
    /// Normalized pattern with placeholders
    pub pattern: String,
    /// Total occurrences this boot
    pub count: usize,
    /// Occurrences in the last hour
    pub count_last_hour: usize,
    /// First occurrence timestamp (ISO 8601)
    pub first_seen: String,
    /// Last occurrence timestamp (ISO 8601)
    pub last_seen: String,
    /// Original sample message (first occurrence)
    pub sample_message: String,
    /// Priority level (warning, error, critical)
    pub priority: String,
}

/// v7.16.0: Pattern with history across time windows
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LogPatternHistory {
    /// Normalized pattern
    pub pattern: String,
    /// Sample message
    pub sample_message: String,
    /// Priority level
    pub priority: String,
    /// Count this boot
    pub count_this_boot: usize,
    /// Count last 24h
    pub count_24h: usize,
    /// Count last 7 days
    pub count_7d: usize,
    /// Count last 30 days
    pub count_30d: usize,
    /// Number of boots seen in
    pub boots_seen: usize,
    /// First seen timestamp
    pub first_seen: String,
    /// Last seen timestamp
    pub last_seen: String,
}

/// Summary of log patterns for a unit
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LogPatternSummary {
    /// Unit name (e.g., "NetworkManager.service")
    pub unit: String,
    /// Total warning/error count
    pub total_count: usize,
    /// Number of unique patterns
    pub pattern_count: usize,
    /// Top patterns sorted by count (descending)
    pub patterns: Vec<LogPattern>,
    /// Source command used
    pub source: String,
}

/// v7.16.0: Multi-window log summary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LogHistorySummary {
    /// Unit or component name
    pub unit: String,
    /// Counts by severity for this boot
    pub this_boot_critical: usize,
    pub this_boot_error: usize,
    pub this_boot_warning: usize,
    /// Patterns with history
    pub patterns: Vec<LogPatternHistory>,
    /// Source description
    pub source: String,
}

impl LogPatternSummary {
    /// Check if there are no warnings/errors
    pub fn is_empty(&self) -> bool {
        self.total_count == 0
    }

    /// Get top N patterns
    pub fn top_patterns(&self, n: usize) -> &[LogPattern] {
        let limit = n.min(self.patterns.len());
        &self.patterns[..limit]
    }
}

impl LogHistorySummary {
    /// Check if there are no warnings/errors this boot
    pub fn is_empty_this_boot(&self) -> bool {
        self.this_boot_critical + self.this_boot_error + self.this_boot_warning == 0
    }

    /// Get total count this boot
    pub fn total_this_boot(&self) -> usize {
        self.this_boot_critical + self.this_boot_error + self.this_boot_warning
    }

    /// Get top N patterns by this_boot count
    pub fn top_patterns(&self, n: usize) -> Vec<&LogPatternHistory> {
        let mut sorted: Vec<_> = self.patterns.iter().collect();
        sorted.sort_by(|a, b| b.count_this_boot.cmp(&a.count_this_boot));
        sorted.into_iter().take(n).collect()
    }

    /// Get patterns that have history beyond this boot
    pub fn patterns_with_history(&self) -> Vec<&LogPatternHistory> {
        self.patterns.iter()
            .filter(|p| p.count_7d > p.count_this_boot || p.boots_seen > 1)
            .collect()
    }
}

/// Extract log patterns from journalctl for a unit
pub fn extract_patterns_for_unit(unit: &str) -> LogPatternSummary {
    let mut summary = LogPatternSummary {
        unit: unit.to_string(),
        source: format!("journalctl -b -u {} -p warning..alert", unit),
        ..Default::default()
    };

    // Get logs from journalctl
    let output = Command::new("journalctl")
        .args([
            "-b",
            "-u", unit,
            "-p", "warning..alert",
            "-o", "json",
            "--no-pager",
            "-q",
        ])
        .output();

    let logs = match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        _ => return summary,
    };

    // Parse JSON lines and extract patterns
    let mut pattern_map: HashMap<String, PatternData> = HashMap::new();
    let now = chrono::Utc::now();
    let one_hour_ago = now - chrono::Duration::hours(1);

    for line in logs.lines() {
        if line.trim().is_empty() {
            continue;
        }

        if let Ok(entry) = serde_json::from_str::<serde_json::Value>(line) {
            let message = entry.get("MESSAGE")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            if message.is_empty() {
                continue;
            }

            let priority = entry.get("PRIORITY")
                .and_then(|v| v.as_str())
                .map(priority_to_name)
                .unwrap_or_else(|| "warning".to_string());

            let timestamp = entry.get("__REALTIME_TIMESTAMP")
                .and_then(|v| v.as_str())
                .map(|ts| parse_realtime_timestamp(ts))
                .unwrap_or_else(|| now.format("%Y-%m-%d %H:%M:%S").to_string());

            let pattern = normalize_message(&message);

            let is_last_hour = if let Ok(ts) = chrono::NaiveDateTime::parse_from_str(
                &timestamp, "%Y-%m-%d %H:%M:%S"
            ) {
                let ts_utc = ts.and_utc();
                ts_utc >= one_hour_ago
            } else {
                false
            };

            pattern_map
                .entry(pattern.clone())
                .and_modify(|data| {
                    data.count += 1;
                    if is_last_hour {
                        data.count_last_hour += 1;
                    }
                    data.last_seen = timestamp.clone();
                })
                .or_insert(PatternData {
                    pattern: pattern.clone(),
                    count: 1,
                    count_last_hour: if is_last_hour { 1 } else { 0 },
                    first_seen: timestamp.clone(),
                    last_seen: timestamp.clone(),
                    sample_message: message,
                    priority,
                });
        }
    }

    // Convert to sorted patterns
    let mut patterns: Vec<LogPattern> = pattern_map
        .into_values()
        .map(|data| LogPattern {
            pattern: data.pattern,
            count: data.count,
            count_last_hour: data.count_last_hour,
            first_seen: data.first_seen,
            last_seen: data.last_seen,
            sample_message: data.sample_message,
            priority: data.priority,
        })
        .collect();

    // Sort by count descending
    patterns.sort_by(|a, b| b.count.cmp(&a.count));

    summary.total_count = patterns.iter().map(|p| p.count).sum();
    summary.pattern_count = patterns.len();
    summary.patterns = patterns;

    summary
}

/// v7.16.0: Extract log patterns with multi-window history
pub fn extract_patterns_with_history(unit: &str) -> LogHistorySummary {
    let mut summary = LogHistorySummary {
        unit: unit.to_string(),
        source: format!("journalctl -u {} -p warning..alert", unit),
        ..Default::default()
    };

    // Collect patterns across windows
    let mut pattern_map: HashMap<String, LogPatternHistory> = HashMap::new();

    // This boot
    let this_boot = extract_logs_for_window(unit, "boot");
    for (pattern, data) in &this_boot {
        let entry = pattern_map.entry(pattern.clone()).or_insert_with(|| LogPatternHistory {
            pattern: pattern.clone(),
            sample_message: data.sample_message.clone(),
            priority: data.priority.clone(),
            ..Default::default()
        });
        entry.count_this_boot = data.count;
        entry.first_seen = data.first_seen.clone();
        entry.last_seen = data.last_seen.clone();
        entry.boots_seen = 1;

        match data.priority.as_str() {
            "critical" | "crit" => summary.this_boot_critical += data.count,
            "error" | "err" => summary.this_boot_error += data.count,
            _ => summary.this_boot_warning += data.count,
        }
    }

    // Last 24h
    let last_24h = extract_logs_for_window(unit, "24h");
    for (pattern, data) in &last_24h {
        let entry = pattern_map.entry(pattern.clone()).or_insert_with(|| LogPatternHistory {
            pattern: pattern.clone(),
            sample_message: data.sample_message.clone(),
            priority: data.priority.clone(),
            ..Default::default()
        });
        entry.count_24h = data.count;
        if entry.first_seen.is_empty() || data.first_seen < entry.first_seen {
            entry.first_seen = data.first_seen.clone();
        }
        if data.last_seen > entry.last_seen {
            entry.last_seen = data.last_seen.clone();
        }
    }

    // Last 7 days
    let last_7d = extract_logs_for_window(unit, "7d");
    for (pattern, data) in &last_7d {
        let entry = pattern_map.entry(pattern.clone()).or_insert_with(|| LogPatternHistory {
            pattern: pattern.clone(),
            sample_message: data.sample_message.clone(),
            priority: data.priority.clone(),
            ..Default::default()
        });
        entry.count_7d = data.count;
        // Estimate boots based on spread
        if data.count > entry.count_this_boot {
            entry.boots_seen = estimate_boots_from_spread(&data.first_seen, &data.last_seen);
        }
    }

    // Convert to vec and sort
    summary.patterns = pattern_map.into_values().collect();
    summary.patterns.sort_by(|a, b| b.count_this_boot.cmp(&a.count_this_boot));

    summary
}

/// Extract logs for a specific time window
fn extract_logs_for_window(unit: &str, window: &str) -> HashMap<String, PatternData> {
    let mut pattern_map: HashMap<String, PatternData> = HashMap::new();

    let args: Vec<&str> = match window {
        "boot" => vec!["-b", "-u", unit, "-p", "warning..alert", "-o", "json", "--no-pager", "-q"],
        "24h" => vec!["--since", "24 hours ago", "-u", unit, "-p", "warning..alert", "-o", "json", "--no-pager", "-q"],
        "7d" => vec!["--since", "7 days ago", "-u", unit, "-p", "warning..alert", "-o", "json", "--no-pager", "-q"],
        "30d" => vec!["--since", "30 days ago", "-u", unit, "-p", "warning..alert", "-o", "json", "--no-pager", "-q"],
        _ => return pattern_map,
    };

    let output = Command::new("journalctl")
        .args(&args)
        .output();

    let logs = match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        _ => return pattern_map,
    };

    let now = chrono::Utc::now();

    for line in logs.lines() {
        if line.trim().is_empty() {
            continue;
        }

        if let Ok(entry) = serde_json::from_str::<serde_json::Value>(line) {
            let message = entry.get("MESSAGE")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            if message.is_empty() {
                continue;
            }

            let priority = entry.get("PRIORITY")
                .and_then(|v| v.as_str())
                .map(priority_to_name)
                .unwrap_or_else(|| "warning".to_string());

            let timestamp = entry.get("__REALTIME_TIMESTAMP")
                .and_then(|v| v.as_str())
                .map(|ts| parse_realtime_timestamp(ts))
                .unwrap_or_else(|| now.format("%Y-%m-%d %H:%M:%S").to_string());

            let pattern = normalize_message(&message);

            pattern_map
                .entry(pattern.clone())
                .and_modify(|data| {
                    data.count += 1;
                    data.last_seen = timestamp.clone();
                })
                .or_insert(PatternData {
                    pattern: pattern.clone(),
                    count: 1,
                    count_last_hour: 0,
                    first_seen: timestamp.clone(),
                    last_seen: timestamp.clone(),
                    sample_message: message,
                    priority,
                });
        }
    }

    pattern_map
}

/// Estimate number of boots from timestamp spread
fn estimate_boots_from_spread(first: &str, last: &str) -> usize {
    if let (Ok(f), Ok(l)) = (
        chrono::NaiveDateTime::parse_from_str(first, "%Y-%m-%d %H:%M:%S"),
        chrono::NaiveDateTime::parse_from_str(last, "%Y-%m-%d %H:%M:%S"),
    ) {
        let days = (l - f).num_days();
        // Rough estimate: 1 boot per day for laptops, at least 1
        (days as usize).max(1)
    } else {
        1
    }
}

/// Extract log patterns for a driver (kernel logs)
pub fn extract_patterns_for_driver(driver: &str) -> LogPatternSummary {
    let mut summary = LogPatternSummary {
        unit: driver.to_string(),
        source: format!("journalctl -b -k -p warning..alert | grep -i {}", driver),
        ..Default::default()
    };

    // Get kernel logs and filter for driver
    let output = Command::new("journalctl")
        .args([
            "-b",
            "-k",
            "-p", "warning..alert",
            "-o", "json",
            "--no-pager",
            "-q",
        ])
        .output();

    let logs = match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        _ => return summary,
    };

    let driver_lower = driver.to_lowercase();
    let mut pattern_map: HashMap<String, PatternData> = HashMap::new();
    let now = chrono::Utc::now();
    let one_hour_ago = now - chrono::Duration::hours(1);

    for line in logs.lines() {
        if line.trim().is_empty() {
            continue;
        }

        if let Ok(entry) = serde_json::from_str::<serde_json::Value>(line) {
            let message = entry.get("MESSAGE")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            // Filter for driver-related messages
            if !message.to_lowercase().contains(&driver_lower) {
                continue;
            }

            let priority = entry.get("PRIORITY")
                .and_then(|v| v.as_str())
                .map(priority_to_name)
                .unwrap_or_else(|| "warning".to_string());

            let timestamp = entry.get("__REALTIME_TIMESTAMP")
                .and_then(|v| v.as_str())
                .map(|ts| parse_realtime_timestamp(ts))
                .unwrap_or_else(|| now.format("%Y-%m-%d %H:%M:%S").to_string());

            let pattern = normalize_message(&message);

            let is_last_hour = if let Ok(ts) = chrono::NaiveDateTime::parse_from_str(
                &timestamp, "%Y-%m-%d %H:%M:%S"
            ) {
                let ts_utc = ts.and_utc();
                ts_utc >= one_hour_ago
            } else {
                false
            };

            pattern_map
                .entry(pattern.clone())
                .and_modify(|data| {
                    data.count += 1;
                    if is_last_hour {
                        data.count_last_hour += 1;
                    }
                    data.last_seen = timestamp.clone();
                })
                .or_insert(PatternData {
                    pattern: pattern.clone(),
                    count: 1,
                    count_last_hour: if is_last_hour { 1 } else { 0 },
                    first_seen: timestamp.clone(),
                    last_seen: timestamp.clone(),
                    sample_message: message,
                    priority,
                });
        }
    }

    // Convert to sorted patterns
    let mut patterns: Vec<LogPattern> = pattern_map
        .into_values()
        .map(|data| LogPattern {
            pattern: data.pattern,
            count: data.count,
            count_last_hour: data.count_last_hour,
            first_seen: data.first_seen,
            last_seen: data.last_seen,
            sample_message: data.sample_message,
            priority: data.priority,
        })
        .collect();

    patterns.sort_by(|a, b| b.count.cmp(&a.count));

    summary.total_count = patterns.iter().map(|p| p.count).sum();
    summary.pattern_count = patterns.len();
    summary.patterns = patterns;

    summary
}

/// v7.16.0: Extract driver patterns with history
pub fn extract_driver_patterns_with_history(driver: &str) -> LogHistorySummary {
    let mut summary = LogHistorySummary {
        unit: driver.to_string(),
        source: format!("journalctl -k -p warning..alert | grep -i {}", driver),
        ..Default::default()
    };

    let mut pattern_map: HashMap<String, LogPatternHistory> = HashMap::new();
    let driver_lower = driver.to_lowercase();

    // This boot
    let this_boot = extract_kernel_logs_for_window(&driver_lower, "boot");
    for (pattern, data) in &this_boot {
        let entry = pattern_map.entry(pattern.clone()).or_insert_with(|| LogPatternHistory {
            pattern: pattern.clone(),
            sample_message: data.sample_message.clone(),
            priority: data.priority.clone(),
            ..Default::default()
        });
        entry.count_this_boot = data.count;
        entry.first_seen = data.first_seen.clone();
        entry.last_seen = data.last_seen.clone();
        entry.boots_seen = 1;

        match data.priority.as_str() {
            "critical" | "crit" => summary.this_boot_critical += data.count,
            "error" | "err" => summary.this_boot_error += data.count,
            _ => summary.this_boot_warning += data.count,
        }
    }

    // Last 7 days
    let last_7d = extract_kernel_logs_for_window(&driver_lower, "7d");
    for (pattern, data) in &last_7d {
        let entry = pattern_map.entry(pattern.clone()).or_insert_with(|| LogPatternHistory {
            pattern: pattern.clone(),
            sample_message: data.sample_message.clone(),
            priority: data.priority.clone(),
            ..Default::default()
        });
        entry.count_7d = data.count;
        if data.count > entry.count_this_boot {
            entry.boots_seen = estimate_boots_from_spread(&data.first_seen, &data.last_seen);
        }
    }

    summary.patterns = pattern_map.into_values().collect();
    summary.patterns.sort_by(|a, b| b.count_this_boot.cmp(&a.count_this_boot));

    summary
}

/// Extract kernel logs for a specific time window
fn extract_kernel_logs_for_window(driver_filter: &str, window: &str) -> HashMap<String, PatternData> {
    let mut pattern_map: HashMap<String, PatternData> = HashMap::new();

    let args: Vec<&str> = match window {
        "boot" => vec!["-b", "-k", "-p", "warning..alert", "-o", "json", "--no-pager", "-q"],
        "7d" => vec!["--since", "7 days ago", "-k", "-p", "warning..alert", "-o", "json", "--no-pager", "-q"],
        _ => return pattern_map,
    };

    let output = Command::new("journalctl")
        .args(&args)
        .output();

    let logs = match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        _ => return pattern_map,
    };

    let now = chrono::Utc::now();

    for line in logs.lines() {
        if line.trim().is_empty() {
            continue;
        }

        if let Ok(entry) = serde_json::from_str::<serde_json::Value>(line) {
            let message = entry.get("MESSAGE")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            if !message.to_lowercase().contains(driver_filter) {
                continue;
            }

            let priority = entry.get("PRIORITY")
                .and_then(|v| v.as_str())
                .map(priority_to_name)
                .unwrap_or_else(|| "warning".to_string());

            let timestamp = entry.get("__REALTIME_TIMESTAMP")
                .and_then(|v| v.as_str())
                .map(|ts| parse_realtime_timestamp(ts))
                .unwrap_or_else(|| now.format("%Y-%m-%d %H:%M:%S").to_string());

            let pattern = normalize_message(&message);

            pattern_map
                .entry(pattern.clone())
                .and_modify(|data| {
                    data.count += 1;
                    data.last_seen = timestamp.clone();
                })
                .or_insert(PatternData {
                    pattern: pattern.clone(),
                    count: 1,
                    count_last_hour: 0,
                    first_seen: timestamp.clone(),
                    last_seen: timestamp.clone(),
                    sample_message: message,
                    priority,
                });
        }
    }

    pattern_map
}

/// Internal data for pattern aggregation
struct PatternData {
    pattern: String,
    count: usize,
    count_last_hour: usize,
    first_seen: String,
    last_seen: String,
    sample_message: String,
    priority: String,
}

/// Convert priority number to name
fn priority_to_name(priority: &str) -> String {
    match priority {
        "0" | "1" | "2" => "critical".to_string(),
        "3" => "error".to_string(),
        "4" => "warning".to_string(),
        "5" => "notice".to_string(),
        "6" => "info".to_string(),
        "7" => "debug".to_string(),
        _ => "warning".to_string(),
    }
}

/// Normalize a log message into a pattern by replacing variable parts
pub fn normalize_message(message: &str) -> String {
    let mut pattern = message.to_string();

    // Replace IPv4 addresses
    let ipv4_re = Regex::new(r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b").unwrap();
    pattern = ipv4_re.replace_all(&pattern, "%IP%").to_string();

    // Replace IPv6 addresses (simplified)
    let ipv6_re = Regex::new(r"\b[0-9a-fA-F:]{3,39}\b").unwrap();
    pattern = ipv6_re.replace_all(&pattern, "%IP%").to_string();

    // Replace MAC addresses
    let mac_re = Regex::new(r"\b[0-9a-fA-F]{2}(:[0-9a-fA-F]{2}){5}\b").unwrap();
    pattern = mac_re.replace_all(&pattern, "%MAC%").to_string();

    // Replace interface names (wlp*, enp*, eth*, wlan*, etc.)
    let iface_re = Regex::new(r"\b(wl|en|eth|wlan|virbr|docker|br|veth|lo)[a-zA-Z0-9]+\b").unwrap();
    pattern = iface_re.replace_all(&pattern, "%IFACE%").to_string();

    // Replace PIDs in brackets or parentheses
    let pid_re = Regex::new(r"\[(\d+)\]|\(pid[= ]*(\d+)\)").unwrap();
    pattern = pid_re.replace_all(&pattern, "[%PID%]").to_string();

    // Replace numeric device indices
    let dev_re = Regex::new(r"\b(sd[a-z]|nvme\d+n\d+|loop\d+|dm-\d+)\b").unwrap();
    pattern = dev_re.replace_all(&pattern, "%DEV%").to_string();

    // Replace absolute paths (keep first component for context)
    let path_re = Regex::new(r"(/[a-zA-Z0-9._-]+){2,}").unwrap();
    pattern = path_re.replace_all(&pattern, "%PATH%").to_string();

    // Replace domain names
    let domain_re = Regex::new(r"\b[a-zA-Z0-9][-a-zA-Z0-9]*(\.[a-zA-Z0-9][-a-zA-Z0-9]*)+\b").unwrap();
    pattern = domain_re.replace_all(&pattern, "%DOMAIN%").to_string();

    // Replace usernames after "user" or "User"
    let user_re = Regex::new(r"(?i)(user[= ]+)[a-zA-Z0-9_-]+").unwrap();
    pattern = user_re.replace_all(&pattern, "$1%USER%").to_string();

    // Replace hex sequences (8+ chars)
    let hex_re = Regex::new(r"\b0x[0-9a-fA-F]{4,}\b|\b[0-9a-fA-F]{8,}\b").unwrap();
    pattern = hex_re.replace_all(&pattern, "%HEX%").to_string();

    // Replace large numbers (5+ digits)
    let num_re = Regex::new(r"\b\d{5,}\b").unwrap();
    pattern = num_re.replace_all(&pattern, "%NUM%").to_string();

    // Collapse multiple spaces
    let space_re = Regex::new(r"\s+").unwrap();
    pattern = space_re.replace_all(&pattern, " ").to_string();

    pattern.trim().to_string()
}

/// Parse journalctl's __REALTIME_TIMESTAMP (microseconds since epoch)
fn parse_realtime_timestamp(ts: &str) -> String {
    if let Ok(usecs) = ts.parse::<i64>() {
        let secs = usecs / 1_000_000;
        if let Some(dt) = chrono::DateTime::from_timestamp(secs, 0) {
            return dt.format("%Y-%m-%d %H:%M:%S").to_string();
        }
    }
    ts.to_string()
}

/// Format a timestamp for display (extract just time if today)
pub fn format_time_short(timestamp: &str) -> String {
    if let Ok(ts) = chrono::NaiveDateTime::parse_from_str(timestamp, "%Y-%m-%d %H:%M:%S") {
        let now = chrono::Local::now().naive_local();
        if ts.date() == now.date() {
            return ts.format("%H:%M").to_string();
        }
    }
    // Just extract time part
    if let Some(time_part) = timestamp.split(' ').nth(1) {
        if let Some(hm) = time_part.split(':').take(2).collect::<Vec<_>>().join(":").into() {
            return hm;
        }
    }
    timestamp.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_ip() {
        let msg = "Connection from 192.168.1.42 refused";
        let pattern = normalize_message(msg);
        assert!(pattern.contains("%IP%"));
        assert!(!pattern.contains("192.168"));
    }

    #[test]
    fn test_normalize_interface() {
        let msg = "Link down on interface wlp0s20f3";
        let pattern = normalize_message(msg);
        assert!(pattern.contains("%IFACE%"));
        assert!(!pattern.contains("wlp0s20f3"));
    }

    #[test]
    fn test_normalize_path() {
        let msg = "Error reading /home/user/config/app.conf";
        let pattern = normalize_message(msg);
        assert!(pattern.contains("%PATH%"));
    }

    #[test]
    fn test_normalize_domain() {
        let msg = "DNS query failed for api.example.com";
        let pattern = normalize_message(msg);
        assert!(pattern.contains("%DOMAIN%"));
    }

    #[test]
    fn test_normalize_user() {
        let msg = "Authentication failed for user admin";
        let pattern = normalize_message(msg);
        assert!(pattern.contains("%USER%"));
    }

    #[test]
    fn test_extract_patterns_for_unit() {
        // This test requires journalctl to be available
        let summary = extract_patterns_for_unit("dbus.service");
        // Just verify it doesn't crash and returns valid structure
        assert!(summary.unit == "dbus.service");
        assert!(summary.source.contains("journalctl"));
    }

    #[test]
    fn test_priority_to_name() {
        assert_eq!(priority_to_name("2"), "critical");
        assert_eq!(priority_to_name("3"), "error");
        assert_eq!(priority_to_name("4"), "warning");
    }

    #[test]
    fn test_log_history_summary() {
        let summary = LogHistorySummary {
            unit: "test.service".to_string(),
            this_boot_critical: 1,
            this_boot_error: 2,
            this_boot_warning: 3,
            ..Default::default()
        };
        assert_eq!(summary.total_this_boot(), 6);
        assert!(!summary.is_empty_this_boot());
    }
}
