//! Deterministic answerer - produces answers without LLM when data is available.
//!
//! Answers common queries by parsing hardware snapshots and probe outputs.
//! Used as primary answerer for known query classes, fallback for LLM timeout.

use anna_shared::rpc::{ProbeResult, RuntimeContext};

use crate::parsers::find_probe;
use crate::probe_answers;
use crate::router::{classify_query, QueryClass};

/// Result from deterministic answerer with metadata
pub struct DeterministicResult {
    pub answer: String,
    #[allow(dead_code)]
    pub grounded: bool,
    pub parsed_data_count: usize, // Number of parsed entries (0 = empty)
    pub route_class: String,      // Query class used for routing (for trace)
}

/// Try to produce a deterministic answer from available data
pub fn try_answer(
    query: &str,
    context: &RuntimeContext,
    probe_results: &[ProbeResult],
) -> Option<DeterministicResult> {
    let query_class = classify_query(query);
    let route_class = query_class.to_string();

    match query_class {
        QueryClass::CpuInfo => probe_answers::answer_cpu_info(&context.hardware, probe_results)
            .map(|mut r| { r.route_class = route_class; r }),
        QueryClass::RamInfo => probe_answers::answer_ram_info(&context.hardware, probe_results)
            .map(|mut r| { r.route_class = route_class; r }),
        QueryClass::GpuInfo => probe_answers::answer_gpu_info(&context.hardware)
            .map(|mut r| { r.route_class = route_class; r }),
        QueryClass::TopMemoryProcesses => probe_answers::answer_top_memory(probe_results)
            .map(|mut r| { r.route_class = route_class; r }),
        QueryClass::TopCpuProcesses => probe_answers::answer_top_cpu(probe_results)
            .map(|mut r| { r.route_class = route_class; r }),
        QueryClass::DiskSpace => probe_answers::answer_disk_space(probe_results)
            .map(|mut r| { r.route_class = route_class; r }),
        QueryClass::NetworkInterfaces => probe_answers::answer_network_interfaces(probe_results)
            .map(|mut r| { r.route_class = route_class; r }),
        QueryClass::Help => Some(answer_help(&route_class)),
        QueryClass::SystemSlow => probe_answers::answer_system_slow(probe_results)
            .map(|mut r| { r.route_class = route_class; r }),
        QueryClass::MemoryUsage => answer_memory_usage(probe_results, &route_class),
        QueryClass::DiskUsage => answer_disk_usage(probe_results, &route_class),
        QueryClass::ServiceStatus => answer_service_status(probe_results, &route_class),
        QueryClass::SystemHealthSummary => answer_system_health_summary(probe_results, &route_class),
        QueryClass::Unknown => None,
    }
}

/// Help response describing available commands
fn answer_help(route_class: &str) -> DeterministicResult {
    let answer = r#"**Anna - Linux System Assistant**

I can answer questions about your system:

**Hardware Information:**
- "What CPU do I have?" - Show CPU model and cores
- "How much RAM?" - Show total memory
- "What GPU?" - Show graphics card info

**Process Monitoring:**
- "Top memory processes" - Show processes using most RAM
- "What's using CPU?" - Show processes using most CPU

**Storage:**
- "Disk space" - Show filesystem usage with warnings

**Network:**
- "Network interfaces" - Show IPs and interface status

**System Health:**
- "System health" - Full system summary
- "Status report" - Overview of CPU, memory, disk

**Diagnostics:**
- "It's slow" - Run full diagnostic (CPU, memory, disk)

Ask a question to get started!"#;

    DeterministicResult {
        answer: answer.to_string(),
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    }
}

// === ROUTE Phase: Typed query class handlers ===
// Uses helper functions from answers module

/// Answer memory usage query using parsed free output
fn answer_memory_usage(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "free")?;
    let answer = crate::answers::answer_from_free_probe(probe)?;
    Some(DeterministicResult {
        answer, grounded: true, parsed_data_count: 1, route_class: route_class.to_string(),
    })
}

/// Answer disk usage query using parsed df output
fn answer_disk_usage(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "df")?;
    let answer = crate::answers::answer_from_df_probe(probe)?;
    Some(DeterministicResult {
        answer, grounded: true, parsed_data_count: 1, route_class: route_class.to_string(),
    })
}

/// Answer service status query using parsed systemctl output
fn answer_service_status(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    // Try is-active probe first
    if let Some(probe) = find_probe(probes, "systemctl is-active") {
        let service_name = probe.command.strip_prefix("systemctl is-active ").unwrap_or("service");
        if let Some(answer) = crate::answers::answer_from_is_active_probe(probe, service_name) {
            return Some(DeterministicResult {
                answer, grounded: true, parsed_data_count: 1, route_class: route_class.to_string(),
            });
        }
    }
    // Try failed units probe
    if let Some(probe) = find_probe(probes, "systemctl --failed") {
        if let Some(answer) = crate::answers::answer_from_failed_units_probe(probe) {
            return Some(DeterministicResult {
                answer, grounded: true, parsed_data_count: 1, route_class: route_class.to_string(),
            });
        }
    }
    None
}

/// Answer system health summary using multiple parsed probe outputs
fn answer_system_health_summary(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    use anna_shared::parsers::{parse_df, parse_free, parse_lsblk, parse_lscpu, ParsedProbeData};

    let mut parsed_data: Vec<ParsedProbeData> = Vec::new();

    // Parse free output
    if let Some(probe) = find_probe(probes, "free") {
        if let Ok(mem) = parse_free("free", &probe.stdout) {
            parsed_data.push(ParsedProbeData::Memory(mem));
        }
    }

    // Parse df output
    if let Some(probe) = find_probe(probes, "df") {
        if let Ok(disks) = parse_df("df", &probe.stdout) {
            parsed_data.push(ParsedProbeData::Disk(disks));
        }
    }

    // Parse lsblk output
    if let Some(probe) = find_probe(probes, "lsblk") {
        if let Ok(devices) = parse_lsblk("lsblk", &probe.stdout) {
            parsed_data.push(ParsedProbeData::BlockDevices(devices));
        }
    }

    // Parse lscpu output
    if let Some(probe) = find_probe(probes, "lscpu") {
        if let Ok(cpu) = parse_lscpu("lscpu", &probe.stdout) {
            parsed_data.push(ParsedProbeData::Cpu(cpu));
        }
    }

    let count = parsed_data.len();
    let answer = crate::answers::generate_health_summary(&parsed_data)?;
    Some(DeterministicResult {
        answer, grounded: true, parsed_data_count: count, route_class: route_class.to_string(),
    })
}

// Unit tests in tests/deterministic_tests.rs
