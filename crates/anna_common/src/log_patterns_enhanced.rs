//! Enhanced Log Patterns v7.18.0 - Pattern IDs and Novelty Detection
//!
//! Extends log pattern analysis with:
//! - Stable pattern IDs (hash of service + priority + normalized message)
//! - Baseline tracking across boots
//! - Novelty detection ("new this boot" vs "known pattern")
//!
//! Sources:
//! - journalctl -b (current and previous boots)
//! - Stored pattern history in /var/lib/anna/telemetry/logs

use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::Command;

use serde::{Deserialize, Serialize};

/// Directory for log pattern storage
pub const LOG_PATTERNS_DIR: &str = "/var/lib/anna/telemetry/logs";

/// A log pattern with stable ID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogPattern {
    /// Stable pattern ID (hash)
    pub id: String,
    /// Service/unit name
    pub service: String,
    /// Priority (error, warning, etc.)
    pub priority: String,
    /// Normalized message template
    pub template: String,
}

impl LogPattern {
    /// Create a new pattern from components
    pub fn new(service: &str, priority: &str, template: &str) -> Self {
        let id = compute_pattern_id(service, priority, template);
        Self {
            id,
            service: service.to_string(),
            priority: priority.to_string(),
            template: template.to_string(),
        }
    }

    /// Short ID for display (first 4 chars)
    pub fn short_id(&self) -> &str {
        &self.id[..4.min(self.id.len())]
    }
}

/// Compute stable pattern ID from components
fn compute_pattern_id(service: &str, priority: &str, template: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    service.hash(&mut hasher);
    priority.hash(&mut hasher);
    template.hash(&mut hasher);

    format!("{:08x}", hasher.finish() as u32)
}

/// Pattern occurrence with counts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternOccurrence {
    /// The pattern
    pub pattern: LogPattern,
    /// Count in current boot
    pub count_this_boot: u32,
    /// Count in last 24h (may span boots)
    pub count_24h: u32,
    /// Count in last 7d
    pub count_7d: u32,
    /// Number of boots where this pattern appeared
    pub boots_seen: u32,
    /// Whether this is new this boot (not seen in previous boots)
    pub is_new: bool,
}

/// Pattern history stored per boot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootPatternHistory {
    /// Boot ID
    pub boot_id: String,
    /// Timestamp
    pub timestamp: u64,
    /// Pattern IDs seen in this boot
    pub pattern_ids: HashSet<String>,
    /// Counts per pattern ID
    pub pattern_counts: HashMap<String, u32>,
}

/// Enhanced log pattern analyzer
pub struct LogPatternAnalyzer {
    /// Known patterns from previous boots
    known_patterns: HashSet<String>,
    /// Pattern counts per boot
    boot_histories: Vec<BootPatternHistory>,
}

impl LogPatternAnalyzer {
    /// Create analyzer, loading history from disk
    pub fn new() -> Self {
        let (known_patterns, boot_histories) = load_pattern_history();
        Self {
            known_patterns,
            boot_histories,
        }
    }

    /// Get patterns for a service with novelty detection
    pub fn get_patterns_for_service(&self, service: &str) -> ServicePatternSummary {
        let current_boot = get_patterns_for_boot("0", service);
        let previous_boot = get_patterns_for_boot("-1", service);

        // Build set of pattern IDs seen before this boot
        let mut known_before_this_boot: HashSet<String> = HashSet::new();
        for history in &self.boot_histories {
            known_before_this_boot.extend(history.pattern_ids.iter().cloned());
        }

        // Also add patterns from previous boot scan
        for (pattern, _, _) in &previous_boot {
            known_before_this_boot.insert(pattern.id.clone());
        }

        // Classify current boot patterns
        let mut current_patterns = Vec::new();
        let mut new_patterns = Vec::new();
        let mut known_patterns = Vec::new();

        for (pattern, count, _) in current_boot {
            let is_new = !known_before_this_boot.contains(&pattern.id);

            // Get 7d count from history
            let count_7d = self.get_7d_count(&pattern.id) + count;

            let occurrence = PatternOccurrence {
                pattern: pattern.clone(),
                count_this_boot: count,
                count_24h: count, // TODO: proper 24h calculation
                count_7d,
                boots_seen: self.get_boots_seen(&pattern.id) + 1,
                is_new,
            };

            current_patterns.push(occurrence.clone());

            if is_new {
                new_patterns.push(occurrence);
            } else {
                known_patterns.push(occurrence);
            }
        }

        // Get patterns from previous boot
        let previous_patterns: Vec<PatternOccurrence> = previous_boot
            .into_iter()
            .map(|(pattern, count, _)| {
                let count_7d = self.get_7d_count(&pattern.id);
                let boots_seen = self.get_boots_seen(&pattern.id);
                PatternOccurrence {
                    pattern,
                    count_this_boot: 0,
                    count_24h: count,
                    count_7d,
                    boots_seen,
                    is_new: false,
                }
            })
            .collect();

        ServicePatternSummary {
            service: service.to_string(),
            current_boot: current_patterns,
            new_this_boot: new_patterns,
            known_patterns,
            previous_boot: previous_patterns,
        }
    }

    /// Get 7d count for a pattern from history
    fn get_7d_count(&self, pattern_id: &str) -> u32 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let week_ago = now.saturating_sub(7 * 24 * 3600);

        self.boot_histories
            .iter()
            .filter(|h| h.timestamp >= week_ago)
            .map(|h| h.pattern_counts.get(pattern_id).copied().unwrap_or(0))
            .sum()
    }

    /// Get number of boots where pattern was seen
    fn get_boots_seen(&self, pattern_id: &str) -> u32 {
        self.boot_histories
            .iter()
            .filter(|h| h.pattern_ids.contains(pattern_id))
            .count() as u32
    }

    /// Save current boot's patterns to history
    pub fn save_current_boot_patterns(&mut self, service: &str) {
        let boot_id = get_current_boot_id().unwrap_or_else(|| "unknown".to_string());
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let patterns = get_patterns_for_boot("0", service);

        let mut pattern_ids = HashSet::new();
        let mut pattern_counts = HashMap::new();

        for (pattern, count, _) in patterns {
            pattern_ids.insert(pattern.id.clone());
            pattern_counts.insert(pattern.id, count);
        }

        let history = BootPatternHistory {
            boot_id,
            timestamp,
            pattern_ids,
            pattern_counts,
        };

        // Update in-memory history
        self.boot_histories.push(history.clone());

        // Prune old entries (keep last 30 days)
        let month_ago = timestamp.saturating_sub(30 * 24 * 3600);
        self.boot_histories.retain(|h| h.timestamp >= month_ago);

        // Update known patterns
        for id in &self.boot_histories.last().unwrap().pattern_ids {
            self.known_patterns.insert(id.clone());
        }

        // Save to disk
        save_pattern_history(&self.boot_histories);
    }
}

impl Default for LogPatternAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of patterns for a service
#[derive(Debug, Clone)]
pub struct ServicePatternSummary {
    pub service: String,
    /// All patterns in current boot
    pub current_boot: Vec<PatternOccurrence>,
    /// Patterns new this boot (not seen before)
    pub new_this_boot: Vec<PatternOccurrence>,
    /// Known patterns (seen in previous boots)
    pub known_patterns: Vec<PatternOccurrence>,
    /// Patterns from previous boot
    pub previous_boot: Vec<PatternOccurrence>,
}

/// Get patterns for a specific boot
fn get_patterns_for_boot(boot_idx: &str, service: &str) -> Vec<(LogPattern, u32, String)> {
    let mut pattern_counts: HashMap<String, (LogPattern, u32, String)> = HashMap::new();

    let unit_name = if service.ends_with(".service") {
        service.to_string()
    } else {
        format!("{}.service", service)
    };

    let output = Command::new("journalctl")
        .args(["-b", boot_idx, "-u", &unit_name, "-p", "warning..alert", "--no-pager"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);

            for line in stdout.lines() {
                let (priority, template, raw_line) = parse_and_normalize_log_line(line);
                if !template.is_empty() {
                    let pattern = LogPattern::new(service, &priority, &template);

                    pattern_counts
                        .entry(pattern.id.clone())
                        .and_modify(|(_, count, _)| *count += 1)
                        .or_insert((pattern, 1, raw_line));
                }
            }
        }
    }

    pattern_counts.into_values().collect()
}

/// Parse and normalize a log line
fn parse_and_normalize_log_line(line: &str) -> (String, String, String) {
    // Determine priority
    let priority = if line.contains("error") || line.contains("Error") || line.contains("ERROR") {
        "error"
    } else if line.contains("crit") || line.contains("CRIT") || line.contains("Critical") {
        "critical"
    } else {
        "warning"
    };

    // Extract message part (skip timestamp and hostname)
    let parts: Vec<&str> = line.splitn(5, ' ').collect();
    let message = if parts.len() >= 5 {
        parts[4..].join(" ")
    } else if parts.len() >= 3 {
        parts[2..].join(" ")
    } else {
        line.to_string()
    };

    // Normalize volatile parts
    let template = normalize_message(&message);

    (priority.to_string(), template, message)
}

/// Normalize message by replacing volatile values with placeholders
fn normalize_message(message: &str) -> String {
    let mut result = message.to_string();

    // IP addresses -> %IP%
    let ip_re = regex::Regex::new(r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}").unwrap();
    result = ip_re.replace_all(&result, "%IP%").to_string();

    // MAC addresses -> %MAC%
    let mac_re = regex::Regex::new(r"([0-9a-fA-F]{2}:){5}[0-9a-fA-F]{2}").unwrap();
    result = mac_re.replace_all(&result, "%MAC%").to_string();

    // Port numbers -> %PORT%
    let port_re = regex::Regex::new(r":\d{2,5}\b").unwrap();
    result = port_re.replace_all(&result, ":%PORT%").to_string();

    // PIDs -> %PID%
    let pid_re = regex::Regex::new(r"\b(pid|PID)[=: ]\d+").unwrap();
    result = pid_re.replace_all(&result, "pid=%PID%").to_string();

    // Hex addresses -> %ADDR%
    let hex_re = regex::Regex::new(r"0x[0-9a-fA-F]+").unwrap();
    result = hex_re.replace_all(&result, "%ADDR%").to_string();

    // UUIDs -> %UUID%
    let uuid_re = regex::Regex::new(
        r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}"
    ).unwrap();
    result = uuid_re.replace_all(&result, "%UUID%").to_string();

    // Interface names (like wlp0s20f3) -> %IFACE%
    let iface_re = regex::Regex::new(r"\b(wl|en|eth|wlan|enp|wlp)[a-z0-9]+\b").unwrap();
    result = iface_re.replace_all(&result, "%IFACE%").to_string();

    // Domain names -> %DOMAIN%
    let domain_re = regex::Regex::new(r"\b[a-zA-Z0-9][-a-zA-Z0-9]*\.[a-zA-Z]{2,}\b").unwrap();
    result = domain_re.replace_all(&result, "%DOMAIN%").to_string();

    result
}

/// Get current boot ID
fn get_current_boot_id() -> Option<String> {
    let output = Command::new("journalctl")
        .args(["--list-boots", "-n", "1", "--no-pager"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout.lines().last()?;
    let parts: Vec<&str> = line.split_whitespace().collect();

    parts.get(1).map(|s| s.to_string())
}

/// Load pattern history from disk
fn load_pattern_history() -> (HashSet<String>, Vec<BootPatternHistory>) {
    let mut known_patterns = HashSet::new();
    let mut histories = Vec::new();

    let history_file = format!("{}/history.jsonl", LOG_PATTERNS_DIR);

    if let Ok(file) = File::open(&history_file) {
        let reader = BufReader::new(file);
        for line in reader.lines().map_while(Result::ok) {
            if let Ok(history) = serde_json::from_str::<BootPatternHistory>(&line) {
                for id in &history.pattern_ids {
                    known_patterns.insert(id.clone());
                }
                histories.push(history);
            }
        }
    }

    (known_patterns, histories)
}

/// Save pattern history to disk
fn save_pattern_history(histories: &[BootPatternHistory]) {
    let dir = Path::new(LOG_PATTERNS_DIR);
    if fs::create_dir_all(dir).is_err() {
        return;
    }

    let history_file = format!("{}/history.jsonl", LOG_PATTERNS_DIR);

    if let Ok(mut file) = File::create(&history_file) {
        for history in histories {
            if let Ok(json) = serde_json::to_string(history) {
                let _ = writeln!(file, "{}", json);
            }
        }
    }
}

/// Get error and warning counts for a service in current boot
pub fn get_service_log_counts(service: &str) -> (u32, u32) {
    let unit_name = if service.ends_with(".service") {
        service.to_string()
    } else {
        format!("{}.service", service)
    };

    let mut errors = 0u32;
    let mut warnings = 0u32;

    // Count errors
    let output = Command::new("journalctl")
        .args(["-b", "0", "-u", &unit_name, "-p", "err", "--no-pager"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            errors = String::from_utf8_lossy(&out.stdout).lines().count() as u32;
        }
    }

    // Count warnings (excluding errors)
    let output = Command::new("journalctl")
        .args(["-b", "0", "-u", &unit_name, "-p", "warning", "--no-pager"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            warnings = String::from_utf8_lossy(&out.stdout).lines().count() as u32;
            warnings = warnings.saturating_sub(errors); // Don't double-count
        }
    }

    (errors, warnings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_pattern_id() {
        let id1 = compute_pattern_id("sshd", "error", "connection refused");
        let id2 = compute_pattern_id("sshd", "error", "connection refused");
        let id3 = compute_pattern_id("sshd", "warning", "connection refused");

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_normalize_message() {
        let msg = "Connection from 192.168.1.1:8080 failed for user@example.com";
        let normalized = normalize_message(msg);

        assert!(normalized.contains("%IP%"));
        assert!(normalized.contains("%PORT%"));
        assert!(normalized.contains("%DOMAIN%"));
    }

    #[test]
    fn test_log_pattern_short_id() {
        let pattern = LogPattern::new("test", "error", "test message");
        assert_eq!(pattern.short_id().len(), 4);
    }
}
