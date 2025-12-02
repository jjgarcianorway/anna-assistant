//! Anna Hotspots v7.24.0 - Resource Usage Hotspot Detection
//!
//! Identifies top resource consumers from Anna's telemetry.
//! All values follow the global percentage rules.
//!
//! Rules:
//! - CPU percentages always include range in parentheses
//! - All fractions [0,1] shown as percentages
//! - No guessing - only data from Anna telemetry

use crate::timeline::format_memory;

/// CPU hotspot entry
#[derive(Debug, Clone)]
pub struct CpuHotspot {
    pub name: String,
    pub avg_percent: f64,
    pub peak_percent: f64,
    pub logical_cores: u32,
}

impl CpuHotspot {
    /// Format as "name  avg X percent (0 - Y percent for N logical cores)"
    pub fn format_line(&self) -> String {
        format!(
            "{}  avg {} percent (0 - {} percent for {} logical cores)",
            self.name,
            self.avg_percent.round() as i64,
            self.logical_cores * 100,
            self.logical_cores
        )
    }

    /// Format compact for status: "name (X percent)"
    pub fn format_compact(&self) -> String {
        format!("{} ({} percent)", self.name, self.avg_percent.round() as i64)
    }
}

/// Memory hotspot entry
#[derive(Debug, Clone)]
pub struct MemoryHotspot {
    pub name: String,
    pub avg_bytes: u64,
    pub peak_bytes: u64,
}

impl MemoryHotspot {
    /// Format as "name  avg X GiB RSS"
    pub fn format_line(&self) -> String {
        format!("{}  avg {}", self.name, format_memory(self.avg_bytes))
    }

    /// Format compact for status: "name (X GiB)"
    pub fn format_compact(&self) -> String {
        format!("{} ({})", self.name, format_memory(self.avg_bytes))
    }
}

/// Process start frequency hotspot
#[derive(Debug, Clone)]
pub struct StartFrequencyHotspot {
    pub name: String,
    pub start_count: u64,
}

impl StartFrequencyHotspot {
    /// Format as "name  N starts"
    pub fn format_line(&self) -> String {
        format!("{}  {} starts", self.name, self.start_count)
    }
}

/// Temperature hotspot entry
#[derive(Debug, Clone)]
pub struct TempHotspot {
    pub device: String,
    pub avg_temp_c: f64,
    pub min_temp_c: f64,
    pub max_temp_c: f64,
}

impl TempHotspot {
    /// Format as "device  avg X C (min Y, max Z)"
    pub fn format_line(&self) -> String {
        format!(
            "{}  avg {} C (min {}, max {})",
            self.device,
            self.avg_temp_c.round() as i64,
            self.min_temp_c.round() as i64,
            self.max_temp_c.round() as i64
        )
    }

    /// Format compact for status
    pub fn format_compact(&self) -> String {
        format!("{} (avg {} C)", self.device, self.avg_temp_c.round() as i64)
    }
}

/// IO hotspot entry
#[derive(Debug, Clone)]
pub struct IoHotspot {
    pub device: String,
    pub read_bytes: u64,
    pub write_bytes: u64,
}

impl IoHotspot {
    /// Format as "device  read X GiB, write Y GiB"
    pub fn format_line(&self) -> String {
        format!(
            "{}  read {}, write {}",
            self.device,
            format_bytes_gib(self.read_bytes),
            format_bytes_gib(self.write_bytes)
        )
    }
}

/// Component load hotspot (CPU, GPU)
#[derive(Debug, Clone)]
pub struct LoadHotspot {
    pub component: String,
    pub avg_percent: f64,
    pub range_max: u32,
    pub range_unit: String, // "for N logical cores" or "per GPU"
}

impl LoadHotspot {
    /// Format as "component  avg X percent (0 - Y percent ...)"
    pub fn format_line(&self) -> String {
        format!(
            "{}  avg {} percent (0 - {} percent {})",
            self.component,
            self.avg_percent.round() as i64,
            self.range_max,
            self.range_unit
        )
    }

    /// Format compact for status
    pub fn format_compact(&self) -> String {
        format!(
            "{} (avg {} percent of {} percent)",
            self.component,
            self.avg_percent.round() as i64,
            self.range_max
        )
    }
}

/// GPU utilization hotspot
#[derive(Debug, Clone)]
pub struct GpuHotspot {
    pub name: String,
    pub avg_percent: f64,
}

impl GpuHotspot {
    /// Format as "name  avg X percent GPU util (0 - 100 percent per GPU)"
    pub fn format_line(&self) -> String {
        format!(
            "{}  avg {} percent GPU util (0 - 100 percent per GPU)",
            self.name,
            self.avg_percent.round() as i64
        )
    }
}

/// Software hotspots collection
#[derive(Debug, Clone, Default)]
pub struct SoftwareHotspots {
    pub top_cpu: Vec<CpuHotspot>,
    pub top_memory: Vec<MemoryHotspot>,
    pub most_started: Vec<StartFrequencyHotspot>,
    pub has_data: bool,
}

/// Hardware hotspots collection
#[derive(Debug, Clone, Default)]
pub struct HardwareHotspots {
    pub warm_devices: Vec<TempHotspot>,
    pub heavy_io: Vec<IoHotspot>,
    pub high_load: Vec<LoadHotspot>,
    pub has_data: bool,
}

/// Get software hotspots from telemetry
pub fn get_software_hotspots() -> SoftwareHotspots {
    let mut hotspots = SoftwareHotspots::default();

    // Get logical core count
    let logical_cores = get_logical_cores();

    // Query telemetry database for top CPU consumers (last 24h)
    if let Some(cpu_data) = query_top_cpu_24h(5) {
        for (name, avg, peak) in cpu_data {
            hotspots.top_cpu.push(CpuHotspot {
                name,
                avg_percent: avg,
                peak_percent: peak,
                logical_cores,
            });
        }
        hotspots.has_data = true;
    }

    // Query telemetry database for top memory consumers (last 24h)
    if let Some(mem_data) = query_top_memory_24h(5) {
        for (name, avg, peak) in mem_data {
            hotspots.top_memory.push(MemoryHotspot {
                name,
                avg_bytes: avg,
                peak_bytes: peak,
            });
        }
        hotspots.has_data = true;
    }

    // Query telemetry database for most frequently started (last 24h)
    if let Some(start_data) = query_most_started_24h(5) {
        for (name, count) in start_data {
            hotspots.most_started.push(StartFrequencyHotspot {
                name,
                start_count: count,
            });
        }
        hotspots.has_data = true;
    }

    hotspots
}

/// Get hardware hotspots from telemetry
pub fn get_hardware_hotspots() -> HardwareHotspots {
    let mut hotspots = HardwareHotspots::default();

    // Get logical core count for CPU load
    let logical_cores = get_logical_cores();

    // Query temperature data (last 24h)
    if let Some(temp_data) = query_warm_devices_24h(5) {
        for (device, avg, min, max) in temp_data {
            hotspots.warm_devices.push(TempHotspot {
                device,
                avg_temp_c: avg,
                min_temp_c: min,
                max_temp_c: max,
            });
        }
        hotspots.has_data = true;
    }

    // Query IO data (last 24h)
    if let Some(io_data) = query_heavy_io_24h(5) {
        for (device, read, write) in io_data {
            hotspots.heavy_io.push(IoHotspot {
                device,
                read_bytes: read,
                write_bytes: write,
            });
        }
        hotspots.has_data = true;
    }

    // Get CPU load
    if let Some((avg, _peak)) = query_cpu_load_24h() {
        hotspots.high_load.push(LoadHotspot {
            component: "CPU".to_string(),
            avg_percent: avg,
            range_max: logical_cores * 100,
            range_unit: format!("for {} logical cores", logical_cores),
        });
        hotspots.has_data = true;
    }

    // Get GPU load (if nvidia-smi or similar available)
    if let Some((avg, _peak)) = query_gpu_load_24h() {
        hotspots.high_load.push(LoadHotspot {
            component: "GPU".to_string(),
            avg_percent: avg,
            range_max: 100,
            range_unit: "per GPU".to_string(),
        });
        hotspots.has_data = true;
    }

    hotspots
}

/// Format software hotspots section
pub fn format_software_hotspots_section(hotspots: &SoftwareHotspots) -> Vec<String> {
    let mut lines = Vec::new();

    lines.push("[HOTSPOTS]".to_string());
    lines.push("  Source: Anna telemetry (sw) and links.db".to_string());
    lines.push(String::new());

    if !hotspots.has_data {
        lines.push("  Telemetry: insufficient to compute hotspots".to_string());
        return lines;
    }

    if !hotspots.top_cpu.is_empty() {
        lines.push("  Top CPU (last 24h):".to_string());
        for h in &hotspots.top_cpu {
            lines.push(format!("    {}", h.format_line()));
        }
        lines.push(String::new());
    }

    if !hotspots.top_memory.is_empty() {
        lines.push("  Top memory (last 24h):".to_string());
        for h in &hotspots.top_memory {
            lines.push(format!("    {}", h.format_line()));
        }
        lines.push(String::new());
    }

    if !hotspots.most_started.is_empty() {
        lines.push("  Most frequently started (last 24h):".to_string());
        for h in &hotspots.most_started {
            lines.push(format!("    {}", h.format_line()));
        }
    }

    lines
}

/// Format hardware hotspots section
pub fn format_hardware_hotspots_section(hotspots: &HardwareHotspots) -> Vec<String> {
    let mut lines = Vec::new();

    lines.push("[HOTSPOTS]".to_string());
    lines.push("  Source: Anna telemetry (hw) and links.db".to_string());
    lines.push(String::new());

    if !hotspots.has_data {
        lines.push("  Telemetry: insufficient to compute hotspots".to_string());
        return lines;
    }

    if !hotspots.warm_devices.is_empty() {
        lines.push("  Warm devices (last 24h):".to_string());
        for h in &hotspots.warm_devices {
            lines.push(format!("    {}", h.format_line()));
        }
        lines.push(String::new());
    }

    if !hotspots.heavy_io.is_empty() {
        lines.push("  Heavy IO (last 24h):".to_string());
        for h in &hotspots.heavy_io {
            lines.push(format!("    {}", h.format_line()));
        }
        lines.push(String::new());
    }

    if !hotspots.high_load.is_empty() {
        lines.push("  High load components (last 24h):".to_string());
        for h in &hotspots.high_load {
            lines.push(format!("    {}", h.format_line()));
        }
    }

    lines
}

/// Format status hotspots section (compact)
pub fn format_status_hotspots_section(
    sw_hotspots: &SoftwareHotspots,
    hw_hotspots: &HardwareHotspots,
) -> Vec<String> {
    let mut lines = Vec::new();

    if !sw_hotspots.has_data && !hw_hotspots.has_data {
        return lines;
    }

    lines.push("[HOTSPOTS]".to_string());
    lines.push("  Source: Anna telemetry (last 24h)".to_string());
    lines.push(String::new());

    // Top CPU (compact)
    if sw_hotspots.top_cpu.len() >= 2 {
        let cpu_str: Vec<String> = sw_hotspots.top_cpu.iter().take(2).map(|h| h.format_compact()).collect();
        lines.push(format!("  Top CPU:      {}", cpu_str.join(", ")));
    } else if !sw_hotspots.top_cpu.is_empty() {
        lines.push(format!("  Top CPU:      {}", sw_hotspots.top_cpu[0].format_compact()));
    }

    // Top memory (compact)
    if sw_hotspots.top_memory.len() >= 2 {
        let mem_str: Vec<String> = sw_hotspots.top_memory.iter().take(2).map(|h| h.format_compact()).collect();
        lines.push(format!("  Top memory:   {}", mem_str.join(", ")));
    } else if !sw_hotspots.top_memory.is_empty() {
        lines.push(format!("  Top memory:   {}", sw_hotspots.top_memory[0].format_compact()));
    }

    // Warm devices (compact)
    let mut warm_parts: Vec<String> = Vec::new();
    for h in hw_hotspots.warm_devices.iter().take(2) {
        warm_parts.push(h.format_compact());
    }
    for h in hw_hotspots.high_load.iter().take(1) {
        warm_parts.push(h.format_compact());
    }
    if !warm_parts.is_empty() {
        lines.push(format!("  Warm devices: {}", warm_parts.join(", ")));
    }

    lines
}

// Helper functions to query telemetry

fn get_logical_cores() -> u32 {
    std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .map(|content| {
            content
                .lines()
                .filter(|l| l.starts_with("processor"))
                .count() as u32
        })
        .unwrap_or(1)
        .max(1)
}

fn query_top_cpu_24h(limit: usize) -> Option<Vec<(String, f64, f64)>> {
    // Try to query from Anna's telemetry database
    let db_path = "/var/lib/anna/telemetry.db";
    if !std::path::Path::new(db_path).exists() {
        return None;
    }

    let conn = rusqlite::Connection::open(db_path).ok()?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs() as i64;
    let since = now - 86400; // 24 hours ago

    let mut stmt = conn
        .prepare(
            "SELECT name, AVG(cpu_percent) as avg_cpu, MAX(cpu_percent) as peak_cpu
             FROM process_samples
             WHERE timestamp > ?1
             GROUP BY name
             ORDER BY avg_cpu DESC
             LIMIT ?2",
        )
        .ok()?;

    let results: Vec<(String, f64, f64)> = stmt
        .query_map(rusqlite::params![since, limit as i64], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })
        .ok()?
        .filter_map(|r| r.ok())
        .collect();

    if results.is_empty() {
        None
    } else {
        Some(results)
    }
}

fn query_top_memory_24h(limit: usize) -> Option<Vec<(String, u64, u64)>> {
    let db_path = "/var/lib/anna/telemetry.db";
    if !std::path::Path::new(db_path).exists() {
        return None;
    }

    let conn = rusqlite::Connection::open(db_path).ok()?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs() as i64;
    let since = now - 86400;

    let mut stmt = conn
        .prepare(
            "SELECT name, AVG(rss_bytes) as avg_rss, MAX(rss_bytes) as peak_rss
             FROM process_samples
             WHERE timestamp > ?1
             GROUP BY name
             ORDER BY avg_rss DESC
             LIMIT ?2",
        )
        .ok()?;

    let results: Vec<(String, u64, u64)> = stmt
        .query_map(rusqlite::params![since, limit as i64], |row| {
            let avg: f64 = row.get(1)?;
            let peak: f64 = row.get(2)?;
            Ok((row.get(0)?, avg as u64, peak as u64))
        })
        .ok()?
        .filter_map(|r| r.ok())
        .collect();

    if results.is_empty() {
        None
    } else {
        Some(results)
    }
}

fn query_most_started_24h(limit: usize) -> Option<Vec<(String, u64)>> {
    let db_path = "/var/lib/anna/telemetry.db";
    if !std::path::Path::new(db_path).exists() {
        return None;
    }

    let conn = rusqlite::Connection::open(db_path).ok()?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs() as i64;
    let since = now - 86400;

    let mut stmt = conn
        .prepare(
            "SELECT name, COUNT(DISTINCT pid) as start_count
             FROM process_samples
             WHERE timestamp > ?1
             GROUP BY name
             ORDER BY start_count DESC
             LIMIT ?2",
        )
        .ok()?;

    let results: Vec<(String, u64)> = stmt
        .query_map(rusqlite::params![since, limit as i64], |row| {
            let count: i64 = row.get(1)?;
            Ok((row.get(0)?, count as u64))
        })
        .ok()?
        .filter_map(|r| r.ok())
        .collect();

    if results.is_empty() {
        None
    } else {
        Some(results)
    }
}

fn query_warm_devices_24h(limit: usize) -> Option<Vec<(String, f64, f64, f64)>> {
    // Query from hwmon telemetry
    let db_path = "/var/lib/anna/telemetry.db";
    if !std::path::Path::new(db_path).exists() {
        return None;
    }

    let conn = rusqlite::Connection::open(db_path).ok()?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs() as i64;
    let since = now - 86400;

    // Try to query hw_samples table if it exists
    let mut stmt = conn
        .prepare(
            "SELECT device, AVG(temp_c) as avg_temp, MIN(temp_c) as min_temp, MAX(temp_c) as max_temp
             FROM hw_samples
             WHERE timestamp > ?1 AND temp_c IS NOT NULL
             GROUP BY device
             ORDER BY avg_temp DESC
             LIMIT ?2",
        )
        .ok()?;

    let results: Vec<(String, f64, f64, f64)> = stmt
        .query_map(rusqlite::params![since, limit as i64], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .ok()?
        .filter_map(|r| r.ok())
        .collect();

    if results.is_empty() {
        None
    } else {
        Some(results)
    }
}

fn query_heavy_io_24h(limit: usize) -> Option<Vec<(String, u64, u64)>> {
    let db_path = "/var/lib/anna/telemetry.db";
    if !std::path::Path::new(db_path).exists() {
        return None;
    }

    let conn = rusqlite::Connection::open(db_path).ok()?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs() as i64;
    let since = now - 86400;

    let mut stmt = conn
        .prepare(
            "SELECT device, SUM(read_bytes) as total_read, SUM(write_bytes) as total_write
             FROM hw_samples
             WHERE timestamp > ?1 AND (read_bytes IS NOT NULL OR write_bytes IS NOT NULL)
             GROUP BY device
             ORDER BY (total_read + total_write) DESC
             LIMIT ?2",
        )
        .ok()?;

    let results: Vec<(String, u64, u64)> = stmt
        .query_map(rusqlite::params![since, limit as i64], |row| {
            let read: i64 = row.get(1)?;
            let write: i64 = row.get(2)?;
            Ok((row.get(0)?, read as u64, write as u64))
        })
        .ok()?
        .filter_map(|r| r.ok())
        .collect();

    if results.is_empty() {
        None
    } else {
        Some(results)
    }
}

fn query_cpu_load_24h() -> Option<(f64, f64)> {
    let db_path = "/var/lib/anna/telemetry.db";
    if !std::path::Path::new(db_path).exists() {
        return None;
    }

    let conn = rusqlite::Connection::open(db_path).ok()?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs() as i64;
    let since = now - 86400;

    let mut stmt = conn
        .prepare(
            "SELECT AVG(cpu_percent) as avg_cpu, MAX(cpu_percent) as peak_cpu
             FROM hw_samples
             WHERE timestamp > ?1 AND device = 'cpu'",
        )
        .ok()?;

    stmt.query_row(rusqlite::params![since], |row| {
        let avg: Option<f64> = row.get(0)?;
        let peak: Option<f64> = row.get(1)?;
        Ok((avg.unwrap_or(0.0), peak.unwrap_or(0.0)))
    })
    .ok()
    .filter(|(avg, _)| *avg > 0.0)
}

fn query_gpu_load_24h() -> Option<(f64, f64)> {
    let db_path = "/var/lib/anna/telemetry.db";
    if !std::path::Path::new(db_path).exists() {
        return None;
    }

    let conn = rusqlite::Connection::open(db_path).ok()?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs() as i64;
    let since = now - 86400;

    let mut stmt = conn
        .prepare(
            "SELECT AVG(gpu_percent) as avg_gpu, MAX(gpu_percent) as peak_gpu
             FROM hw_samples
             WHERE timestamp > ?1 AND device LIKE 'gpu%'",
        )
        .ok()?;

    stmt.query_row(rusqlite::params![since], |row| {
        let avg: Option<f64> = row.get(0)?;
        let peak: Option<f64> = row.get(1)?;
        Ok((avg.unwrap_or(0.0), peak.unwrap_or(0.0)))
    })
    .ok()
    .filter(|(avg, _)| *avg > 0.0)
}

fn format_bytes_gib(bytes: u64) -> String {
    let gib = bytes as f64 / (1024.0 * 1024.0 * 1024.0);
    if gib >= 1.0 {
        format!("{:.1} GiB", gib)
    } else {
        let mib = bytes as f64 / (1024.0 * 1024.0);
        format!("{:.0} MiB", mib)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_hotspot_format() {
        let h = CpuHotspot {
            name: "firefox".to_string(),
            avg_percent: 12.5,
            peak_percent: 45.2,
            logical_cores: 16,
        };

        let line = h.format_line();
        assert!(line.contains("firefox"));
        assert!(line.contains("13 percent")); // rounded
        assert!(line.contains("0 - 1600 percent"));
        assert!(line.contains("16 logical cores"));
    }

    #[test]
    fn test_cpu_hotspot_compact() {
        let h = CpuHotspot {
            name: "chrome".to_string(),
            avg_percent: 35.0,
            peak_percent: 80.0,
            logical_cores: 8,
        };

        let compact = h.format_compact();
        assert_eq!(compact, "chrome (35 percent)");
    }

    #[test]
    fn test_memory_hotspot_format() {
        let h = MemoryHotspot {
            name: "code".to_string(),
            avg_bytes: 2_500_000_000,
            peak_bytes: 4_000_000_000,
        };

        let line = h.format_line();
        assert!(line.contains("code"));
        assert!(line.contains("GiB"));
    }

    #[test]
    fn test_temp_hotspot_format() {
        let h = TempHotspot {
            device: "nvme0n1".to_string(),
            avg_temp_c: 44.5,
            min_temp_c: 38.0,
            max_temp_c: 58.0,
        };

        let line = h.format_line();
        assert!(line.contains("nvme0n1"));
        assert!(line.contains("avg 45 C"));
        assert!(line.contains("min 38"));
        assert!(line.contains("max 58"));
    }

    #[test]
    fn test_load_hotspot_format() {
        let h = LoadHotspot {
            component: "CPU".to_string(),
            avg_percent: 15.0,
            range_max: 1600,
            range_unit: "for 16 logical cores".to_string(),
        };

        let line = h.format_line();
        assert!(line.contains("CPU"));
        assert!(line.contains("avg 15 percent"));
        assert!(line.contains("0 - 1600 percent"));
    }

    #[test]
    fn test_gpu_hotspot_format() {
        let h = GpuHotspot {
            name: "ollama".to_string(),
            avg_percent: 20.0,
        };

        let line = h.format_line();
        assert!(line.contains("ollama"));
        assert!(line.contains("20 percent GPU util"));
        assert!(line.contains("0 - 100 percent per GPU"));
    }
}
