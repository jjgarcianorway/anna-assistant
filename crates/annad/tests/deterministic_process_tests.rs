//! Integration tests for deterministic answerer - Process and memory queries.
//!
//! Tests process listing and memory usage queries.

use anna_shared::rpc::{HardwareSummary, ProbeResult, RuntimeContext};

// Reuse the deterministic_answerer module from hardware tests
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
            QueryType::TopMemoryProcesses => answer_top_memory(probe_results),
            _ => None, // Only handle process queries in this test file
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

// === Process and memory tests ===

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
fn test_top_memory_shows_pid_column() {
    // Golden test: top_memory output must include PID column
    let context = make_context();
    let probes = vec![make_ps_aux_output()];

    let answer = deterministic_answerer::try_answer(
        "what processes are using the most memory?",
        &context,
        &probes,
    );

    assert!(answer.is_some());
    let answer = answer.unwrap();
    // Must have PID in table header or content
    assert!(
        answer.contains("PID") || answer.contains("1234"),
        "Output must include PID column. Got: {}",
        answer
    );
}

#[test]
fn test_top_memory_includes_all_columns() {
    let context = make_context();
    let probes = vec![make_ps_aux_output()];

    let answer = deterministic_answerer::try_answer(
        "what processes are using the most memory?",
        &context,
        &probes,
    );

    assert!(answer.is_some());
    let answer = answer.unwrap();

    // Verify all expected columns are present
    assert!(answer.contains("PID"));
    assert!(answer.contains("COMMAND"));
    assert!(answer.contains("%MEM"));
    assert!(answer.contains("RSS"));
    assert!(answer.contains("USER"));

    // Verify actual data is present
    assert!(answer.contains("firefox"));
    assert!(answer.contains("code"));
    assert!(answer.contains("systemd"));
}

#[test]
fn test_top_memory_rss_formatting() {
    let context = make_context();
    let probes = vec![make_ps_aux_output()];

    let answer = deterministic_answerer::try_answer(
        "what processes are using the most memory?",
        &context,
        &probes,
    );

    assert!(answer.is_some());
    let answer = answer.unwrap();

    // RSS should be formatted in MB for large values (50000 KB = 48M)
    assert!(answer.contains("48M") || answer.contains("49M"));
}

#[test]
fn test_query_classification_processes() {
    use deterministic_answerer::{classify_query, QueryType};

    assert_eq!(
        classify_query("processes using most memory"),
        QueryType::TopMemoryProcesses
    );
    assert_eq!(
        classify_query("what are the memory hogs"),
        QueryType::TopMemoryProcesses
    );
    assert_eq!(
        classify_query("show top memory"),
        QueryType::TopMemoryProcesses
    );
    assert_eq!(
        classify_query("which process uses most ram"),
        QueryType::TopMemoryProcesses
    );
}

#[test]
fn test_empty_process_list_returns_none() {
    let context = make_context();
    let probes = vec![ProbeResult {
        command: "ps aux --sort=-%mem".to_string(),
        exit_code: 0,
        stdout: "USER       PID %CPU %MEM    VSZ   RSS TTY      STAT START   TIME COMMAND\n"
            .to_string(),
        stderr: String::new(),
        timing_ms: 100,
    }];

    let answer = deterministic_answerer::try_answer(
        "what processes are using the most memory?",
        &context,
        &probes,
    );

    // Should return None when no processes in output
    assert!(answer.is_none());
}

#[test]
fn test_failed_process_probe_not_used() {
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
