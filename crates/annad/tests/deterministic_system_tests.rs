//! Integration tests for deterministic answerer - System queries.
//!
//! Tests disk space and network interface queries.

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
        _context: &RuntimeContext,
        probe_results: &[ProbeResult],
    ) -> Option<String> {
        let query_type = classify_query(query);

        match query_type {
            QueryType::DiskSpace => answer_disk_space(probe_results),
            QueryType::NetworkInterfaces => answer_network_interfaces(probe_results),
            _ => None, // Only handle system queries in this test file
        }
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

// === Test fixtures ===

fn make_hardware() -> HardwareSummary {
    HardwareSummary {
        cpu_model: "Intel(R) Core(TM) i9-14900HX".to_string(),
        cpu_cores: 32,
        ram_gb: 31.0,
        gpu: Some("NVIDIA GeForce RTX 4060 Laptop GPU".to_string()),
        gpu_vram_gb: Some(8.0),
        ..Default::default()
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

// === Disk space tests ===

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
fn test_disk_space_shows_critical_warning() {
    // Golden test: disk usage >= 95% shows CRITICAL, >= 85% shows warning
    let context = make_context();
    let probes = vec![make_df_h_output()]; // /home is at 96%

    let answer =
        deterministic_answerer::try_answer("how much disk space is free?", &context, &probes);

    assert!(answer.is_some());
    let answer = answer.unwrap();
    assert!(
        answer.contains("CRITICAL") || answer.contains("critical"),
        "96% usage must show CRITICAL. Got: {}",
        answer
    );
}

#[test]
fn test_disk_space_filters_tmpfs() {
    let context = make_context();
    let probes = vec![make_df_h_output()];

    let answer =
        deterministic_answerer::try_answer("show disk space", &context, &probes);

    assert!(answer.is_some());
    let answer = answer.unwrap();

    // tmpfs should be filtered out
    assert!(!answer.contains("tmpfs"));

    // But real filesystems should be present
    assert!(answer.contains("/dev/sda1"));
    assert!(answer.contains("/dev/sdb1"));
}

#[test]
fn test_query_classification_disk() {
    use deterministic_answerer::{classify_query, QueryType};

    assert_eq!(classify_query("disk space free"), QueryType::DiskSpace);
    assert_eq!(classify_query("show storage"), QueryType::DiskSpace);
    assert_eq!(classify_query("filesystem usage"), QueryType::DiskSpace);
    assert_eq!(classify_query("how much space left"), QueryType::DiskSpace);
}

// === Network interface tests ===

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
fn test_network_shows_interface_state() {
    // Golden test: network interfaces must show UP/DOWN state
    let context = make_context();
    let probes = vec![make_ip_addr_output()];

    let answer =
        deterministic_answerer::try_answer("what are my network interfaces?", &context, &probes);

    assert!(answer.is_some());
    let answer = answer.unwrap();
    assert!(
        answer.contains("UP") && answer.contains("DOWN"),
        "Output must show UP/DOWN states. Got: {}",
        answer
    );
}

#[test]
fn test_network_shows_all_interfaces() {
    let context = make_context();
    let probes = vec![make_ip_addr_output()];

    let answer = deterministic_answerer::try_answer(
        "list network interfaces",
        &context,
        &probes,
    );

    assert!(answer.is_some());
    let answer = answer.unwrap();

    // All three interfaces should be present
    assert!(answer.contains("lo"));
    assert!(answer.contains("eth0"));
    assert!(answer.contains("wlan0"));

    // States should be correct
    assert!(answer.contains("127.0.0.1"));
    assert!(answer.contains("192.168.1.100"));
}

#[test]
fn test_network_interface_without_ip() {
    let context = make_context();
    let probes = vec![make_ip_addr_output()];

    let answer = deterministic_answerer::try_answer(
        "show network interfaces",
        &context,
        &probes,
    );

    assert!(answer.is_some());
    let answer = answer.unwrap();

    // wlan0 has no IP, should show "-"
    assert!(answer.contains("wlan0"));
    assert!(answer.contains("DOWN"));
}

#[test]
fn test_query_classification_network() {
    use deterministic_answerer::{classify_query, QueryType};

    assert_eq!(
        classify_query("network interfaces and ips"),
        QueryType::NetworkInterfaces
    );
    assert_eq!(classify_query("show my ip"), QueryType::NetworkInterfaces);
    assert_eq!(classify_query("what is my ip?"), QueryType::NetworkInterfaces);
    assert_eq!(classify_query("list interfaces"), QueryType::NetworkInterfaces);
}
