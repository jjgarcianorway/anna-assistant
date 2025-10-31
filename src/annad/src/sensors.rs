//! Sensor Layer - Passive Telemetry Collection
//!
//! Collects system metrics in a resource-efficient manner:
//! - CPU load (per core, 1 min avg)
//! - Memory usage (free, cache, swap)
//! - Temperature from hwmon/lm-sensors
//! - Battery status if present
//! - Network throughput per interface
//!
//! All data stored in memory ring buffer (last 60 samples).

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use sysinfo::System;

const RING_BUFFER_SIZE: usize = 60;
const POLL_INTERVAL_SECS: u64 = 30;
const POLL_JITTER_SECS: u64 = 5;

/// Complete telemetry snapshot at a single point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetrySample {
    pub timestamp: u64,
    pub cpu: CpuMetrics,
    pub memory: MemoryMetrics,
    pub temperature: Option<TemperatureMetrics>,
    pub battery: Option<BatteryMetrics>,
    pub network: NetworkMetrics,
}

/// CPU metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuMetrics {
    pub cores: usize,
    pub load_1min: f32,
    pub load_5min: f32,
    pub load_15min: f32,
    pub per_core_usage: Vec<f32>,
}

/// Memory metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetrics {
    pub total_mb: u64,
    pub used_mb: u64,
    pub free_mb: u64,
    pub available_mb: u64,
    pub cached_mb: u64,
    pub swap_total_mb: u64,
    pub swap_used_mb: u64,
}

/// Temperature metrics from hwmon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemperatureMetrics {
    pub cpu_temp_c: Option<f32>,
    pub highest_temp_c: f32,
    pub sensor_name: String,
}

/// Battery status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryMetrics {
    pub percentage: f32,
    pub status: String, // "Charging", "Discharging", "Full"
    pub time_remaining_mins: Option<u32>,
}

/// Network throughput
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    pub interfaces: Vec<InterfaceMetrics>,
    pub total_rx_bytes: u64,
    pub total_tx_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceMetrics {
    pub name: String,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub is_up: bool,
}

/// Ring buffer holding last N samples
pub struct SensorRingBuffer {
    samples: Arc<Mutex<VecDeque<TelemetrySample>>>,
    max_size: usize,
}

impl SensorRingBuffer {
    pub fn new(max_size: usize) -> Self {
        Self {
            samples: Arc::new(Mutex::new(VecDeque::with_capacity(max_size))),
            max_size,
        }
    }

    /// Add new sample, evicting oldest if full
    pub fn push(&self, sample: TelemetrySample) {
        let mut samples = self.samples.lock().unwrap();
        if samples.len() >= self.max_size {
            samples.pop_front();
        }
        samples.push_back(sample);
    }

    /// Get most recent sample
    pub fn latest(&self) -> Option<TelemetrySample> {
        let samples = self.samples.lock().unwrap();
        samples.back().cloned()
    }

    /// Get all samples (oldest to newest)
    pub fn all(&self) -> Vec<TelemetrySample> {
        let samples = self.samples.lock().unwrap();
        samples.iter().cloned().collect()
    }

    /// Get samples since timestamp
    pub fn since(&self, timestamp: u64) -> Vec<TelemetrySample> {
        let samples = self.samples.lock().unwrap();
        samples
            .iter()
            .filter(|s| s.timestamp >= timestamp)
            .cloned()
            .collect()
    }

    /// Get last N samples
    pub fn last(&self, n: usize) -> Vec<TelemetrySample> {
        let samples = self.samples.lock().unwrap();
        samples
            .iter()
            .rev()
            .take(n)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    pub fn len(&self) -> usize {
        let samples = self.samples.lock().unwrap();
        samples.len()
    }
}

/// Main sensor collector
pub struct SensorCollector {
    buffer: Arc<SensorRingBuffer>,
    system: System,
}

impl SensorCollector {
    pub fn new() -> Self {
        Self {
            buffer: Arc::new(SensorRingBuffer::new(RING_BUFFER_SIZE)),
            system: System::new_all(),
        }
    }

    pub fn buffer(&self) -> Arc<SensorRingBuffer> {
        Arc::clone(&self.buffer)
    }

    /// Collect one complete sample
    pub fn collect_sample(&mut self) -> Result<TelemetrySample> {
        // Refresh system info
        self.system.refresh_all();

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let sample = TelemetrySample {
            timestamp,
            cpu: self.collect_cpu()?,
            memory: self.collect_memory()?,
            temperature: self.collect_temperature().ok(),
            battery: self.collect_battery().ok(),
            network: self.collect_network()?,
        };

        self.buffer.push(sample.clone());

        Ok(sample)
    }

    /// Start continuous collection in background
    pub fn start_collection_loop(self: Arc<Self>) {
        tokio::spawn(async move {
            use tokio::time::{interval, Duration};
            use rand::Rng;

            loop {
                // Add jitter to prevent synchronized load
                let jitter = rand::thread_rng().gen_range(0..POLL_JITTER_SECS);
                let wait_secs = POLL_INTERVAL_SECS + jitter;

                let mut interval = interval(Duration::from_secs(wait_secs));
                interval.tick().await;

                // Create new mutable instance for this iteration
                let mut collector = SensorCollector {
                    buffer: Arc::clone(&self.buffer),
                    system: System::new_all(),
                };

                if let Err(e) = collector.collect_sample() {
                    tracing::warn!("[SENSORS] Collection failed: {}", e);
                }
            }
        });
    }

    fn collect_cpu(&mut self) -> Result<CpuMetrics> {
        // Read load average from /proc/loadavg
        let (load_1, load_5, load_15) = read_load_average();

        let cpus = self.system.cpus();
        let per_core_usage: Vec<f32> = cpus.iter().map(|cpu| cpu.cpu_usage()).collect();

        Ok(CpuMetrics {
            cores: cpus.len(),
            load_1min: load_1,
            load_5min: load_5,
            load_15min: load_15,
            per_core_usage,
        })
    }

    fn collect_memory(&self) -> Result<MemoryMetrics> {
        let total = self.system.total_memory() / (1024 * 1024); // Convert to MB
        let used = self.system.used_memory() / (1024 * 1024);
        let free = self.system.free_memory() / (1024 * 1024);
        let available = self.system.available_memory() / (1024 * 1024);

        let swap_total = self.system.total_swap() / (1024 * 1024);
        let swap_used = self.system.used_swap() / (1024 * 1024);

        // Approximate cached memory (total - used - free)
        let cached = if total > (used + free) {
            total - used - free
        } else {
            0
        };

        Ok(MemoryMetrics {
            total_mb: total,
            used_mb: used,
            free_mb: free,
            available_mb: available,
            cached_mb: cached,
            swap_total_mb: swap_total,
            swap_used_mb: swap_used,
        })
    }

    fn collect_temperature(&self) -> Result<TemperatureMetrics> {
        // Try hwmon first
        if let Some(temp) = self.read_hwmon_temp() {
            return Ok(temp);
        }

        // Fallback: try thermal zones
        if let Some(temp) = self.read_thermal_zones() {
            return Ok(temp);
        }

        anyhow::bail!("No temperature sensors found")
    }

    fn read_hwmon_temp(&self) -> Option<TemperatureMetrics> {
        let hwmon_path = Path::new("/sys/class/hwmon");
        if !hwmon_path.exists() {
            return None;
        }

        let mut highest_temp = 0.0f32;
        let mut cpu_temp = None;
        let mut sensor_name = String::from("unknown");

        // Iterate through hwmon devices
        if let Ok(entries) = fs::read_dir(hwmon_path) {
            for entry in entries.flatten() {
                let device_path = entry.path();

                // Read device name
                if let Ok(name) = fs::read_to_string(device_path.join("name")) {
                    let name = name.trim();

                    // Look for temperature inputs
                    for i in 1..=10 {
                        let temp_file = device_path.join(format!("temp{}_input", i));
                        if let Ok(temp_str) = fs::read_to_string(&temp_file) {
                            if let Ok(temp_millideg) = temp_str.trim().parse::<i32>() {
                                let temp_c = temp_millideg as f32 / 1000.0;

                                if temp_c > highest_temp {
                                    highest_temp = temp_c;
                                    sensor_name = name.to_string();
                                }

                                // Check if this is a CPU sensor
                                if name.contains("coretemp") || name.contains("k10temp") {
                                    cpu_temp = Some(temp_c);
                                }
                            }
                        }
                    }
                }
            }
        }

        if highest_temp > 0.0 {
            Some(TemperatureMetrics {
                cpu_temp_c: cpu_temp,
                highest_temp_c: highest_temp,
                sensor_name,
            })
        } else {
            None
        }
    }

    fn read_thermal_zones(&self) -> Option<TemperatureMetrics> {
        let thermal_path = Path::new("/sys/class/thermal");
        if !thermal_path.exists() {
            return None;
        }

        let mut highest_temp = 0.0f32;

        if let Ok(entries) = fs::read_dir(thermal_path) {
            for entry in entries.flatten() {
                let zone_path = entry.path();
                let temp_file = zone_path.join("temp");

                if let Ok(temp_str) = fs::read_to_string(&temp_file) {
                    if let Ok(temp_millideg) = temp_str.trim().parse::<i32>() {
                        let temp_c = temp_millideg as f32 / 1000.0;
                        if temp_c > highest_temp {
                            highest_temp = temp_c;
                        }
                    }
                }
            }
        }

        if highest_temp > 0.0 {
            Some(TemperatureMetrics {
                cpu_temp_c: Some(highest_temp),
                highest_temp_c: highest_temp,
                sensor_name: "thermal_zone".to_string(),
            })
        } else {
            None
        }
    }

    fn collect_battery(&self) -> Result<BatteryMetrics> {
        let power_supply_path = Path::new("/sys/class/power_supply");
        if !power_supply_path.exists() {
            anyhow::bail!("No battery found");
        }

        // Look for BAT0, BAT1, etc.
        for entry in fs::read_dir(power_supply_path)? {
            let entry = entry?;
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            if name_str.starts_with("BAT") {
                let bat_path = entry.path();

                // Read capacity
                let capacity = fs::read_to_string(bat_path.join("capacity"))
                    .ok()
                    .and_then(|s| s.trim().parse::<f32>().ok())
                    .unwrap_or(0.0);

                // Read status
                let status = fs::read_to_string(bat_path.join("status"))
                    .ok()
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(|| "Unknown".to_string());

                return Ok(BatteryMetrics {
                    percentage: capacity,
                    status,
                    time_remaining_mins: None, // TODO: Calculate based on current rate
                });
            }
        }

        anyhow::bail!("No battery device found")
    }

    fn collect_network(&mut self) -> Result<NetworkMetrics> {
        // Read network stats from /proc/net/dev
        let interfaces = read_network_stats()?;

        let total_rx: u64 = interfaces.iter().map(|i| i.rx_bytes).sum();
        let total_tx: u64 = interfaces.iter().map(|i| i.tx_bytes).sum();

        Ok(NetworkMetrics {
            interfaces,
            total_rx_bytes: total_rx,
            total_tx_bytes: total_tx,
        })
    }
}

/// Read load average from /proc/loadavg
fn read_load_average() -> (f32, f32, f32) {
    if let Ok(content) = fs::read_to_string("/proc/loadavg") {
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() >= 3 {
            let load_1 = parts[0].parse::<f32>().unwrap_or(0.0);
            let load_5 = parts[1].parse::<f32>().unwrap_or(0.0);
            let load_15 = parts[2].parse::<f32>().unwrap_or(0.0);
            return (load_1, load_5, load_15);
        }
    }
    (0.0, 0.0, 0.0)
}

/// Read network statistics from /proc/net/dev
fn read_network_stats() -> Result<Vec<InterfaceMetrics>> {
    let content = fs::read_to_string("/proc/net/dev")
        .context("Failed to read /proc/net/dev")?;

    let mut interfaces = Vec::new();

    for line in content.lines().skip(2) {
        // Skip header lines
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 10 {
            continue;
        }

        let name = parts[0].trim_end_matches(':').to_string();
        let rx_bytes = parts[1].parse::<u64>().unwrap_or(0);
        let tx_bytes = parts[9].parse::<u64>().unwrap_or(0);

        // Skip loopback
        if name == "lo" {
            continue;
        }

        interfaces.push(InterfaceMetrics {
            name,
            rx_bytes,
            tx_bytes,
            is_up: rx_bytes > 0 || tx_bytes > 0,
        });
    }

    Ok(interfaces)
}
