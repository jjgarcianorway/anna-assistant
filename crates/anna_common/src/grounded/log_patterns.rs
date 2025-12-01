//! Log Pattern Extraction v7.14.0
//!
//! Normalizes log messages into patterns for grouping and counting.
//! Replaces variable parts (IPs, paths, PIDs, timestamps) with placeholders.
//!
//! Sources:
//! - journalctl -b -u UNIT -p warning..alert

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
        })
        .collect();

    // Sort by count descending
    patterns.sort_by(|a, b| b.count.cmp(&a.count));

    summary.total_count = patterns.iter().map(|p| p.count).sum();
    summary.pattern_count = patterns.len();
    summary.patterns = patterns;

    summary
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
        })
        .collect();

    patterns.sort_by(|a, b| b.count.cmp(&a.count));

    summary.total_count = patterns.iter().map(|p| p.count).sum();
    summary.pattern_count = patterns.len();
    summary.patterns = patterns;

    summary
}

/// Internal data for pattern aggregation
struct PatternData {
    pattern: String,
    count: usize,
    count_last_hour: usize,
    first_seen: String,
    last_seen: String,
    sample_message: String,
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
}
