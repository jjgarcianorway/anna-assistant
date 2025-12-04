//! Log Atlas v7.20.0 - Pattern IDs and Cross-Boot Visibility
//!
//! Each distinct log pattern is assigned a short stable ID (A01, A02, etc.)
//! Patterns are stored with severity, first seen, and last seen timestamps.
//! Persisted under /var/lib/anna/journal.
//!
//! Pattern matching rules:
//! - Messages are normalized (timestamps, PIDs stripped)
//! - A pattern is "the same" when normalized message matches exactly
//! - Full message text is always shown (no truncation)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

/// Directory for log atlas persistence
pub const JOURNAL_DIR: &str = "/var/lib/anna/journal";
pub const BASELINE_DIR: &str = "/var/lib/anna/journal/baseline";

/// A log pattern with stable ID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogPattern {
    /// Short stable ID like "A01", "A02" (component-local)
    pub id: String,
    /// Severity: "emergency", "alert", "critical", "error", "warning", "notice", "info", "debug"
    pub severity: String,
    /// Normalized pattern (timestamps/PIDs stripped)
    pub normalized: String,
    /// Full original message (first occurrence)
    pub full_message: String,
    /// First seen timestamp (Unix)
    pub first_seen: u64,
    /// Last seen timestamp (Unix)
    pub last_seen: u64,
    /// Total occurrences across all boots
    pub total_count: u32,
    /// Number of boots where this pattern appeared
    pub boots_seen: u32,
    /// Boot IDs where this pattern appeared (list of boot numbers, newest first)
    pub boot_ids: Vec<i32>,
}

/// Log atlas for a component
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComponentAtlas {
    pub component: String,
    pub component_type: String, // "service", "device", "kernel"
    pub patterns: Vec<LogPattern>,
    pub last_updated: u64,
}

impl ComponentAtlas {
    /// Get or create pattern ID for a normalized message
    pub fn get_or_create_pattern_id(&mut self, normalized: &str) -> String {
        // Find existing pattern
        for pattern in &self.patterns {
            if pattern.normalized == normalized {
                return pattern.id.clone();
            }
        }

        // Create new pattern ID
        let next_num = self.patterns.len() + 1;
        let prefix = self
            .component_type
            .chars()
            .next()
            .unwrap_or('X')
            .to_ascii_uppercase();
        format!("{}{:02}", prefix, next_num)
    }

    /// Add or update a pattern
    pub fn record_pattern(
        &mut self,
        severity: &str,
        normalized: &str,
        full_message: &str,
        timestamp: u64,
        boot_id: i32,
    ) {
        // Find existing pattern
        for pattern in &mut self.patterns {
            if pattern.normalized == normalized {
                pattern.last_seen = timestamp;
                pattern.total_count += 1;
                if !pattern.boot_ids.contains(&boot_id) {
                    pattern.boot_ids.push(boot_id);
                    pattern.boots_seen += 1;
                }
                return;
            }
        }

        // Create new pattern
        let id = self.get_or_create_pattern_id(normalized);
        self.patterns.push(LogPattern {
            id,
            severity: severity.to_string(),
            normalized: normalized.to_string(),
            full_message: full_message.to_string(),
            first_seen: timestamp,
            last_seen: timestamp,
            total_count: 1,
            boots_seen: 1,
            boot_ids: vec![boot_id],
        });
    }

    /// Get patterns from current boot
    pub fn current_boot_patterns(&self) -> Vec<&LogPattern> {
        self.patterns
            .iter()
            .filter(|p| p.boot_ids.contains(&0))
            .collect()
    }

    /// Save atlas to disk
    pub fn save(&self) -> std::io::Result<()> {
        let dir = Path::new(JOURNAL_DIR);
        if !dir.exists() {
            fs::create_dir_all(dir)?;
        }

        let path = dir.join(format!("{}.json", self.component.replace('/', "_")));
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Load atlas from disk
    pub fn load(component: &str) -> Option<Self> {
        let path = Path::new(JOURNAL_DIR).join(format!("{}.json", component.replace('/', "_")));

        if !path.exists() {
            return None;
        }

        let content = fs::read_to_string(&path).ok()?;
        serde_json::from_str(&content).ok()
    }
}

/// Log entry with boot context
#[derive(Debug, Clone)]
pub struct BootLogEntry {
    pub pattern_id: String,
    pub severity: String,
    pub message: String,
    pub count_this_boot: u32,
    pub timestamp: u64,
    pub boot_offset: i32, // 0 = current, -1 = previous, etc.
}

/// Cross-boot log summary for a component
#[derive(Debug, Clone, Default)]
pub struct CrossBootLogSummary {
    pub component: String,
    pub current_boot_entries: Vec<BootLogEntry>,
    pub historical_patterns: Vec<LogPattern>,
    pub source: String,
}

/// Normalize a log message by stripping timestamps, PIDs, and variable parts
pub fn normalize_message(message: &str) -> String {
    let mut normalized = message.to_string();

    // Strip common timestamp patterns
    // ISO format: 2025-12-01T14:37:00.123456
    let timestamp_re =
        regex::Regex::new(r"\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}(\.\d+)?").unwrap();
    normalized = timestamp_re
        .replace_all(&normalized, "%TIMESTAMP%")
        .to_string();

    // Strip IP addresses BEFORE PIDs (IP addresses have dots that break PID pattern)
    let ip_re = regex::Regex::new(r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}(:\d+)?").unwrap();
    normalized = ip_re.replace_all(&normalized, "%IP%").to_string();

    // Strip PIDs like [1234] or (1234) - must have brackets or parens
    let pid_re = regex::Regex::new(r"\[\d{2,6}\]|\(\d{2,6}\)").unwrap();
    normalized = pid_re.replace_all(&normalized, "%PID%").to_string();

    // Strip MAC addresses
    let mac_re = regex::Regex::new(r"[0-9a-fA-F]{2}(:[0-9a-fA-F]{2}){5}").unwrap();
    normalized = mac_re.replace_all(&normalized, "%MAC%").to_string();

    // Strip UUIDs
    let uuid_re = regex::Regex::new(
        r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}",
    )
    .unwrap();
    normalized = uuid_re.replace_all(&normalized, "%UUID%").to_string();

    // Strip hex memory addresses like 0x7fff1234
    let hex_re = regex::Regex::new(r"0x[0-9a-fA-F]+").unwrap();
    normalized = hex_re.replace_all(&normalized, "%HEX%").to_string();

    normalized.trim().to_string()
}

/// Get log atlas for a service
pub fn get_service_log_atlas(unit_name: &str, max_boots: u32) -> CrossBootLogSummary {
    let mut summary = CrossBootLogSummary {
        component: unit_name.to_string(),
        source: format!("journalctl -u {} -p warning..alert", unit_name),
        ..Default::default()
    };

    // Load or create atlas
    let mut atlas = ComponentAtlas::load(unit_name).unwrap_or_else(|| ComponentAtlas {
        component: unit_name.to_string(),
        component_type: "service".to_string(),
        ..Default::default()
    });

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // Get logs for current boot and recent boots
    for boot_offset in 0..max_boots as i32 {
        let boot_arg = if boot_offset == 0 {
            "-b".to_string()
        } else {
            format!("-b -{}", boot_offset)
        };

        let output = Command::new("journalctl")
            .args([
                "-u",
                unit_name,
                &boot_arg,
                "-p",
                "warning..alert",
                "--no-pager",
                "-o",
                "short-iso",
                "-q",
            ])
            .output();

        if let Ok(out) = output {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let mut pattern_counts: HashMap<String, u32> = HashMap::new();

                for line in stdout.lines() {
                    if line.is_empty() {
                        continue;
                    }

                    // Parse severity and message
                    let (severity, message) = parse_journal_line(line);
                    let normalized = normalize_message(&message);

                    // Record in atlas
                    atlas.record_pattern(&severity, &normalized, &message, now, -boot_offset);

                    // Count for current boot display
                    if boot_offset == 0 {
                        let pattern_id = atlas.get_or_create_pattern_id(&normalized);
                        *pattern_counts.entry(pattern_id.clone()).or_insert(0) += 1;

                        // Add to current boot entries if not already there
                        if !summary
                            .current_boot_entries
                            .iter()
                            .any(|e| e.pattern_id == pattern_id)
                        {
                            summary.current_boot_entries.push(BootLogEntry {
                                pattern_id,
                                severity: severity.clone(),
                                message: message.clone(),
                                count_this_boot: 1,
                                timestamp: now,
                                boot_offset: 0,
                            });
                        }
                    }
                }

                // Update counts for current boot
                for entry in &mut summary.current_boot_entries {
                    if let Some(&count) = pattern_counts.get(&entry.pattern_id) {
                        entry.count_this_boot = count;
                    }
                }
            }
        }
    }

    // Sort current boot entries by severity then ID
    summary.current_boot_entries.sort_by(|a, b| {
        severity_priority(&a.severity)
            .cmp(&severity_priority(&b.severity))
            .then(a.pattern_id.cmp(&b.pattern_id))
    });

    // Set historical patterns (those seen in multiple boots)
    summary.historical_patterns = atlas
        .patterns
        .iter()
        .filter(|p| p.boots_seen > 1 || p.boot_ids.iter().any(|&b| b < 0))
        .cloned()
        .collect();

    // Save atlas
    atlas.last_updated = now;
    let _ = atlas.save();

    summary
}

/// Get log atlas for a hardware device (kernel messages)
pub fn get_device_log_atlas(device: &str, max_boots: u32) -> CrossBootLogSummary {
    let mut summary = CrossBootLogSummary {
        component: device.to_string(),
        source: format!("journalctl -k | grep {}", device),
        ..Default::default()
    };

    let mut atlas = ComponentAtlas::load(device).unwrap_or_else(|| ComponentAtlas {
        component: device.to_string(),
        component_type: "device".to_string(),
        ..Default::default()
    });

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // Get kernel logs for current and recent boots
    for boot_offset in 0..max_boots as i32 {
        let boot_arg = if boot_offset == 0 {
            "-b".to_string()
        } else {
            format!("-b -{}", boot_offset)
        };

        let output = Command::new("sh")
            .args([
                "-c",
                &format!(
                    "journalctl -k {} --no-pager -o short-iso -q | grep -i {}",
                    boot_arg, device
                ),
            ])
            .output();

        if let Ok(out) = output {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let mut pattern_counts: HashMap<String, u32> = HashMap::new();

                for line in stdout.lines() {
                    if line.is_empty() {
                        continue;
                    }

                    let (severity, message) = parse_kernel_line(line);
                    let normalized = normalize_message(&message);

                    atlas.record_pattern(&severity, &normalized, &message, now, -boot_offset);

                    if boot_offset == 0 {
                        let pattern_id = atlas.get_or_create_pattern_id(&normalized);
                        *pattern_counts.entry(pattern_id.clone()).or_insert(0) += 1;

                        if !summary
                            .current_boot_entries
                            .iter()
                            .any(|e| e.pattern_id == pattern_id)
                        {
                            summary.current_boot_entries.push(BootLogEntry {
                                pattern_id,
                                severity: severity.clone(),
                                message: message.clone(),
                                count_this_boot: 1,
                                timestamp: now,
                                boot_offset: 0,
                            });
                        }
                    }
                }

                for entry in &mut summary.current_boot_entries {
                    if let Some(&count) = pattern_counts.get(&entry.pattern_id) {
                        entry.count_this_boot = count;
                    }
                }
            }
        }
    }

    summary.current_boot_entries.sort_by(|a, b| {
        severity_priority(&a.severity)
            .cmp(&severity_priority(&b.severity))
            .then(a.pattern_id.cmp(&b.pattern_id))
    });

    summary.historical_patterns = atlas
        .patterns
        .iter()
        .filter(|p| p.boots_seen > 1 || p.boot_ids.iter().any(|&b| b < 0))
        .cloned()
        .collect();

    atlas.last_updated = now;
    let _ = atlas.save();

    summary
}

/// Parse a journalctl short-iso line into severity and message
fn parse_journal_line(line: &str) -> (String, String) {
    // Format: "2025-12-01T14:37:00+0100 hostname unit[PID]: message"
    let parts: Vec<&str> = line.splitn(4, ' ').collect();
    if parts.len() >= 4 {
        let message = parts[3..].join(" ");
        // Try to extract severity from priority prefix if present
        let severity = if message.contains("<error>") || message.contains("error:") {
            "error"
        } else if message.contains("<warning>")
            || message.contains("warning:")
            || message.contains("<warn>")
        {
            "warning"
        } else if message.contains("<alert>") {
            "alert"
        } else if message.contains("<critical>") || message.contains("<crit>") {
            "critical"
        } else {
            "warning" // Default for warning..alert filter
        };
        (severity.to_string(), message)
    } else {
        ("warning".to_string(), line.to_string())
    }
}

/// Parse a kernel log line
fn parse_kernel_line(line: &str) -> (String, String) {
    // Format: "2025-12-01T14:37:00+0100 hostname kernel: message"
    let parts: Vec<&str> = line.splitn(4, ' ').collect();
    if parts.len() >= 4 {
        let message = parts[3..].join(" ");
        let severity = if message.contains("error") || message.contains("failed") {
            "error"
        } else if message.contains("warning") || message.contains("warn") {
            "warning"
        } else {
            "info"
        };
        (severity.to_string(), message)
    } else {
        ("info".to_string(), line.to_string())
    }
}

/// Severity priority for sorting (lower = more severe)
fn severity_priority(severity: &str) -> u8 {
    match severity.to_lowercase().as_str() {
        "emergency" => 0,
        "alert" => 1,
        "critical" | "crit" => 2,
        "error" | "err" => 3,
        "warning" | "warn" => 4,
        "notice" => 5,
        "info" => 6,
        "debug" => 7,
        _ => 8,
    }
}

/// Format timestamp for display
pub fn format_timestamp_short(ts: u64) -> String {
    use chrono::{DateTime, Local, Utc};

    let dt = DateTime::<Utc>::from_timestamp(ts as i64, 0).map(|d| d.with_timezone(&Local));

    match dt {
        Some(d) => d.format("%Y-%m-%d %H:%M").to_string(),
        None => "unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_message() {
        let msg = "2025-12-01T14:37:00.123 connection to 192.168.1.1 failed [1234]";
        let normalized = normalize_message(msg);
        assert!(normalized.contains("%TIMESTAMP%"));
        assert!(normalized.contains("%IP%"));
        assert!(normalized.contains("%PID%"));
        assert!(!normalized.contains("2025"));
        assert!(!normalized.contains("192.168"));
    }

    #[test]
    fn test_severity_priority() {
        assert!(severity_priority("error") < severity_priority("warning"));
        assert!(severity_priority("critical") < severity_priority("error"));
        assert!(severity_priority("warning") < severity_priority("info"));
    }

    #[test]
    fn test_pattern_id_generation() {
        let mut atlas = ComponentAtlas {
            component: "test.service".to_string(),
            component_type: "service".to_string(),
            ..Default::default()
        };

        let id1 = atlas.get_or_create_pattern_id("message one");
        assert_eq!(id1, "S01");

        atlas.patterns.push(LogPattern {
            id: id1.clone(),
            severity: "warning".to_string(),
            normalized: "message one".to_string(),
            full_message: "message one".to_string(),
            first_seen: 0,
            last_seen: 0,
            total_count: 1,
            boots_seen: 1,
            boot_ids: vec![0],
        });

        let id2 = atlas.get_or_create_pattern_id("message two");
        assert_eq!(id2, "S02");

        // Same message should return same ID
        let id1_again = atlas.get_or_create_pattern_id("message one");
        assert_eq!(id1_again, "S01");
    }
}
