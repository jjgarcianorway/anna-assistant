//! System information answer functions (v0.0.175).
//!
//! Uptime, timezone, hostname, OS, architecture, locale, process tree.

use anna_shared::rpc::ProbeResult;

use crate::deterministic::DeterministicResult;
use crate::parsers::find_probe;

/// Answer package updates query using checkupdates probe
pub fn answer_package_updates(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "checkupdates").or_else(|| find_probe(probes, "pacman"));
    let probe = probe?;

    let output = probe.stdout.trim();

    if output.is_empty() || probe.exit_code != 0 {
        return Some(DeterministicResult {
            answer: "No package updates available. Your system is up to date.".to_string(),
            grounded: true,
            parsed_data_count: 1,
            route_class: route_class.to_string(),
        });
    }

    let update_count = output.lines().count();
    let preview: Vec<&str> = output.lines().take(5).collect();
    let preview_str = preview.join("\n  ");

    let answer = if update_count == 1 {
        format!("1 package update available:\n  {}", preview_str)
    } else if update_count <= 5 {
        format!("{} package updates available:\n  {}", update_count, preview_str)
    } else {
        format!(
            "{} package updates available:\n  {}\n  ...and {} more",
            update_count, preview_str, update_count - 5
        )
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: update_count,
        route_class: route_class.to_string(),
    })
}

/// Answer swap info query using free probe
pub fn answer_swap_info(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "free")?;
    if probe.exit_code != 0 {
        return None;
    }

    for line in probe.stdout.lines() {
        if line.starts_with("Swap:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let answer = format!("Swap: {} total, {} used, {} free", parts[1], parts[2], parts[3]);
                return Some(DeterministicResult {
                    answer,
                    grounded: true,
                    parsed_data_count: 1,
                    route_class: route_class.to_string(),
                });
            }
        }
    }

    Some(DeterministicResult {
        answer: "No swap space is configured on this system.".to_string(),
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer timezone info query using timedatectl probe
pub fn answer_timezone_info(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "timedatectl")?;
    if probe.exit_code != 0 {
        return None;
    }

    let mut timezone = String::new();
    let mut local_time = String::new();
    let mut ntp_status = String::new();

    for line in probe.stdout.lines() {
        let line = line.trim();
        if line.starts_with("Time zone:") {
            timezone = line.strip_prefix("Time zone:").unwrap_or("").trim().to_string();
        } else if line.starts_with("Local time:") {
            local_time = line.strip_prefix("Local time:").unwrap_or("").trim().to_string();
        } else if line.starts_with("NTP service:") || line.starts_with("System clock synchronized:") {
            ntp_status = line.to_string();
        }
    }

    let mut answer = String::new();
    if !timezone.is_empty() {
        answer.push_str(&format!("Timezone: {}\n", timezone));
    }
    if !local_time.is_empty() {
        answer.push_str(&format!("Local time: {}\n", local_time));
    }
    if !ntp_status.is_empty() {
        answer.push_str(&ntp_status);
    }

    if answer.is_empty() {
        return None;
    }

    Some(DeterministicResult {
        answer: answer.trim().to_string(),
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer system uptime query using uptime probe
pub fn answer_system_uptime(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "uptime")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return None;
    }

    Some(DeterministicResult {
        answer: format!("System has been {}", output),
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer system load query using /proc/loadavg
pub fn answer_system_load(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "load_average")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return None;
    }

    let parts: Vec<&str> = output.split_whitespace().collect();
    if parts.len() >= 3 {
        let answer = format!(
            "System load averages:\n  1 min:  {}\n  5 min:  {}\n  15 min: {}",
            parts[0], parts[1], parts[2]
        );
        return Some(DeterministicResult {
            answer,
            grounded: true,
            parsed_data_count: 3,
            route_class: route_class.to_string(),
        });
    }

    None
}

/// Answer last boot query using who -b
pub fn answer_last_boot(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "last_boot")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return None;
    }

    let boot_time = output
        .strip_prefix("system boot")
        .or_else(|| output.split("system boot").nth(1))
        .map(|s| s.trim())
        .unwrap_or(output);

    Some(DeterministicResult {
        answer: format!("System last booted: {}", boot_time),
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer hostname query using hostname command
pub fn answer_hostname(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "hostname")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return None;
    }

    Some(DeterministicResult {
        answer: format!("Hostname: {}", output),
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer OS info query using /etc/os-release
pub fn answer_os_info(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "os_release")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return None;
    }

    let mut name = String::new();
    let mut version = String::new();
    let mut pretty_name = String::new();

    for line in output.lines() {
        if line.starts_with("PRETTY_NAME=") {
            pretty_name = line.strip_prefix("PRETTY_NAME=").unwrap_or("").trim_matches('"').to_string();
        } else if line.starts_with("NAME=") {
            name = line.strip_prefix("NAME=").unwrap_or("").trim_matches('"').to_string();
        } else if line.starts_with("VERSION=") {
            version = line.strip_prefix("VERSION=").unwrap_or("").trim_matches('"').to_string();
        }
    }

    let answer = if !pretty_name.is_empty() {
        format!("OS: {}", pretty_name)
    } else if !name.is_empty() && !version.is_empty() {
        format!("OS: {} {}", name, version)
    } else if !name.is_empty() {
        format!("OS: {}", name)
    } else {
        return None;
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer system architecture query using uname -m
pub fn answer_system_architecture(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "arch")?;
    if probe.exit_code != 0 {
        return None;
    }

    let arch = probe.stdout.trim();
    if arch.is_empty() {
        return None;
    }

    let desc = match arch {
        "x86_64" => "64-bit x86 (AMD64/Intel64)",
        "i686" | "i386" => "32-bit x86",
        "aarch64" => "64-bit ARM",
        "armv7l" => "32-bit ARM (ARMv7)",
        "riscv64" => "64-bit RISC-V",
        _ => arch,
    };

    Some(DeterministicResult {
        answer: format!("Architecture: {} ({})", arch, desc),
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer process tree query
pub fn answer_process_tree(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "pstree")?;
    if probe.exit_code != 0 {
        return Some(DeterministicResult {
            answer: "pstree not available (install psmisc package)".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No process tree available.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let line_count = output.lines().count();
    Some(DeterministicResult {
        answer: format!("Process tree ({} lines):\n```\n{}\n```", line_count, output),
        grounded: true,
        parsed_data_count: line_count,
        route_class: route_class.to_string(),
    })
}

/// Answer open files count query
pub fn answer_open_files(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "open_files")?;
    if probe.exit_code != 0 {
        return Some(DeterministicResult {
            answer: "lsof not available or requires elevated permissions".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let count: usize = probe.stdout.trim().parse().unwrap_or(0);
    Some(DeterministicResult {
        answer: format!("Open files: {} file descriptors system-wide", count),
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer system locale query
pub fn answer_system_locale(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "locale")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No locale settings available.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let mut lang = None;
    let mut lc_all = None;

    for line in output.lines() {
        if let Some(val) = line.strip_prefix("LANG=") {
            lang = Some(val.trim_matches('"'));
        }
        if let Some(val) = line.strip_prefix("LC_ALL=") {
            lc_all = Some(val.trim_matches('"'));
        }
    }

    let primary = lc_all.unwrap_or_else(|| lang.unwrap_or("not set"));
    Some(DeterministicResult {
        answer: format!("System locale: {}\n\nFull output:\n{}", primary, output),
        grounded: true,
        parsed_data_count: output.lines().count(),
        route_class: route_class.to_string(),
    })
}

/// Answer virtualization info query
pub fn answer_virtualization_info(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "virtualization_info")?;

    let output = probe.stdout.trim();
    let answer = if output == "none" || output.is_empty() {
        "Running on bare metal (no virtualization detected).".to_string()
    } else {
        format!("Virtualization: **{}**", output)
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer coredump list query
pub fn answer_coredump_list(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "coredump_list")?;

    let output = probe.stdout.trim();
    if output.contains("not available") || output.contains("No coredumps") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "No coredumps found on this system.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let dump_count = output.lines().count().saturating_sub(1);
    Some(DeterministicResult {
        answer: format!("Coredumps ({} found):\n```\n{}\n```", dump_count, output),
        grounded: true,
        parsed_data_count: dump_count,
        route_class: route_class.to_string(),
    })
}

/// Answer tmp files query
pub fn answer_tmp_files(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "tmp_files")?;

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "/tmp directory is empty.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let file_count = output.lines().count().saturating_sub(1);
    Some(DeterministicResult {
        answer: format!("Files in /tmp ({}):\n```\n{}\n```", file_count, output),
        grounded: true,
        parsed_data_count: file_count,
        route_class: route_class.to_string(),
    })
}
