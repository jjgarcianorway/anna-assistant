//! Deterministic answer generators for typed probe data.
//!
//! ROUTE phase: Generates verifiable answers from ParsedProbeData.
//! All answers emit claims in extractable format (bytes with B suffix,
//! percents with %, service states as lowercase).

use anna_shared::parsers::{
    BlockDevice, BlockDeviceType, CpuInfo, DiskUsage, MemoryInfo, ParsedProbeData, ServiceState,
    ServiceStatus,
};

use crate::router::QueryClass;

/// Generate deterministic answer from parsed probe data.
///
/// Returns None if the query class doesn't match available data
/// or if the data is insufficient for a complete answer.
pub fn generate_answer(class: QueryClass, data: &ParsedProbeData) -> Option<String> {
    match class {
        QueryClass::MemoryUsage => generate_memory_answer(data),
        QueryClass::DiskUsage => generate_disk_answer(data),
        QueryClass::ServiceStatus => generate_service_answer(data),
        _ => None, // Other classes not yet implemented or need LLM
    }
}

/// Generate memory usage answer from MemoryInfo.
/// Emits claims in bytes format: "memory uses 8804682957B"
fn generate_memory_answer(data: &ParsedProbeData) -> Option<String> {
    let mem = match data {
        ParsedProbeData::Memory(m) => m,
        _ => return None,
    };

    let used_pct = (mem.used_bytes as f64 / mem.total_bytes as f64 * 100.0) as u8;
    let avail_pct = (mem.available_bytes as f64 / mem.total_bytes as f64 * 100.0) as u8;

    Some(format!(
        "Memory: {}B used of {}B total ({}% used). {}B available ({}% available).",
        mem.used_bytes,
        mem.total_bytes,
        used_pct,
        mem.available_bytes,
        avail_pct
    ))
}

/// Generate disk usage answer from Vec<DiskUsage>.
/// Emits claims in percent format: "/ is 85% full"
fn generate_disk_answer(data: &ParsedProbeData) -> Option<String> {
    let disks = match data {
        ParsedProbeData::Disk(d) => d,
        _ => return None,
    };

    if disks.is_empty() {
        return None;
    }

    let lines: Vec<String> = disks
        .iter()
        .map(|d| format!("{} is {}% full", d.mount, d.percent_used))
        .collect();

    Some(format!("Disk usage:\n{}", lines.join("\n")))
}

/// Generate service status answer from Vec<ServiceStatus>.
/// Emits claims in state format: "nginx.service is running"
fn generate_service_answer(data: &ParsedProbeData) -> Option<String> {
    match data {
        ParsedProbeData::Service(s) => Some(format_service(s)),
        ParsedProbeData::Services(svcs) => {
            if svcs.is_empty() {
                return None;
            }
            let lines: Vec<String> = svcs.iter().map(format_service).collect();
            Some(lines.join("\n"))
        }
        _ => None,
    }
}

/// Format a single service status with extractable claim.
fn format_service(svc: &ServiceStatus) -> String {
    let state_str = match svc.state {
        ServiceState::Running => "running",
        ServiceState::Active => "active",
        ServiceState::Failed => "failed",
        ServiceState::Inactive => "inactive",
        ServiceState::Activating => "activating",
        ServiceState::Deactivating => "deactivating",
        ServiceState::Reloading => "reloading",
        ServiceState::Unknown => "unknown",
    };

    format!("{} is {}", svc.name, state_str)
}

/// Generate system health summary from multiple probe outputs.
/// Composes memory, disk, block devices, and CPU info into a single report.
pub fn generate_health_summary(probes: &[ParsedProbeData]) -> Option<String> {
    let mut sections: Vec<String> = Vec::new();

    // Extract each data type from probes
    let mut memory: Option<&MemoryInfo> = None;
    let mut disks: Vec<&DiskUsage> = Vec::new();
    let mut block_devices: Vec<&BlockDevice> = Vec::new();
    let mut cpu: Option<&CpuInfo> = None;

    for probe in probes {
        match probe {
            ParsedProbeData::Memory(m) => memory = Some(m),
            ParsedProbeData::Disk(d) => disks.extend(d.iter()),
            ParsedProbeData::BlockDevices(b) => block_devices.extend(b.iter()),
            ParsedProbeData::Cpu(c) => cpu = Some(c),
            _ => {}
        }
    }

    // CPU section
    if let Some(c) = cpu {
        let mut cpu_lines = vec![format!("CPU: {}", c.model_name)];
        cpu_lines.push(format!("  Logical CPUs: {}", c.cpu_count));
        if let Some(cores) = c.physical_cores() {
            cpu_lines.push(format!("  Physical cores: {}", cores));
        }
        if let Some(ht) = c.hyperthreading() {
            cpu_lines.push(format!("  Hyperthreading: {}", if ht { "yes" } else { "no" }));
        }
        sections.push(cpu_lines.join("\n"));
    }

    // Memory section
    if let Some(m) = memory {
        let used_pct = (m.used_bytes as f64 / m.total_bytes as f64 * 100.0) as u8;
        let avail_pct = (m.available_bytes as f64 / m.total_bytes as f64 * 100.0) as u8;
        let mut mem_lines = vec!["Memory:".to_string()];
        mem_lines.push(format!(
            "  Total: {} ({}B)",
            format_bytes_short(m.total_bytes),
            m.total_bytes
        ));
        mem_lines.push(format!(
            "  Used: {} ({}%) - {}B used",
            format_bytes_short(m.used_bytes),
            used_pct,
            m.used_bytes
        ));
        mem_lines.push(format!(
            "  Available: {} ({}%) - {}B available",
            format_bytes_short(m.available_bytes),
            avail_pct,
            m.available_bytes
        ));
        sections.push(mem_lines.join("\n"));
    }

    // Storage section (from block devices)
    if !block_devices.is_empty() {
        let total_disk: u64 = block_devices
            .iter()
            .filter(|d| d.device_type == BlockDeviceType::Disk)
            .map(|d| d.size_bytes)
            .sum();
        let mut storage_lines = vec!["Storage:".to_string()];
        storage_lines.push(format!(
            "  Total disk: {} ({}B)",
            format_bytes_short(total_disk),
            total_disk
        ));

        // List disks
        for dev in block_devices.iter().filter(|d| d.device_type == BlockDeviceType::Disk) {
            storage_lines.push(format!(
                "  {} - {}",
                dev.name,
                format_bytes_short(dev.size_bytes)
            ));
        }
        sections.push(storage_lines.join("\n"));
    }

    // Disk usage section (from df)
    if !disks.is_empty() {
        let mut disk_lines = vec!["Disk Usage:".to_string()];
        for d in &disks {
            let status = if d.percent_used >= 90 {
                " [CRITICAL]"
            } else if d.percent_used >= 80 {
                " [WARNING]"
            } else {
                ""
            };
            disk_lines.push(format!(
                "  {} is {}% full{}",
                d.mount, d.percent_used, status
            ));
        }
        sections.push(disk_lines.join("\n"));
    }

    if sections.is_empty() {
        return None;
    }

    Some(format!("System Health Summary\n{}\n{}", "=".repeat(20), sections.join("\n\n")))
}

/// Format bytes in short human-readable form.
fn format_bytes_short(bytes: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = 1024 * 1024;
    const GIB: u64 = 1024 * 1024 * 1024;
    const TIB: u64 = 1024 * 1024 * 1024 * 1024;

    if bytes >= TIB {
        format!("{:.1} TiB", bytes as f64 / TIB as f64)
    } else if bytes >= GIB {
        format!("{:.1} GiB", bytes as f64 / GIB as f64)
    } else if bytes >= MIB {
        format!("{:.1} MiB", bytes as f64 / MIB as f64)
    } else if bytes >= KIB {
        format!("{:.1} KiB", bytes as f64 / KIB as f64)
    } else {
        format!("{}B", bytes)
    }
}

// === Probe-to-answer helpers (used by deterministic.rs) ===

use anna_shared::parsers::{parse_df, parse_failed_units, parse_free, parse_is_active};
use anna_shared::rpc::ProbeResult;

/// Try to generate memory answer from probe output.
pub fn answer_from_free_probe(probe: &ProbeResult) -> Option<String> {
    let mem = parse_free("free", &probe.stdout).ok()?;
    let data = ParsedProbeData::Memory(mem);
    generate_answer(QueryClass::MemoryUsage, &data)
}

/// Try to generate disk usage answer from probe output.
pub fn answer_from_df_probe(probe: &ProbeResult) -> Option<String> {
    let disks = parse_df("df", &probe.stdout).ok()?;
    let data = ParsedProbeData::Disk(disks);
    generate_answer(QueryClass::DiskUsage, &data)
}

/// Try to generate service status answer from is-active probe.
pub fn answer_from_is_active_probe(probe: &ProbeResult, service_name: &str) -> Option<String> {
    let svc = parse_is_active("systemctl", service_name, &probe.stdout).ok()?;
    let data = ParsedProbeData::Service(svc);
    generate_answer(QueryClass::ServiceStatus, &data)
}

/// Try to generate service status answer from failed units probe.
pub fn answer_from_failed_units_probe(probe: &ProbeResult) -> Option<String> {
    let svcs = parse_failed_units("systemctl", &probe.stdout).ok()?;
    let data = ParsedProbeData::Services(svcs);
    generate_answer(QueryClass::ServiceStatus, &data)
}

/// Format bytes in human-readable form with exact bytes in parentheses.
/// Example: "8.2 GiB (8804682957B)"
#[allow(dead_code)]
pub fn format_bytes_human(bytes: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = 1024 * 1024;
    const GIB: u64 = 1024 * 1024 * 1024;
    const TIB: u64 = 1024 * 1024 * 1024 * 1024;

    let (value, unit) = if bytes >= TIB {
        (bytes as f64 / TIB as f64, "TiB")
    } else if bytes >= GIB {
        (bytes as f64 / GIB as f64, "GiB")
    } else if bytes >= MIB {
        (bytes as f64 / MIB as f64, "MiB")
    } else if bytes >= KIB {
        (bytes as f64 / KIB as f64, "KiB")
    } else {
        return format!("{}B", bytes);
    };

    format!("{:.1} {} ({}B)", value, unit, bytes)
}
