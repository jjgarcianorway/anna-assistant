//! Direct answer generation from probe results (v0.0.403).
//!
//! This module bypasses the LLM specialist entirely for queries where
//! probe data directly and unambiguously answers the question.
//!
//! The key insight: if we have the right probes and they succeeded, we can
//! generate accurate answers deterministically - the LLM often fails at this.

use anna_shared::parsers::{parse_probe_result, ParsedProbeData};
use anna_shared::rpc::ProbeResult;
use regex::Regex;
use tracing::info;

/// Result of direct probe answer generation
pub struct DirectAnswerResult {
    pub answer: String,
    pub confidence: u8,
}

/// Try to generate a direct answer from probe results based on query pattern.
/// Returns Some if we can confidently answer without LLM.
pub fn try_direct_answer(query: &str, probes: &[ProbeResult]) -> Option<DirectAnswerResult> {
    let q = query.to_lowercase();

    // Service status queries - most common LLM failure case
    if let Some(r) = try_service_answer(&q, probes) {
        return Some(r);
    }

    // Swap queries
    if let Some(r) = try_swap_answer(&q, probes) {
        return Some(r);
    }

    // Disk queries
    if let Some(r) = try_disk_answer(&q, probes) {
        return Some(r);
    }

    // Memory queries
    if let Some(r) = try_memory_answer(&q, probes) {
        return Some(r);
    }

    // Network/IP queries
    if let Some(r) = try_network_answer(&q, probes) {
        return Some(r);
    }

    // Bluetooth queries
    if let Some(r) = try_bluetooth_answer(&q, probes) {
        return Some(r);
    }

    // GPU queries
    if let Some(r) = try_gpu_answer(&q, probes) {
        return Some(r);
    }

    // Webcam queries
    if let Some(r) = try_webcam_answer(&q, probes) {
        return Some(r);
    }

    // CPU queries
    if let Some(r) = try_cpu_answer(&q, probes) {
        return Some(r);
    }

    // Audio queries
    if let Some(r) = try_audio_answer(&q, probes) {
        return Some(r);
    }

    None
}

/// Service status answer
fn try_service_answer(query: &str, probes: &[ProbeResult]) -> Option<DirectAnswerResult> {
    // Extract service name from query
    let service = extract_service_name(query)?;

    for probe in probes {
        let cmd = probe.command.to_lowercase();
        if !cmd.contains("systemctl") {
            continue;
        }
        if !cmd.contains(&service) && !cmd.contains("--all") && !cmd.contains("--failed") {
            continue;
        }

        let output = &probe.stdout;
        let stderr = &probe.stderr;

        // Check systemctl status output patterns
        if output.contains("Active: active (running)") {
            info!("v0.0.403: Direct service answer - running");
            return Some(DirectAnswerResult {
                answer: format!("**{}.service** is **active and running**.", service),
                confidence: 95,
            });
        }

        if output.contains("Active: inactive") || output.contains("inactive (dead)") {
            return Some(DirectAnswerResult {
                answer: format!(
                    "**{}.service** is **inactive** (not running).\n\nTo start: `sudo systemctl start {}`",
                    service, service
                ),
                confidence: 95,
            });
        }

        if output.contains("Active: failed") {
            return Some(DirectAnswerResult {
                answer: format!(
                    "**{}.service** is in **failed** state.\n\nCheck logs: `journalctl -u {} -n 50`",
                    service, service
                ),
                confidence: 95,
            });
        }

        if output.contains("could not be found") || stderr.contains("could not be found") {
            return Some(DirectAnswerResult {
                answer: format!("**{}.service** does not exist on this system.", service),
                confidence: 95,
            });
        }

        // Simple is-active output
        let trimmed = output.trim();
        if trimmed == "active" {
            return Some(DirectAnswerResult {
                answer: format!("**{}.service** is **active**.", service),
                confidence: 90,
            });
        }
        if trimmed == "inactive" {
            return Some(DirectAnswerResult {
                answer: format!("**{}.service** is **inactive**.", service),
                confidence: 90,
            });
        }
    }

    None
}

/// Extract service name from query
fn extract_service_name(query: &str) -> Option<String> {
    // Common service name patterns
    let patterns = [
        r"(?:is|check|status)\s+(\w+)(?:\.service)?\s+(?:running|active|started)",
        r"(\w+)(?:\.service)?\s+(?:running|active|status|started)",
        r"(?:running|active|started)\s+(\w+)(?:\.service)?",
    ];

    for pattern in &patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(caps) = re.captures(query) {
                if let Some(m) = caps.get(1) {
                    return Some(m.as_str().to_string());
                }
            }
        }
    }

    // Check for common service keywords
    for svc in [
        "bluetooth", "docker", "nginx", "ssh", "sshd", "cups", "pipewire",
        "pulseaudio", "networkmanager", "apache", "mysql", "postgresql", "redis",
    ] {
        if query.contains(svc) {
            return Some(svc.to_string());
        }
    }

    None
}

/// Swap answer
fn try_swap_answer(query: &str, probes: &[ProbeResult]) -> Option<DirectAnswerResult> {
    if !query.contains("swap") {
        return None;
    }

    for probe in probes {
        let cmd = probe.command.to_lowercase();

        // /proc/swaps output
        if cmd.contains("/proc/swap") || cmd.contains("swapon") {
            let lines: Vec<&str> = probe.stdout.lines().collect();

            // Just header = no swap
            if lines.len() <= 1 {
                info!("v0.0.403: Direct swap answer - no swap");
                return Some(DirectAnswerResult {
                    answer: "**No swap** is configured on this system.".to_string(),
                    confidence: 95,
                });
            }

            // Has swap entries
            let mut answer = "**Swap is configured:**\n".to_string();
            for line in lines.iter().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let filename = parts[0];
                    let size_kb: u64 = parts[2].parse().unwrap_or(0);
                    let size_mb = size_kb / 1024;
                    answer.push_str(&format!("- {} ({} MB)\n", filename, size_mb));
                }
            }
            return Some(DirectAnswerResult { answer, confidence: 95 });
        }

        // free -h output
        if cmd.contains("free") {
            for line in probe.stdout.lines() {
                if line.starts_with("Swap:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let total = parts[1];
                        if total == "0" || total == "0B" || total == "0K" || total == "0M" {
                            return Some(DirectAnswerResult {
                                answer: "**No swap** is configured on this system.".to_string(),
                                confidence: 90,
                            });
                        }
                        return Some(DirectAnswerResult {
                            answer: format!("**Swap:** {} total", total),
                            confidence: 90,
                        });
                    }
                }
            }
        }
    }

    None
}

/// Disk answer
fn try_disk_answer(query: &str, probes: &[ProbeResult]) -> Option<DirectAnswerResult> {
    if !query.contains("disk") && !query.contains("space") && !query.contains("storage") {
        return None;
    }

    for probe in probes {
        let cmd = probe.command.to_lowercase();
        if cmd.contains("df") {
            let parsed = parse_probe_result(probe);
            if let ParsedProbeData::Disk(disks) = parsed {
                let mut answer = "**Disk Usage:**\n".to_string();
                for disk in &disks {
                    let status = if disk.percent_used >= 90 {
                        " [CRITICAL]"
                    } else if disk.percent_used >= 80 {
                        " [WARNING]"
                    } else {
                        ""
                    };
                    answer.push_str(&format!(
                        "- {} - {}% used{}\n",
                        disk.mount, disk.percent_used, status
                    ));
                }
                info!("v0.0.403: Direct disk answer");
                return Some(DirectAnswerResult { answer, confidence: 95 });
            }
        }
    }

    None
}

/// Memory answer
fn try_memory_answer(query: &str, probes: &[ProbeResult]) -> Option<DirectAnswerResult> {
    if !query.contains("memory") && !query.contains("ram") {
        return None;
    }

    for probe in probes {
        let cmd = probe.command.to_lowercase();
        if cmd.contains("free") {
            let parsed = parse_probe_result(probe);
            if let ParsedProbeData::Memory(mem) = parsed {
                let used_gb = mem.used_bytes as f64 / 1024.0 / 1024.0 / 1024.0;
                let total_gb = mem.total_bytes as f64 / 1024.0 / 1024.0 / 1024.0;
                let avail_gb = mem.available_bytes as f64 / 1024.0 / 1024.0 / 1024.0;
                let used_pct = (mem.used_bytes as f64 / mem.total_bytes as f64 * 100.0) as u8;

                let status = if used_pct >= 90 {
                    " [HIGH]"
                } else if used_pct >= 75 {
                    " [MODERATE]"
                } else {
                    ""
                };

                let answer = format!(
                    "**Memory Usage:**\n- Used: {:.1} GB / {:.1} GB ({}%){}\n- Available: {:.1} GB",
                    used_gb, total_gb, used_pct, status, avail_gb
                );
                info!("v0.0.403: Direct memory answer");
                return Some(DirectAnswerResult { answer, confidence: 95 });
            }
        }
    }

    None
}

/// Network answer
fn try_network_answer(query: &str, probes: &[ProbeResult]) -> Option<DirectAnswerResult> {
    if !query.contains("ip") && !query.contains("network") && !query.contains("address") {
        return None;
    }

    for probe in probes {
        let cmd = probe.command.to_lowercase();
        if cmd.contains("ip addr") || cmd.contains("ip a") {
            let mut interfaces: Vec<(String, Vec<String>)> = Vec::new();
            let mut current_iface = String::new();
            let mut current_ips: Vec<String> = Vec::new();

            for line in probe.stdout.lines() {
                if line.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                    if !current_iface.is_empty() {
                        interfaces.push((current_iface.clone(), current_ips.clone()));
                        current_ips.clear();
                    }
                    if let Some(name) = line.split(':').nth(1) {
                        current_iface = name.trim().to_string();
                    }
                } else if line.contains("inet ") && !line.contains("inet6") {
                    if let Some(addr) = line.split_whitespace().nth(1) {
                        current_ips.push(addr.to_string());
                    }
                }
            }
            if !current_iface.is_empty() {
                interfaces.push((current_iface, current_ips));
            }

            if !interfaces.is_empty() {
                let mut answer = "**Network Interfaces:**\n".to_string();
                for (iface, ips) in &interfaces {
                    if ips.is_empty() {
                        answer.push_str(&format!("- {}: no IPv4\n", iface));
                    } else {
                        answer.push_str(&format!("- {}: {}\n", iface, ips.join(", ")));
                    }
                }
                info!("v0.0.403: Direct network answer");
                return Some(DirectAnswerResult { answer, confidence: 90 });
            }
        }
    }

    None
}

/// Bluetooth answer
fn try_bluetooth_answer(query: &str, probes: &[ProbeResult]) -> Option<DirectAnswerResult> {
    if !query.contains("bluetooth") {
        return None;
    }

    for probe in probes {
        let cmd = probe.command.to_lowercase();
        if cmd.contains("systemctl") && cmd.contains("bluetooth") {
            let output = &probe.stdout;

            if output.contains("Active: active (running)") {
                info!("v0.0.403: Direct bluetooth answer - running");
                return Some(DirectAnswerResult {
                    answer: "**Bluetooth** service is **active and running**.".to_string(),
                    confidence: 95,
                });
            }

            if output.contains("Active: inactive") {
                return Some(DirectAnswerResult {
                    answer: "**Bluetooth** service is **inactive**.\n\nTo start: `sudo systemctl start bluetooth`".to_string(),
                    confidence: 95,
                });
            }

            if output.contains("could not be found") {
                return Some(DirectAnswerResult {
                    answer: "**Bluetooth** service is not installed.\n\nInstall with your package manager (e.g., `bluez`).".to_string(),
                    confidence: 95,
                });
            }
        }

        if cmd.contains("bluetoothctl") {
            if probe.stdout.contains("Controller") {
                return Some(DirectAnswerResult {
                    answer: format!("**Bluetooth** hardware detected:\n{}",
                        probe.stdout.lines().take(5).collect::<Vec<_>>().join("\n")),
                    confidence: 85,
                });
            }
        }
    }

    None
}

/// GPU answer
fn try_gpu_answer(query: &str, probes: &[ProbeResult]) -> Option<DirectAnswerResult> {
    if !query.contains("gpu") && !query.contains("graphic") && !query.contains("video") {
        return None;
    }

    for probe in probes {
        let cmd = probe.command.to_lowercase();
        if cmd.contains("lspci") && (cmd.contains("vga") || cmd.contains("3d")) {
            let gpus: Vec<&str> = probe.stdout.lines().filter(|l| !l.is_empty()).collect();

            if gpus.is_empty() {
                return Some(DirectAnswerResult {
                    answer: "No GPU detected via lspci.".to_string(),
                    confidence: 80,
                });
            }

            let mut answer = "**Graphics Hardware:**\n".to_string();
            for gpu in &gpus {
                answer.push_str(&format!("- {}\n", gpu.trim()));
            }
            info!("v0.0.403: Direct GPU answer");
            return Some(DirectAnswerResult { answer, confidence: 90 });
        }
    }

    None
}

/// Webcam answer
fn try_webcam_answer(query: &str, probes: &[ProbeResult]) -> Option<DirectAnswerResult> {
    if !query.contains("webcam") && !query.contains("camera") {
        return None;
    }

    for probe in probes {
        let cmd = probe.command.to_lowercase();
        if cmd.contains("v4l2") || cmd.contains("/dev/video") {
            if probe.stdout.trim().is_empty() || probe.exit_code != 0 {
                return Some(DirectAnswerResult {
                    answer: "**No webcam detected**. Check if camera is connected or drivers are loaded.".to_string(),
                    confidence: 85,
                });
            }

            let devices: Vec<&str> = probe.stdout.lines().filter(|l| !l.is_empty()).collect();
            if !devices.is_empty() {
                let mut answer = "**Webcam Devices:**\n".to_string();
                for dev in &devices {
                    answer.push_str(&format!("- {}\n", dev.trim()));
                }
                info!("v0.0.403: Direct webcam answer");
                return Some(DirectAnswerResult { answer, confidence: 90 });
            }
        }
    }

    None
}

/// CPU answer
fn try_cpu_answer(query: &str, probes: &[ProbeResult]) -> Option<DirectAnswerResult> {
    if !query.contains("cpu") && !query.contains("processor") {
        return None;
    }

    for probe in probes {
        let cmd = probe.command.to_lowercase();
        if cmd.contains("lscpu") {
            let parsed = parse_probe_result(probe);
            if let ParsedProbeData::Cpu(cpu) = parsed {
                let answer = format!(
                    "**CPU:** {}\n- Cores: {}\n- Threads: {}",
                    cpu.model_name,
                    cpu.physical_cores().unwrap_or(cpu.cpu_count),
                    cpu.cpu_count
                );
                info!("v0.0.403: Direct CPU answer");
                return Some(DirectAnswerResult { answer, confidence: 95 });
            }
        }
    }

    None
}

/// Audio answer
fn try_audio_answer(query: &str, probes: &[ProbeResult]) -> Option<DirectAnswerResult> {
    if !query.contains("audio") && !query.contains("sound") && !query.contains("speaker") {
        return None;
    }

    for probe in probes {
        let cmd = probe.command.to_lowercase();
        if cmd.contains("pactl") || cmd.contains("aplay") || (cmd.contains("lspci") && cmd.contains("audio")) {
            if !probe.stdout.trim().is_empty() {
                let mut answer = "**Audio Devices:**\n".to_string();
                for line in probe.stdout.lines().take(10) {
                    if !line.trim().is_empty() {
                        answer.push_str(&format!("- {}\n", line.trim()));
                    }
                }
                info!("v0.0.403: Direct audio answer");
                return Some(DirectAnswerResult { answer, confidence: 85 });
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_probe(cmd: &str, stdout: &str) -> ProbeResult {
        ProbeResult {
            command: cmd.to_string(),
            exit_code: 0,
            stdout: stdout.to_string(),
            stderr: String::new(),
            timing_ms: 100,
        }
    }

    #[test]
    fn test_service_running() {
        let probe = make_probe(
            "systemctl status bluetooth.service",
            "● bluetooth.service - Bluetooth service\n   Loaded: loaded\n   Active: active (running)",
        );
        let result = try_direct_answer("is bluetooth running", &[probe]).unwrap();
        assert!(result.answer.contains("active and running"));
    }

    #[test]
    fn test_no_swap() {
        let probe = make_probe("cat /proc/swaps", "Filename\tType\tSize\tUsed\tPriority");
        let result = try_direct_answer("do i have swap", &[probe]).unwrap();
        assert!(result.answer.contains("No swap"));
    }

    #[test]
    fn test_disk_usage() {
        let probe = make_probe(
            "df -h",
            "Filesystem      Size  Used Avail Use% Mounted on\n/dev/sda1       100G   50G   50G  50% /",
        );
        let result = try_direct_answer("disk space", &[probe]).unwrap();
        assert!(result.answer.contains("Disk Usage"));
    }

    #[test]
    fn test_service_extraction() {
        assert_eq!(extract_service_name("is bluetooth running"), Some("bluetooth".to_string()));
        assert_eq!(extract_service_name("docker status"), Some("docker".to_string()));
        assert_eq!(extract_service_name("check nginx service"), Some("nginx".to_string()));
    }
}
