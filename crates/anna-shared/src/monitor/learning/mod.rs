//! System Learning - Anna learns your system's normal behavior.
//! v0.0.990: Package tracking, performance learning, behavior analysis.
//!
//! Unlike static thresholds, this module LEARNS what's normal for YOUR system.

mod analysis;
mod types;

pub use analysis::{
    detect_package_tool, get_boot_time, parse_alpm_action,
    parse_meminfo_value, MAX_HISTORY,
};
pub use types::{
    DailySnapshot, DetectedChanges, IoBaseline, LongTermHistory, PackageAction, PackageTransaction,
    PerfSample, SystemLearning,
};

use std::path::PathBuf;
use types::PackageAction as PA;

impl SystemLearning {
    /// Path to learning data (system-wide)
    pub fn path() -> PathBuf {
        crate::paths::paths().learning_file()
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
                                    PA::Installed => {
                                        new_packages.extend(packages.clone());
                                        self.package_history.push_back(PackageTransaction {
                                            timestamp: timestamp.clone(),
                                            action: PA::Installed,
                                            packages,
                                            tool: detect_package_tool(),
                                        });
                                    }
                                    PA::Removed => {
                                        self.package_history.push_back(PackageTransaction {
                                            timestamp: timestamp.clone(),
                                            action: PA::Removed,
                                            packages,
                                            tool: detect_package_tool(),
                                        });
                                    }
                                    PA::Upgraded => {
                                        self.package_history.push_back(PackageTransaction {
                                            timestamp: timestamp.clone(),
                                            action: PA::Upgraded,
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
                    if name == "sda"
                        || name == "nvme0n1"
                        || name == "vda"
                        || name.starts_with("sd") && name.len() == 3
                    {
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
                if line.contains("eth")
                    || line.contains("wlan")
                    || line.contains("enp")
                    || line.contains("wlp")
                {
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
}
