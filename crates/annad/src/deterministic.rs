//! Deterministic answerer - produces answers without LLM when data is available.
//!
//! Answers common queries by parsing hardware snapshots and probe outputs.
//! Used as primary answerer for known query classes, fallback for LLM timeout.

use anna_shared::rpc::{HardwareSummary, ProbeResult, RuntimeContext};

use crate::parsers::{
    find_probe, parse_df_h, parse_free_total, parse_ip_addr, parse_lscpu, parse_ps_aux,
};
use crate::router::{classify_query, QueryClass};

/// Result from deterministic answerer with metadata
pub struct DeterministicResult {
    pub answer: String,
    #[allow(dead_code)]
    pub grounded: bool,
    pub parsed_data_count: usize, // Number of parsed entries (0 = empty)
}

/// Try to produce a deterministic answer from available data
pub fn try_answer(
    query: &str,
    context: &RuntimeContext,
    probe_results: &[ProbeResult],
) -> Option<DeterministicResult> {
    let query_class = classify_query(query);

    match query_class {
        QueryClass::CpuInfo => answer_cpu_info(&context.hardware, probe_results),
        QueryClass::RamInfo => answer_ram_info(&context.hardware, probe_results),
        QueryClass::GpuInfo => answer_gpu_info(&context.hardware),
        QueryClass::TopMemoryProcesses => answer_top_memory(probe_results),
        QueryClass::TopCpuProcesses => answer_top_cpu(probe_results),
        QueryClass::DiskSpace => answer_disk_space(probe_results),
        QueryClass::NetworkInterfaces => answer_network_interfaces(probe_results),
        QueryClass::Help => Some(answer_help()),
        QueryClass::SystemSlow => answer_system_slow(probe_results),
        // ROUTE Phase: Typed query classes - use answers module for these
        QueryClass::MemoryUsage => answer_memory_usage(probe_results),
        QueryClass::DiskUsage => answer_disk_usage(probe_results),
        QueryClass::ServiceStatus => answer_service_status(probe_results),
        QueryClass::Unknown => None,
    }
}

/// Help response describing available commands
fn answer_help() -> DeterministicResult {
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

**Diagnostics:**
- "It's slow" - Run full diagnostic (CPU, memory, disk)

Ask a question to get started!"#;

    DeterministicResult {
        answer: answer.to_string(),
        grounded: true,
        parsed_data_count: 1,
    }
}

/// Answer CPU info from hardware snapshot or lscpu probe
fn answer_cpu_info(hardware: &HardwareSummary, probes: &[ProbeResult]) -> Option<DeterministicResult> {
    // Try hardware snapshot first
    if !hardware.cpu_model.is_empty() && hardware.cpu_model != "Unknown" {
        return Some(DeterministicResult {
            answer: format!(
                "**CPU:** {} ({} cores)",
                hardware.cpu_model, hardware.cpu_cores
            ),
            grounded: true,
            parsed_data_count: 1,
        });
    }

    // Fallback to lscpu probe
    if let Some(probe) = find_probe(probes, "lscpu") {
        if let Some((model, cores)) = parse_lscpu(&probe.stdout) {
            return Some(DeterministicResult {
                answer: format!("**CPU:** {} ({} cores)", model, cores),
                grounded: true,
                parsed_data_count: 1,
            });
        }
    }

    None
}

/// Answer RAM info from hardware snapshot or free -h probe
fn answer_ram_info(hardware: &HardwareSummary, probes: &[ProbeResult]) -> Option<DeterministicResult> {
    // Try hardware snapshot first
    if hardware.ram_gb > 0.0 {
        return Some(DeterministicResult {
            answer: format!("**RAM:** {:.1} GB total", hardware.ram_gb),
            grounded: true,
            parsed_data_count: 1,
        });
    }

    // Fallback to free -h probe
    if let Some(probe) = find_probe(probes, "free") {
        if let Some(total) = parse_free_total(&probe.stdout) {
            return Some(DeterministicResult {
                answer: format!("**RAM:** {} total", total),
                grounded: true,
                parsed_data_count: 1,
            });
        }
    }

    None
}

/// Answer GPU info from hardware snapshot
fn answer_gpu_info(hardware: &HardwareSummary) -> Option<DeterministicResult> {
    let answer = match (&hardware.gpu, hardware.gpu_vram_gb) {
        (Some(model), Some(vram)) => format!("**GPU:** {} ({:.1} GB VRAM)", model, vram),
        (Some(model), None) => format!("**GPU:** {}", model),
        (None, _) => "**GPU:** No dedicated GPU detected".to_string(),
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: 1,
    })
}

/// Answer top memory processes from ps aux probe
fn answer_top_memory(probes: &[ProbeResult]) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "ps aux --sort=-%mem")?;
    let processes = parse_ps_aux(&probe.stdout, 10);

    if processes.is_empty() {
        return None;
    }

    let mut answer = String::from("**Top 10 processes by memory usage:**\n\n");
    answer.push_str("| PID | COMMAND | %MEM | RSS | USER |\n");
    answer.push_str("|-----|---------|------|-----|------|\n");

    for proc in &processes {
        let rss_display = proc.rss.as_deref().unwrap_or("-");
        answer.push_str(&format!(
            "| {} | {} | {}% | {} | {} |\n",
            proc.pid,
            truncate(&proc.command, 30),
            proc.mem_percent,
            rss_display,
            truncate(&proc.user, 10)
        ));
    }

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: processes.len(),
    })
}

/// Answer top CPU processes
fn answer_top_cpu(probes: &[ProbeResult]) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "ps aux --sort=-%cpu")?;
    let processes = parse_ps_aux(&probe.stdout, 10);

    if processes.is_empty() {
        return None;
    }

    let mut answer = String::from("**Top 10 processes by CPU usage:**\n\n");
    answer.push_str("| PID | COMMAND | %CPU | USER |\n");
    answer.push_str("|-----|---------|------|------|\n");

    for proc in &processes {
        answer.push_str(&format!(
            "| {} | {} | {}% | {} |\n",
            proc.pid,
            truncate(&proc.command, 30),
            proc.cpu_percent,
            truncate(&proc.user, 10)
        ));
    }

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: processes.len(),
    })
}

/// Answer disk space from df -h probe
fn answer_disk_space(probes: &[ProbeResult]) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "df -h")?;
    let filesystems = parse_df_h(&probe.stdout);

    if filesystems.is_empty() {
        return None;
    }

    let mut answer = String::from("**Filesystem usage:**\n\n");
    answer.push_str("| Filesystem | Size | Used | Avail | Use% | Mount |\n");
    answer.push_str("|------------|------|------|-------|------|-------|\n");

    for fs in &filesystems {
        let status = if fs.use_percent >= 95 {
            " ⚠️ CRITICAL"
        } else if fs.use_percent >= 85 {
            " ⚠️"
        } else {
            ""
        };

        answer.push_str(&format!(
            "| {} | {} | {} | {} | {}%{} | {} |\n",
            truncate(&fs.filesystem, 15),
            fs.size,
            fs.used,
            fs.avail,
            fs.use_percent,
            status,
            fs.mount
        ));
    }

    // Summary
    let critical: Vec<_> = filesystems.iter().filter(|f| f.use_percent >= 95).collect();
    let warning: Vec<_> = filesystems.iter().filter(|f| f.use_percent >= 85 && f.use_percent < 95).collect();

    if !critical.is_empty() {
        answer.push_str(&format!("\n**Critical:** {} filesystem(s) at >=95%", critical.len()));
    }
    if !warning.is_empty() {
        answer.push_str(&format!("\n**Warning:** {} filesystem(s) at >=85%", warning.len()));
    }

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: filesystems.len(),
    })
}

/// Answer network interfaces from ip addr show probe
fn answer_network_interfaces(probes: &[ProbeResult]) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "ip addr")?;
    let interfaces = parse_ip_addr(&probe.stdout);

    if interfaces.is_empty() {
        return None;
    }

    // Find active interface (UP, has IPv4, not loopback)
    let active = interfaces.iter().find(|i| {
        i.state == "UP" && i.ipv4.is_some() && i.name != "lo"
    });

    let mut answer = String::new();

    // Show active connection at top
    if let Some(iface) = active {
        let iface_type = classify_interface_type(&iface.name);
        let ip = iface.ipv4.as_deref().unwrap_or("-");
        answer.push_str(&format!(
            "**Active: {} ({})** - {}\n\n",
            iface_type, iface.name, ip
        ));
    }

    answer.push_str("**All interfaces:**\n\n");
    answer.push_str("| Interface | Type | IPv4 | State |\n");
    answer.push_str("|-----------|------|------|-------|\n");

    for iface in &interfaces {
        let iface_type = classify_interface_type(&iface.name);
        let ipv4 = iface.ipv4.as_deref().unwrap_or("-");
        let state_display = if iface.state == "UP" { "UP ✓" } else { &iface.state };
        answer.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            iface.name, iface_type, ipv4, state_display
        ));
    }

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: interfaces.len(),
    })
}

/// Answer system slow diagnostic
fn answer_system_slow(probes: &[ProbeResult]) -> Option<DeterministicResult> {
    let mut sections = Vec::new();
    let mut total_parsed = 0;

    // CPU section
    if let Some(probe) = find_probe(probes, "ps aux --sort=-%cpu") {
        let processes = parse_ps_aux(&probe.stdout, 3);
        if !processes.is_empty() {
            let mut s = String::from("**Top CPU consumers:**\n");
            for proc in &processes {
                s.push_str(&format!("- {} ({}% CPU)\n", truncate(&proc.command, 30), proc.cpu_percent));
            }
            sections.push(s);
            total_parsed += processes.len();
        }
    }

    // Memory section
    if let Some(probe) = find_probe(probes, "ps aux --sort=-%mem") {
        let processes = parse_ps_aux(&probe.stdout, 3);
        if !processes.is_empty() {
            let mut s = String::from("**Top memory consumers:**\n");
            for proc in &processes {
                s.push_str(&format!("- {} ({}% RAM)\n", truncate(&proc.command, 30), proc.mem_percent));
            }
            sections.push(s);
            total_parsed += processes.len();
        }
    }

    // Disk section
    if let Some(probe) = find_probe(probes, "df -h") {
        let filesystems = parse_df_h(&probe.stdout);
        let critical: Vec<_> = filesystems.iter().filter(|f| f.use_percent >= 85).collect();
        if !critical.is_empty() {
            let mut s = String::from("**Disk warnings:**\n");
            for fs in &critical {
                s.push_str(&format!("- {} at {}% ({} free)\n", fs.mount, fs.use_percent, fs.avail));
            }
            sections.push(s);
            total_parsed += critical.len();
        }
    }

    if sections.is_empty() {
        return None;
    }

    Some(DeterministicResult {
        answer: format!("**System Diagnostic:**\n\n{}", sections.join("\n")),
        grounded: true,
        parsed_data_count: total_parsed,
    })
}

/// Classify interface type (wifi vs ethernet)
fn classify_interface_type(name: &str) -> &'static str {
    if name.starts_with("wlan") || name.starts_with("wlp") || name.starts_with("wl") {
        "WiFi"
    } else if name.starts_with("eth") || name.starts_with("enp") || name.starts_with("en") {
        "Ethernet"
    } else if name == "lo" {
        "Loopback"
    } else {
        "Other"
    }
}

/// Truncate string with ellipsis
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

// === ROUTE Phase: Typed query class handlers ===
// Uses helper functions from answers module

/// Answer memory usage query using parsed free output
fn answer_memory_usage(probes: &[ProbeResult]) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "free")?;
    let answer = crate::answers::answer_from_free_probe(probe)?;
    Some(DeterministicResult { answer, grounded: true, parsed_data_count: 1 })
}

/// Answer disk usage query using parsed df output
fn answer_disk_usage(probes: &[ProbeResult]) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "df")?;
    let answer = crate::answers::answer_from_df_probe(probe)?;
    Some(DeterministicResult { answer, grounded: true, parsed_data_count: 1 })
}

/// Answer service status query using parsed systemctl output
fn answer_service_status(probes: &[ProbeResult]) -> Option<DeterministicResult> {
    // Try is-active probe first
    if let Some(probe) = find_probe(probes, "systemctl is-active") {
        let service_name = probe.command.strip_prefix("systemctl is-active ").unwrap_or("service");
        if let Some(answer) = crate::answers::answer_from_is_active_probe(probe, service_name) {
            return Some(DeterministicResult { answer, grounded: true, parsed_data_count: 1 });
        }
    }
    // Try failed units probe
    if let Some(probe) = find_probe(probes, "systemctl --failed") {
        if let Some(answer) = crate::answers::answer_from_failed_units_probe(probe) {
            return Some(DeterministicResult { answer, grounded: true, parsed_data_count: 1 });
        }
    }
    None
}

// Unit tests in tests/deterministic_tests.rs
