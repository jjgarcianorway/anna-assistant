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

use anna_shared::parsers::{parse_df, parse_failed_units, parse_free, parse_is_active, parse_lsblk, parse_lscpu};
use anna_shared::rpc::ProbeResult;

/// Try to generate memory answer from probe output.
pub fn answer_from_free_probe(probe: &ProbeResult) -> Option<String> {
    let mem = parse_free("free", &probe.stdout).ok()?;
    let data = ParsedProbeData::Memory(mem);
    generate_answer(QueryClass::MemoryUsage, &data)
}

/// Try to generate available/free memory answer from probe output (v0.0.45).
/// Distinct from answer_from_free_probe which shows total/used.
pub fn answer_from_free_probe_available(probe: &ProbeResult) -> Option<String> {
    let mem = parse_free("free", &probe.stdout).ok()?;
    let avail_pct = (mem.available_bytes as f64 / mem.total_bytes as f64 * 100.0) as u8;
    Some(format!(
        "Available memory: {} ({avail_pct}% of total {})",
        format_bytes_short(mem.available_bytes),
        format_bytes_short(mem.total_bytes),
    ))
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

// === v0.0.30: Best-effort summary from ANY evidence ===

use crate::parsers::{parse_ip_addr, parse_ps_aux, parse_df_h};

/// Generate best-effort summary from whatever probe results exist.
/// This is used when specialist times out but we have evidence.
/// Returns (answer, parsed_count) where parsed_count > 0 means useful data.
pub fn generate_best_effort_summary(probe_results: &[ProbeResult]) -> Option<(String, usize)> {
    let mut sections: Vec<String> = Vec::new();
    let mut parsed_count = 0;

    // Try to parse each probe type we know about
    for probe in probe_results {
        if probe.exit_code != 0 {
            continue; // Skip failed probes
        }

        // Memory from free (anna-shared parser)
        if probe.command.starts_with("free") {
            if let Ok(mem) = parse_free("free", &probe.stdout) {
                let used_pct = (mem.used_bytes as f64 / mem.total_bytes as f64 * 100.0) as u8;
                sections.push(format!(
                    "Memory: {} used of {} total ({}% used)",
                    format_bytes_short(mem.used_bytes),
                    format_bytes_short(mem.total_bytes),
                    used_pct
                ));
                parsed_count += 1;
            }
        }

        // Disk from df (anna-shared parser for typed data)
        if probe.command.starts_with("df") {
            if let Ok(disks) = parse_df("df", &probe.stdout) {
                if !disks.is_empty() {
                    let disk_lines: Vec<String> = disks.iter()
                        .map(|d| {
                            let status = if d.percent_used >= 90 { " [CRITICAL]" }
                                else if d.percent_used >= 80 { " [WARNING]" }
                                else { "" };
                            format!("  {} is {}% full{}", d.mount, d.percent_used, status)
                        })
                        .collect();
                    sections.push(format!("Disk Usage:\n{}", disk_lines.join("\n")));
                    parsed_count += 1;
                }
            } else {
                // Fallback to simpler parser
                let disks = parse_df_h(&probe.stdout);
                if !disks.is_empty() {
                    let disk_lines: Vec<String> = disks.iter()
                        .map(|d| {
                            let status = if d.use_percent >= 90 { " [CRITICAL]" }
                                else if d.use_percent >= 80 { " [WARNING]" }
                                else { "" };
                            format!("  {} is {}% full{}", d.mount, d.use_percent, status)
                        })
                        .collect();
                    sections.push(format!("Disk Usage:\n{}", disk_lines.join("\n")));
                    parsed_count += 1;
                }
            }
        }

        // Block devices from lsblk
        if probe.command.starts_with("lsblk") {
            if let Ok(devices) = parse_lsblk("lsblk", &probe.stdout) {
                let total: u64 = devices.iter()
                    .filter(|d| d.device_type == BlockDeviceType::Disk)
                    .map(|d| d.size_bytes)
                    .sum();
                if total > 0 {
                    sections.push(format!("Storage: {} total", format_bytes_short(total)));
                    parsed_count += 1;
                }
            }
        }

        // CPU from lscpu
        if probe.command.starts_with("lscpu") {
            if let Ok(cpu) = parse_lscpu("lscpu", &probe.stdout) {
                sections.push(format!("CPU: {} ({} cores)", cpu.model_name, cpu.cpu_count));
                parsed_count += 1;
            }
        }

        // Network from ip addr (annad parser)
        if probe.command.starts_with("ip addr") || probe.command.starts_with("ip a") {
            let ifaces = parse_ip_addr(&probe.stdout);
            let up_ifaces: Vec<_> = ifaces.iter().filter(|i| i.state == "UP").collect();
            if !up_ifaces.is_empty() {
                let iface_lines: Vec<String> = up_ifaces.iter()
                    .map(|i| {
                        let addr = i.ipv4.as_deref().unwrap_or("no IPv4");
                        format!("  {}: {}", i.name, addr)
                    })
                    .collect();
                sections.push(format!("Network Interfaces (UP):\n{}", iface_lines.join("\n")));
                parsed_count += 1;
            }
        }

        // Top memory/CPU processes from ps aux (annad parser)
        if probe.command.contains("ps") && probe.command.contains("--sort") {
            let procs = parse_ps_aux(&probe.stdout, 5);
            if !procs.is_empty() {
                let is_mem_sort = probe.command.contains("-%mem") || probe.command.contains("-rss");
                let label = if is_mem_sort { "Top Memory" } else { "Top CPU" };
                let proc_lines: Vec<String> = procs.iter().take(3)
                    .map(|p| {
                        let rss = p.rss.as_deref().unwrap_or("?");
                        format!("  {} - {}% CPU, {}% MEM, {}", p.command, p.cpu_percent, p.mem_percent, rss)
                    })
                    .collect();
                sections.push(format!("{} Processes:\n{}", label, proc_lines.join("\n")));
                parsed_count += 1;
            }
        }
    }

    if parsed_count == 0 {
        return None;
    }

    // Build final summary with limitation notice
    let header = "Best-Effort Summary (specialist timed out)";
    let body = sections.join("\n\n");
    let footer = "\nNote: This summary was generated from available probe data. Some information may be incomplete.";

    Some((format!("{}\n{}\n{}{}", header, "=".repeat(40), body, footer), parsed_count))
}
