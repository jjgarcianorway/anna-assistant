//! Integration tests for deterministic answerer - Hardware data edge cases.

use anna_shared::rpc::{HardwareSummary, ProbeResult, RuntimeContext};

// Reuse the deterministic_answerer module
mod deterministic_answerer {
    use super::*;

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

    pub fn try_answer(
        query: &str,
        context: &RuntimeContext,
        probe_results: &[ProbeResult],
    ) -> Option<String> {
        let query_type = classify_query(query);

        match query_type {
            QueryType::CpuInfo => answer_cpu_info(&context.hardware),
            QueryType::RamInfo => answer_ram_info(&context.hardware),
            QueryType::GpuInfo => answer_gpu_info(&context.hardware),
            QueryType::TopMemoryProcesses => answer_top_memory(probe_results),
            QueryType::DiskSpace => answer_disk_space(probe_results),
            QueryType::NetworkInterfaces => answer_network_interfaces(probe_results),
            QueryType::Unknown => None,
        }
    }

    fn answer_cpu_info(hardware: &HardwareSummary) -> Option<String> {
        if !hardware.cpu_model.is_empty() && hardware.cpu_model != "Unknown" {
            Some(format!(
                "Your CPU is: **{}** with **{} cores**.",
                hardware.cpu_model, hardware.cpu_cores
            ))
        } else {
            None
        }
    }

    fn answer_ram_info(hardware: &HardwareSummary) -> Option<String> {
        if hardware.ram_gb > 0.0 {
            Some(format!(
                "Your system has **{:.1} GB** of RAM.",
                hardware.ram_gb
            ))
        } else {
            None
        }
    }

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

    fn answer_top_memory(probes: &[ProbeResult]) -> Option<String> {
        let probe = probes
            .iter()
            .find(|p| p.exit_code == 0 && p.command.contains("ps aux --sort=-%mem"))?;

        let lines: Vec<&str> = probe.stdout.lines().skip(1).take(10).collect();
        if lines.is_empty() {
            return None;
        }

        let mut answer = String::from("**Top 10 processes by memory usage:**\n\n");
        answer.push_str("| PID | COMMAND | %MEM | RSS | USER |\n");
        answer.push_str("|-----|---------|------|-----|------|\n");

        for line in lines.iter() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 11 {
                // RSS in KB, format human-readable
                let rss_kb: u64 = parts[5].parse().unwrap_or(0);
                let rss = if rss_kb >= 1024 {
                    format!("{}M", rss_kb / 1024)
                } else {
                    format!("{}K", rss_kb)
                };
                answer.push_str(&format!(
                    "| {} | {} | {}% | {} | {} |\n",
                    parts[1],              // PID
                    parts[10..].join(" "), // COMMAND
                    parts[3],              // %MEM
                    rss,
                    parts[0] // USER
                ));
            }
        }

        Some(answer)
    }

    fn answer_disk_space(probes: &[ProbeResult]) -> Option<String> {
        let probe = probes
            .iter()
            .find(|p| p.exit_code == 0 && p.command.contains("df -h"))?;

        let mut answer = String::from("**Filesystem usage:**\n\n");
        answer.push_str("| Filesystem | Size | Used | Avail | Use% | Mounted on |\n");
        answer.push_str("|------------|------|------|-------|------|------------|\n");

        for line in probe.stdout.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 6 && !parts[0].starts_with("tmpfs") {
                let use_percent: u8 = parts[4].trim_end_matches('%').parse().unwrap_or(0);
                let status = if use_percent >= 95 {
                    " **CRITICAL**"
                } else if use_percent >= 85 {
                    " *warning*"
                } else {
                    ""
                };
                answer.push_str(&format!(
                    "| {} | {} | {} | {} | {}%{} | {} |\n",
                    parts[0], parts[1], parts[2], parts[3], use_percent, status, parts[5]
                ));
            }
        }

        Some(answer)
    }

    fn answer_network_interfaces(probes: &[ProbeResult]) -> Option<String> {
        let probe = probes
            .iter()
            .find(|p| p.exit_code == 0 && p.command.contains("ip addr"))?;

        let mut answer = String::from("**Network interfaces:**\n\n");
        answer.push_str("| Interface | IPv4 | State |\n");
        answer.push_str("|-----------|------|-------|\n");

        let mut current_iface = String::new();
        let mut current_state = String::new();
        let mut current_ipv4 = String::new();

        for line in probe.stdout.lines() {
            if line
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
            {
                // Flush previous interface
                if !current_iface.is_empty() {
                    let ipv4 = if current_ipv4.is_empty() {
                        "-"
                    } else {
                        &current_ipv4
                    };
                    answer.push_str(&format!(
                        "| {} | {} | {} |\n",
                        current_iface, ipv4, current_state
                    ));
                }
                // Parse new interface
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    current_iface = parts[1].trim_end_matches(':').to_string();
                    current_state = if line.contains("state UP") {
                        "UP".to_string()
                    } else if line.contains("state DOWN") {
                        "DOWN".to_string()
                    } else {
                        "UNKNOWN".to_string()
                    };
                    current_ipv4.clear();
                }
            } else if line.trim().starts_with("inet ") {
                if let Some(addr) = line.trim().split_whitespace().nth(1) {
                    current_ipv4 = addr.split('/').next().unwrap_or(addr).to_string();
                }
            }
        }
        // Flush last interface
        if !current_iface.is_empty() {
            let ipv4 = if current_ipv4.is_empty() {
                "-"
            } else {
                &current_ipv4
            };
            answer.push_str(&format!(
                "| {} | {} | {} |\n",
                current_iface, ipv4, current_state
            ));
        }

        Some(answer)
    }
}

// === Hardware edge case tests ===

#[test]
fn test_empty_hardware_returns_none() {
    let context = RuntimeContext {
        version: "0.0.12".to_string(),
        daemon_running: true,
        capabilities: anna_shared::rpc::Capabilities::default(),
        hardware: HardwareSummary {
            cpu_model: String::new(),
            cpu_cores: 0,
            ram_gb: 0.0,
            gpu: None,
            gpu_vram_gb: None,
            ..Default::default()
        },
        probes: std::collections::HashMap::new(),
    };
    let probes = vec![];

    // Should return None when hardware info is empty
    let answer = deterministic_answerer::try_answer("what cpu do i have?", &context, &probes);
    assert!(answer.is_none());
}

#[test]
fn test_unknown_cpu_model_returns_none() {
    let context = RuntimeContext {
        version: "0.0.12".to_string(),
        daemon_running: true,
        capabilities: anna_shared::rpc::Capabilities::default(),
        hardware: HardwareSummary {
            cpu_model: "Unknown".to_string(),
            cpu_cores: 8,
            ram_gb: 16.0,
            gpu: None,
            gpu_vram_gb: None,
            ..Default::default()
        },
        probes: std::collections::HashMap::new(),
    };
    let probes = vec![];

    // Should return None when CPU model is "Unknown"
    let answer = deterministic_answerer::try_answer("what cpu do i have?", &context, &probes);
    assert!(answer.is_none());
}

#[test]
fn test_zero_ram_returns_none() {
    let context = RuntimeContext {
        version: "0.0.12".to_string(),
        daemon_running: true,
        capabilities: anna_shared::rpc::Capabilities::default(),
        hardware: HardwareSummary {
            cpu_model: "Intel Core i5".to_string(),
            cpu_cores: 4,
            ram_gb: 0.0,
            gpu: None,
            gpu_vram_gb: None,
            ..Default::default()
        },
        probes: std::collections::HashMap::new(),
    };
    let probes = vec![];

    // Should return None when RAM is 0
    let answer = deterministic_answerer::try_answer("how much ram?", &context, &probes);
    assert!(answer.is_none());
}
