//! Live System State - Real-time snapshot of system health.
//!
//! Provides current system metrics for contextual LLM prompts.
//! Makes Anna aware of current conditions when answering questions.

use serde::{Deserialize, Serialize};
use std::process::Command;

/// Live system state snapshot.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LiveState {
    /// CPU load averages (1, 5, 15 minute)
    pub load_avg: (f32, f32, f32),
    /// Memory: used / total in GB
    pub memory: MemoryState,
    /// Disk usage for root partition
    pub disk: DiskState,
    /// Number of running processes
    pub process_count: u32,
    /// Uptime in hours
    pub uptime_hours: f32,
    /// Failed systemd units
    pub failed_units: Vec<String>,
    /// High CPU processes (> 50%)
    pub high_cpu_procs: Vec<String>,
    /// Network connectivity status
    pub network_status: NetworkStatus,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryState {
    pub used_gb: f32,
    pub total_gb: f32,
    pub swap_used_gb: f32,
    pub swap_total_gb: f32,
}

impl MemoryState {
    pub fn percent_used(&self) -> f32 {
        if self.total_gb > 0.0 {
            (self.used_gb / self.total_gb) * 100.0
        } else {
            0.0
        }
    }

    pub fn swap_percent_used(&self) -> f32 {
        if self.swap_total_gb > 0.0 {
            (self.swap_used_gb / self.swap_total_gb) * 100.0
        } else {
            0.0
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiskState {
    pub mount_point: String,
    pub used_gb: f32,
    pub total_gb: f32,
}

impl DiskState {
    pub fn percent_used(&self) -> f32 {
        if self.total_gb > 0.0 {
            (self.used_gb / self.total_gb) * 100.0
        } else {
            0.0
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum NetworkStatus {
    #[default]
    Unknown,
    Connected { interface: String, ip: String },
    Disconnected,
}

impl LiveState {
    /// Capture current system state.
    pub fn capture() -> Self {
        let mut state = Self::default();

        state.load_avg = read_load_avg();
        state.memory = read_memory();
        state.disk = read_disk("/");
        state.process_count = read_process_count();
        state.uptime_hours = read_uptime();
        state.failed_units = read_failed_units();
        state.high_cpu_procs = read_high_cpu_procs();
        state.network_status = check_network();

        state
    }

    /// Generate a concise summary for LLM context.
    pub fn summary(&self) -> String {
        let mut lines = Vec::new();

        // Load average
        let load_status = if self.load_avg.0 > 2.0 {
            "high load"
        } else if self.load_avg.0 > 1.0 {
            "moderate load"
        } else {
            "normal"
        };
        lines.push(format!(
            "CPU: {} (load {:.1}/{:.1}/{:.1})",
            load_status, self.load_avg.0, self.load_avg.1, self.load_avg.2
        ));

        // Memory
        let mem_pct = self.memory.percent_used();
        let mem_status = if mem_pct > 90.0 {
            "critical"
        } else if mem_pct > 80.0 {
            "high"
        } else {
            "normal"
        };
        lines.push(format!(
            "Memory: {} ({:.1}GB/{:.1}GB, {:.0}%)",
            mem_status, self.memory.used_gb, self.memory.total_gb, mem_pct
        ));

        // Swap if used
        if self.memory.swap_percent_used() > 10.0 {
            lines.push(format!(
                "Swap: {:.1}GB/{:.1}GB ({:.0}%)",
                self.memory.swap_used_gb,
                self.memory.swap_total_gb,
                self.memory.swap_percent_used()
            ));
        }

        // Disk
        let disk_pct = self.disk.percent_used();
        let disk_status = if disk_pct > 90.0 {
            "critical"
        } else if disk_pct > 80.0 {
            "warning"
        } else {
            "normal"
        };
        lines.push(format!(
            "Disk: {} ({:.0}GB/{:.0}GB, {:.0}%)",
            disk_status, self.disk.used_gb, self.disk.total_gb, disk_pct
        ));

        // Failed units
        if !self.failed_units.is_empty() {
            let count = self.failed_units.len();
            let names = self.failed_units.iter().take(3).cloned().collect::<Vec<_>>().join(", ");
            if count > 3 {
                lines.push(format!("Failed services: {} (and {} more)", names, count - 3));
            } else {
                lines.push(format!("Failed services: {}", names));
            }
        }

        // Network
        match &self.network_status {
            NetworkStatus::Connected { interface, ip } => {
                lines.push(format!("Network: connected ({} {})", interface, ip));
            }
            NetworkStatus::Disconnected => {
                lines.push("Network: disconnected".to_string());
            }
            NetworkStatus::Unknown => {}
        }

        // Uptime
        if self.uptime_hours > 24.0 {
            lines.push(format!("Uptime: {:.0} days", self.uptime_hours / 24.0));
        }

        lines.join(", ")
    }

    /// Check if system is under stress.
    pub fn is_stressed(&self) -> bool {
        self.load_avg.0 > 2.0
            || self.memory.percent_used() > 90.0
            || self.disk.percent_used() > 95.0
            || !self.failed_units.is_empty()
    }
}

fn read_load_avg() -> (f32, f32, f32) {
    if let Ok(content) = std::fs::read_to_string("/proc/loadavg") {
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() >= 3 {
            return (
                parts[0].parse().unwrap_or(0.0),
                parts[1].parse().unwrap_or(0.0),
                parts[2].parse().unwrap_or(0.0),
            );
        }
    }
    (0.0, 0.0, 0.0)
}

fn read_memory() -> MemoryState {
    let mut state = MemoryState::default();

    if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
        let mut mem_total: u64 = 0;
        let mut mem_available: u64 = 0;
        let mut swap_total: u64 = 0;
        let mut swap_free: u64 = 0;

        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let value: u64 = parts[1].parse().unwrap_or(0);
                match parts[0] {
                    "MemTotal:" => mem_total = value,
                    "MemAvailable:" => mem_available = value,
                    "SwapTotal:" => swap_total = value,
                    "SwapFree:" => swap_free = value,
                    _ => {}
                }
            }
        }

        state.total_gb = mem_total as f32 / 1024.0 / 1024.0;
        state.used_gb = state.total_gb - (mem_available as f32 / 1024.0 / 1024.0);
        state.swap_total_gb = swap_total as f32 / 1024.0 / 1024.0;
        state.swap_used_gb = state.swap_total_gb - (swap_free as f32 / 1024.0 / 1024.0);
    }

    state
}

fn read_disk(mount_point: &str) -> DiskState {
    let mut state = DiskState {
        mount_point: mount_point.to_string(),
        ..Default::default()
    };

    if let Ok(output) = Command::new("df")
        .args(["-B1", mount_point])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(line) = stdout.lines().nth(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let total: u64 = parts[1].parse().unwrap_or(0);
                    let used: u64 = parts[2].parse().unwrap_or(0);
                    state.total_gb = total as f32 / 1024.0 / 1024.0 / 1024.0;
                    state.used_gb = used as f32 / 1024.0 / 1024.0 / 1024.0;
                }
            }
        }
    }

    state
}

fn read_process_count() -> u32 {
    if let Ok(entries) = std::fs::read_dir("/proc") {
        entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_str()
                    .map(|s| s.chars().all(char::is_numeric))
                    .unwrap_or(false)
            })
            .count() as u32
    } else {
        0
    }
}

fn read_uptime() -> f32 {
    if let Ok(content) = std::fs::read_to_string("/proc/uptime") {
        if let Some(secs) = content.split_whitespace().next() {
            return secs.parse::<f32>().unwrap_or(0.0) / 3600.0;
        }
    }
    0.0
}

fn read_failed_units() -> Vec<String> {
    if let Ok(output) = Command::new("systemctl")
        .args(["--failed", "--no-legend", "--no-pager"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            return stdout
                .lines()
                .filter_map(|line| line.split_whitespace().next())
                .map(|s| s.to_string())
                .collect();
        }
    }
    vec![]
}

fn read_high_cpu_procs() -> Vec<String> {
    if let Ok(output) = Command::new("ps")
        .args(["--no-headers", "-eo", "%cpu,comm", "--sort=-%cpu"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            return stdout
                .lines()
                .take(5)
                .filter_map(|line| {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let cpu: f32 = parts[0].parse().unwrap_or(0.0);
                        if cpu > 50.0 {
                            return Some(format!("{}({:.0}%)", parts[1], cpu));
                        }
                    }
                    None
                })
                .collect();
        }
    }
    vec![]
}

fn check_network() -> NetworkStatus {
    // Try to get default interface and IP
    if let Ok(output) = Command::new("ip")
        .args(["route", "get", "1.1.1.1"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let parts: Vec<&str> = stdout.split_whitespace().collect();

            // Find dev and src
            let mut interface = None;
            let mut ip = None;
            for (i, part) in parts.iter().enumerate() {
                if *part == "dev" && i + 1 < parts.len() {
                    interface = Some(parts[i + 1].to_string());
                }
                if *part == "src" && i + 1 < parts.len() {
                    ip = Some(parts[i + 1].to_string());
                }
            }

            if let (Some(iface), Some(addr)) = (interface, ip) {
                return NetworkStatus::Connected {
                    interface: iface,
                    ip: addr,
                };
            }
        }
    }

    NetworkStatus::Disconnected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_state() {
        let state = LiveState::capture();
        // Should at least get some data
        assert!(state.memory.total_gb > 0.0);
    }

    #[test]
    fn test_summary_generation() {
        let state = LiveState::capture();
        let summary = state.summary();
        assert!(summary.contains("CPU:"));
        assert!(summary.contains("Memory:"));
        assert!(summary.contains("Disk:"));
    }
}
