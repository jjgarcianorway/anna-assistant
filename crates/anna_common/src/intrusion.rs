//! Intrusion Detection v5.2.0 - Security Pattern Matching
//!
//! Detect potential security issues and intrusion attempts:
//! - Failed SSH/authentication attempts
//! - Sudo abuse patterns
//! - Suspicious process activity
//! - Privilege escalation attempts
//! - Port scanning detection
//! - Unusual file access patterns

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// Constants
// ============================================================================

/// Intrusion store path
pub const INTRUSION_STORE_PATH: &str = "/var/lib/anna/knowledge/intrusion_v5.json";

/// Threshold for failed auth alerts (within 1 hour)
pub const FAILED_AUTH_THRESHOLD: u32 = 5;

/// Threshold for sudo failures (within 1 hour)
pub const SUDO_FAILURE_THRESHOLD: u32 = 3;

// ============================================================================
// Intrusion Type
// ============================================================================

/// Type of detected intrusion pattern
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntrusionType {
    /// Multiple failed SSH/login attempts
    FailedAuth,
    /// Failed sudo attempts
    SudoAbuse,
    /// Potential brute force attack
    BruteForce,
    /// Privilege escalation attempt
    PrivilegeEscalation,
    /// Port scanning detected
    PortScan,
    /// Suspicious binary execution
    SuspiciousBinary,
    /// Unauthorized file access
    UnauthorizedAccess,
    /// Rootkit indicators
    RootkitIndicator,
    /// Known malware signature
    MalwareSignature,
    /// Unusual network activity
    NetworkAnomaly,
    /// Configuration tampering
    ConfigTampering,
    /// Log tampering attempt
    LogTampering,
}

impl IntrusionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            IntrusionType::FailedAuth => "failed_auth",
            IntrusionType::SudoAbuse => "sudo_abuse",
            IntrusionType::BruteForce => "brute_force",
            IntrusionType::PrivilegeEscalation => "privilege_escalation",
            IntrusionType::PortScan => "port_scan",
            IntrusionType::SuspiciousBinary => "suspicious_binary",
            IntrusionType::UnauthorizedAccess => "unauthorized_access",
            IntrusionType::RootkitIndicator => "rootkit_indicator",
            IntrusionType::MalwareSignature => "malware_signature",
            IntrusionType::NetworkAnomaly => "network_anomaly",
            IntrusionType::ConfigTampering => "config_tampering",
            IntrusionType::LogTampering => "log_tampering",
        }
    }

    /// Severity level (1-5)
    pub fn severity(&self) -> u8 {
        match self {
            IntrusionType::FailedAuth => 2,
            IntrusionType::SudoAbuse => 3,
            IntrusionType::BruteForce => 4,
            IntrusionType::PrivilegeEscalation => 5,
            IntrusionType::PortScan => 2,
            IntrusionType::SuspiciousBinary => 4,
            IntrusionType::UnauthorizedAccess => 3,
            IntrusionType::RootkitIndicator => 5,
            IntrusionType::MalwareSignature => 5,
            IntrusionType::NetworkAnomaly => 2,
            IntrusionType::ConfigTampering => 4,
            IntrusionType::LogTampering => 5,
        }
    }

    /// Color for terminal display
    pub fn color_code(&self) -> &'static str {
        match self.severity() {
            5 => "red_bold",
            4 => "red",
            3 => "yellow",
            2 => "yellow_dim",
            _ => "default",
        }
    }
}

// ============================================================================
// Intrusion Pattern
// ============================================================================

/// Pattern for detecting intrusions from log messages
#[derive(Debug, Clone)]
pub struct IntrusionPattern {
    /// Pattern name
    pub name: &'static str,
    /// Type of intrusion
    pub intrusion_type: IntrusionType,
    /// Regex patterns to match
    pub patterns: &'static [&'static str],
    /// Description
    pub description: &'static str,
}

/// Built-in intrusion detection patterns
pub const INTRUSION_PATTERNS: &[IntrusionPattern] = &[
    IntrusionPattern {
        name: "ssh_failed_auth",
        intrusion_type: IntrusionType::FailedAuth,
        patterns: &[
            "Failed password for",
            "Failed publickey for",
            "authentication failure",
            "Invalid user",
            "Connection closed by authenticating user",
            "Disconnected from authenticating user",
        ],
        description: "Failed SSH authentication attempts",
    },
    IntrusionPattern {
        name: "sudo_failures",
        intrusion_type: IntrusionType::SudoAbuse,
        patterns: &[
            "incorrect password attempts",
            "NOT in sudoers",
            "user NOT in sudoers",
            "3 incorrect password attempts",
            "authentication failure.*sudo",
        ],
        description: "Failed sudo/privilege elevation attempts",
    },
    IntrusionPattern {
        name: "privilege_escalation",
        intrusion_type: IntrusionType::PrivilegeEscalation,
        patterns: &[
            "COMMAND=/bin/su",
            "COMMAND=/usr/bin/su",
            "setuid.*root",
            "privilege escalation",
            "Insecure setuid",
        ],
        description: "Privilege escalation attempts",
    },
    IntrusionPattern {
        name: "port_scanning",
        intrusion_type: IntrusionType::PortScan,
        patterns: &[
            "SYN flood",
            "port scan",
            "Connection from.*refused",
            "possible SYN flooding",
            "TCP: Possible SYN flooding",
        ],
        description: "Port scanning or SYN flood attacks",
    },
    IntrusionPattern {
        name: "rootkit_indicators",
        intrusion_type: IntrusionType::RootkitIndicator,
        patterns: &[
            "hidden process",
            "hidden file",
            "/dev/shm.*deleted",
            "ld.so.preload",
            "libkeyutils",
            "/proc/.*/exe.*deleted",
        ],
        description: "Rootkit or process hiding indicators",
    },
    IntrusionPattern {
        name: "config_tampering",
        intrusion_type: IntrusionType::ConfigTampering,
        patterns: &[
            "/etc/passwd.*modified",
            "/etc/shadow.*modified",
            "/etc/sudoers.*modified",
            "sshd_config.*modified",
            "PAM configuration changed",
        ],
        description: "Critical configuration file modifications",
    },
    IntrusionPattern {
        name: "log_tampering",
        intrusion_type: IntrusionType::LogTampering,
        patterns: &[
            "log file.*truncated",
            "/var/log.*deleted",
            "journal.*corrupted",
            "rsyslog.*terminated",
            "Logs cleared",
        ],
        description: "Log file tampering or deletion",
    },
    IntrusionPattern {
        name: "suspicious_binaries",
        intrusion_type: IntrusionType::SuspiciousBinary,
        patterns: &[
            "nc -e",
            "bash -i",
            "/dev/tcp",
            "wget.*\\|.*sh",
            "curl.*\\|.*sh",
            "base64.*-d.*\\|.*sh",
            "python.*-c.*import socket",
        ],
        description: "Suspicious binary execution patterns",
    },
    IntrusionPattern {
        name: "brute_force",
        intrusion_type: IntrusionType::BruteForce,
        patterns: &[
            "maximum authentication attempts",
            "Too many authentication failures",
            "PAM.*failure.*repeated",
            "error: maximum authentication attempts exceeded",
        ],
        description: "Brute force attack indicators",
    },
    IntrusionPattern {
        name: "network_anomaly",
        intrusion_type: IntrusionType::NetworkAnomaly,
        patterns: &[
            "martian source",
            "spoofed",
            "impossible",
            "suspicious packet",
            "land attack",
        ],
        description: "Network packet anomalies",
    },
];

// ============================================================================
// Intrusion Event
// ============================================================================

/// A detected intrusion event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntrusionEvent {
    /// Timestamp of detection
    pub timestamp: u64,

    /// Type of intrusion
    pub intrusion_type: IntrusionType,

    /// Pattern that matched
    pub pattern_name: String,

    /// Source (log file, service, etc.)
    pub source: String,

    /// Associated object (if any)
    pub object_name: Option<String>,

    /// Raw log message
    pub message: String,

    /// Source IP (if applicable)
    pub source_ip: Option<String>,

    /// Username involved (if applicable)
    pub username: Option<String>,

    /// Severity (1-5)
    pub severity: u8,
}

impl IntrusionEvent {
    pub fn new(
        intrusion_type: IntrusionType,
        pattern_name: &str,
        message: String,
        source: &str,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let severity = intrusion_type.severity();

        Self {
            timestamp: now,
            intrusion_type,
            pattern_name: pattern_name.to_string(),
            source: source.to_string(),
            object_name: None,
            message,
            source_ip: None,
            username: None,
            severity,
        }
    }

    /// Extract source IP from message
    pub fn extract_source_ip(&mut self) {
        // Common patterns for IP addresses in logs
        let ip_patterns = [
            r"from\s+(\d+\.\d+\.\d+\.\d+)",
            r"(\d+\.\d+\.\d+\.\d+)\s+port",
            r"SRC=(\d+\.\d+\.\d+\.\d+)",
            r"rhost=(\d+\.\d+\.\d+\.\d+)",
        ];

        for pattern in &ip_patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if let Some(caps) = re.captures(&self.message) {
                    if let Some(ip) = caps.get(1) {
                        self.source_ip = Some(ip.as_str().to_string());
                        return;
                    }
                }
            }
        }
    }

    /// Extract username from message
    pub fn extract_username(&mut self) {
        let user_patterns = [
            r"user[=:\s]+(\w+)",
            r"for\s+(\w+)\s+from",
            r"Invalid user (\w+)",
            r"USER=(\w+)",
        ];

        for pattern in &user_patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if let Some(caps) = re.captures(&self.message) {
                    if let Some(user) = caps.get(1) {
                        let username = user.as_str().to_string();
                        // Filter out common false positives
                        if !["from", "root", "for"].contains(&username.as_str()) {
                            self.username = Some(username);
                            return;
                        }
                    }
                }
            }
        }
    }

    /// Format for display
    pub fn format_short(&self) -> String {
        let ts = chrono::DateTime::from_timestamp(self.timestamp as i64, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let ip = self
            .source_ip
            .as_deref()
            .map(|ip| format!(" [{}]", ip))
            .unwrap_or_default();

        format!(
            "[{}] {} (severity: {}){}: {}",
            ts,
            self.intrusion_type.as_str(),
            self.severity,
            ip,
            self.message
        )
    }
}

// ============================================================================
// Object Intrusions (per-object intrusion collection)
// ============================================================================

/// Intrusion events for a single object
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ObjectIntrusions {
    /// Object name
    pub object_name: String,

    /// Intrusion events
    pub events: Vec<IntrusionEvent>,

    /// Event count by type
    pub type_counts: HashMap<String, u64>,

    /// Highest severity seen
    pub max_severity: u8,

    /// First indexed timestamp
    pub first_indexed_at: u64,

    /// Last indexed timestamp
    pub last_indexed_at: u64,
}

impl ObjectIntrusions {
    pub fn new(name: &str) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            object_name: name.to_string(),
            events: Vec::new(),
            type_counts: HashMap::new(),
            max_severity: 0,
            first_indexed_at: now,
            last_indexed_at: now,
        }
    }

    /// Add an intrusion event
    pub fn add_event(&mut self, event: IntrusionEvent) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Update counts
        *self
            .type_counts
            .entry(event.intrusion_type.as_str().to_string())
            .or_insert(0) += 1;

        // Update max severity
        if event.severity > self.max_severity {
            self.max_severity = event.severity;
        }

        self.events.push(event);
        self.last_indexed_at = now;

        // Cap events at 50
        if self.events.len() > 50 {
            self.events.remove(0);
        }
    }

    /// Total event count
    pub fn total_events(&self) -> usize {
        self.events.len()
    }

    /// Get events by severity threshold
    pub fn events_above_severity(&self, min_severity: u8) -> Vec<&IntrusionEvent> {
        self.events
            .iter()
            .filter(|e| e.severity >= min_severity)
            .collect()
    }
}

// ============================================================================
// Intrusion Index (global intrusion database)
// ============================================================================

/// Global intrusion detection index
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IntrusionIndex {
    /// Intrusions by object name
    pub objects: HashMap<String, ObjectIntrusions>,

    /// Global events (not tied to specific object)
    pub global_events: Vec<IntrusionEvent>,

    /// Total events detected
    pub total_events: u64,

    /// Events by type (global)
    pub type_counts: HashMap<String, u64>,

    /// Created at timestamp
    pub created_at: u64,

    /// Last updated timestamp
    pub last_updated: u64,
}

impl IntrusionIndex {
    pub fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            objects: HashMap::new(),
            global_events: Vec::new(),
            total_events: 0,
            type_counts: HashMap::new(),
            created_at: now,
            last_updated: now,
        }
    }

    /// Load from disk
    pub fn load() -> Self {
        if let Ok(content) = fs::read_to_string(INTRUSION_STORE_PATH) {
            serde_json::from_str(&content).unwrap_or_else(|_| Self::new())
        } else {
            Self::new()
        }
    }

    /// Save to disk
    pub fn save(&self) -> std::io::Result<()> {
        if let Some(parent) = Path::new(INTRUSION_STORE_PATH).parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(INTRUSION_STORE_PATH, json)
    }

    /// Check a log message against all patterns
    pub fn check_message(&mut self, message: &str, source: &str, object_name: Option<&str>) {
        let lower_message = message.to_lowercase();

        for pattern in INTRUSION_PATTERNS {
            for pat in pattern.patterns {
                if lower_message.contains(&pat.to_lowercase()) {
                    let mut event =
                        IntrusionEvent::new(pattern.intrusion_type.clone(), pattern.name, message.to_string(), source);

                    event.extract_source_ip();
                    event.extract_username();

                    if let Some(name) = object_name {
                        event.object_name = Some(name.to_string());
                        let obj_intrusions = self
                            .objects
                            .entry(name.to_string())
                            .or_insert_with(|| ObjectIntrusions::new(name));
                        obj_intrusions.add_event(event.clone());
                    } else {
                        self.global_events.push(event.clone());
                        if self.global_events.len() > 100 {
                            self.global_events.remove(0);
                        }
                    }

                    // Update global counts
                    self.total_events += 1;
                    *self
                        .type_counts
                        .entry(pattern.intrusion_type.as_str().to_string())
                        .or_insert(0) += 1;

                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    self.last_updated = now;

                    break; // Only match first pattern per message
                }
            }
        }
    }

    /// Get object intrusions
    pub fn get_object_intrusions(&self, name: &str) -> Option<&ObjectIntrusions> {
        self.objects.get(name)
    }

    /// Get recent high-severity events
    pub fn recent_high_severity(&self, within_secs: u64, min_severity: u8) -> Vec<&IntrusionEvent> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let cutoff = now.saturating_sub(within_secs);

        let mut events: Vec<_> = self
            .global_events
            .iter()
            .filter(|e| e.timestamp >= cutoff && e.severity >= min_severity)
            .collect();

        for obj in self.objects.values() {
            events.extend(
                obj.events
                    .iter()
                    .filter(|e| e.timestamp >= cutoff && e.severity >= min_severity),
            );
        }

        events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        events
    }

    /// Clear all data
    pub fn clear(&mut self) {
        self.objects.clear();
        self.global_events.clear();
        self.total_events = 0;
        self.type_counts.clear();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.last_updated = now;
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intrusion_type_severity() {
        assert!(IntrusionType::PrivilegeEscalation.severity() > IntrusionType::FailedAuth.severity());
        assert!(IntrusionType::RootkitIndicator.severity() == 5);
    }

    #[test]
    fn test_pattern_matching() {
        let mut index = IntrusionIndex::new();

        index.check_message(
            "Failed password for invalid user admin from 192.168.1.100 port 22",
            "sshd",
            Some("sshd"),
        );

        assert_eq!(index.total_events, 1);
        assert!(index.get_object_intrusions("sshd").is_some());
    }

    #[test]
    fn test_ip_extraction() {
        let mut event = IntrusionEvent::new(
            IntrusionType::FailedAuth,
            "test",
            "Failed password from 192.168.1.100 port 22".to_string(),
            "sshd",
        );

        event.extract_source_ip();
        assert_eq!(event.source_ip, Some("192.168.1.100".to_string()));
    }

    #[test]
    fn test_username_extraction() {
        let mut event = IntrusionEvent::new(
            IntrusionType::FailedAuth,
            "test",
            "Invalid user admin from 192.168.1.100".to_string(),
            "sshd",
        );

        event.extract_username();
        assert_eq!(event.username, Some("admin".to_string()));
    }

    #[test]
    fn test_sudo_pattern() {
        let mut index = IntrusionIndex::new();

        index.check_message(
            "user NOT in sudoers; TTY=pts/0; PWD=/home/user",
            "sudo",
            Some("sudo"),
        );

        assert_eq!(index.total_events, 1);
        let obj = index.get_object_intrusions("sudo").unwrap();
        assert_eq!(obj.events[0].intrusion_type, IntrusionType::SudoAbuse);
    }

    #[test]
    fn test_object_intrusions_cap() {
        let mut obj = ObjectIntrusions::new("test");

        for i in 0..60 {
            let event = IntrusionEvent::new(
                IntrusionType::FailedAuth,
                "test",
                format!("Event {}", i),
                "test",
            );
            obj.add_event(event);
        }

        assert!(obj.events.len() <= 50);
    }
}
