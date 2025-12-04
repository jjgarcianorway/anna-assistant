//! Integration tests for deterministic answerer fallback.
//!
//! These tests verify that Anna can produce answers from probe data
//! even when the LLM translator/specialist times out.

use anna_shared::rpc::{HardwareSummary, ProbeResult, ReliabilitySignals, RuntimeContext};

mod deterministic_answerer {
    //! Re-implementation of deterministic answerer for testing.
    //! This mirrors the logic in annad::deterministic.

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

        let lines: Vec<&str> = probe.stdout.lines().skip(1).take(5).collect();
        if lines.is_empty() {
            return None;
        }

        let mut answer = String::from("**Top processes by memory usage:**\n\n");
        answer.push_str("| # | Process | Memory | CPU | User |\n");
        answer.push_str("|---|---------|--------|-----|------|\n");

        for (i, line) in lines.iter().enumerate() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 11 {
                answer.push_str(&format!(
                    "| {} | {} | {}% | {}% | {} |\n",
                    i + 1,
                    parts[10..].join(" "),
                    parts[3],
                    parts[2],
                    parts[0]
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
            if line.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
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

// === Test fixtures ===

fn make_hardware() -> HardwareSummary {
    HardwareSummary {
        cpu_model: "Intel(R) Core(TM) i9-14900HX".to_string(),
        cpu_cores: 32,
        ram_gb: 31.0,
        gpu: Some("NVIDIA GeForce RTX 4060 Laptop GPU".to_string()),
        gpu_vram_gb: Some(8.0),
    }
}

fn make_context() -> RuntimeContext {
    RuntimeContext {
        version: "0.0.12".to_string(),
        daemon_running: true,
        capabilities: anna_shared::rpc::Capabilities::default(),
        hardware: make_hardware(),
        probes: std::collections::HashMap::new(),
    }
}

fn make_ps_aux_output() -> ProbeResult {
    ProbeResult {
        command: "ps aux --sort=-%mem".to_string(),
        exit_code: 0,
        stdout: r#"USER       PID %CPU %MEM    VSZ   RSS TTY      STAT START   TIME COMMAND
root         1  0.0  0.1 169936 12456 ?        Ss   Dec01   0:03 /sbin/init
user      1234  5.0 10.2 500000 50000 ?        Sl   10:00   1:23 firefox
user      5678  2.0  8.5 400000 40000 ?        Sl   10:00   0:45 code
root       999  1.0  5.2 300000 30000 ?        Sl   10:00   0:30 systemd
user      1111  0.5  3.1 200000 20000 ?        Sl   10:00   0:15 bash"#
            .to_string(),
        stderr: String::new(),
        timing_ms: 150,
    }
}

fn make_df_h_output() -> ProbeResult {
    ProbeResult {
        command: "df -h".to_string(),
        exit_code: 0,
        stdout: r#"Filesystem      Size  Used Avail Use% Mounted on
/dev/sda1       100G   80G   20G  80% /
/dev/sdb1        50G   48G    2G  96% /home
tmpfs           16G     0   16G   0% /dev/shm"#
            .to_string(),
        stderr: String::new(),
        timing_ms: 50,
    }
}

fn make_ip_addr_output() -> ProbeResult {
    ProbeResult {
        command: "ip addr show".to_string(),
        exit_code: 0,
        stdout: r#"1: lo: <LOOPBACK,UP,LOWER_UP> mtu 65536 state UNKNOWN
    inet 127.0.0.1/8 scope host lo
2: eth0: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500 state UP
    inet 192.168.1.100/24 brd 192.168.1.255 scope global eth0
3: wlan0: <BROADCAST,MULTICAST> mtu 1500 state DOWN"#
            .to_string(),
        stderr: String::new(),
        timing_ms: 30,
    }
}

// === Integration tests ===

#[test]
fn test_cpu_query_deterministic() {
    let context = make_context();
    let probes = vec![];

    let answer = deterministic_answerer::try_answer("what cpu do i have?", &context, &probes);

    assert!(answer.is_some());
    let answer = answer.unwrap();
    assert!(answer.contains("Intel"));
    assert!(answer.contains("32 cores"));
}

#[test]
fn test_ram_query_deterministic() {
    let context = make_context();
    let probes = vec![];

    let answer = deterministic_answerer::try_answer("how much ram do i have?", &context, &probes);

    assert!(answer.is_some());
    let answer = answer.unwrap();
    assert!(answer.contains("31.0 GB"));
}

#[test]
fn test_top_memory_deterministic() {
    let context = make_context();
    let probes = vec![make_ps_aux_output()];

    let answer = deterministic_answerer::try_answer(
        "what processes are using the most memory?",
        &context,
        &probes,
    );

    assert!(answer.is_some());
    let answer = answer.unwrap();
    assert!(answer.contains("firefox"));
    assert!(answer.contains("10.2%"));
}

#[test]
fn test_disk_space_deterministic() {
    let context = make_context();
    let probes = vec![make_df_h_output()];

    let answer =
        deterministic_answerer::try_answer("how much disk space is free?", &context, &probes);

    assert!(answer.is_some());
    let answer = answer.unwrap();
    assert!(answer.contains("/dev/sda1"));
    assert!(answer.contains("80%"));
    assert!(answer.contains("CRITICAL")); // /home is at 96%
}

#[test]
fn test_network_interfaces_deterministic() {
    let context = make_context();
    let probes = vec![make_ip_addr_output()];

    let answer = deterministic_answerer::try_answer(
        "what are my network interfaces and IPs?",
        &context,
        &probes,
    );

    assert!(answer.is_some());
    let answer = answer.unwrap();
    assert!(answer.contains("eth0"));
    assert!(answer.contains("192.168.1.100"));
    assert!(answer.contains("UP"));
}

#[test]
fn test_deterministic_scoring_with_probes() {
    // When probes succeed and deterministic answer is generated,
    // reliability score should be > 20 even if translator timed out
    let signals = ReliabilitySignals {
        translator_confident: false, // Translator timed out
        probe_coverage: true,        // All probes succeeded
        answer_grounded: true,       // Deterministic = always grounded
        no_invention: true,          // Deterministic = never invents
        clarification_not_needed: true, // We produced an answer
    };

    // Score should be 80 (4 signals * 20)
    assert_eq!(signals.score(), 80);
    assert!(signals.score() > 20, "Score should be > 20 when probes succeed");
}

#[test]
fn test_unknown_query_returns_none() {
    let context = make_context();
    let probes = vec![];

    // Query that doesn't match any pattern
    let answer =
        deterministic_answerer::try_answer("what is the meaning of life?", &context, &probes);

    assert!(answer.is_none());
}

#[test]
fn test_query_classification() {
    use deterministic_answerer::{classify_query, QueryType};

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
    assert_eq!(classify_query("random question"), QueryType::Unknown);
}

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
        },
        probes: std::collections::HashMap::new(),
    };
    let probes = vec![];

    // Should return None when hardware info is empty
    let answer = deterministic_answerer::try_answer("what cpu do i have?", &context, &probes);
    assert!(answer.is_none());
}

#[test]
fn test_failed_probe_not_used() {
    let context = make_context();
    let probes = vec![ProbeResult {
        command: "ps aux --sort=-%mem".to_string(),
        exit_code: 1, // Failed
        stdout: String::new(),
        stderr: "Permission denied".to_string(),
        timing_ms: 100,
    }];

    let answer = deterministic_answerer::try_answer(
        "what processes are using the most memory?",
        &context,
        &probes,
    );

    // Should return None because probe failed
    assert!(answer.is_none());
}
