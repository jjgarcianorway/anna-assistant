//! Deterministic answer generators for typed probe data.
//!
//! ROUTE phase: Generates verifiable answers from ParsedProbeData.
//! All answers emit claims in extractable format (bytes with B suffix,
//! percents with %, service states as lowercase).

use anna_shared::parsers::{ParsedProbeData, ServiceState, ServiceStatus};

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

#[cfg(test)]
mod tests {
    use super::*;
    use anna_shared::parsers::{DiskUsage, MemoryInfo};

    #[test]
    fn test_memory_answer_format() {
        let data = ParsedProbeData::Memory(MemoryInfo {
            total_bytes: 16_000_000_000,
            used_bytes: 8_000_000_000,
            free_bytes: 4_000_000_000,
            shared_bytes: 100_000_000,
            buff_cache_bytes: 2_000_000_000,
            available_bytes: 6_000_000_000,
            swap_total_bytes: None,
            swap_used_bytes: None,
            swap_free_bytes: None,
        });

        let answer = generate_answer(QueryClass::MemoryUsage, &data).unwrap();

        // Verify extractable claims
        assert!(answer.contains("8000000000B used"));
        assert!(answer.contains("16000000000B total"));
        assert!(answer.contains("6000000000B available"));
    }

    #[test]
    fn test_disk_answer_format() {
        let data = ParsedProbeData::Disk(vec![
            DiskUsage {
                filesystem: "/dev/sda1".to_string(),
                mount: "/".to_string(),
                size_bytes: 100_000_000_000,
                used_bytes: 85_000_000_000,
                available_bytes: 15_000_000_000,
                percent_used: 85,
            },
            DiskUsage {
                filesystem: "/dev/sdb1".to_string(),
                mount: "/home".to_string(),
                size_bytes: 500_000_000_000,
                used_bytes: 250_000_000_000,
                available_bytes: 250_000_000_000,
                percent_used: 50,
            },
        ]);

        let answer = generate_answer(QueryClass::DiskUsage, &data).unwrap();

        // Verify extractable claims
        assert!(answer.contains("/ is 85% full"));
        assert!(answer.contains("/home is 50% full"));
    }

    #[test]
    fn test_service_answer_format() {
        let data = ParsedProbeData::Services(vec![
            ServiceStatus {
                name: "nginx.service".to_string(),
                state: ServiceState::Running,
                description: None,
            },
            ServiceStatus {
                name: "sshd.service".to_string(),
                state: ServiceState::Active,
                description: None,
            },
        ]);

        let answer = generate_answer(QueryClass::ServiceStatus, &data).unwrap();

        // Verify extractable claims
        assert!(answer.contains("nginx.service is running"));
        assert!(answer.contains("sshd.service is active"));
    }

    #[test]
    fn test_wrong_data_type_returns_none() {
        let memory_data = ParsedProbeData::Memory(MemoryInfo {
            total_bytes: 16_000_000_000,
            used_bytes: 8_000_000_000,
            free_bytes: 4_000_000_000,
            shared_bytes: 0,
            buff_cache_bytes: 0,
            available_bytes: 6_000_000_000,
            swap_total_bytes: None,
            swap_used_bytes: None,
            swap_free_bytes: None,
        });

        // DiskUsage class with memory data should return None
        assert!(generate_answer(QueryClass::DiskUsage, &memory_data).is_none());
    }

    #[test]
    fn test_format_bytes_human() {
        assert_eq!(format_bytes_human(500), "500B");
        assert_eq!(format_bytes_human(1024), "1.0 KiB (1024B)");
        assert_eq!(format_bytes_human(1_073_741_824), "1.0 GiB (1073741824B)");
        assert_eq!(
            format_bytes_human(8_804_682_957),
            "8.2 GiB (8804682957B)"
        );
    }
}
