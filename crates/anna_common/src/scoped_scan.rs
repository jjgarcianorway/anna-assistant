//! Scoped Scan v7.32.0 - On-Demand Targeted Data Collection
//!
//! Provides on-demand scoped scans with time budgets:
//! - 250ms default, 1s max
//! - Collect only what's needed
//! - No whole-system scans
//!
//! Staleness model for datasets:
//! - Track last_collected, ttl_seconds, collection_cost
//! - Decide when to refresh

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

/// Default time budget for scoped scans (250ms)
pub const DEFAULT_TIME_BUDGET_MS: u64 = 250;

/// Maximum time budget for scoped scans (1 second)
pub const MAX_TIME_BUDGET_MS: u64 = 1000;

/// Scan scope - what data to collect
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ScanScope {
    /// CPU metrics only
    Cpu,
    /// Memory metrics only
    Memory,
    /// Disk/storage metrics only
    Storage,
    /// Network metrics only
    Network,
    /// GPU metrics only
    Gpu,
    /// Temperature sensors only
    Thermal,
    /// Specific process by name
    Process(String),
    /// Specific service by name
    Service(String),
    /// Specific device by name
    Device(String),
    /// Package info by name
    Package(String),
}

impl ScanScope {
    pub fn label(&self) -> String {
        match self {
            Self::Cpu => "cpu".to_string(),
            Self::Memory => "memory".to_string(),
            Self::Storage => "storage".to_string(),
            Self::Network => "network".to_string(),
            Self::Gpu => "gpu".to_string(),
            Self::Thermal => "thermal".to_string(),
            Self::Process(name) => format!("process:{}", name),
            Self::Service(name) => format!("service:{}", name),
            Self::Device(name) => format!("device:{}", name),
            Self::Package(name) => format!("package:{}", name),
        }
    }

    /// Estimated collection cost in milliseconds
    pub fn estimated_cost_ms(&self) -> u64 {
        match self {
            Self::Cpu => 10,
            Self::Memory => 5,
            Self::Storage => 50,
            Self::Network => 30,
            Self::Gpu => 100,
            Self::Thermal => 20,
            Self::Process(_) => 15,
            Self::Service(_) => 25,
            Self::Device(_) => 40,
            Self::Package(_) => 100,
        }
    }
}

/// Staleness info for a dataset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StalenessInfo {
    /// Scope identifier
    pub scope: String,
    /// Last collection timestamp (Unix epoch)
    pub last_collected: u64,
    /// Time-to-live in seconds
    pub ttl_seconds: u64,
    /// Actual collection cost in milliseconds
    pub collection_cost_ms: u64,
    /// Number of times collected
    pub collection_count: u64,
}

impl StalenessInfo {
    /// Create new staleness info
    pub fn new(scope: &str, ttl_seconds: u64) -> Self {
        Self {
            scope: scope.to_string(),
            last_collected: 0,
            ttl_seconds,
            collection_cost_ms: 0,
            collection_count: 0,
        }
    }

    /// Check if data is stale
    pub fn is_stale(&self) -> bool {
        if self.last_collected == 0 {
            return true;
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        now.saturating_sub(self.last_collected) > self.ttl_seconds
    }

    /// Seconds until data becomes stale
    pub fn seconds_until_stale(&self) -> Option<u64> {
        if self.last_collected == 0 {
            return None;
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let age = now.saturating_sub(self.last_collected);
        if age >= self.ttl_seconds {
            Some(0)
        } else {
            Some(self.ttl_seconds - age)
        }
    }

    /// Age of data in seconds
    pub fn age_seconds(&self) -> u64 {
        if self.last_collected == 0 {
            return u64::MAX;
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        now.saturating_sub(self.last_collected)
    }

    /// Mark as collected now
    pub fn mark_collected(&mut self, cost_ms: u64) {
        self.last_collected = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.collection_cost_ms = cost_ms;
        self.collection_count += 1;
    }
}

/// Scoped scan result
#[derive(Debug, Clone)]
pub struct ScanResult {
    /// Scope that was scanned
    pub scope: ScanScope,
    /// Whether scan succeeded
    pub success: bool,
    /// Scan duration in milliseconds
    pub duration_ms: u64,
    /// Error message if failed
    pub error: Option<String>,
    /// Data collected (scope-specific)
    pub data: Option<ScanData>,
}

/// Collected data from a scan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScanData {
    /// CPU data
    Cpu {
        usage_percent: f32,
        load_1m: f32,
        load_5m: f32,
        load_15m: f32,
        process_count: usize,
    },
    /// Memory data
    Memory {
        used_bytes: u64,
        total_bytes: u64,
        swap_used_bytes: u64,
        swap_total_bytes: u64,
    },
    /// Storage data
    Storage { mounts: Vec<MountInfo> },
    /// Network data
    Network { interfaces: Vec<InterfaceInfo> },
    /// Temperature data
    Thermal { sensors: Vec<TempSensor> },
    /// Process data
    Process {
        pid: u32,
        name: String,
        cpu_percent: f32,
        memory_bytes: u64,
        state: String,
    },
    /// Service data
    Service {
        name: String,
        active: bool,
        running: bool,
        enabled: bool,
    },
    /// GPU data
    Gpu {
        name: String,
        utilization: Option<f32>,
        memory_used: Option<u64>,
        memory_total: Option<u64>,
        temperature: Option<f32>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountInfo {
    pub device: String,
    pub mountpoint: String,
    pub used_bytes: u64,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceInfo {
    pub name: String,
    pub up: bool,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TempSensor {
    pub name: String,
    pub temperature_c: f32,
    pub high_threshold: Option<f32>,
    pub critical_threshold: Option<f32>,
}

/// Scoped scanner with time budget
pub struct ScopedScanner {
    /// Time budget in milliseconds
    budget_ms: u64,
    /// Start time
    start: Instant,
    /// Staleness tracker
    staleness: HashMap<String, StalenessInfo>,
}

impl ScopedScanner {
    /// Create new scanner with default time budget
    pub fn new() -> Self {
        Self::with_budget_ms(DEFAULT_TIME_BUDGET_MS)
    }

    /// Create scanner with custom time budget (capped at MAX)
    pub fn with_budget_ms(budget_ms: u64) -> Self {
        Self {
            budget_ms: budget_ms.min(MAX_TIME_BUDGET_MS),
            start: Instant::now(),
            staleness: HashMap::new(),
        }
    }

    /// Remaining time budget in milliseconds
    pub fn remaining_ms(&self) -> u64 {
        let elapsed = self.start.elapsed().as_millis() as u64;
        self.budget_ms.saturating_sub(elapsed)
    }

    /// Check if budget is exhausted
    pub fn budget_exhausted(&self) -> bool {
        self.remaining_ms() == 0
    }

    /// Check if we can fit a scope within remaining budget
    pub fn can_fit(&self, scope: &ScanScope) -> bool {
        self.remaining_ms() >= scope.estimated_cost_ms()
    }

    /// Execute a scoped scan
    pub fn scan(&mut self, scope: ScanScope) -> ScanResult {
        let scope_key = scope.label();
        let scan_start = Instant::now();

        // Check if we have budget
        if self.budget_exhausted() {
            return ScanResult {
                scope,
                success: false,
                duration_ms: 0,
                error: Some("Time budget exhausted".to_string()),
                data: None,
            };
        }

        // Check staleness - skip if data is fresh
        if let Some(info) = self.staleness.get(&scope_key) {
            if !info.is_stale() {
                return ScanResult {
                    scope,
                    success: true,
                    duration_ms: 0,
                    error: None,
                    data: None, // Data not collected, use cached
                };
            }
        }

        // Execute the scan based on scope
        let result = match &scope {
            ScanScope::Cpu => self.scan_cpu(),
            ScanScope::Memory => self.scan_memory(),
            ScanScope::Storage => self.scan_storage(),
            ScanScope::Network => self.scan_network(),
            ScanScope::Thermal => self.scan_thermal(),
            ScanScope::Process(name) => self.scan_process(name),
            ScanScope::Service(name) => self.scan_service(name),
            ScanScope::Gpu => self.scan_gpu(),
            ScanScope::Device(_) | ScanScope::Package(_) => {
                // Not implemented yet
                Err("Not implemented".to_string())
            }
        };

        let duration_ms = scan_start.elapsed().as_millis() as u64;

        // Update staleness info
        let staleness = self
            .staleness
            .entry(scope_key.clone())
            .or_insert_with(|| StalenessInfo::new(&scope_key, default_ttl(&scope)));
        staleness.mark_collected(duration_ms);

        match result {
            Ok(data) => ScanResult {
                scope,
                success: true,
                duration_ms,
                error: None,
                data: Some(data),
            },
            Err(e) => ScanResult {
                scope,
                success: false,
                duration_ms,
                error: Some(e),
                data: None,
            },
        }
    }

    fn scan_cpu(&self) -> Result<ScanData, String> {
        // Read /proc/loadavg
        let loadavg = std::fs::read_to_string("/proc/loadavg").map_err(|e| e.to_string())?;
        let parts: Vec<&str> = loadavg.split_whitespace().collect();
        if parts.len() < 4 {
            return Err("Invalid loadavg format".to_string());
        }

        let load_1m: f32 = parts[0].parse().unwrap_or(0.0);
        let load_5m: f32 = parts[1].parse().unwrap_or(0.0);
        let load_15m: f32 = parts[2].parse().unwrap_or(0.0);
        let process_count: usize = parts[3]
            .split('/')
            .nth(1)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        // Read /proc/stat for CPU usage (simplified)
        let stat = std::fs::read_to_string("/proc/stat").map_err(|e| e.to_string())?;
        let cpu_line = stat.lines().next().ok_or("No CPU line")?;
        let values: Vec<u64> = cpu_line
            .split_whitespace()
            .skip(1)
            .filter_map(|s| s.parse().ok())
            .collect();

        let usage_percent = if values.len() >= 4 {
            let idle = values[3];
            let total: u64 = values.iter().sum();
            if total > 0 {
                ((total - idle) as f32 / total as f32) * 100.0
            } else {
                0.0
            }
        } else {
            load_1m * 100.0 / num_cpus()
        };

        Ok(ScanData::Cpu {
            usage_percent,
            load_1m,
            load_5m,
            load_15m,
            process_count,
        })
    }

    fn scan_memory(&self) -> Result<ScanData, String> {
        let meminfo = std::fs::read_to_string("/proc/meminfo").map_err(|e| e.to_string())?;

        let mut mem_total: u64 = 0;
        let mut mem_available: u64 = 0;
        let mut swap_total: u64 = 0;
        let mut swap_free: u64 = 0;

        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                mem_total = parse_meminfo_kb(line);
            } else if line.starts_with("MemAvailable:") {
                mem_available = parse_meminfo_kb(line);
            } else if line.starts_with("SwapTotal:") {
                swap_total = parse_meminfo_kb(line);
            } else if line.starts_with("SwapFree:") {
                swap_free = parse_meminfo_kb(line);
            }
        }

        Ok(ScanData::Memory {
            used_bytes: (mem_total - mem_available) * 1024,
            total_bytes: mem_total * 1024,
            swap_used_bytes: (swap_total - swap_free) * 1024,
            swap_total_bytes: swap_total * 1024,
        })
    }

    fn scan_storage(&self) -> Result<ScanData, String> {
        let output = std::process::Command::new("df")
            .args(["-B1", "--output=source,target,used,size"])
            .output()
            .map_err(|e| e.to_string())?;

        if !output.status.success() {
            return Err("df command failed".to_string());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut mounts = Vec::new();

        for line in stdout.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                // Skip pseudo filesystems
                if parts[0].starts_with("/dev/") {
                    mounts.push(MountInfo {
                        device: parts[0].to_string(),
                        mountpoint: parts[1].to_string(),
                        used_bytes: parts[2].parse().unwrap_or(0),
                        total_bytes: parts[3].parse().unwrap_or(0),
                    });
                }
            }
        }

        Ok(ScanData::Storage { mounts })
    }

    fn scan_network(&self) -> Result<ScanData, String> {
        let mut interfaces = Vec::new();

        if let Ok(entries) = std::fs::read_dir("/sys/class/net") {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name == "lo" {
                    continue;
                }

                let carrier = std::fs::read_to_string(format!("/sys/class/net/{}/carrier", name))
                    .map(|s| s.trim() == "1")
                    .unwrap_or(false);

                let rx_bytes =
                    std::fs::read_to_string(format!("/sys/class/net/{}/statistics/rx_bytes", name))
                        .ok()
                        .and_then(|s| s.trim().parse().ok())
                        .unwrap_or(0);

                let tx_bytes =
                    std::fs::read_to_string(format!("/sys/class/net/{}/statistics/tx_bytes", name))
                        .ok()
                        .and_then(|s| s.trim().parse().ok())
                        .unwrap_or(0);

                interfaces.push(InterfaceInfo {
                    name,
                    up: carrier,
                    rx_bytes,
                    tx_bytes,
                });
            }
        }

        Ok(ScanData::Network { interfaces })
    }

    fn scan_thermal(&self) -> Result<ScanData, String> {
        let mut sensors = Vec::new();

        // Read from /sys/class/thermal
        if let Ok(entries) = std::fs::read_dir("/sys/class/thermal") {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path
                    .file_name()
                    .map(|n| n.to_string_lossy().starts_with("thermal_zone"))
                    .unwrap_or(false)
                {
                    continue;
                }

                let name = std::fs::read_to_string(path.join("type"))
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(|_| "unknown".to_string());

                let temp_millic: i64 = std::fs::read_to_string(path.join("temp"))
                    .ok()
                    .and_then(|s| s.trim().parse().ok())
                    .unwrap_or(0);

                sensors.push(TempSensor {
                    name,
                    temperature_c: temp_millic as f32 / 1000.0,
                    high_threshold: None,
                    critical_threshold: None,
                });
            }
        }

        Ok(ScanData::Thermal { sensors })
    }

    fn scan_process(&self, name: &str) -> Result<ScanData, String> {
        // Use pgrep to find PID
        let output = std::process::Command::new("pgrep")
            .args(["-x", name])
            .output()
            .map_err(|e| e.to_string())?;

        if !output.status.success() {
            return Err(format!("Process '{}' not found", name));
        }

        let pid: u32 = String::from_utf8_lossy(&output.stdout)
            .lines()
            .next()
            .and_then(|s| s.trim().parse().ok())
            .ok_or("No PID found")?;

        // Read /proc/<pid>/stat
        let stat =
            std::fs::read_to_string(format!("/proc/{}/stat", pid)).map_err(|e| e.to_string())?;
        let parts: Vec<&str> = stat.split_whitespace().collect();

        let state = parts.get(2).map(|s| s.to_string()).unwrap_or_default();

        // Read /proc/<pid>/statm for memory
        let statm =
            std::fs::read_to_string(format!("/proc/{}/statm", pid)).map_err(|e| e.to_string())?;
        let mem_pages: u64 = statm
            .split_whitespace()
            .nth(1)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let page_size: u64 = 4096; // Typical page size

        Ok(ScanData::Process {
            pid,
            name: name.to_string(),
            cpu_percent: 0.0, // Would need sampling
            memory_bytes: mem_pages * page_size,
            state,
        })
    }

    fn scan_service(&self, name: &str) -> Result<ScanData, String> {
        let output = std::process::Command::new("systemctl")
            .args([
                "show",
                name,
                "--no-pager",
                "-p",
                "ActiveState,SubState,UnitFileState",
            ])
            .output()
            .map_err(|e| e.to_string())?;

        if !output.status.success() {
            return Err(format!("Service '{}' not found", name));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut active_state = "unknown".to_string();
        let mut sub_state = "unknown".to_string();
        let mut enabled = false;

        for line in stdout.lines() {
            if let Some(val) = line.strip_prefix("ActiveState=") {
                active_state = val.to_string();
            } else if let Some(val) = line.strip_prefix("SubState=") {
                sub_state = val.to_string();
            } else if let Some(val) = line.strip_prefix("UnitFileState=") {
                enabled = val == "enabled";
            }
        }

        Ok(ScanData::Service {
            name: name.to_string(),
            active: active_state == "active",
            running: sub_state == "running",
            enabled,
        })
    }

    fn scan_gpu(&self) -> Result<ScanData, String> {
        // Try nvidia-smi first
        if let Ok(output) = std::process::Command::new("nvidia-smi")
            .args([
                "--query-gpu=name,utilization.gpu,memory.used,memory.total,temperature.gpu",
                "--format=csv,noheader,nounits",
            ])
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let parts: Vec<&str> = stdout.trim().split(',').collect();
                if parts.len() >= 5 {
                    return Ok(ScanData::Gpu {
                        name: parts[0].trim().to_string(),
                        utilization: parts[1].trim().parse().ok(),
                        memory_used: parts[2].trim().parse().ok().map(|m: u64| m * 1024 * 1024),
                        memory_total: parts[3].trim().parse().ok().map(|m: u64| m * 1024 * 1024),
                        temperature: parts[4].trim().parse().ok(),
                    });
                }
            }
        }

        Err("No GPU data available".to_string())
    }
}

impl Default for ScopedScanner {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_meminfo_kb(line: &str) -> u64 {
    line.split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

fn num_cpus() -> f32 {
    std::fs::read_to_string("/proc/cpuinfo")
        .map(|content| {
            content
                .lines()
                .filter(|l| l.starts_with("processor"))
                .count() as f32
        })
        .unwrap_or(1.0)
}

fn default_ttl(scope: &ScanScope) -> u64 {
    match scope {
        ScanScope::Cpu => 5, // 5 seconds
        ScanScope::Memory => 5,
        ScanScope::Storage => 60, // 1 minute
        ScanScope::Network => 10,
        ScanScope::Gpu => 10,
        ScanScope::Thermal => 10,
        ScanScope::Process(_) => 5,
        ScanScope::Service(_) => 30,
        ScanScope::Device(_) => 60,
        ScanScope::Package(_) => 300, // 5 minutes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_staleness_info() {
        let mut info = StalenessInfo::new("test", 10);
        assert!(info.is_stale());

        info.mark_collected(5);
        assert!(!info.is_stale());
    }

    #[test]
    fn test_scanner_budget() {
        let scanner = ScopedScanner::new();
        assert!(!scanner.budget_exhausted());
        assert!(scanner.remaining_ms() <= DEFAULT_TIME_BUDGET_MS);
    }

    #[test]
    fn test_scope_labels() {
        assert_eq!(ScanScope::Cpu.label(), "cpu");
        assert_eq!(ScanScope::Process("foo".to_string()).label(), "process:foo");
    }
}
