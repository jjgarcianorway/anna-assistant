//! Deterministic answerer - produces answers without LLM when data is available.
//!
//! This module answers common queries by parsing hardware snapshots and probe outputs.
//! Used as fallback when specialist LLM times out or errors.

use anna_shared::rpc::{HardwareSummary, ProbeResult, RuntimeContext};

use crate::parsers::{
    find_probe, parse_df_h, parse_free_total, parse_ip_addr, parse_lscpu, parse_ps_aux,
};

/// Query types that can be answered deterministically
#[derive(Debug, Clone, PartialEq)]
pub enum QueryType {
    CpuInfo,
    RamInfo,
    GpuInfo,
    TopMemoryProcesses,
    DiskSpace,
    NetworkInterfaces,
    Unknown,
}

/// Classify query to determine if deterministic answer is possible
pub fn classify_query(query: &str) -> QueryType {
    let q = query.to_lowercase();

    if q.contains("cpu") || q.contains("processor") || q.contains("core") {
        QueryType::CpuInfo
    } else if q.contains("ram") || q.contains("memory") && !q.contains("process") {
        QueryType::RamInfo
    } else if q.contains("gpu") || q.contains("graphics") || q.contains("vram") {
        QueryType::GpuInfo
    } else if q.contains("process") && (q.contains("memory") || q.contains("ram"))
        || q.contains("memory hog")
        || q.contains("top memory")
        || q.contains("most memory")
    {
        QueryType::TopMemoryProcesses
    } else if q.contains("disk")
        || q.contains("space")
        || q.contains("storage")
        || q.contains("filesystem")
        || q.contains("mount")
    {
        QueryType::DiskSpace
    } else if q.contains("network")
        || q.contains("interface")
        || q.contains("ip ")
        || q.contains("ip?")
        || q.contains("ips")
    {
        QueryType::NetworkInterfaces
    } else {
        QueryType::Unknown
    }
}

/// Try to produce a deterministic answer from available data
pub fn try_answer(
    query: &str,
    context: &RuntimeContext,
    probe_results: &[ProbeResult],
) -> Option<String> {
    let query_type = classify_query(query);

    match query_type {
        QueryType::CpuInfo => answer_cpu_info(&context.hardware, probe_results),
        QueryType::RamInfo => answer_ram_info(&context.hardware, probe_results),
        QueryType::GpuInfo => answer_gpu_info(&context.hardware),
        QueryType::TopMemoryProcesses => answer_top_memory(probe_results),
        QueryType::DiskSpace => answer_disk_space(probe_results),
        QueryType::NetworkInterfaces => answer_network_interfaces(probe_results),
        QueryType::Unknown => None,
    }
}

/// Answer CPU info from hardware snapshot or lscpu probe
fn answer_cpu_info(hardware: &HardwareSummary, probes: &[ProbeResult]) -> Option<String> {
    // First try hardware snapshot
    if !hardware.cpu_model.is_empty() && hardware.cpu_model != "Unknown" {
        return Some(format!(
            "Your CPU is: **{}** with **{} cores**.",
            hardware.cpu_model, hardware.cpu_cores
        ));
    }

    // Fallback to lscpu probe
    if let Some(probe) = find_probe(probes, "lscpu") {
        if let Some(info) = parse_lscpu(&probe.stdout) {
            return Some(format!(
                "Your CPU is: **{}** with **{} cores**.",
                info.0, info.1
            ));
        }
    }

    None
}

/// Answer RAM info from hardware snapshot or free -h probe
fn answer_ram_info(hardware: &HardwareSummary, probes: &[ProbeResult]) -> Option<String> {
    // First try hardware snapshot
    if hardware.ram_gb > 0.0 {
        return Some(format!(
            "Your system has **{:.1} GB** of RAM.",
            hardware.ram_gb
        ));
    }

    // Fallback to free -h probe
    if let Some(probe) = find_probe(probes, "free") {
        if let Some(total) = parse_free_total(&probe.stdout) {
            return Some(format!("Your system has **{}** of RAM.", total));
        }
    }

    None
}

/// Answer GPU info from hardware snapshot
fn answer_gpu_info(hardware: &HardwareSummary) -> Option<String> {
    match (&hardware.gpu, hardware.gpu_vram_gb) {
        (Some(model), Some(vram)) => Some(format!(
            "Your GPU is: **{}** with **{:.1} GB VRAM**.",
            model, vram
        )),
        (Some(model), None) => Some(format!("Your GPU is: **{}**.", model)),
        (None, _) => Some("No dedicated GPU detected.".to_string()),
    }
}

/// Answer top memory processes from ps aux probe
fn answer_top_memory(probes: &[ProbeResult]) -> Option<String> {
    let probe = find_probe(probes, "ps aux --sort=-%mem")?;
    let processes = parse_ps_aux(&probe.stdout, 5);

    if processes.is_empty() {
        return None;
    }

    let mut answer = String::from("**Top processes by memory usage:**\n\n");
    answer.push_str("| # | Process | Memory | CPU | User |\n");
    answer.push_str("|---|---------|--------|-----|------|\n");

    for (i, proc) in processes.iter().enumerate() {
        answer.push_str(&format!(
            "| {} | {} | {}% | {}% | {} |\n",
            i + 1,
            proc.command,
            proc.mem_percent,
            proc.cpu_percent,
            proc.user
        ));
    }

    Some(answer)
}

/// Answer disk space from df -h probe
fn answer_disk_space(probes: &[ProbeResult]) -> Option<String> {
    let probe = find_probe(probes, "df -h")?;
    let filesystems = parse_df_h(&probe.stdout);

    if filesystems.is_empty() {
        return None;
    }

    let mut answer = String::from("**Filesystem usage:**\n\n");
    answer.push_str("| Filesystem | Size | Used | Avail | Use% | Mounted on |\n");
    answer.push_str("|------------|------|------|-------|------|------------|\n");

    for fs in &filesystems {
        let status = if fs.use_percent >= 95 {
            " **CRITICAL**"
        } else if fs.use_percent >= 85 {
            " *warning*"
        } else {
            ""
        };

        answer.push_str(&format!(
            "| {} | {} | {} | {} | {}%{} | {} |\n",
            fs.filesystem, fs.size, fs.used, fs.avail, fs.use_percent, status, fs.mount
        ));
    }

    // Add summary
    let critical: Vec<_> = filesystems.iter().filter(|f| f.use_percent >= 95).collect();
    let warning: Vec<_> = filesystems
        .iter()
        .filter(|f| f.use_percent >= 85 && f.use_percent < 95)
        .collect();

    if !critical.is_empty() {
        answer.push_str(&format!(
            "\n**Critical:** {} filesystem(s) at >=95% capacity.",
            critical.len()
        ));
    }
    if !warning.is_empty() {
        answer.push_str(&format!(
            "\n**Warning:** {} filesystem(s) at >=85% capacity.",
            warning.len()
        ));
    }

    Some(answer)
}

/// Answer network interfaces from ip addr show probe
fn answer_network_interfaces(probes: &[ProbeResult]) -> Option<String> {
    let probe = find_probe(probes, "ip addr")?;
    let interfaces = parse_ip_addr(&probe.stdout);

    if interfaces.is_empty() {
        return None;
    }

    let mut answer = String::from("**Network interfaces:**\n\n");
    answer.push_str("| Interface | IPv4 | IPv6 | State |\n");
    answer.push_str("|-----------|------|------|-------|\n");

    for iface in &interfaces {
        let ipv4 = iface.ipv4.as_deref().unwrap_or("-");
        let ipv6 = iface.ipv6.as_deref().unwrap_or("-");
        answer.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            iface.name, ipv4, ipv6, iface.state
        ));
    }

    Some(answer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_query() {
        assert_eq!(classify_query("what cpu do i have?"), QueryType::CpuInfo);
        assert_eq!(classify_query("how much ram"), QueryType::RamInfo);
        assert_eq!(classify_query("show gpu info"), QueryType::GpuInfo);
        assert_eq!(
            classify_query("processes using most memory"),
            QueryType::TopMemoryProcesses
        );
        assert_eq!(classify_query("disk space free"), QueryType::DiskSpace);
        assert_eq!(
            classify_query("network interfaces and ips"),
            QueryType::NetworkInterfaces
        );
    }

    #[test]
    fn test_answer_cpu_info() {
        let hw = HardwareSummary {
            cpu_model: "Intel i9-14900HX".to_string(),
            cpu_cores: 32,
            ram_gb: 32.0,
            gpu: None,
            gpu_vram_gb: None,
        };
        let answer = answer_cpu_info(&hw, &[]).unwrap();
        assert!(answer.contains("Intel i9-14900HX"));
        assert!(answer.contains("32 cores"));
    }

    #[test]
    fn test_answer_ram_info() {
        let hw = HardwareSummary {
            cpu_model: "Test".to_string(),
            cpu_cores: 4,
            ram_gb: 16.0,
            gpu: None,
            gpu_vram_gb: None,
        };
        let answer = answer_ram_info(&hw, &[]).unwrap();
        assert!(answer.contains("16.0 GB"));
    }

    #[test]
    fn test_answer_gpu_info() {
        let hw = HardwareSummary {
            cpu_model: "Test".to_string(),
            cpu_cores: 4,
            ram_gb: 16.0,
            gpu: Some("NVIDIA RTX 4090".to_string()),
            gpu_vram_gb: Some(24.0),
        };
        let answer = answer_gpu_info(&hw).unwrap();
        assert!(answer.contains("RTX 4090"));
        assert!(answer.contains("24.0 GB VRAM"));
    }
}
